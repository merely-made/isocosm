// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 6's vertical probe (ruling 262), its matter in parts since
//! checkpoint 7 and its bodies grown from recipes since checkpoint 8: two
//! lineages, producers fixing, growing glands and budding, grazers eating
//! them, bearing and nursing their young, so that the natives run as
//! definitions. A world draws each lineage's recipe (plans.rs), its members'
//! soma and tissue by cohort as shares of each part's adult mass, the
//! grazers' reserve as shares of what their lump stores (ruling 506), which
//! producer cohorts carry the gland among their candidates, which cohorts
//! breed once (529), its sites' soil and the rate at which the ground
//! returns living matter as soil. Since checkpoint 9 a body carries the
//! systems it realizes and its natives read them, each lineage feeding by
//! capability, and a world draws its feeding rates, what a cell carries,
//! and each recipe's riff and variation odds (rulings 564, 571, 573, 578).
//! Nothing is contested and no mind is kept: what the crowd must match is
//! the body.

mod feeding;
mod life;
mod physiology;
mod plans;

use super::{ProbeWorld, Similitude, member, world_traits};
use crate::{
    Result, anatomy,
    bodied::{BROOD, BUD, EGG, MILK, SEMELPAROUS},
    development::{develop, soma},
    population::Population,
    rules::*,
    schema::*,
    simulation::Genesis,
};
use physiology::{CANDIDATE, SOIL, provision, reserve, tissue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Inclusive ranges, drawn per world from `seed`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
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
    /// A limb's long half-extent, and how many limbs a grazer bears.
    pub limb: [i32; 2],
    pub limbs: [u64; 2],
    /// Of the eight cells of a grazer's lump, how many store (ruling 506);
    /// the rest take in or reproduce.
    pub store: [u64; 2],
    /// Of each root's cells, how many reproduce (ruling 513); none, with
    /// no variance and no absences, founds checkpoint 7's bodies.
    pub reproduce: [u32; 2],
    /// Each recipe's segment variance, and its absence odds, one in so
    /// many, 0 for never (ruling 531).
    pub variance: [u8; 2],
    pub absence: [u32; 2],
    /// How many eggs a grazer's birth lays (530).
    pub clutch: [u32; 2],
    /// Each recipe's odds that a child's system riffs and that one of its
    /// cells varies, one in so many, 0 for never (573, 578).
    pub riff: [u32; 2],
    pub vary: [u32; 2],
    /// The feeding rates, numerators over 144 faces and 269 voxels (571),
    /// and what a cell carries a tick, in milligrams (564).
    pub fixes: [u64; 2],
    pub grazes: [u64; 2],
    pub carriage: [u64; 2],
    /// Per mille of producer cohorts carrying the gland among their
    /// candidates, and of all cohorts breeding once (529).
    pub candidates: u64,
    pub semelparous: u64,
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
    /// quarter of producer cohorts able to grow a gland and half of all
    /// cohorts breeding once; recipes varying by Mesocosm's bounds (550);
    /// some cohorts founded near starving, so that bodies die within the
    /// run; soil from starved to surplus (553); and a run long enough for
    /// births (517), the latest first birth measured at tick 56 of 100
    /// worlds.
    fn default() -> Self {
        Self {
            seed: 1,
            ticks: 60,
            sites: [1, 2],
            cohort: 8,
            producers: [24, 64],
            grazers: [4, 16],
            frond: [3, 4],
            limb: [3, 4],
            limbs: [1, 2],
            store: [1, 4],
            reproduce: [1, 1],
            variance: [1, 2],
            absence: [12, 12],
            clutch: [1, 4],
            riff: [12, 100],
            vary: [12, 100],
            fixes: [6, 22],
            grazes: [6, 24],
            carriage: [7, 14],
            candidates: 250,
            semelparous: 500,
            tissue: [150, 1000],
            reserve: [0, 1000],
            soil: [200, 12_000],
            mineralization: [0, 8],
            bound_per_mille: 200,
        }
    }
}

