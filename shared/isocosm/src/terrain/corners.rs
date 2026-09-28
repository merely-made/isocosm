// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A corner is one point shared by every site around it, so it is found by
//! walking borders and drawn once, keyed by its least slot.

use super::{View, signed};
use crate::{Result, schema::Id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Corner `corner` of site `site`: where that site's side `corner` starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CornerKey {
    pub site: Id,
    pub corner: u8,
}

impl View<'_> {
    /// Every slot naming the same point as corner `corner` of `site`, least
    /// first. The walk goes forward across the side the corner starts and
    /// back across the side it ends, until it closes or meets an open edge.
    pub fn corner_class(&self, site: Id, corner: u8) -> Vec<CornerKey> {
        let n = self.footprint.sides;
        let start = CornerKey { site, corner };
        let mut class = BTreeSet::from([start]);
        let mut at = start;
        while let Some((to, b)) = self.border(at.site, at.corner) {
            let corner = if b.flipped {
                b.enters
            } else {
                (b.enters + 1) % n
            };
            if !class.insert(CornerKey { site: to, corner }) {
                break;
            }
            at = CornerKey { site: to, corner };
        }
        let mut at = start;
        while let Some((to, b)) = self.border(at.site, (at.corner + n - 1) % n) {
            let corner = if b.flipped {
                (b.enters + 1) % n
            } else {
                b.enters
            };
            if !class.insert(CornerKey { site: to, corner }) {
                break;
            }
            at = CornerKey { site: to, corner };
        }
        class.into_iter().collect()
    }

    /// The corner's height: the mean elevation of the slots around it, and a
    /// draw keyed by its least slot within half their mean relief.
    pub fn corner_height(&self, site: Id, corner: u8) -> Result<i64> {
        let class = self.corner_class(site, corner);
        let (mut elevation, mut relief) = (0i64, 0i64);
        for slot in &class {
            elevation += self.elevation(slot.site)?;
            relief += self.relief(slot.site)?;
        }
        let n = class.len() as i64;
        let key = class[0];
        let draw = crate::draw(self.seed, "terrain:corner", &[key.site, key.corner.into()]);
        Ok(elevation / n + signed(draw, relief / n / 2))
    }
}
