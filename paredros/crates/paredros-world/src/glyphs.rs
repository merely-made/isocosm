// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P4: accepted Paredros history read through a caller-supplied glyph canon.
//!
//! This is the wing's second `wing-glyphs` consumer, after
//! `mesocosm-runtime::glyphs`, and it keeps that consumer's posture:
//!
//! - **Opt-in.** Nothing constructs a [`GlyphReading`] on its own. A host that
//!   wants one supplies [`GlyphRules`] and holds the reading itself.
//! - **Non-durable.** The reading is not in `GameSave` and is not written by
//!   any intent. It is rebuilt from accepted history when it is wanted, and
//!   losing it loses no world fact.
//! - **No world change.** Every method here takes `&GameState`. Enabling a
//!   reading cannot move the state hash.
//! - **No transfer on control change.** The rules name one [`SubjectId`], and
//!   [`AcceptedKind::Died`] for that subject ends the reading. Succession
//!   starts a new reading or none; it never inherits this one.
//!
//! Grants are keyed by [`AcceptedKind`], an enum over the `GameEvent` kinds
//! that carry an actor. The evidence string is `subject` plus the event's
//! index in `GameState::events()` and nothing else, so a reading rebuilt from
//! a restored save produces a byte-identical journal — a construction-time
//! world hash would not, because the two readings are opened at different
//! points in the same history.
//!
//! # Declared limits
//!
//! - The reading references effect IDs; it never executes one. What a glyph
//!   *does* is not modelled here at all.
//! - Reincarnation, wishes and divine spending are the shared kernel's and
//!   stay unused, exactly as they do in Mesocosm.
//! # Hagioglyph (F3b5)
//!
//! The world may publish a newer correspondence as
//! [`GameIntent::ReviseCanon`](crate::GameIntent::ReviseCanon), which is
//! accepted history like any other intent. This reading follows it: from that
//! event on, every evidence record and grant is stamped with the live
//! revision, and [`GlyphReading::live_effect`] answers from the revised canon
//! while [`GlyphReading::founding_effect`] keeps answering from the founding
//! one. Eligibility, the ascension basis and completion are the kernel's and
//! read the founding canon only. A revision no newer than the live one is
//! recorded world history the reading ignores.
//!
//! - A load that shortens the accepted log rewinds the cursor. Replayed
//!   indices then arrive as `Duplicate` (same glyph, same evidence) or, if a
//!   different history now occupies an index already used for the same glyph,
//!   as a recorded `Rejected`. Nothing panics and no acquisition is lost.

use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use wing_glyphs::{GrantOutcome, Journey, Provenance, VariantPolicy};

use crate::{CanonRevisionCause, GameEvent, GameState};

// The kernel types a host needs to author rules and draw a journal, so a
// consumer of this module needs no `wing-glyphs` dependency of its own.
pub use wing_glyphs::{
    Acquisition, Canon, CanonSpec, CorrespondenceMove, Eligibility, GlyphDefinition, GrantRecord,
    ProvenanceKind,
};

/// The accepted `GameEvent` kinds a rule may select. Each one names the
/// subject it is evidence *for*: the striker of a volley, the subject of an
/// injury, rest, pickup, attachment, motion step, move or death.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcceptedKind {
    VolleyResolved,
    Injured,
    Rested,
    Took,
    ItemAttached,
    MotionAdvanced,
    Moved,
    Died,
}

impl AcceptedKind {
    /// The kebab word a provenance and a panel row show for this kind.
    pub const fn label(self) -> &'static str {
        match self {
            Self::VolleyResolved => "volley-resolved",
            Self::Injured => "injured",
            Self::Rested => "rested",
            Self::Took => "took",
            Self::ItemAttached => "item-attached",
            Self::MotionAdvanced => "motion-advanced",
            Self::Moved => "moved",
            Self::Died => "died",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventGrant {
    pub event: AcceptedKind,
    /// A base identity from the supplied canon, never a Unicode display mark.
    pub glyph: String,
}

/// One subject's opt-in reading rules. The canon is embedded, not named, so
/// the journal carries the exact correspondence it was built against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphRules {
    pub canon: CanonSpec,
    pub individual: String,
    /// The one body this reading is bound to. Changing control neither
    /// transfers this collection nor begins a new life on it.
    pub subject: SubjectId,
    pub unlock_thresholds: Vec<u64>,
    /// One rule per accepted event kind, at least one. Bases only.
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

/// One accepted event this reading answered, whatever the kernel said about
/// it. Redrawing the journal never consumes or grants anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GlyphEvidence {
    /// The event's index in `GameState::events()`.
    pub index: usize,
    pub tick: u64,
    pub kind: AcceptedKind,
    pub glyph: String,
    pub evidence: String,
    pub outcome: GlyphGrantOutcome,
    /// The live canon revision this record was accepted under.
    pub canon_revision: u64,
}

/// The revision this reading currently answers `live_effect` from, and the
/// accepted cause that published it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LiveCanon {
    pub revision: u64,
    pub seed: u64,
    pub cause: CanonRevisionCause,
}

