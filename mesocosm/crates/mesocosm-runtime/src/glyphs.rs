// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Opt-in glyph reading over the native journal (ruling 774): accepted acts
//! of one bound critter, read from the session's log and its flows, granted
//! through a caller's canon. It neither changes the world nor grants
//! durable powers; the sim stays the authority for the act.

use isocosm::effects::{GlyphGrantOutcome, Journal, embodied};
use isocosm::flows::{Flow, Holder};
use isocosm::history::Command;
use isocosm::schema::Id;
use isocosm::{Session, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wing_glyphs::{Canon, CanonSpec, ExpressionTable, GlyphId, Provenance, ProvenanceKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptedKind {
    Carved,
    Moved,
    Fed,
    /// Matter taken from a site, as a producer's feeding is: evidence for the
    /// same glyph as feeding where a rule says so.
    Uptake,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventGrant {
    pub event: AcceptedKind,
    /// A base identity from the supplied canon.
    pub glyph: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphRules {
    pub canon: CanonSpec,
    pub individual: String,
    /// The critter whose acts are read.
    pub organism: Id,
    pub unlock_thresholds: Vec<u64>,
    /// One rule per kind, one to four.
    pub grants: Vec<EventGrant>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GlyphEvidence {
    pub tick: u64,
    pub kind: AcceptedKind,
    pub glyph: String,
    pub evidence: String,
    pub outcome: GlyphGrantOutcome,
}

pub struct GlyphReading {
    rules: GlyphRules,
    journal: Journal,
    records: Vec<GlyphEvidence>,
    /// Log entries already read.
    seen: usize,
    place: Option<Id>,
}

impl GlyphReading {
    pub fn new(rules: GlyphRules, session: &Session) -> Result<Self, String> {
        let canon = Canon::new(rules.canon.clone())?;
        let pop = &session.sim.state().population;
        let place = pop.get(rules.organism).filter(|e| e.alive).map(|e| e.place);
        if place.is_none() {
            return Err("glyph reading requires a living bound critter".into());
        }
        if rules.grants.is_empty() || rules.grants.len() > 4 {
            return Err("glyph reading requires one to four grant rules".into());
        }
        let mut kinds = BTreeSet::new();
        for rule in &rules.grants {
            if !kinds.insert(rule.event) {
                return Err("glyph reading has duplicate grant selectors".into());
            }
            if !canon.is_base(&rule.glyph) {
                return Err(format!("glyph reading requires a known base: {}", rule.glyph));
            }
        }
        let journal = Journal::new(
            canon,
            rules.individual.clone(),
            rules.unlock_thresholds.clone(),
        )?;
        Ok(Self {
            rules,
            journal,
            records: vec![],
            seen: session.entries.len(),
            place,
        })
    }

    pub fn rules(&self) -> &GlyphRules {
        &self.rules
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn records(&self) -> &[GlyphEvidence] {
        &self.records
    }

    /// Reads one round: the log's new entries, the critter's place and the
    /// round's flows.
    pub fn absorb(&mut self, session: &Session, flows: &[Flow]) {
        let me = self.rules.organism;
        let tick = session.sim.state().tick;
        for entry in &session.entries[self.seen.min(session.entries.len())..] {
            let carved = matches!(&entry.command, Command::Edit { by: Some(by), edit, .. }
                if *by == me && matches!(edit.op, isometer_space::Op::Carve));
            if carved {
                self.grant(AcceptedKind::Carved, entry.tick, &entry.id);
            }
        }
        self.seen = session.entries.len();
        let now = session.sim.state().population.get(me).map(|e| e.place);
        if now.is_some() && now != self.place {
            self.grant(AcceptedKind::Moved, tick, &format!("place/{tick}"));
        }
        self.place = now.or(self.place);
        for (i, flow) in flows.iter().enumerate() {
            if flow.to.0.body() != Some(me) || flow.amount == 0 {
                continue;
            }
            let kind = match flow.from.0 {
                Holder::Site(_) => AcceptedKind::Uptake,
                from if from.body().is_some_and(|b| b != me) => AcceptedKind::Fed,
                _ => continue,
            };
            self.grant(kind, flow.tick, &format!("flow/{}/{i}", flow.tick));
        }
    }

    fn grant(&mut self, kind: AcceptedKind, tick: u64, evidence: &str) {
        let Some(rule) = self.rules.grants.iter().find(|r| r.event == kind) else {
            return;
        };
        let glyph = rule.glyph.clone();
        let evidence = format!("mesocosm.session/{evidence}");
        let provenance = Provenance {
            kind: ProvenanceKind::Event,
            evidence: evidence.clone(),
            context: Some(format!("{kind:?}; organism={}", self.rules.organism)),
        };
        let (outcome, _) = self.journal.grant(&glyph, provenance, tick);
        self.records.push(GlyphEvidence {
            tick,
            kind,
            glyph,
            evidence,
            outcome,
        });
    }

    /// Whether the journey owns a base bearing `effect`.
    pub fn owns_effect(&self, effect: &str) -> bool {
        let spec = self.journal.canon().spec();
        let bases = spec.glyphs.iter().filter(|g| g.effect == effect);
        bases.into_iter().any(|g| self.journal.journey().owns_base(&g.id))
    }

    /// The glyphs the bound critter's living parts embody under `table`.
    pub fn embodied_glyphs(&self, sim: &Simulation, table: &ExpressionTable) -> BTreeSet<GlyphId> {
        let pop = &sim.state().population;
        let me = pop.get(self.rules.organism).filter(|e| e.alive);
        me.map(|e| embodied(e, table)).unwrap_or_default()
    }
}
