// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's vertical probe (ruling 262), its matter in parts since
//! checkpoint 7: two lineages over a minimal allocated body, producers fixing
//! and growing glands and grazers eating them, so that the five natives run
//! as definitions. A world draws each lineage's body plan, its members'
//! tissue by cohort as shares of each part's adult mass and the grazers'
//! reserve as shares of what their lump stores (ruling 506), which producer
//! cohorts carry the gland among their candidates, its sites' soil and the
//! rate at which the ground returns living matter as soil. Nothing is
//! contested and no mind is kept: what the crowd must match is the body.

#[cfg(test)]
mod calibration;
mod physiology;

use super::{ProbeWorld, Similitude, member, world_traits};
use crate::{Result, anatomy, population::Population, rules::*, schema::*, simulation::Genesis};
use physiology::{CANDIDATE, SOIL, reserve, tissue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Inclusive ranges, drawn per world from `seed`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyFounding {
    pub seed: u64,
    pub ticks: Tick,
    pub sites: [u64; 2],
    pub cohort: u64,
    /// Producers and grazers per site.
    pub producers: [u64; 2],
    pub grazers: [u64; 2],
    /// A frond's half-extent along each of its two long axes; its cells are
    /// Mesocosm's lattice of them.
    pub frond: [i32; 2],
    /// A limb's long half-extent, and how many limbs a grazer has.
    pub limb: [i32; 2],
    pub limbs: [u64; 2],
    /// Of the eight cells of a grazer's lump, how many store (ruling 506);
    /// the rest take in.
    pub store: [u64; 2],
    /// Per mille of producer cohorts carrying the gland among their
    /// candidates.
    pub candidates: u64,
    /// A cohort's tissue as founded, per mille of each part's adult mass,
    /// and its reserve, per mille of what its stores may hold.
    pub tissue: [u64; 2],
    pub reserve: [u64; 2],
    /// Each site's soil, and what of the ground's living matter the site
    /// returns as soil a tick (step 4's rate).
    pub soil: [u64; 2],
    pub mineralization: [u64; 2],
    pub bound_per_mille: u32,
}

impl Default for BodyFounding {
    /// Mesocosm's primitive palette, widened: fronds of the palette's plate
    /// or smaller, grazers of one lump, one limb or two and an eye, a
    /// quarter of producer cohorts able to grow a gland; some cohorts
    /// founded near starving, so that bodies die within the run.
    fn default() -> Self {
        Self {
            seed: 1,
            ticks: 24,
            sites: [1, 2],
            cohort: 8,
            producers: [24, 64],
            grazers: [4, 16],
            frond: [3, 4],
            limb: [3, 4],
            limbs: [1, 2],
            store: [1, 4],
            candidates: 250,
            tissue: [150, 1000],
            reserve: [0, 1000],
            soil: [200, 2400],
            mineralization: [0, 8],
            bound_per_mille: 200,
        }
    }
}

/// A part of `shape` expressing `function` on every cell its extents give
/// it (ruling 460), attached to `parent`.
fn shaped(shape: &str, function: &str, half_extent: [i32; 3], parent: Option<Id>) -> Part {
    let mut p = Part {
        parent,
        shape: format!("part-shape:{shape}"),
        functions: BTreeSet::from([format!("function:{function}")]),
        half_extent,
        ..Default::default()
    };
    p.cells = BTreeMap::from([(format!("function:{function}"), anatomy::capacity(&p))]);
    p
}

impl BodyFounding {
    fn pick(&self, domain: &str, i: u64, range: [u64; 2]) -> u64 {
        range[0] + crate::draw(self.seed, domain, &[i]) % (range[1] - range[0] + 1)
    }

    fn valid(&self) -> bool {
        let ranges = [
            self.sites,
            self.producers,
            self.grazers,
            self.limbs,
            self.tissue,
            self.reserve,
            self.soil,
            self.mineralization,
        ];
        ranges.iter().all(|r| r[0] <= r[1])
            && self.frond[0] <= self.frond[1]
            && self.limb[0] <= self.limb[1]
            && self.frond[0] > 0
            && self.limb[0] > 0
            && self.store[0] <= self.store[1]
            && self.sites[0] > 0
            && self.cohort > 0
            && self.ticks > 0
            && self.candidates <= 1000
            && self.tissue[1] <= 1000
            && self.reserve[1] <= 1000
            && self.bound_per_mille <= 1000
    }

    /// The producers' body: one frond, every cell of Mesocosm's lattice
    /// fixing, keeping no reserve (ruling 506).
    fn frond(&self) -> BTreeMap<Id, Part> {
        let extent = |axis| self.pick("body-frond", axis, self.frond.map(|h| h as u64)) as i32;
        let frond = shaped("sheet", "fix", [extent(0), extent(1), 1], None);
        BTreeMap::from([(0, frond)])
    }

    /// The grazers' body: a lump that takes in and stores, its cells split
    /// by a draw (ruling 506), its limbs and an eye attached to it.
    fn grazer(&self) -> BTreeMap<Id, Part> {
        let mut lump = shaped("lump", "intake", [2, 2, 2], None);
        let capacity = anatomy::capacity(&lump);
        let store = (self.pick("body-store", 0, self.store) as u32).min(capacity - 1);
        lump.functions.insert(anatomy::STORE.into());
        lump.cells = BTreeMap::from([
            ("function:intake".into(), capacity - store),
            (anatomy::STORE.into(), store),
        ]);
        let mut parts = vec![lump];
        let limbs = self.pick("body-limbs", 0, self.limbs);
        for k in 0..limbs {
            let reach = self.pick("body-limb", k, self.limb.map(|h| h as u64)) as i32;
            parts.push(shaped("rod", "contract", [reach, 1, 1], Some(0)));
        }
        parts.push(shaped("point", "sense", [1, 1, 1], Some(0)));
        (0..).zip(parts).collect()
    }

    pub fn generate(&self) -> Result<ProbeWorld> {
        if !self.valid() {
            return Err("body founding outside its declared domain".into());
        }
        let sites = self.pick("body-sites", 0, self.sites);
        let plans = [self.frond(), self.grazer()];
        let per_site = [
            self.pick("body-members", 0, self.producers),
            self.pick("body-members", 1, self.grazers),
        ];
        let dose = self.pick("body-mineralization", 0, self.mineralization);
        let matter = |lineage: &str, reserve: bool| AccountKind::Matter {
            lineage: lineage.into(),
            reserve,
            provision: false,
        };
        let mut accounts = BTreeMap::from([(SOIL.into(), matter("world:ground", false))]);
        let mut traits = BTreeSet::from([CANDIDATE.to_string()]);
        let mut lineages = BTreeMap::from([(
            "world:ground".into(),
            Lineage {
                parent: None,
                revision: 1,
                traits: BTreeSet::new(),
                kingdom: "kingdom:world".into(),
                development: None,
            },
        )]);
        let mut processes = BTreeMap::new();
        for (i, kingdom) in [(0u32, "kingdom:flora"), (1, "kingdom:fauna")] {
            let lineage = format!("lineage:{i}");
            accounts.insert(tissue(i), matter(&lineage, false));
            accounts.insert(reserve(i), matter(&lineage, true));
            let identity = format!("ability:probe-{i}");
            traits.insert(identity.clone());
            lineages.insert(
                lineage,
                Lineage {
                    parent: None,
                    revision: 1,
                    traits: BTreeSet::from([identity]),
                    kingdom: kingdom.into(),
                    development: None,
                },
            );
            let prey = (i == 1).then_some(0);
            for p in physiology::natives(i, prey) {
                processes.insert(p.id.clone(), p);
            }
        }
        let mineralize = physiology::mineralize(2, dose);
        processes.insert(mineralize.id.clone(), mineralize);
        let founders: u64 = per_site.iter().sum();
        let rules = Rules {
            body: None,
            kinds: BTreeMap::new(),
            affinity: None,
            version: crate::VERSION,
            accounts,
            conditions: BTreeSet::new(),
            traits,
            relations: BTreeSet::new(),
            note_kinds: BTreeSet::from(["sim:act".into()]),
            processes,
            field: FieldPolicy {
                strength: 1_000_000,
                decay_per_tick: 10_000,
                legend_floor: 250_000,
            },
            limits: Limits {
                entities: 1 + sites + sites * founders,
                ..Limits::default()
            },
            // A year at the default minute (ruling 451).
            epoch_ticks: crate::rules::year_ticks(crate::rules::DEFAULT_TICK_MICROSECONDS),
            collection_buffer: 0,
            competitions: BTreeMap::new(),
            similitude: Some(Similitude {
                default_bound: self.bound_per_mille,
                bounds: BTreeMap::new(),
            }),
            mind: None,
            tick_microseconds: None,
            shapes: default_shapes(),
            functions: default_functions(),
            skeleton: None,
            epoch: Default::default(),
            deep_time: Default::default(),
        };
        let (site_map, population) = self.found(&lineages, sites, per_site, &plans)?;
        let genesis = Genesis {
            version: crate::VERSION,
            seed: self.seed,
            dynamics: None,
            founding: None,
            world: world_traits(),
            rules,
            lineages,
            sites: site_map,
            population,
        };
        genesis.validate()?;
        Ok(ProbeWorld {
            genesis,
            ticks: self.ticks,
        })
    }

    /// The sites with their soil, and the founders in cohorts at every
    /// site, each cohort's tissue drawn as a share of each part's adult mass
    /// and its reserve as a share of what its stores may hold.
    fn found(
        &self,
        lineages: &BTreeMap<Key, Lineage>,
        sites: u64,
        per_site: [u64; 2],
        plans: &[BTreeMap<Id, Part>; 2],
    ) -> Result<(BTreeMap<Id, Site>, Population)> {
        let mut site_map = BTreeMap::new();
        let mut population = Population::default();
        for s in 0..sites {
            let site = Site {
                terrain_seed: crate::draw(self.seed, "body-terrain", &[s]),
                conditions: BTreeMap::new(),
                accounts: BTreeMap::from([(SOIL.into(), self.pick("body-soil", s, self.soil))]),
                routes: vec![],
            };
            site_map.insert(s, site);
            population.insert(member(lineages, "world:ground", s, BTreeMap::new()), 1)?;
        }
        let mut cohort = 0;
        for s in 0..sites {
            for (i, &n) in per_site.iter().enumerate() {
                let lineage = format!("lineage:{i}");
                let mut left = n;
                while left > 0 {
                    let count = left.min(self.cohort);
                    let tissue_mille = self.pick("body-tissue", cohort, self.tissue);
                    let reserve_mille = self.pick("body-reserve", cohort, self.reserve);
                    let mut e = member(lineages, &lineage, s, BTreeMap::new());
                    e.parts = plans[i].clone();
                    let b = BodyRules::default();
                    for part in e.parts.values_mut() {
                        let held = anatomy::ceiling(part, b) * tissue_mille / 1000;
                        part.matter.insert(tissue(i as u32), held);
                        let cells = u64::from(part.cells.get(anatomy::STORE).copied().unwrap_or(0));
                        let stored = cells * anatomy::cell_mass(part, b) * reserve_mille / 1000;
                        // Every store keeps the account, empty or not, so
                        // that every draw reads the same set (as checkpoint
                        // 6's ledger kept both accounts).
                        if cells > 0 {
                            part.matter.insert(reserve(i as u32), stored);
                        }
                    }
                    let candidate = self.pick("body-candidate", cohort, [0, 999]) < self.candidates;
                    if i == 0 && candidate {
                        e.traits.insert(CANDIDATE.into());
                    }
                    population.insert(e, count)?;
                    left -= count;
                    cohort += 1;
                }
            }
        }
        Ok((site_map, population))
    }
}