/// The opt-in reading: a `wing_glyphs::Journey` plus the evidence behind it.
pub struct GlyphReading {
    rules: GlyphRules,
    canon: Canon,
    /// The published correspondence, equal to `canon` until a revision is
    /// read. Only the effect an owned glyph *now* carries follows it.
    live: Canon,
    revised: Option<LiveCanon>,
    journey: Journey,
    records: Vec<GlyphEvidence>,
    cursor: usize,
    ended: bool,
}

impl GlyphReading {
    /// Opens a reading and consumes the accepted history the game already
    /// holds, from the first event. Never touches `game`.
    pub fn new(rules: GlyphRules, game: &GameState) -> Result<Self, String> {
        let canon = Canon::new(rules.canon.clone())?;
        if game.bodies().get(rules.subject).is_none() {
            return Err("glyph reading requires an existing bound subject".into());
        }
        if rules.grants.is_empty() || rules.grants.len() > 8 {
            return Err("glyph reading requires one to eight event grant rules".into());
        }
        let mut kinds = BTreeSet::new();
        for rule in &rules.grants {
            if !kinds.insert(rule.event) {
                return Err("glyph reading has duplicate event grant selectors".into());
            }
            if !canon.is_base(&rule.glyph) {
                return Err(format!(
                    "glyph reading requires a known base ID: {}",
                    rule.glyph
                ));
            }
        }
        let journey = Journey::new(
            rules.individual.clone(),
            &canon,
            rules.unlock_thresholds.clone(),
        )?;
        let mut reading = Self {
            rules,
            live: canon.clone(),
            canon,
            revised: None,
            journey,
            records: Vec::new(),
            cursor: 0,
            ended: false,
        };
        reading.advance(game);
        Ok(reading)
    }

    /// Consumes every accepted event since this reading's cursor. Idempotent
    /// when nothing new was accepted, and a no-op once the subject has died.
    pub fn advance(&mut self, game: &GameState) {
        if self.ended {
            return;
        }
        let events = game.events();
        let mut index = self.cursor.min(events.len());
        while index < events.len() {
            let event = &events[index];
            if let GameEvent::CanonRevised {
                revision,
                seed,
                cause,
                ..
            } = event
            {
                self.revise(*revision, *seed, cause.clone());
            }
            if let Some((kind, subject)) = accepted(event)
                && subject == self.rules.subject
            {
                if let Some(rule) = self.rules.grants.iter().find(|rule| rule.event == kind) {
                    let glyph = rule.glyph.clone();
                    self.record(index, kind, glyph, tick_of(event));
                }
                if kind == AcceptedKind::Died {
                    self.ended = true;
                    self.cursor = index + 1;
                    return;
                }
            }
            index += 1;
        }
        self.cursor = events.len();
    }

    pub fn rules(&self) -> &GlyphRules {
        &self.rules
    }
    /// The founding correspondence: what completion and the ascension basis
    /// are judged against, whatever the world publishes later.
    pub fn canon(&self) -> &Canon {
        &self.canon
    }
    /// The published correspondence this reading currently answers from.
    pub fn live_canon(&self) -> &Canon {
        &self.live
    }
    /// The accepted revision in force, when one has been read.
    pub fn revision(&self) -> Option<&LiveCanon> {
        self.revised.as_ref()
    }
    /// What this glyph meant when the journey was founded.
    pub fn founding_effect(&self, glyph: &str) -> Option<&str> {
        self.canon.effect(glyph)
    }
    /// What this glyph means under the live revision. Equal to the founding
    /// effect until a revision moves that base.
    pub fn live_effect(&self, glyph: &str) -> Option<&str> {
        self.live.effect(glyph)
    }
    /// The bases whose effect the live revision moved.
    pub fn moved_bases(&self) -> Vec<CorrespondenceMove> {
        self.canon
            .correspondence_diff(&self.live)
            .unwrap_or_default()
    }
    pub fn journey(&self) -> &Journey {
        &self.journey
    }
    pub fn records(&self) -> &[GlyphEvidence] {
        &self.records
    }
    pub fn eligibility(&self) -> Eligibility {
        self.journey.eligibility()
    }
    /// True once the bound subject's death was read. Later events are ignored.
    pub fn ended(&self) -> bool {
        self.ended
    }
    /// How much of `GameState::events()` this reading has consumed.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// Whether `advance` has anything left to read.
    pub fn pending(&self, game: &GameState) -> bool {
        !self.ended && self.cursor < game.events().len()
    }

