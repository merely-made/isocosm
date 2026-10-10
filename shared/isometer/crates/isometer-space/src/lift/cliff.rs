// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Cliffs along borders (ruling 744): a span of a border may be drawn a
//! cliff, from both sites' skeletons and the pair's seed, and the lower
//! site's surface steps down there by at least [`CLIFF`], climbing back
//! over a ramp an eighth of a side deep. The shared profile still bounds
//! both lattices; only the dropping site's columns step, so the step is
//! between the two sites' edge columns. The end spans of a side never
//! step, so no corner and no other border moves.

use super::{SPANS, canonical};
use crate::{Atlas, Result, SiteId};
use serde::{Deserialize, Serialize};

/// The least drop a drawn cliff makes, in base units: as high as places
/// look for a step, so no walker steps it.
pub const CLIFF: i64 = 64;

/// A span of one of this site's sides where its surface steps down.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cliff {
    pub side: u8,
    /// The span, counted in this site's own direction along the side.
    pub span: u32,
    pub drop: i64,
}

/// Every cliff span along `side` of `site`, in its own direction, with
/// whether `site` is the side that drops.
pub fn cliff_spans<A: Atlas + ?Sized>(a: &A, site: SiteId, side: u8) -> Result<Vec<(Cliff, bool)>> {
    let Some((to, b)) = a.border(site, side) else {
        return Ok(Vec::new());
    };
    let (cs, ck) = canonical(a, site, side)?;
    let forward = b.flipped || (cs, ck) == (site, side);
    let length = a.footprint().side as i64;
    let (ea, eb) = (a.elevation(site)?, a.elevation(to)?);
    let reliefs = a.relief(site)? + a.relief(to)?;
    let chance = ((reliefs + ea.abs_diff(eb) as i64) * 1024 / length).clamp(0, 512) as u64;
    let lower = if ea != eb { if ea < eb { site } else { to } } else if cs == site { to } else { site };
    let mut out = Vec::new();
    for k in 1..SPANS - 1 {
        let r = crate::draw(a.seed(), "terrain:cliff", &[cs, ck.into(), k.into()]);
        if r % 1024 < chance {
            // Past CLIFF by the pair's relief, so the slope either side of
            // the border cannot bring the step back under it.
            let reliefs = reliefs.max(0);
            let drop = CLIFF + reliefs + 2 + ((r >> 10) % (reliefs as u64 + 1)) as i64;
            let span = if forward { k } else { SPANS - 1 - k };
            out.push((Cliff { side, span, drop }, lower == site));
        }
    }
    Ok(out)
}

/// The cliffs whose drop this site's surface takes.
pub(crate) fn drops<A: Atlas + ?Sized>(a: &A, site: SiteId) -> Result<Vec<Cliff>> {
    let mut out = Vec::new();
    for side in 0..a.footprint().sides {
        out.extend(cliff_spans(a, site, side)?.into_iter().filter(|c| c.1).map(|c| c.0));
    }
    Ok(out)
}

/// How deep a cliff's ramp runs into its site.
pub(crate) fn depth(length: i64) -> i64 {
    (length / 8).max(4)
}

impl Cliff {
    /// The drop at point `(x, z)` of a site `length` a side: the full drop
    /// on the border and one cell in, then easing to nothing at the ramp's
    /// depth, strictly inside the span.
    pub fn at(&self, x: i64, z: i64, length: i64) -> i64 {
        let (c, d, n) = crate::edit::frame(self.side, length);
        let rel = [x - c[0], z - c[1]];
        let t = rel[0] * d[0] + rel[1] * d[1];
        let inward = rel[0] * n[0] + rel[1] * n[1];
        let (len, deep) = (length / i64::from(SPANS), depth(length));
        let span = i64::from(self.span);
        if t <= span * len || t >= (span + 1) * len || !(0..deep).contains(&inward) {
            return 0;
        }
        if inward <= 1 { self.drop } else { self.drop * (deep - inward) / (deep - 1) }
    }

    /// The drop summed over a site's base-grain columns.
    pub(crate) fn total(&self, length: i64) -> i128 {
        let (c, d, n) = crate::edit::frame(self.side, length);
        let len = length / i64::from(SPANS);
        let span = i64::from(self.span);
        let mut sum = 0i128;
        for t in span * len + 1..(span + 1) * len {
            for inward in 0..depth(length) {
                let [x, z] = [0, 1].map(|k| c[k] + t * d[k] + inward * n[k]);
                if (0..length).contains(&x) && (0..length).contains(&z) {
                    sum += i128::from(self.at(x, z, length));
                }
            }
        }
        sum
    }
}
