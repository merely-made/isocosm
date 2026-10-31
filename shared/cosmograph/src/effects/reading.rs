// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A subject's glyph journal over accepted acts: the world-independent half
//! of a game's glyph reading (moved from Eponym's legacy `glyphs`, behaviour
//! unchanged). The game says which accepted act is evidence for whom and
//! under what provenance; this holds the journey, the founding canon that
//! completion is judged against, and the live canon a published revision
//! moves. It references effect ids and never executes one.

use serde::Serialize;
use wing_glyphs::{
    Canon, CorrespondenceMove, Eligibility, GrantOutcome, Journey, Provenance, VariantPolicy,
};

/// What the kernel said about one grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphGrantOutcome {
    Acquired,
    Recorded,
    Duplicate,
    Rejected(String),
}

/// The journey with its founding and live canons.
pub struct Journal {
    canon: Canon,
    /// Equal to `canon` until a revision is read.
    live: Canon,
    journey: Journey,
}

impl Journal {
    /// Opens a journal for `individual` on an admitted canon.
    pub fn new(canon: Canon, individual: String, thresholds: Vec<u64>) -> Result<Self, String> {
        let journey = Journey::new(individual, &canon, thresholds)?;
        Ok(Self {
            live: canon.clone(),
            canon,
            journey,
        })
    }

    /// Grants `glyph` under the live revision, requiring an owned base for a
    /// variant; returns the outcome and the revision it was taken under.
    pub fn grant(
        &mut self,
        glyph: &str,
        provenance: Provenance,
        tick: u64,
    ) -> (GlyphGrantOutcome, u64) {
        let revision = self.live.spec().revision;
        let policy = VariantPolicy::RequireOwnedBase;
        let outcome = match self
            .journey
            .grant_at_revision(glyph, provenance, tick, policy, revision)
        {
            Ok(GrantOutcome::Acquired { .. }) => GlyphGrantOutcome::Acquired,
            Ok(GrantOutcome::Recorded { .. }) => GlyphGrantOutcome::Recorded,
            Ok(GrantOutcome::Duplicate { .. }) => GlyphGrantOutcome::Duplicate,
            Err(why) => GlyphGrantOutcome::Rejected(why),
        };
        (outcome, revision)
    }

    /// Follows a published revision, derived from the founding canon so the
    /// live correspondence is a function of seed and revision alone. False
    /// when it is not newer than the live one, or does not derive.
    pub fn revise(&mut self, revision: u64, seed: u64) -> bool {
        if revision <= self.live.spec().revision {
            return false;
        }
        match self.canon.shuffled(seed, revision) {
            Ok(live) => {
                self.live = live;
                true
            },
            Err(_) => false,
        }
    }

    /// The founding correspondence: what completion and the ascension basis
    /// are judged against, whatever the world publishes later.
    pub fn canon(&self) -> &Canon {
        &self.canon
    }
    /// The published correspondence the journal answers from now.
    pub fn live_canon(&self) -> &Canon {
        &self.live
    }
    /// What this glyph meant when the journey was founded.
    pub fn founding_effect(&self, glyph: &str) -> Option<&str> {
        self.canon.effect(glyph)
    }
    /// What this glyph means under the live revision.
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
    pub fn eligibility(&self) -> Eligibility {
        self.journey.eligibility()
    }
}
