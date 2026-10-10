// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! In-site space (wing rulings 670, 696, 698 and 701).
//!
//! A product keeps its world map, each site's skeleton and every edit as
//! facts of its own, and reads them across through [`Atlas`]. What lives
//! here is derived from those facts and may be discarded: a site lifted into
//! columns ([`lift`]), edits applied as shape operations ([`edit`]), the
//! volume they realize in `Ground` ([`volume`]), and the places, passages
//! and travel cost over its bricks ([`places`]).
//!
//! Integers throughout, so every derivation is the same bytes on every
//! machine. Coordinates are base units in a site's own frame: `x` and `z`
//! across its footprint from corner 0, `y` up.

pub mod border;
pub mod edit;
#[cfg(test)]
mod fixture;
pub mod lift;
pub mod places;
pub mod volume;

use serde::{Deserialize, Serialize};

pub use edit::{Edit, Op, Shape};
pub use lift::{CHUNK, Chunk, CornerKey, EdgeProfile, Lattice, Materials, SPANS, fading};

/// A site, as the product numbers it.
pub type SiteId = u64;

pub type Result<T> = std::result::Result<T, String>;

/// A site's outline in its own frame: its sides, and each side's length in
/// base units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    pub sides: u8,
    pub side: u64,
}

/// One side of a site meeting one side of a neighbour. Side `k` runs from
/// corner `k` to corner `k + 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Border {
    /// The side of this site.
    pub side: u8,
    /// The side of the neighbour it meets.
    pub enters: u8,
    /// The sides run the same way rather than the usual opposite one.
    pub flipped: bool,
}

/// What the lift reads of a product's world: its sites, how they border,
/// and each one's skeleton. Every derivation in this crate is a function of
/// these answers, so a product that answers the same lifts the same bytes.
pub trait Atlas {
    /// The world's seed, which keys corners and borders.
    fn seed(&self) -> u64;
    fn footprint(&self) -> Footprint;
    /// Every site, ascending.
    fn sites(&self) -> Vec<SiteId>;
    /// The seed of a site's own interior detail.
    fn terrain_seed(&self, site: SiteId) -> Result<u64>;
    /// The skeleton, in base units.
    fn elevation(&self, site: SiteId) -> Result<i64>;
    fn relief(&self, site: SiteId) -> Result<i64>;
    fn water(&self, site: SiteId) -> Result<i64>;
    /// The neighbour across `side` of `site`, and the border between them.
    fn border(&self, site: SiteId, side: u8) -> Option<(SiteId, Border)>;
    /// The product's ids for what a column holds.
    fn materials(&self) -> Result<Materials>;

    /// Every slot naming the same point as corner `corner` of `site`.
    fn corner_class(&self, site: SiteId, corner: u8) -> Vec<CornerKey> {
        lift::corner_class(self, site, corner)
    }

    /// The corner's one height, whichever slot asks.
    fn corner_height(&self, site: SiteId, corner: u8) -> Result<i64> {
        lift::corner_height(self, site, corner)
    }

    /// The profile of the border on `side` of `site`: the same bytes asked
    /// from either side.
    fn edge_profile(&self, site: SiteId, side: u8) -> Result<EdgeProfile> {
        lift::edge_profile(self, site, side)
    }

    /// The border's height `t` base units along `side` of `site`.
    fn border_height(&self, site: SiteId, side: u8, t: u64) -> Result<i64> {
        lift::border_height(self, site, side, t)
    }

    /// A site's lattice, fading and corrected (rulings 400 and 401).
    fn lattice(&self, site: SiteId) -> Result<Lattice> {
        lift::lattice_with(self, site, fading, true)
    }

    /// A lattice with its detail shaped by `window` and the correction on or
    /// off, so the checks' controls can break either.
    fn lattice_with(
        &self,
        site: SiteId,
        window: impl Fn(u32) -> i64,
        corrected: bool,
    ) -> Result<Lattice>
    where
        Self: Sized,
    {
        lift::lattice_with(self, site, window, corrected)
    }

    /// Chunks along each side of a site at `level`.
    fn chunks(&self, level: u8) -> u32 {
        lift::chunks(self, level)
    }

    /// Chunk `at` of `site` at `level`, unedited (ruling 402).
    fn lift(&self, site: SiteId, level: u8, at: [u32; 2]) -> Result<Chunk> {
        lift::chunk(self, site, level, at)
    }

    /// The profile's spans along `side` of `site`, each passing, climbing
    /// or stopping for a walker under `climb`.
    fn spans(&self, site: SiteId, side: u8, climb: places::Climb) -> Result<Vec<border::Span>> {
        border::spans(self, site, side, climb)
    }

    /// The same chunk with `edits` replayed onto it in order: every edit
    /// made in this site and every one a neighbour made that reaches across
    /// a border (rulings 413 to 415). `edits` are `(site made in, edit)`.
    fn lift_edited(
        &self,
        site: SiteId,
        level: u8,
        at: [u32; 2],
        edits: &[(SiteId, Edit)],
    ) -> Result<Chunk>
    where
        Self: Sized,
    {
        edit::lift_edited(self, site, level, at, edits)
    }
}

/// Domain-separated deterministic draws, independent of visitation order.
/// The sim's own draw, byte for byte, so the lift kept its bytes when it
/// moved here.
pub fn draw(seed: u64, domain: &str, values: &[u64]) -> u64 {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(seed.to_le_bytes());
    hash.update((domain.len() as u64).to_le_bytes());
    hash.update(domain.as_bytes());
    for value in values {
        hash.update(value.to_le_bytes());
    }
    u64::from_le_bytes(hash.finalize()[..8].try_into().unwrap())
}
