// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An edge profile is the shared border of two sites, drawn once from the
//! lesser of its two sides, so both compute the same bytes (ruling 391).

use super::{CornerKey, View, signed};
use crate::{Result, schema::Id};
use serde::{Deserialize, Serialize};

/// The spans a profile is drawn in; its detail vanishes at both ends.
pub const SPANS: u32 = 16;

/// A border's surface from its start corner to its end corner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeProfile {
    /// The corner it starts from and the one it ends at, by least slot.
    pub start: CornerKey,
    pub end: CornerKey,
    /// The border's length, in base units.
    pub length: u64,
    /// Heights at `SPANS + 1` evenly spaced points, start to end.
    pub heights: Vec<i64>,
}

impl EdgeProfile {
    /// The height `t` base units from the start, `t` clamped to the border.
    pub fn at(&self, t: u64) -> i64 {
        let spans = u64::from(SPANS);
        let scaled = t.min(self.length) * spans;
        let i = (scaled / self.length).min(spans - 1);
        let (a, b) = (self.heights[i as usize], self.heights[i as usize + 1]);
        a + (b - a) * (scaled - i * self.length) as i64 / self.length as i64
    }
}

impl View<'_> {
    /// The lesser of a border's two sides, which draws its profile.
    pub(super) fn canonical(&self, site: Id, side: u8) -> Result<(Id, u8)> {
        let (to, b) = self.border(site, side).ok_or("a side without a border")?;
        Ok((site, side).min((to, b.enters)))
    }

    /// The profile of the border on `side` of `site`: the same bytes asked
    /// from either side.
    pub fn edge_profile(&self, site: Id, side: u8) -> Result<EdgeProfile> {
        let (site, side) = self.canonical(site, side)?;
        let (to, _) = self.border(site, side).ok_or("a side without a border")?;
        let next = (side + 1) % self.footprint.sides;
        let (h0, h1) = (
            self.corner_height(site, side)?,
            self.corner_height(site, next)?,
        );
        let amplitude = (self.relief(site)? + self.relief(to)?) / 4;
        let spans = i64::from(SPANS);
        let heights = (0..=SPANS)
            .map(|i| {
                let at = i64::from(i);
                let line = h0 + (h1 - h0) * at / spans;
                let draw = crate::draw(self.seed, "terrain:edge", &[site, side.into(), i.into()]);
                line + signed(draw, amplitude) * 4 * at * (spans - at) / (spans * spans)
            })
            .collect();
        Ok(EdgeProfile {
            start: self.corner_class(site, side)[0],
            end: self.corner_class(site, next)[0],
            length: self.footprint.side,
            heights,
        })
    }

    /// The border's height `t` base units along `side` of `site`, measured
    /// from that site's corner `side`. The far side runs the border the
    /// other way, unless the two sides meet flipped.
    pub fn border_height(&self, site: Id, side: u8, t: u64) -> Result<i64> {
        let profile = self.edge_profile(site, side)?;
        let (_, b) = self.border(site, side).ok_or("a side without a border")?;
        let forward = b.flipped || self.canonical(site, side)? == (site, side);
        let t = t.min(profile.length);
        Ok(profile.at(if forward { t } else { profile.length - t }))
    }
}
