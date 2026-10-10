// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world map's geometry: sites laid in the world's one shape, each side
//! of a site meeting one side of a neighbour (rulings 72, 73, 395 and 397).

mod grid;

pub use grid::{Grid, SHAPES};

use crate::{
    Result,
    rules::Skeleton,
    schema::{Border, Footprint, Id, Key, Material, Site},
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
/// conditions the skeleton declares, the world's outline and shape, and
/// the materials its ground is made of.
pub(crate) struct Laid {
    pub sites: BTreeMap<Id, Site>,
    pub conditions: BTreeSet<Key>,
    pub skeleton: Skeleton,
    pub footprint: Footprint,
    pub shape: Key,
    pub materials: Vec<Material>,
}

impl Layout {
    /// Where `site` lies on the map: its column and row (ruling 783).
    pub fn at(&self, site: Id) -> Option<[u32; 2]> {
        match self {
            Layout::Grid(grid) => grid.at(site),
        }
    }

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

/// The materials a laid world's ground starts with (ruling 403): air, water,
/// soil and rock, all nis of the world itself. `world:soil` is the nis the
/// ledger's soil account names.
pub fn default_materials() -> Vec<Material> {
    ["world:air", "world:water", "world:soil", "world:rock"]
        .into_iter()
        .map(|key| Material {
            key: key.into(),
            lineage: "world:ground".into(),
            density: None,
            account: None,
        })
        .collect()
}

/// Borders only where the world has an outline, each within it, each side
/// used once and each met by its reverse; skeletons declared and present;
/// materials named once each, of lineages the world has.
pub(crate) fn validate(g: &Genesis) -> Result<()> {
    let mut named = BTreeSet::new();
    for material in &g.world.materials {
        if !named.insert(&material.key) || !g.lineages.contains_key(&material.lineage) {
            return Err("a material named twice or of an absent lineage".into());
        }
    }
    for m in &g.world.materials {
        let matter = |k: &Key| {
            matches!(g.rules.accounts.get(k), Some(crate::rules::AccountKind::Matter { .. }))
        };
        match (&m.density, &m.account) {
            (None, None) => {},
            (Some(d), Some(k)) if *d > 0 && matter(k) => {},
            _ => return Err("a material's density and matter account set apart".into()),
        }
    }
    if g.world.materials.len() > 256 {
        return Err("more materials than a voxel can name".into());
    }
    let bordered = g
        .sites
        .values()
        .flat_map(|s| &s.routes)
        .any(|r| r.border.is_some());
    match g.world.footprint {
        None if bordered => return Err("a border on a world without a footprint".into()),
        None => {},
        Some(f) => {
            if !(3..=12).contains(&f.sides) || f.side < u64::from(isometer_space::SPANS) {
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

/// Where `site` lies on the world's map, when its founding laid one.
pub fn position(genesis: &Genesis, site: Id) -> Option<[u32; 2]> {
    genesis.founding.as_ref()?.map.as_ref()?.at(site)
}
