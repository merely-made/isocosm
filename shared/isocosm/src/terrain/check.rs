// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP1's checks, shared by the tests and the bench's map draws. Each takes
//! the function it checks, so a deliberately broken one can be shown to fail.

use super::{EdgeProfile, View};
use crate::{Result, schema::Id};

/// Every border's profile is the same bytes asked from either side. Returns
/// how many border sides were compared.
pub fn profiles(
    view: &View<'_>,
    profile: impl Fn(&View<'_>, Id, u8) -> Result<EdgeProfile>,
) -> Result<u64> {
    let mut checked = 0;
    for (&site, s) in view.sites {
        for route in &s.routes {
            let Some(b) = route.border else { continue };
            let near =
                serde_json::to_vec(&profile(view, site, b.side)?).map_err(|e| e.to_string())?;
            let far = serde_json::to_vec(&profile(view, route.to, b.enters)?)
                .map_err(|e| e.to_string())?;
            if near != far {
                return Err(format!(
                    "site {site} side {} reads differently from its far side",
                    b.side
                ));
            }
            checked += 1;
        }
    }
    Ok(checked)
}

/// Every corner has one height whichever slot asks, and every border starts
/// and ends at its corners' heights. Returns how many corners were checked.
pub fn corners(view: &View<'_>, height: impl Fn(&View<'_>, Id, u8) -> Result<i64>) -> Result<u64> {
    let n = view.footprint.sides;
    let mut checked = 0;
    for &site in view.sites.keys() {
        for corner in 0..n {
            let own = height(view, site, corner)?;
            for slot in view.corner_class(site, corner) {
                if height(view, slot.site, slot.corner)? != own {
                    return Err(format!(
                        "site {site} corner {corner} disagrees with site {} corner {}",
                        slot.site, slot.corner
                    ));
                }
            }
            if view.border(site, corner).is_some() {
                let next = height(view, site, (corner + 1) % n)?;
                if view.border_height(site, corner, 0)? != own
                    || view.border_height(site, corner, view.footprint.side)? != next
                {
                    return Err(format!("site {site} side {corner} misses its corners"));
                }
            }
            checked += 1;
        }
    }
    Ok(checked)
}
