// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A microbial surface owns local biomass. A world-class founder owns a
//! connected footprint in the generated terrain. Overlapping footprints add.
use super::FoundingForm;
use crate::{Founding, Result, draw, simulation::Genesis, spread::Identity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Signed changes in terrain units at the core, tapering along site borders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldBody {
    pub reach: u8,
    pub elevation: i8,
    pub relief: u8,
    pub water: i8,
}

impl WorldBody {
    fn from_recipe(recipe: u64) -> Self {
        Self {
            reach: (recipe % 5) as u8,
            elevation: ((recipe / 5 % 7) as i8 - 3) * 2,
            relief: (recipe / 35 % 4) as u8,
            water: ((recipe / 140 % 5) as i8 - 2) * 2,
        }
    }

    /// Actual elevation, relief and water contributions at this point in the footprint.
    pub fn contribution(self, remaining: i64) -> [i64; 3] {
        let extent = i64::from(self.reach) + 1;
        [self.elevation as i64, self.relief as i64, self.water as i64]
            .map(|n| n * remaining.clamp(0, extent) / extent)
    }

    fn apply(self, g: &mut Genesis, at: u64, index: u32) -> Result<()> {
        if self.reach > 32
            || self.relief > 64
            || self.elevation.unsigned_abs() > 64
            || self.water.unsigned_abs() > 64
        {
            return Err(format!(
                "world-class lineage {index} exceeds terrain growth bounds"
            ));
        }
        let skeleton = g
            .rules
            .skeleton
            .get_or_insert_with(crate::map::default_skeleton)
            .clone();
        let keys = [&skeleton.elevation, &skeleton.relief, &skeleton.water];
        for key in keys {
            g.rules.conditions.insert(key.clone());
            for site in g.sites.values_mut() {
                site.conditions.entry(key.clone()).or_insert(0);
            }
        }
        let marker = format!("habitat:body-{index}");
        g.rules.conditions.insert(marker.clone());
        let extent = i64::from(self.reach) + 1;
        let mut visited = BTreeSet::from([at]);
        let mut frontier = VecDeque::from([(at, 0)]);
        while let Some((id, distance)) = frontier.pop_front() {
            let site = g.sites.get_mut(&id).unwrap();
            let share = extent - distance;
            site.conditions.insert(marker.clone(), share);
            for (key, n) in keys.into_iter().zip(self.contribution(share)) {
                *site.conditions.get_mut(key).unwrap() += n;
            }
            if share > 1 {
                for r in &site.routes {
                    if visited.insert(r.to) {
                        frontier.push_back((r.to, distance + 1));
                    }
                }
            }
        }
        Ok(())
    }
}

fn recipes(seed: u64, kind: FoundingForm) -> Vec<u64> {
    let mut recipes: Vec<_> = if kind == FoundingForm::Microbe {
        (0..511).collect()
    } else {
        (0..700)
            .filter(|&n| {
                let p = WorldBody::from_recipe(n);
                p.elevation != 0 || p.relief != 0 || p.water != 0
            })
            .collect()
    };
    recipes.sort_by_cached_key(|n| (draw(seed, "habitat-recipe", &[kind as u64, *n]), *n));
    recipes
}

impl Founding {
    /// World-class recipes and their numbered lineage keys as they appear in this draw.
    pub fn world_body_recipes(&self) -> Result<BTreeMap<String, WorldBody>> {
        let mut out = BTreeMap::new();
        let Some(rep) = &self.repertoire else {
            return Ok(out);
        };
        let Some(dist) = &rep.kingdom_draw else {
            return Ok(out);
        };
        let mut first = 0;
        for (kind, n) in dist.draw(self.seed)? {
            if kind == FoundingForm::WorldClass {
                for (slot, r) in recipes(self.seed, kind)
                    .into_iter()
                    .take(n as usize)
                    .enumerate()
                {
                    let body = rep
                        .world_bodies
                        .get(&slot.to_string())
                        .copied()
                        .unwrap_or_else(|| WorldBody::from_recipe(r));
                    out.insert(format!("lineage:{}", first + slot as u32), body);
                }
            }
            first += n;
        }
        Ok(out)
    }
}

pub(super) fn found(f: &Founding, roster: &[(FoundingForm, u32)], g: &mut Genesis) -> Result<()> {
    let mut index = f.lineages;
    for &(kingdom, count) in roster {
        if kingdom.bodied() {
            continue;
        }
        if kingdom == FoundingForm::Microbe && g.rules.functions.is_empty() {
            g.rules.functions = crate::rules::default_functions();
            g.rules.shapes = crate::rules::default_shapes();
        }
        if kingdom == FoundingForm::Microbe {
            let rule = g
                .rules
                .hosting
                .get_or_insert_with(crate::rules::Hosting::default);
            g.rules
                .traits
                .extend([&rule.rot, &rule.feed, &rule.leak, &rule.enter].map(Clone::clone));
        }
        for (slot, recipe) in recipes(f.seed, kingdom)
            .into_iter()
            .take(count as usize)
            .enumerate()
        {
            let key = format!("lineage:{index}");
            let mut line = g.lineages["world:ground"].clone();
            line.kingdom = kingdom.key().into();
            let at = draw(f.seed, "habitat-origin", &[kingdom as u64, recipe]) % u64::from(f.sites);
            match kingdom {
                FoundingForm::Microbe => {
                    let functions = [
                        "fix",
                        "store",
                        "conduct",
                        "sense",
                        "contract",
                        "reproduce",
                        "intake",
                        "secrete",
                        "support",
                    ];
                    for (bit, name) in functions.iter().enumerate() {
                        if (recipe + 1) & (1 << bit) != 0 {
                            line.traits.insert(format!("function:{name}"));
                        }
                    }
                    let host = g.rules.hosting.as_ref().unwrap();
                    for (function, gene) in [
                        ("function:intake", &host.feed),
                        ("function:secrete", &host.leak),
                        ("function:conduct", &host.rot),
                        ("function:contract", &host.enter),
                    ] {
                        if line.traits.contains(function) {
                            line.traits.insert(gene.clone());
                        }
                    }
                    g.spreads.identities.insert(Identity::Surface(key.clone()));
                    g.sites
                        .get_mut(&at)
                        .unwrap()
                        .accounts
                        .insert(key.clone(), f.stock_min + recipe % 13);
                },
                FoundingForm::WorldClass => {
                    let overrides = &f.repertoire.as_ref().unwrap().world_bodies;
                    let body = overrides
                        .get(&slot.to_string())
                        .copied()
                        .unwrap_or_else(|| WorldBody::from_recipe(recipe));
                    body.apply(g, at, index)?;
                },
                _ => unreachable!(),
            }
            g.rules.traits.extend(line.traits.iter().cloned());
            g.lineages.insert(key, line);
            index += 1;
        }
    }
    Ok(())
}
