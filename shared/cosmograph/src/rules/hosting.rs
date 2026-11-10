// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rule of hosting (cases 1223, 1225, 1226): what a surface does
//! where it lives. Unset, a surface only holds matter and hashes as before.

use crate::{Result, rules::Rules, schema::{Key, Tick}};
use serde::{Deserialize, Serialize};

/// What a surface does where it lives (cases 1223, 1225, 1226): each named
/// trait is a gene a strain may carry, switched on or off.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hosting {
    /// Ticks between a surface's turns.
    pub period: Tick,
    /// With this trait a surface on a site takes the site's dead matter.
    pub rot: Key,
    /// With this trait a surface in a host takes the host's tissue.
    pub feed: Key,
    /// With this trait a surface in a host gives the host matter.
    pub leak: Key,
    /// With this trait a surface on a site enters the bodies residing there.
    pub enter: Key,
    /// The share (numerator, denominator) of its own matter a surface takes,
    /// gives or sends in one turn.
    pub take: [u32; 2],
    pub give: [u32; 2],
    pub send: [u32; 2],
}

impl Default for Hosting {
    /// A first cut, to be tuned.
    fn default() -> Self {
        Self {
            // A turn every six ticks.
            period: 6,
            rot: "gene:rot".into(),
            feed: "gene:feed".into(),
            leak: "gene:leak".into(),
            enter: "gene:enter".into(),
            // A tenth taken, a twentieth given, a tenth sent.
            take: [1, 10],
            give: [1, 20],
            send: [1, 10],
        }
    }
}

impl Hosting {
    /// # Errors
    /// Fails if the period is nought, a share has no denominator or is more
    /// than whole, or a gene is not a trait the world declares.
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        if self.period == 0 {
            return Err("hosting's period must last some ticks".into());
        }
        if [self.take, self.give, self.send].iter().any(|[n, d]| *d == 0 || n > d) {
            return Err("hosting's shares need denominators and cannot exceed a whole".into());
        }
        if [&self.rot, &self.feed, &self.leak, &self.enter].iter().any(|key| !rules.traits.contains(*key)) {
            return Err("hosting's genes must be traits the world declares".into());
        }
        Ok(())
    }
}
