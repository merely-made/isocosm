// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The terrain models the lift reads (rulings 391 and 393): corners shared
//! by every site around them, and edge profiles both sides compute alike.
//! Integers throughout, like every authoritative value in the sim.

pub mod check;
mod corners;
mod profile;

pub use corners::CornerKey;
pub use profile::{EdgeProfile, SPANS};

use crate::{
    Result,
    rules::Skeleton,
    schema::{Border, Footprint, Id, Key, Site},
    simulation::Genesis,
};
use std::collections::BTreeMap;

/// A world's sites read as terrain.
#[derive(Clone, Copy)]
pub struct View<'a> {
    pub seed: u64,
    pub sites: &'a BTreeMap<Id, Site>,
    pub footprint: Footprint,
    pub skeleton: &'a Skeleton,
}

impl<'a> View<'a> {
    /// A founded world's terrain, when it has an outline and a skeleton.
    pub fn of(genesis: &'a Genesis) -> Result<Self> {
        Ok(Self {
            seed: genesis.seed,
            sites: &genesis.sites,
            footprint: genesis
                .world
                .footprint
                .ok_or("a world without a footprint")?,
            skeleton: genesis
                .rules
                .skeleton
                .as_ref()
                .ok_or("a world without a skeleton")?,
        })
    }

    fn condition(&self, site: Id, key: &Key) -> Result<i64> {
        self.sites
            .get(&site)
            .and_then(|s| s.conditions.get(key))
            .copied()
            .ok_or_else(|| "a site without its skeleton".into())
    }

    pub fn elevation(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.elevation)
    }

    pub fn relief(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.relief)
    }

    pub fn water(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.water)
    }

    /// The neighbour across `side` of `site`, and the border between them.
    pub fn border(&self, site: Id, side: u8) -> Option<(Id, Border)> {
        self.sites
            .get(&site)?
            .routes
            .iter()
            .find_map(|r| r.border.filter(|b| b.side == side).map(|b| (r.to, b)))
    }
}

/// A draw spread over `-amplitude..=amplitude`.
fn signed(draw: u64, amplitude: i64) -> i64 {
    if amplitude <= 0 {
        return 0;
    }
    (draw % (2 * amplitude as u64 + 1)) as i64 - amplitude
}
