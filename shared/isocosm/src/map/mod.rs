// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world map's geometry: sites laid in the world's one shape, each side
//! of a site meeting one side of a neighbour (rulings 72, 73, 395 and 397).

mod grid;

pub use grid::{Grid, SHAPES};

use crate::{
    Result,
    rules::Skeleton,
    schema::{Border, Footprint, Id, Key, Site},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a founding lays its world map (ruling 398). Square grids come first
/// (ruling 395); polygon layouts join as further variants.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layout {
    Grid(Grid),
}

/// What a layout lays: sites with their borders and skeletons, the
/// conditions the skeleton declares, and the world's outline and shape.
pub(crate) struct Laid {
    pub sites: BTreeMap<Id, Site>,
    pub conditions: BTreeSet<Key>,
    pub skeleton: Skeleton,
    pub footprint: Footprint,
    pub shape: Key,
}

impl Layout {
    pub(crate) fn lay(&self, seed: u64, sites: u32) -> Result<Laid> {
        match self {
            Layout::Grid(grid) => grid.lay(seed, sites),
        }
    }
}

/// The skeleton's default condition keys (ruling 394).
pub fn default_skeleton() -> Skeleton {
    Skeleton {
        elevation: "terrain:elevation".into(),
        relief: "terrain:relief".into(),
        water: "terrain:water".into(),
    }
}

/// Borders only where the world has an outline, each within it, each side
/// used once and each met by its reverse; skeletons declared and present.
pub(crate) fn validate(g: &Genesis) -> Result<()> {
    let bordered = g
        .sites
        .values()
        .flat_map(|s| &s.routes)
        .any(|r| r.border.is_some());
    match g.world.footprint {
        None if bordered => return Err("a border on a world without a footprint".into()),
        None => {},
        Some(f) => {
            if !(3..=12).contains(&f.sides) || f.side < u64::from(crate::terrain::SPANS) {
                return Err("invalid site footprint".into());
            }
            for (&id, site) in &g.sites {
                let mut used = BTreeSet::new();
                for route in &site.routes {
                    let Some(b) = route.border else { continue };
                    if b.side >= f.sides || b.enters >= f.sides {
                        return Err("a border outside its footprint".into());
                    }
                    if !used.insert(b.side) {
                        return Err("a site side with two borders".into());
                    }
                    let reverse = Border {
                        side: b.enters,
                        enters: b.side,
                        flipped: b.flipped,
                    };
                    let met = g.sites.get(&route.to).is_some_and(|to| {
                        to.routes
                            .iter()
                            .any(|r| r.to == id && r.border == Some(reverse))
                    });
                    if !met {
                        return Err("a border without its reverse".into());
                    }
                }
            }
        },
    }
    if let Some(skeleton) = &g.rules.skeleton {
        for key in [&skeleton.elevation, &skeleton.relief, &skeleton.water] {
            if !g.rules.conditions.contains(key) {
                return Err("an undeclared skeleton condition".into());
            }
            if g.sites.values().any(|s| !s.conditions.contains_key(key)) {
                return Err("a site without its skeleton".into());
            }
        }
        if g.sites.values().any(|s| s.conditions[&skeleton.relief] < 0) {
            return Err("a site with negative relief".into());
        }
    }
    Ok(())
}
