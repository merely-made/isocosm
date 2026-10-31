// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A founded world's sites, read across to isometer's lift. The world map,
//! each site's skeleton and the material table stay the sim's; lifting a
//! site into voxels is isometer's (ruling 701), through `Atlas`.

use crate::{
    Result,
    rules::Skeleton,
    schema::{Footprint, Id, Key, Material, Site},
    simulation::Genesis,
};
use isometer_space::{Atlas, Border, Materials};
use std::collections::BTreeMap;

/// A world's sites read as terrain.
#[derive(Clone, Copy)]
pub struct View<'a> {
    pub seed: u64,
    pub sites: &'a BTreeMap<Id, Site>,
    pub footprint: Footprint,
    pub skeleton: &'a Skeleton,
    pub materials: &'a [Material],
}

impl<'a> View<'a> {
    /// A founded world's terrain, when it has an outline and a skeleton.
    pub fn of(genesis: &'a Genesis) -> Result<Self> {
        Self::over(genesis, &genesis.sites)
    }

    /// The same world with its sites as `sites` holds them, as a running
    /// state does.
    pub fn over(genesis: &'a Genesis, sites: &'a BTreeMap<Id, Site>) -> Result<Self> {
        Ok(Self {
            seed: genesis.seed,
            sites,
            footprint: genesis
                .world
                .footprint
                .ok_or("a world without a footprint")?,
            skeleton: genesis
                .rules
                .skeleton
                .as_ref()
                .ok_or("a world without a skeleton")?,
            materials: &genesis.world.materials,
        })
    }

    fn condition(&self, site: Id, key: &Key) -> Result<i64> {
        self.sites
            .get(&site)
            .and_then(|s| s.conditions.get(key))
            .copied()
            .ok_or_else(|| "a site without its skeleton".into())
    }

    /// The id the world's material table gives `key`.
    fn material(&self, key: &str) -> Result<u8> {
        self.materials
            .iter()
            .position(|m| m.key == key)
            .map(|id| id as u8)
            .ok_or_else(|| format!("a world without {key}"))
    }
}

impl Atlas for View<'_> {
    fn seed(&self) -> u64 {
        self.seed
    }

    fn footprint(&self) -> isometer_space::Footprint {
        isometer_space::Footprint {
            sides: self.footprint.sides,
            side: self.footprint.side,
        }
    }

    fn sites(&self) -> Vec<Id> {
        self.sites.keys().copied().collect()
    }

    fn terrain_seed(&self, site: Id) -> Result<u64> {
        Ok(self.sites.get(&site).ok_or("an absent site")?.terrain_seed)
    }

    fn elevation(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.elevation)
    }

    fn relief(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.relief)
    }

    fn water(&self, site: Id) -> Result<i64> {
        self.condition(site, &self.skeleton.water)
    }

    fn border(&self, site: Id, side: u8) -> Option<(Id, Border)> {
        self.sites.get(&site)?.routes.iter().find_map(|r| {
            let b = r.border.filter(|b| b.side == side)?;
            Some((
                r.to,
                Border {
                    side: b.side,
                    enters: b.enters,
                    flipped: b.flipped,
                },
            ))
        })
    }

    fn materials(&self) -> Result<Materials> {
        Ok(Materials {
            air: self.material("world:air")?,
            water: self.material("world:water")?,
            soil: self.material("world:soil")?,
            rock: self.material("world:rock")?,
        })
    }
}

#[cfg(test)]
mod tests {
    /// The lift kept its bytes when it moved only if isometer draws as the
    /// sim does.
    #[test]
    fn the_lift_draws_as_the_sim_does() {
        for seed in [0, 7, u64::MAX] {
            for domain in ["terrain:corner", "terrain:edge", "terrain:detail", ""] {
                let values = [seed, 3, 1 << 40];
                assert_eq!(
                    crate::draw(seed, domain, &values),
                    isometer_space::draw(seed, domain, &values)
                );
            }
        }
    }
}
