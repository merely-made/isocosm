// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::state::{Bench, Specimen};
use mesocosm_core::{
    PartId, World, state_hash,
    world::generation::{Candidate, ProportionSelection},
};
use mesocosm_mesh::content::ContentPack;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) struct Alternative {
    pub world: Option<Box<World>>,
    pub candidate: Option<Candidate>,
    pub description: String,
    pub changed_parts: Vec<PartId>,
}

pub(super) struct Comparison {
    pub epoch: u64,
    pub source: ProportionSelection,
    pub cards: Vec<Alternative>,
    pub selected: usize,
    pub shared_scale: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SavedComparison {
    pub selection: ProportionSelection,
    pub content: ContentPack,
    pub expected_hash: u64,
    #[serde(default)]
    pub base_content: Option<ContentPack>,
    #[serde(default = "base_size")]
    pub size: u8,
}

fn base_size() -> u8 {
    1
}

impl SavedComparison {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        let saved: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if let Some(base) = &saved.base_content {
            if base
                .resized(saved.size)
                .map_err(|e| format!("Size refused: {e:?}"))?
                != saved.content
            {
                return Err("Saved size does not reproduce its content.".into());
            }
        } else if saved.size != 1 {
            return Err("Saved size requires its base content.".into());
        }
        saved
            .content
            .resolve()
            .map_err(|e| format!("Content refused: {e:?}"))?;
        if saved.selection.palette != saved.content.palette {
            return Err("Comparison palette does not match its content.".into());
        }
        let world = saved
            .selection
            .enter()
            .map_err(|e| format!("Comparison refused: {e:?}"))?;
        if state_hash(&world) != saved.expected_hash {
            return Err("Comparison no longer reproduces its saved world.".into());
        }
        Ok(saved)
    }
}

impl Specimen {
    pub fn world(&self) -> &World {
        self.comparison
            .as_ref()
            .and_then(|c| c.cards.get(c.selected))
            .and_then(|c| c.world.as_deref())
            .unwrap_or_else(|| self.creator.world())
    }

    pub fn card_world(&self, card: Option<usize>) -> Option<&World> {
        match card {
            None => Some(self.world()),
            Some(index) => self.comparison.as_ref()?.cards.get(index)?.world.as_deref(),
        }
    }

    pub fn compare(&mut self, round: u64) -> Result<(), String> {
        if self.creator.pending {
            return Err("Wait for the specimen to finish generating.".into());
        }
        let prepared = self
            .creator
            .prepared
            .as_ref()
            .ok_or("Choose an admitted specimen first.")?;
        let index = self.creator.selected;
        let original = prepared
            .draft()
            .candidates
            .get(index)
            .ok_or("No admitted specimen.")?;
        let source = prepared.proportion_selection(index, round, 0);
        let mut cards = vec![Alternative {
            world: Some(Box::new(
                prepared.enter(index).map_err(|e| format!("{e:?}"))?,
            )),
            candidate: Some(original.clone()),
            description: "Retained original".into(),
            changed_parts: Vec::new(),
        }];
        for (number, option) in prepared
            .proportions(index, round)
            .map_err(|e| format!("{e:?}"))?
            .into_iter()
            .enumerate()
        {
            let description = match (option.stretch, option.shape) {
                (Some(stretch), Some(shape)) => {
                    let old = original.recipe.tagmata[stretch].segment_shape;
                    let before = source.palette.mass.at(old).half_extent;
                    let after = source.palette.mass.at(shape).half_extent;
                    let words: Vec<_> = [
                        (0, "narrower", "wider"),
                        (1, "lower", "taller"),
                        (2, "shorter", "longer"),
                    ]
                    .into_iter()
                    .filter_map(|(axis, less, more)| match after[axis].cmp(&before[axis]) {
                        std::cmp::Ordering::Less => Some(less),
                        std::cmp::Ordering::Greater => Some(more),
                        _ => None,
                    })
                    .collect();
                    format!("Stretch {}: {}", stretch + 1, words.join(", "))
                },
                _ => "No further admitted shape".into(),
            };
            let world = prepared.enter_proportion(index, round, number);
            let card = match (option.candidate, world) {
                (Ok(candidate), Ok(world)) => {
                    let changed_parts = candidate
                        .body
                        .parts
                        .iter()
                        .zip(&original.body.parts)
                        .filter(|(a, b)| a.half_extent != b.half_extent || a.volume != b.volume)
                        .map(|(a, _)| a.id)
                        .collect();
                    Alternative {
                        world: Some(Box::new(world)),
                        candidate: Some(candidate),
                        description,
                        changed_parts,
                    }
                },
                (Err(why), _) | (_, Err(why)) => Alternative {
                    world: None,
                    candidate: None,
                    description: format!("{description}. Refused: {why:?}"),
                    changed_parts: Vec::new(),
                },
            };
            cards.push(card);
        }
        self.comparison = Some(Comparison {
            epoch: self.epoch + 1,
            source,
            cards,
            selected: 0,
            shared_scale: true,
        });
        self.selected = None;
        self.isolated = true;
        self.epoch += 1;
        self.changed();
        Ok(())
    }

