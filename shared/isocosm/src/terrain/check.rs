// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The spine's checks, shared by the tests and the bench's draws: SP1's
//! profiles and corners, SP2's borders, means and relief. Each takes the
//! function it checks, so a deliberately broken one can be shown to fail.

use super::{EdgeProfile, Lattice, View};
use crate::{Result, schema::Id};
use std::collections::BTreeMap;

/// Every pair of neighbours gives the same exact surface at every base-grain
/// point of their shared border. Returns how many border sides were compared.
pub fn borders(view: &View<'_>, lattice: impl Fn(&View<'_>, Id) -> Result<Lattice>) -> Result<u64> {
    let mut lattices = BTreeMap::new();
    for &site in view.sites.keys() {
        lattices.insert(site, lattice(view, site)?);
    }
    let side = view.footprint.side;
    let mut checked = 0;
    for (&site, s) in view.sites {
        for route in &s.routes {
            let Some(b) = route.border else { continue };
            let (near, far) = (&lattices[&site], &lattices[&route.to]);
            for t in 0..=side {
                let across = if b.flipped { t } else { side - t };
                if near.on_side(b.side, t) != far.on_side(b.enters, across) {
                    return Err(format!(
                        "site {site} side {} parts from its neighbour {t} along",
                        b.side
                    ));
                }
            }
            checked += 1;
        }
    }
    Ok(checked)
}

/// Each named site's exact surface, summed column by column over its
/// base-grain columns, is its elevation times their count: the mean restricts
/// to the skeleton. Brute force on purpose, so it tests the closed form.
pub fn means(
    view: &View<'_>,
    sites: &[Id],
    lattice: impl Fn(&View<'_>, Id) -> Result<Lattice>,
) -> Result<u64> {
    let side = view.footprint.side;
    for &site in sites {
        let lattice = lattice(view, site)?;
        let mut sum = 0i128;
        for z in 0..side {
            for x in 0..side {
                sum += lattice.numerator(x, z);
            }
        }
        let columns = i128::from(side) * i128::from(side);
        if sum != i128::from(view.elevation(site)?) * columns * lattice.denominator() {
            return Err(format!("site {site}'s mean surface is not its elevation"));
        }
    }
    Ok(sites.len() as u64)
}

/// Every lattice point's detail stays within its site's relief. Returns how
/// many sites were checked.
pub fn reliefs(view: &View<'_>, lattice: impl Fn(&View<'_>, Id) -> Result<Lattice>) -> Result<u64> {
    for &site in view.sites.keys() {
        let relief = view.relief(site)?;
        if lattice(view, site)?
            .detail
            .iter()
            .flatten()
            .any(|d| d.abs() > relief)
        {
            return Err(format!("site {site}'s detail exceeds its relief"));
        }
    }
    Ok(view.sites.len() as u64)
}

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
