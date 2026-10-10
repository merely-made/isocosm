// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A corner is one point shared by every site around it, so it is found by
//! walking borders and drawn once, keyed by its least slot.

use super::signed;
use crate::{Atlas, Result, SiteId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Corner `corner` of site `site`: where that site's side `corner` starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CornerKey {
    pub site: SiteId,
    pub corner: u8,
}

/// Every slot naming the same point as corner `corner` of `site`, least
/// first. The walk goes forward across the side the corner starts and back
/// across the side it ends, until it closes or meets an open edge.
pub(crate) fn corner_class<A: Atlas + ?Sized>(a: &A, site: SiteId, corner: u8) -> Vec<CornerKey> {
    let n = a.footprint().sides;
    let start = CornerKey { site, corner };
    let mut class = BTreeSet::from([start]);
    let mut at = start;
    while let Some((to, b)) = a.border(at.site, at.corner) {
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
    while let Some((to, b)) = a.border(at.site, (at.corner + n - 1) % n) {
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
pub(crate) fn corner_height<A: Atlas + ?Sized>(a: &A, site: SiteId, corner: u8) -> Result<i64> {
    let class = corner_class(a, site, corner);
    let (mut elevation, mut relief) = (0i64, 0i64);
    for slot in &class {
        elevation += a.elevation(slot.site)?;
        relief += a.relief(slot.site)?;
    }
    let n = class.len() as i64;
    let key = class[0];
    let draw = crate::draw(a.seed(), "terrain:corner", &[key.site, key.corner.into()]);
    Ok(elevation / n + signed(draw, relief / n / 2))
}
