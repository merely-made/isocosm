// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Opt-in specimen experiment: interpret accepted Mesocosm history through a
//! caller-configured glyph canon. This reading neither changes World nor grants
//! durable divinity powers. Core remains the authority for the underlying act.

use crate::TrialUptake;
use mesocosm_core::{History, OrganismId, World, embodiment::embodied, history::Event, state_hash};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wing_glyphs::{
    Canon, CanonSpec, ExpressionTable, GlyphId, GrantOutcome, Journey, Provenance, ProvenanceKind,
    VariantPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptedKind {
    Carved,
    Moved,
    Fed,
    /// A producer's feeding: positive soil uptake, recorded as a flow rather
    /// than an event (ruled 2026-09-15). It grants through the same base
    /// glyph as feeding.
    ///
    /// **Evidence, not a form selector** (2026-09-15). Which act a grant came
    /// through is still on the record and still explains the acquisition; it
    /// no longer decides how the glyph returns, because the axis is now what
    /// bears the glyph. This is also the seed of the *used* manner in the
    /// experience condition: a trait took part in a recorded flow.
    Uptake,
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
    /// Which accepted act this record answers to, stated rather than
    /// re-derived. Uptake is a recorded flow, so `event` is absent for it.
    pub kind: AcceptedKind,
    pub event: Option<Event>,
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
        if rules.grants.is_empty() || rules.grants.len() > 4 {
            return Err("glyph experiment requires one to four event grant rules".into());
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

    /// Does the bound individual own a base whose *current* canon effect is
    /// this one? Read-only: it joins `Journey::owns_base` to `Canon::effect`
    /// and grants nothing. Keying on the current revision, not the effect
    /// recorded at acquisition, is what `EffectPackTable` expects.
    pub fn owns_effect(&self, effect: &str) -> bool {
        self.effect_bases(effect)
            .any(|base| self.journey.owns_base(base))
    }

    /// Which accepted act earned this effect, read back out of the event or
    /// flow the grant was made from. `None` when it is not owned.
    ///
    /// **Evidence only.** Under the acquiring-act axis this chose the mark's
    /// form; it no longer does. The form follows what bears the glyph, and
    /// this stays because the acquisition still has to be explainable and
    /// because the manner that satisfied an experience condition is what
    /// shapes the divinity later.
    pub fn acquired_by(&self, effect: &str) -> Option<AcceptedKind> {
        let canon = self.journey.canon();
        self.records.iter().find_map(|record| {
            if record.outcome != GlyphGrantOutcome::Acquired {
                return None;
            }
            let base = canon.base_id(&record.glyph)?;
            (canon.effect(base)? == effect).then_some(record.kind)
        })
    }

    /// Every glyph the bound body **currently embodies**, read through the
    /// expression table.
    ///
    /// Not the journey's answer and not a grant: a body embodies from its
    /// first frame, having done nothing, and loses a glyph when the part
    /// expressing it is cut. `&`-only in both directions — this cannot grant,
    /// cannot express, and cannot change one saved byte.
    ///
    /// The world arrives by reference rather than being held: the reading is
    /// a disposable trial's record and must not own or outlive world state,
    /// and the registry the sites resolve against is the world's own ruleset.
    /// A body that is absent or no longer alive bears nothing.
    pub fn embodied_glyphs(&self, world: &World, table: &ExpressionTable) -> BTreeSet<GlyphId> {
        world
            .organisms
            .iter()
            .find(|o| o.id == self.rules.organism && o.is_alive())
            .map(|o| embodied(&o.phenotype, world.ruleset(), table))
            .unwrap_or_default()
    }

    fn effect_bases<'a>(&'a self, effect: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        let canon = self.journey.canon();
        canon
            .spec()
            .glyphs
            .iter()
            .filter(move |glyph| glyph.effect == effect)
            .map(|glyph| glyph.id.as_str())
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
            let glyph = rule.glyph.clone();
            let evidence = format!(
                "mesocosm.trial/{:016x}/{post_hash:016x}/history/{sequence}",
                self.baseline_hash
            );
            let context = format!("{kind:?}; organism={}", actor.0);
            self.grant_one(
                glyph,
                evidence,
                context,
                recorded.tick,
                sequence,
                kind,
                Some(recorded.record),
            );
        }
    }

    /// Accepted soil uptake: a producer's feeding (ruled 2026-09-15). Only
    /// positive milligrams to the bound body count, and `Trial::uptakes`
    /// already carries nothing else. The first one acquires the base glyph and
    /// later ones are duplicates kept as evidence, exactly as an event is.
    /// A flow carries no `Event`, so the record states its kind instead.
    pub(crate) fn absorb_uptake(&mut self, uptakes: &[TrialUptake], post_hash: u64) {
        let Some(glyph) = self
            .rules
            .grants
            .iter()
            .find(|rule| rule.event == AcceptedKind::Uptake)
            .map(|rule| rule.glyph.clone())
        else {
            return;
        };
        for record in uptakes {
            if record.organism != self.rules.organism || record.record.record.amount_mg == 0 {
                continue;
            }
            // Uptake identity is (tick, flow ordinal), not a history sequence.
            let evidence = format!(
                "mesocosm.trial/{:016x}/{post_hash:016x}/uptake/{}/{}",
                self.baseline_hash, record.tick, record.sequence
            );
            let context = format!(
                "Uptake; organism={}; {} mg",
                record.organism.0, record.record.record.amount_mg
            );
            self.grant_one(
                glyph.clone(),
                evidence,
                context,
                record.tick,
                record.sequence,
                AcceptedKind::Uptake,
                None,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_one(
        &mut self,
        glyph: String,
        evidence: String,
        context: String,
        tick: u64,
        sequence: u64,
        kind: AcceptedKind,
        event: Option<Event>,
    ) {
        let outcome = match self.journey.grant(
            &glyph,
            Provenance {
                kind: ProvenanceKind::Event,
                evidence: evidence.clone(),
                context: Some(context),
            },
            tick,
            VariantPolicy::RequireOwnedBase,
        ) {
            Ok(GrantOutcome::Acquired { .. }) => GlyphGrantOutcome::Acquired,
            Ok(GrantOutcome::Recorded { .. }) => GlyphGrantOutcome::Recorded,
            Ok(GrantOutcome::Duplicate { .. }) => GlyphGrantOutcome::Duplicate,
            Err(why) => GlyphGrantOutcome::Rejected(why),
        };
        self.records.push(GlyphEvidence {
            sequence,
            tick,
            kind,
            event,
            glyph,
            evidence,
            outcome,
        });
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
