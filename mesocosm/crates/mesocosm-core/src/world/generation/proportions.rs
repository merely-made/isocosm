// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded alternatives over one admitted recipe. Only mass-shape selectors vary.

use super::*;

#[derive(Clone, Debug)]
pub struct ProportionOption {
    pub stretch: Option<usize>,
    pub shape: Option<u8>,
    pub candidate: Result<Candidate, Error>,
}

/// Replayable inputs, including the palette that interprets shape selectors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProportionSelection {
    pub version: u32,
    pub source: Selection,
    pub palette: PartPalette,
    pub round: u64,
    /// Zero is the retained original; 1..=4 are the displayed alternatives.
    pub selected: usize,
}

impl ProportionSelection {
    pub fn enter(&self) -> Result<World, Error> {
        if self.version != 1 || self.selected > 4 {
            return Err(Error::Invalid("unsupported proportion selection"));
        }
        let prepared = self.source.request.prepare(self.palette)?;
        if self.selected == 0 {
            return prepared.enter(self.source.candidate);
        }
        prepared.enter_proportion(self.source.candidate, self.round, self.selected - 1)
    }
}

impl Prepared {
    pub fn proportion_selection(
        &self,
        index: usize,
        round: u64,
        selected: usize,
    ) -> ProportionSelection {
        ProportionSelection {
            version: 1,
            source: Selection {
                request: self.draft.request.clone(),
                candidate: index,
            },
            palette: self.foundation.development_palette,
            round,
            selected,
        }
    }

    /// Four distinct single-stretch edits, including explicit refusals.
    /// The original, individual realization, and appendage instructions are held.
    pub fn proportions(&self, index: usize, round: u64) -> Result<Vec<ProportionOption>, Error> {
        let original = self
            .draft
            .candidates
            .get(index)
            .ok_or(Error::CandidateUnavailable {
                requested: index,
                available: self.draft.candidates.len(),
            })?;
        let palette = self.foundation.development_palette;
        let mut choices = Vec::new();
        for (stretch, tagma) in original.recipe.tagmata.iter().enumerate() {
            let old = palette.mass.at(tagma.segment_shape);
            let mut envelopes = vec![old.half_extent];
            for slot in 0..crate::development::PALETTE_SHAPES {
                let template = if slot == 0 {
                    Some(palette.mass.default)
                } else {
                    palette.mass.extra[slot - 1]
                };
                if let Some(template) = template {
                    if !envelopes.contains(&template.half_extent) {
                        envelopes.push(template.half_extent);
                        choices.push((stretch, slot as u8));
                    }
                }
            }
        }
        let centre = self.draft.habitat[self.draft.request.place as usize].centre;
        let start = if choices.is_empty() {
            0
        } else {
            (round % choices.len() as u64) as usize
        };
        Ok((0..4)
            .map(|offset| {
                if offset >= choices.len() {
                    return ProportionOption {
                        stretch: None,
                        shape: None,
                        candidate: Err(Error::Invalid("no further distinct admitted proportion")),
                    };
                }
                let (stretch, shape) = choices[(start + offset) % choices.len()];
                let mut recipe = original.recipe.clone();
                recipe.tagmata[stretch].segment_shape = shape;
                let candidate = self
                    .draft
                    .request
                    .candidate_recipe(
                        &self.foundation,
                        palette,
                        centre,
                        original.seed,
                        original.role,
                        recipe,
                    )
                    .map_err(|why| Error::Development(why.into()));
                ProportionOption {
                    stretch: Some(stretch),
                    shape: Some(shape),
                    candidate,
                }
            })
            .collect())
    }

    pub fn enter_proportion(
        &self,
        index: usize,
        round: u64,
        alternative: usize,
    ) -> Result<World, Error> {
        let options = self.proportions(index, round)?;
        let option = options
            .get(alternative)
            .ok_or(Error::Invalid("proportion index must be 0..4"))?;
        self.enter_admitted(option.candidate.as_ref().map_err(Clone::clone)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternatives_hold_realized_topology_mass_and_appendages_and_replay() {
        let palette = crate::axis::archetype::spaced::palette();
        for plan in [BodyPlan::Axial, BodyPlan::Branched] {
            let request = Request {
                seed: 1,
                criteria: Criteria {
                    body_plan: plan,
                    role: Some(Kingdom::Producer),
                    ..Default::default()
                },
                ..Default::default()
            };
            let prepared = request.prepare(palette).unwrap();
            let original = &prepared.draft.candidates[0];
            let original_hash = crate::state_hash(&prepared.enter(0).unwrap());
            let options = prepared.proportions(0, 0).unwrap();
            assert_eq!(options.len(), 4);
            let mut accepted = 0;
            for (index, option) in options.iter().enumerate() {
                let Ok(candidate) = &option.candidate else {
                    continue;
                };
                accepted += 1;
                assert_ne!(candidate.recipe, original.recipe);
                let mut restored = candidate.recipe.clone();
                for (a, b) in restored.tagmata.iter_mut().zip(&original.recipe.tagmata) {
                    a.segment_shape = b.segment_shape;
                }
                assert_eq!(restored, original.recipe);
                assert_eq!(
                    Soma::develop(&candidate.recipe, candidate.seed),
                    Soma::develop(&original.recipe, original.seed)
                );
                assert_eq!(candidate.body.parts.len(), original.body.parts.len());
                for (a, b) in candidate.body.parts.iter().zip(&original.body.parts) {
                    assert_eq!(a.id, b.id);
                    assert_eq!(
                        a.attachment.map(|a| a.parent),
                        b.attachment.map(|a| a.parent)
                    );
                    assert_eq!(a.mass_mg, b.mass_mg);
                }
                let selection = prepared.proportion_selection(0, 0, index + 1);
                let bytes = serde_json::to_vec(&selection).unwrap();
                let decoded: ProportionSelection = serde_json::from_slice(&bytes).unwrap();
                let entered = prepared.enter_proportion(0, 0, index).unwrap();
                assert_eq!(
                    crate::state_hash(&entered),
                    crate::state_hash(&decoded.enter().unwrap())
                );
                assert_eq!(
                    entered.total_matter_mg(),
                    prepared.enter(0).unwrap().total_matter_mg()
                );
            }
            assert!(accepted >= 2);
            assert_eq!(
                crate::state_hash(&prepared.enter(0).unwrap()),
                original_hash
            );
        }
    }

    #[test]
    fn a_palette_without_alternatives_refuses_instead_of_repeating_original() {
        let prepared = Request::default()
            .prepare(PartPalette::primitive())
            .unwrap();
        assert!(
            prepared
                .proportions(0, u64::MAX)
                .unwrap()
                .iter()
                .all(|o| o.candidate.is_err())
        );
        assert!(prepared.enter_proportion(0, 0, 4).is_err());
    }
}
