// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The lift's checks, shared by the tests and the bench's draws: SP1's
//! profiles and corners, SP2's borders, means and relief. Each takes the
//! function it checks, so a deliberately broken one can be shown to fail.

use super::{EdgeProfile, Lattice};
use crate::{Atlas, Border, Result, SiteId};
use std::collections::BTreeMap;

/// Every bordered side of every site, with its neighbour.
fn bordered<A: Atlas>(a: &A) -> Vec<(SiteId, SiteId, Border)> {
    let mut out = Vec::new();
    for site in a.sites() {
        for side in 0..a.footprint().sides {
            if let Some((to, b)) = a.border(site, side) {
                out.push((site, to, b));
            }
        }
    }
    out
}

/// Every pair of neighbours gives the same exact surface at every base-grain
/// point of their shared border. Returns how many border sides were compared.
pub fn borders<A: Atlas>(a: &A, lattice: impl Fn(&A, SiteId) -> Result<Lattice>) -> Result<u64> {
    let mut lattices = BTreeMap::new();
    for site in a.sites() {
        lattices.insert(site, lattice(a, site)?);
    }
    let side = a.footprint().side;
    let mut checked = 0;
    for (site, to, b) in bordered(a) {
        let (near, far) = (&lattices[&site], &lattices[&to]);
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
    Ok(checked)
}

/// Each named site's exact surface, summed column by column over its
/// base-grain columns, is its elevation times their count: the mean restricts
/// to the skeleton. Brute force on purpose, so it tests the closed form.
pub fn means<A: Atlas>(
    a: &A,
    sites: &[SiteId],
    lattice: impl Fn(&A, SiteId) -> Result<Lattice>,
) -> Result<u64> {
    let side = a.footprint().side;
    for &site in sites {
        let lattice = lattice(a, site)?;
        let mut sum = 0i128;
        for z in 0..side {
            for x in 0..side {
                sum += lattice.numerator(x, z);
            }
        }
        let columns = i128::from(side) * i128::from(side);
        if sum != i128::from(a.elevation(site)?) * columns * lattice.denominator() {
            return Err(format!("site {site}'s mean surface is not its elevation"));
        }
    }
    Ok(sites.len() as u64)
}

/// Every lattice point's detail stays within its site's relief. Returns how
/// many sites were checked.
pub fn reliefs<A: Atlas>(a: &A, lattice: impl Fn(&A, SiteId) -> Result<Lattice>) -> Result<u64> {
    let sites = a.sites();
    for &site in &sites {
        let relief = a.relief(site)?;
        if lattice(a, site)?
            .detail
            .iter()
            .flatten()
            .any(|d| d.abs() > relief)
        {
            return Err(format!("site {site}'s detail exceeds its relief"));
        }
    }
    Ok(sites.len() as u64)
}

/// Every border's profile is the same bytes asked from either side. Returns
/// how many border sides were compared.
pub fn profiles<A: Atlas>(
    a: &A,
    profile: impl Fn(&A, SiteId, u8) -> Result<EdgeProfile>,
) -> Result<u64> {
    let bytes = |p: EdgeProfile| postcard::to_allocvec(&p).map_err(|e| e.to_string());
    let mut checked = 0;
    for (site, to, b) in bordered(a) {
        if bytes(profile(a, site, b.side)?)? != bytes(profile(a, to, b.enters)?)? {
            return Err(format!(
                "site {site} side {} reads differently from its far side",
                b.side
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

/// Every corner has one height whichever slot asks, and every border starts
/// and ends at its corners' heights. Returns how many corners were checked.
pub fn corners<A: Atlas>(a: &A, height: impl Fn(&A, SiteId, u8) -> Result<i64>) -> Result<u64> {
    let footprint = a.footprint();
    let n = footprint.sides;
    let mut checked = 0;
    for site in a.sites() {
        for corner in 0..n {
            let own = height(a, site, corner)?;
            for slot in a.corner_class(site, corner) {
                if height(a, slot.site, slot.corner)? != own {
                    return Err(format!(
                        "site {site} corner {corner} disagrees with site {} corner {}",
                        slot.site, slot.corner
                    ));
                }
            }
            if a.border(site, corner).is_some() {
                let next = height(a, site, (corner + 1) % n)?;
                if a.border_height(site, corner, 0)? != own
                    || a.border_height(site, corner, footprint.side)? != next
                {
                    return Err(format!("site {site} side {corner} misses its corners"));
                }
            }
            checked += 1;
        }
    }
    Ok(checked)
}