    pub fn selected_change(&self) -> Option<String> {
        let comparison = self.comparison.as_ref()?;
        let card = &comparison.cards[comparison.selected];
        let selected = self.selected?;
        Some(if card.changed_parts.contains(&selected.part) {
            format!(
                "{}; this part's shape changed. Parent and appendage assignment held.",
                card.description
            )
        } else {
            "This part's shape is held. Its position can move with the changed stretch.".into()
        })
    }
}

impl Alternative {
    pub fn counts(&self) -> String {
        self.candidate.as_ref().map_or("Not admitted".into(), |c| {
            let meshes: BTreeSet<_> = c.body.parts.iter().map(|p| p.volume).collect();
            format!("{} parts · {} distinct shapes", c.parts, meshes.len())
        })
    }
}

impl Bench {
    pub fn compare(&mut self) {
        let result = self.model.borrow_mut().compare(0);
        self.notice = result.err().unwrap_or_default();
        self.events.push("proportions-compared".into());
    }

    pub fn more_proportions(&mut self) {
        let round = self
            .model
            .borrow()
            .comparison
            .as_ref()
            .map_or(0, |c| c.source.round.wrapping_add(4));
        let result = self.model.borrow_mut().compare(round);
        self.notice = result.err().unwrap_or_default();
    }

    pub fn choose_proportion(&mut self, index: usize) {
        let mut model = self.model.borrow_mut();
        let Some(comparison) = &mut model.comparison else {
            return;
        };
        if !comparison
            .cards
            .get(index)
            .is_some_and(|c| c.world.is_some())
        {
            return;
        }
        comparison.selected = index;
        model.selected = None;
        model.epoch += 1;
        model.changed();
        self.notice.clear();
        self.events.push(format!("proportion-selected-{index}"));
    }

    pub fn shared_scale(&mut self) {
        let mut model = self.model.borrow_mut();
        if let Some(comparison) = &mut model.comparison {
            comparison.shared_scale = !comparison.shared_scale;
            model.changed();
        }
    }

    pub fn save_comparison(&mut self) {
        let result = self.write_comparison();
        self.notice = match result {
            Ok(path) => format!(
                "Saved {}. Reopen with --bench --comparison FILE.",
                path.display()
            ),
            Err(why) => why,
        };
    }

    fn write_comparison(&self) -> Result<PathBuf, String> {
        let model = self.model.borrow();
        let selection = if let Some(comparison) = &model.comparison {
            let mut selection = comparison.source.clone();
            selection.selected = comparison.selected;
            selection
        } else {
            model
                .creator
                .prepared
                .as_ref()
                .ok_or("Wait for generation.")?
                .proportion_selection(model.creator.selected, 0, 0)
        };
        let saved = SavedComparison {
            base_content: self.generation.base.clone(),
            size: self.generation.size,
            selection,
            content: model
                .content
                .clone()
                .ok_or("Saving requires an admitted content pack.")?,
            expected_hash: state_hash(model.world()),
        };
        let reproduced = saved
            .selection
            .enter()
            .map_err(|e| format!("Specimen cannot be saved: {e:?}"))?;
        if state_hash(&reproduced) != saved.expected_hash {
            return Err("Wait for the selected specimen before saving.".into());
        }
        let parent = self.export_directory.as_path();
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let file = tempfile::Builder::new()
            .prefix("specimen-proportions-")
            .suffix(".json")
            .tempfile_in(parent)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file.as_file(), &saved).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        let (_, path) = file.keep().map_err(|e| e.to_string())?;
        Ok(path)
    }
}