    fn record(&mut self, index: usize, kind: AcceptedKind, glyph: String, tick: u64) {
        let evidence = format!(
            "paredros.session/subject/{}/events/{index}",
            self.rules.subject.0
        );
        let provenance = Provenance {
            kind: ProvenanceKind::Custom(kind.label().to_owned()),
            evidence: evidence.clone(),
            context: Some(format!("subject={}", self.rules.subject.0)),
        };
        let canon_revision = self.live.spec().revision;
        let outcome = match self.journey.grant_at_revision(
            &glyph,
            provenance,
            tick,
            VariantPolicy::RequireOwnedBase,
            canon_revision,
        ) {
            Ok(GrantOutcome::Acquired { .. }) => GlyphGrantOutcome::Acquired,
            Ok(GrantOutcome::Recorded { .. }) => GlyphGrantOutcome::Recorded,
            Ok(GrantOutcome::Duplicate { .. }) => GlyphGrantOutcome::Duplicate,
            Err(why) => GlyphGrantOutcome::Rejected(why),
        };
        self.records.push(GlyphEvidence {
            index,
            tick,
            kind,
            glyph,
            evidence,
            outcome,
            canon_revision,
        });
    }

    /// Follows one accepted revision. Every revision is derived from the
    /// founding canon, so the live correspondence is a function of the
    /// published seed and revision alone. A revision that is not newer is
    /// history this reading has already answered under.
    fn revise(&mut self, revision: u64, seed: u64, cause: CanonRevisionCause) {
        if revision <= self.live.spec().revision {
            return;
        }
        if let Ok(live) = self.canon.shuffled(seed, revision) {
            self.live = live;
            self.revised = Some(LiveCanon {
                revision,
                seed,
                cause,
            });
        }
    }
}

/// Which subject an accepted event is evidence for, when it is evidence at
/// all. A volley answers for its *striker*; a zero-distance move and a
/// harmless injury are not accepted acts and answer for nobody.
fn accepted(event: &GameEvent) -> Option<(AcceptedKind, SubjectId)> {
    match event {
        GameEvent::VolleyResolved { actor, strikes, .. } if !strikes.is_empty() => {
            Some((AcceptedKind::VolleyResolved, *actor))
        },
        GameEvent::Injured { subject, harm, .. } if *harm > 0 => {
            Some((AcceptedKind::Injured, *subject))
        },
        GameEvent::Rested { subject, .. } => Some((AcceptedKind::Rested, *subject)),
        GameEvent::Took { subject, .. } => Some((AcceptedKind::Took, *subject)),
        GameEvent::ItemAttached { subject, .. } => Some((AcceptedKind::ItemAttached, *subject)),
        GameEvent::MotionAdvanced { subject, .. } => Some((AcceptedKind::MotionAdvanced, *subject)),
        GameEvent::Moved {
            subject, from, to, ..
        } if from != to => Some((AcceptedKind::Moved, *subject)),
        GameEvent::Died { subject, .. } => Some((AcceptedKind::Died, *subject)),
        _ => None,
    }
}

/// The accepting tick, as the kernel's caller-supplied evidence time.
fn tick_of(event: &GameEvent) -> u64 {
    match event {
        GameEvent::AnatomyReconciled { tick, .. }
        | GameEvent::ItemAttached { tick, .. }
        | GameEvent::ItemDetached { tick, .. }
        | GameEvent::ItemReleased { tick, .. }
        | GameEvent::AnatomyAdmitted { tick, .. }
        | GameEvent::Generated { tick, .. }
        | GameEvent::Named { tick, .. }
        | GameEvent::Moved { tick, .. }
        | GameEvent::Held { tick, .. }
        | GameEvent::Observed { tick, .. }
        | GameEvent::Took { tick, .. }
        | GameEvent::Ate { tick, .. }
        | GameEvent::Rested { tick, .. }
        | GameEvent::Injured { tick, .. }
        | GameEvent::Waited { tick, .. }
        | GameEvent::Died { tick, .. }
        | GameEvent::VolleyResolved { tick, .. }
        | GameEvent::MotionAdvanced { tick, .. }
        | GameEvent::MovementProfileConfigured { tick, .. }
        | GameEvent::CanonRevised { tick, .. } => tick.0,
    }
}

#[cfg(test)]
#[path = "glyphs/tests.rs"]
mod tests;