/// The traits a world's members may carry: their strategies and care
/// (548), whether they breed once, a bud's mark, and a young unweaned.
fn life_traits() -> [&'static str; 8] {
    [
        BROOD,
        EGG,
        BUD,
        MILK,
        SEMELPAROUS,
        life::ITEROPAROUS,
        life::BUD_MARK,
        life::UNWEANED,
    ]
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
            self.fixes,
            self.grazes,
            self.carriage,
        ];
        ranges.iter().all(|r| r[0] <= r[1])
            && self.frond[0] <= self.frond[1]
            && self.limb[0] <= self.limb[1]
            && self.frond[0] > 0
            && self.limb[0] > 0
            && self.store[0] <= self.store[1]
            && self.reproduce[0] <= self.reproduce[1]
            && self.variance[0] <= self.variance[1]
            && self.absence[0] <= self.absence[1]
            && self.clutch[0] <= self.clutch[1]
            && self.clutch[0] > 0
            && self.riff[0] <= self.riff[1]
            && self.vary[0] <= self.vary[1]
            && self.fixes[0] > 0
            && self.grazes[0] > 0
            && self.carriage[0] > 0
            && (1..=2).contains(&self.limbs[0])
            && self.limbs[1] <= 2
            && self.sites[0] > 0
            && self.cohort > 0
            && self.ticks > 0
            && self.candidates <= 1000
            && self.semelparous <= 1000
            && self.tissue[1] <= 1000
            && self.reserve[1] <= 1000
            && self.bound_per_mille <= 1000
    }

    pub fn generate(&self) -> Result<ProbeWorld> {
        if !self.valid() {
            return Err("body founding outside its declared domain".into());
        }
        let sites = self.pick("body-sites", 0, self.sites);
        let per_site = [
            self.pick("body-members", 0, self.producers),
            self.pick("body-members", 1, self.grazers),
        ];
        let dose = self.pick("body-mineralization", 0, self.mineralization);
        let rates = feeding::Rates {
            fixes: self.pick("body-fixes", 0, self.fixes) as i64,
            grazes: self.pick("body-grazes", 0, self.grazes) as i64,
        };
        let per_cell = self.pick("body-carriage", 0, self.carriage);
        let affinity = Affinity::default();
        let developments = self.developments(&affinity);
        let matter = |lineage: &str, reserve: bool, provision: bool| AccountKind::Matter {
            lineage: lineage.into(),
            reserve,
            provision,
        };
        let mut accounts = BTreeMap::from([(SOIL.into(), matter("world:ground", false, false))]);
        let mut traits: BTreeSet<Key> = life_traits().map(String::from).into();
        traits.insert(CANDIDATE.into());
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
            accounts.insert(tissue(i), matter(&lineage, false, false));
            accounts.insert(reserve(i), matter(&lineage, true, false));
            accounts.insert(provision(i), matter(&lineage, false, true));
            let identity = format!("ability:probe-{i}");
            traits.insert(identity.clone());
            // The probe sets its own life history (548): producers bud;
            // grazers brood or lay eggs and give milk.
            let history: &[&str] = match i {
                0 => &[BUD],
                _ => &[BROOD, EGG, MILK],
            };
            let mut carried = BTreeSet::from([identity]);
            carried.extend(history.iter().map(|t| t.to_string()));
            lineages.insert(
                lineage,
                Lineage {
                    parent: None,
                    revision: 1,
                    traits: carried,
                    kingdom: kingdom.into(),
                    development: Some(developments[i as usize].clone()),
                },
            );
            for p in physiology::natives(i, 2, i == 0, rates) {
                processes.insert(p.id.clone(), p);
            }
        }
        let mineralize = physiology::mineralize(2, dose);
        processes.insert(mineralize.id.clone(), mineralize);
        let rules = Rules {
            body: None,
            kinds: self.kinds(),
            affinity: Some(affinity),
            systems: default_systems(),
            carriage: Some(Carriage { per_cell }),
            directing: None,
            version: crate::VERSION,
            accounts,
            conditions: BTreeSet::new(),
            traits,
            relations: BTreeSet::from(["sim:parent".into(), "sim:child".into()]),
            note_kinds: BTreeSet::from(["sim:act".into()]),
            processes,
            field: FieldPolicy {
                strength: 1_000_000,
                decay_per_tick: 10_000,
                legend_floor: 250_000,
            },
            limits: Limits::default(),
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
        let (site_map, population) = self.found(&rules, &lineages, sites, per_site)?;
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
    /// site, each cohort sharing a soma drawn from its recipe, its tissue a
    /// share of each part's adult mass, its reserve a share of what its
    /// stores may hold and its provision empty.
    fn found(
        &self,
        rules: &Rules,
        lineages: &BTreeMap<Key, Lineage>,
        sites: u64,
        per_site: [u64; 2],
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
        let b = rules.body();
        let mut cohort = 0;
        for s in 0..sites {
            for (i, &n) in per_site.iter().enumerate() {
                let lineage = format!("lineage:{i}");
                let d = lineages[&lineage]
                    .development
                    .as_ref()
                    .ok_or("a probe lineage without a recipe")?;
                let mut left = n;
                while left > 0 {
                    let count = left.min(self.cohort);
                    let tissue_mille = self.pick("body-tissue", cohort, self.tissue);
                    let reserve_mille = self.pick("body-reserve", cohort, self.reserve);
                    let mut e = member(lineages, &lineage, s, BTreeMap::new());
                    let drawn = soma(
                        rules,
                        &d.recipe,
                        crate::draw(self.seed, "body-soma", &[cohort]),
                    );
                    e.parts = develop(rules, d, &drawn)?;
                    e.soma = drawn.segments;
                    e.systems = crate::systems::founded(&e, rules);
                    let i = i as u32;
                    for part in e.parts.values_mut() {
                        let held = anatomy::ceiling(part, b) * tissue_mille / 1000;
                        part.matter.insert(tissue(i), held);
                        let cells = |f: &str| u64::from(part.cells.get(f).copied().unwrap_or(0));
                        let (stores, sown) = (cells(anatomy::STORE), cells(anatomy::REPRODUCE));
                        let stored = stores * anatomy::cell_mass(part, b) * reserve_mille / 1000;
                        // Every store and reproducing part keeps its account,
                        // empty or not, so that every draw reads the same set
                        // (as checkpoint 6's ledger kept both accounts).
                        if stores > 0 {
                            part.matter.insert(reserve(i), stored);
                        }
                        if sown > 0 {
                            part.matter.insert(provision(i), 0);
                        }
                    }
                    let candidate = self.pick("body-candidate", cohort, [0, 999]) < self.candidates;
                    if i == 0 && candidate {
                        e.traits.insert(CANDIDATE.into());
                    }
                    let once = self.pick("body-semelparous", cohort, [0, 999]) < self.semelparous;
                    e.traits
                        .insert(if once { SEMELPAROUS } else { life::ITEROPAROUS }.into());
                    population.insert(e, count)?;
                    left -= count;
                    cohort += 1;
                }
            }
        }
        Ok((site_map, population))
    }
}
