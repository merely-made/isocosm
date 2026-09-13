// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Opt-in specimen experiment: interpret accepted Mesocosm history through a
//! caller-configured glyph canon. This reading neither changes World nor grants
//! durable divinity powers. Core remains the authority for the underlying act.

use mesocosm_core::{History, OrganismId, World, history::Event, state_hash};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wing_glyphs::{
    Canon, CanonSpec, GrantOutcome, Journey, Provenance, ProvenanceKind, VariantPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptedKind {
    Carved,
    Moved,
    Fed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventGrant {
    pub event: AcceptedKind,
    /// A base identity from the supplied canon, never a Unicode display string.
    pub glyph: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphRules {
    pub canon: CanonSpec,
    pub individual: String,
    /// Explicit body binding for this disposable run; changing control does
    /// not establish a new life, transfer this collection or earn experience.
    pub organism: OrganismId,
    pub unlock_thresholds: Vec<u64>,
    /// One rule per event kind, at least one. This slice grants bases only.
    pub grants: Vec<EventGrant>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphGrantOutcome {
    Acquired,
    Recorded,
    Duplicate,
    Rejected(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GlyphEvidence {
    pub sequence: u64,
    pub tick: u64,
    pub event: Event,
    pub glyph: String,
    /// Identifies the exact baseline, post-step world and history ordinal.
    /// Retained separately from the core event, whose authority is unchanged.
    pub evidence: String,
    pub outcome: GlyphGrantOutcome,
}

pub struct GlyphReading {
    rules: GlyphRules,
    baseline_hash: u64,
    journey: Journey,
    records: Vec<GlyphEvidence>,
}

impl GlyphReading {
    pub(crate) fn new(rules: GlyphRules, baseline: &World) -> Result<Self, String> {
        let canon = Canon::new(rules.canon.clone())?;
        if !baseline
            .organisms
            .iter()
            .any(|o| o.id == rules.organism && o.is_alive())
        {
            return Err("glyph experiment requires a living bound organism".into());
        }
        if rules.grants.is_empty() || rules.grants.len() > 3 {
            return Err("glyph experiment requires one to three event grant rules".into());
        }
        let mut kinds = BTreeSet::new();
        for rule in &rules.grants {
            if !kinds.insert(rule.event) {
                return Err("glyph experiment has duplicate event grant selectors".into());
            }
            if !canon.is_base(&rule.glyph) {
                return Err(format!(
                    "glyph experiment requires a known base ID: {}",
                    rule.glyph
                ));
            }
        }
        let journey = Journey::new(
            rules.individual.clone(),
            &canon,
            rules.unlock_thresholds.clone(),
        )?;
        Ok(Self {
            rules,
            baseline_hash: state_hash(baseline),
            journey,
            records: Vec::new(),
        })
    }

    pub fn rules(&self) -> &GlyphRules {
        &self.rules
    }
    pub fn journey(&self) -> &Journey {
        &self.journey
    }
    /// Entire ordered experiment record, including repeated accepted acts and
    /// any shared-kernel refusal. Redrawing never consumes or grants anything.
    pub fn records(&self) -> &[GlyphEvidence] {
        &self.records
    }

    pub(crate) fn reset(&mut self, baseline: &World) {
        *self = Self::new(self.rules.clone(), baseline)
            .expect("previously validated exact baseline and rules");
    }

    pub(crate) fn absorb(&mut self, history: &History, start: usize, post_hash: u64) {
        for (offset, recorded) in history.log().entries()[start..].iter().enumerate() {
            let Some((kind, actor)) = accepted(recorded.record) else {
                continue;
            };
            if actor != self.rules.organism {
                continue;
            }
            let Some(rule) = self.rules.grants.iter().find(|rule| rule.event == kind) else {
                continue;
            };
            let sequence = (start + offset) as u64;
            let evidence = format!(
                "mesocosm.trial/{:016x}/{post_hash:016x}/history/{sequence}",
                self.baseline_hash
            );
            let outcome = match self.journey.grant(
                &rule.glyph,
                Provenance {
                    kind: ProvenanceKind::Event,
                    evidence: evidence.clone(),
                    context: Some(format!("{kind:?}; organism={}", actor.0)),
                },
                recorded.tick,
                VariantPolicy::RequireOwnedBase,
            ) {
                Ok(GrantOutcome::Acquired { .. }) => GlyphGrantOutcome::Acquired,
                Ok(GrantOutcome::Recorded { .. }) => GlyphGrantOutcome::Recorded,
                Ok(GrantOutcome::Duplicate { .. }) => GlyphGrantOutcome::Duplicate,
                Err(why) => GlyphGrantOutcome::Rejected(why),
            };
            self.records.push(GlyphEvidence {
                sequence,
                tick: recorded.tick,
                event: recorded.record,
                glyph: rule.glyph.clone(),
                evidence,
                outcome,
            });
        }
    }
}

fn accepted(event: Event) -> Option<(AcceptedKind, OrganismId)> {
    match event {
        Event::Carved {
            organism, removed, ..
        } if removed > 0 => Some((AcceptedKind::Carved, organism)),
        Event::Moved { organism, from, to } if from != to => Some((AcceptedKind::Moved, organism)),
        Event::Fed { eater, mass_mg, .. } if mass_mg > 0 => Some((AcceptedKind::Fed, eater)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
