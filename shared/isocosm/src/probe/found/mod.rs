// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The probe's declared domain. Every quantity a world's rules hold is drawn
//! from its seed within the stated ranges, the fight's and the mind's
//! included. A world contests food, and water too when `water` is set, each
//! keyed by its site account (ruling 236). The lineage count is fixed so
//! every draw reads alike.

mod contested;
mod mind;

pub use mind::{MindFounding, STRAIN};

use super::{Competition, Competitor, ProbeWorld, Similitude};
use crate::{
    Result, generate::process, population::Population, rules::*, schema::*, simulation::Genesis,
};
use contested::{FOOD, Thing, WATER};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Inclusive ranges, drawn per world from `seed`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbeFounding {
    pub seed: u64,
    pub ticks: Tick,
    pub lineages: u32,
    pub sites: [u64; 2],
    /// Members per site and lineage.
    pub members: [u64; 2],
    pub cohort: u64,
    /// Contest water as well as food.
    pub water: bool,
    /// Each contested thing's ration, drawn for each.
    pub ration: [u64; 2],
    /// A member wants a contested thing while its store of it is below
    /// this, drawn for each.
    pub hunger: [u64; 2],
    pub margin: [u64; 2],
    /// Reserve spent by the side losing an exchange.
    pub cost: [u64; 2],
    /// Per mille, the chance an exchange goes against the side standing
    /// higher.
    pub upset: [u64; 2],
    /// Standing a break adds or takes for the rest of a fight.
    pub advantage: [u64; 2],
    /// Strain each side takes per round, and per unit of reserve spent.
    pub round_strain: [u64; 2],
    pub spend_strain: [u64; 2],
    /// Each thing regrown per tick at a site, per mille of the site's
    /// founders, drawn for each.
    pub regrowth: [u64; 2],
    pub mind: MindFounding,
    pub bound_per_mille: u32,
}

impl Default for ProbeFounding {
    fn default() -> Self {
        Self {
            seed: 1,
            ticks: 24,
            lineages: 2,
            sites: [1, 2],
            // Crowds of about 18 living members per state by the last tick,
            // at a cost the exact runner can repeat a thousand times.
            members: [64, 128],
            cohort: 16,
            water: false,
            ration: [2, 4],
            hunger: [3, 6],
            margin: [0, 2],
            cost: [1, 2],
            upset: [0, 400],
            advantage: [1, 3],
            round_strain: [1, 3],
            spend_strain: [0, 2],
            regrowth: [300, 900],
            mind: MindFounding::default(),
            bound_per_mille: 200,
        }
    }
}

fn set(values: &[String]) -> BTreeSet<Key> {
    values.iter().cloned().collect()
}

fn matter(lineage: &str) -> AccountKind {
    AccountKind::Matter {
        lineage: lineage.into(),
    }
}

/// One contested thing as drawn for a world.
struct Drawn {
    thing: &'static Thing,
    ration: u64,
    want: u64,
    regrowth: u64,
}

impl ProbeFounding {
    fn pick(&self, domain: &str, i: u64, range: [u64; 2]) -> u64 {
        range[0] + crate::draw(self.seed, domain, &[i]) % (range[1] - range[0] + 1)
    }

    fn valid(&self) -> bool {
        let ranges = [
            self.sites,
            self.members,
            self.ration,
            self.hunger,
            self.margin,
            self.cost,
            self.upset,
            self.advantage,
            self.round_strain,
            self.spend_strain,
            self.regrowth,
        ];
        ranges.iter().all(|r| r[0] <= r[1])
            && self.mind.valid()
            && self.sites[0] > 0
            && self.ration[0] >= 2
            && self.hunger[0] > 0
            && self.cost[0] > 0
            && self.upset[1] <= 1000
            && self.lineages > 0
            && self.cohort > 0
            && self.ticks > 0
            && self.bound_per_mille <= 1000
    }

    pub fn generate(&self) -> Result<ProbeWorld> {
        if !self.valid() {
            return Err("probe founding outside its declared domain".into());
        }
        let sites = self.pick("probe-sites", 0, self.sites);
        let per_site: Vec<u64> = (0..self.lineages)
            .map(|i| self.pick("probe-members", u64::from(i), self.members))
            .collect();
        let founders: u64 = per_site.iter().sum();
        let things: &[&'static Thing] = if self.water {
            &[&FOOD, &WATER]
        } else {
            &[&FOOD]
        };
        let drawn: Vec<Drawn> = things
            .iter()
            .enumerate()
            .map(|(j, &thing)| {
                let j = j as u64;
                let per_mille = self.pick("probe-regrowth", j, self.regrowth);
                Drawn {
                    thing,
                    ration: self.pick("probe-ration", j, self.ration),
                    want: self.pick("probe-hunger", j, self.hunger),
                    regrowth: (per_mille * founders).div_ceil(1000),
                }
            })
            .collect();
        let spend_strain = self.pick("probe-spend-strain", 0, self.spend_strain);
        let mut accounts = BTreeMap::from([
            ("world:soil".into(), matter("world:ground")),
            (STRAIN.into(), AccountKind::Strain),
        ]);
        let mut lineages = BTreeMap::from([(
            "world:ground".into(),
            Lineage {
                parent: None,
                revision: 1,
                traits: BTreeSet::new(),
                kingdom: "kingdom:world".into(),
            },
        )]);
        let mut traits = set(&["leaning:contest".into(), "leaning:scramble".into()]);
        let mut processes = BTreeMap::new();
        let mut kinds: Vec<Vec<Competitor>> = drawn.iter().map(|_| vec![]).collect();
        let mut minds = Vec::new();
        for d in &drawn {
            accounts.insert(d.thing.key.into(), matter("world:ground"));
            let p = contested::regrow(d.thing, d.regrowth);
            processes.insert(p.id.clone(), p);
        }
        for i in 0..self.lineages {
            let identity = format!("ability:probe-{i}");
            let lineage = format!("lineage:{i}");
            let leaning =
                if crate::draw(self.seed, "probe-leaning", &[u64::from(i)]).is_multiple_of(2) {
                    "leaning:contest"
                } else {
                    "leaning:scramble"
                };
            for d in &drawn {
                accounts.insert(contested::store(i, d.thing.store), matter(&lineage));
            }
            traits.insert(identity.clone());
            lineages.insert(
                lineage,
                Lineage {
                    parent: None,
                    revision: 1,
                    traits: set(&[identity.clone(), leaning.into()]),
                    kingdom: "kingdom:fauna".into(),
                },
            );
            let takes: Vec<(&Thing, u64)> = drawn.iter().map(|d| (d.thing, d.ration)).collect();
            let made = contested::lineage(i, &takes, spend_strain);
            let body = contested::store(i, 0);
            let mut wants = Vec::new();
            for (j, d) in drawn.iter().enumerate() {
                let want = contested::below(&contested::store(i, d.thing.store), d.want);
                wants.push(want.clone());
                kinds[j].push(Competitor {
                    identity: identity.clone(),
                    body: body.clone(),
                    hungry: want,
                    eat: format!("probe:{}-{i}", d.thing.take),
                    share: format!("probe:{}-{i}", d.thing.half),
                    spend: made.spend.clone(),
                });
            }
            minds.push((identity, body, wants));
            for p in made.processes {
                processes.insert(p.id.clone(), p);
            }
        }
        let minds: Vec<mind::Kind> = minds
            .iter()
            .map(|(identity, body, wants)| mind::Kind {
                identity,
                body,
                wants: wants.clone(),
            })
            .collect();
        let (mind, keeping) = self.mind.draw(self.seed, &minds, drawn[0].want);
        let round_strain = self.pick("probe-round-strain", 0, self.round_strain);
        let round = process(
            "probe:round",
            Shape::Choice,
            contested::strain(round_strain),
        );
        for p in keeping.into_iter().chain([round]) {
            processes.insert(p.id.clone(), p);
        }
        let (margin, cost) = (
            self.pick("probe-margin", 0, self.margin),
            self.pick("probe-cost", 0, self.cost),
        );
        let upset = self.pick("probe-upset", 0, self.upset) as u32;
        let advantage = self.pick("probe-advantage", 0, self.advantage);
        let competitions = drawn
            .iter()
            .zip(kinds)
            .map(|(d, kinds)| {
                let c = Competition {
                    ration: d.ration,
                    contest: "leaning:contest".into(),
                    margin,
                    cost,
                    round: "probe:round".into(),
                    upset,
                    advantage,
                    kinds,
                };
                (d.thing.key.to_string(), c)
            })
            .collect();
        let rules = Rules {
            version: crate::VERSION,
            accounts,
            conditions: BTreeSet::new(),
            traits,
            relations: BTreeSet::new(),
            note_kinds: set(&["sim:act".into()]),
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
            epoch_ticks: 32,
            collection_buffer: 0,
            competitions,
            similitude: Some(Similitude {
                default_bound: self.bound_per_mille,
                bounds: BTreeMap::new(),
            }),
            mind: Some(mind),
        };
        let (site_map, population) = self.found(&drawn, &lineages, sites, &per_site)?;
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

    /// The sites, with soil for every regrowth and some of each thing, and
    /// the founders, in cohorts at every site.
    fn found(
        &self,
        drawn: &[Drawn],
        lineages: &BTreeMap<Key, Lineage>,
        sites: u64,
        per_site: &[u64],
    ) -> Result<(BTreeMap<Id, Site>, Population)> {
        let soil: u64 = drawn.iter().map(|d| d.regrowth).sum();
        let mut site_map = BTreeMap::new();
        let mut population = Population::default();
        for s in 0..sites {
            let mut accounts = BTreeMap::from([(
                "world:soil".into(),
                soil * self.pick("probe-soil", s, [2, 4]),
            )]);
            for (j, d) in drawn.iter().enumerate() {
                let domain = if j == 0 { "probe-food" } else { "probe-water" };
                accounts.insert(d.thing.key.into(), self.pick(domain, s, [0, d.regrowth]));
            }
            let site = Site {
                terrain_seed: crate::draw(self.seed, "probe-terrain", &[s]),
                conditions: BTreeMap::new(),
                accounts,
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
                    let mut ledger = BTreeMap::new();
                    for (j, d) in drawn.iter().enumerate() {
                        let domain = if j == 0 { "probe-body" } else { "probe-store" };
                        let held = self.pick(domain, cohort, [1, d.want + d.ration]);
                        ledger.insert(contested::store(i as u32, d.thing.store), held);
                    }
                    population.insert(member(lineages, &lineage, s, ledger), count)?;
                    left -= count;
                    cohort += 1;
                }
            }
        }
        Ok((site_map, population))
    }
}

fn member(lineages: &BTreeMap<Key, Lineage>, lineage: &str, place: Id, accounts: Ledger) -> Entity {
    let l = &lineages[lineage];
    let world = l.kingdom == "kingdom:world";
    Entity {
        lineage: lineage.into(),
        kingdom: l.kingdom.clone(),
        scale: if world { "scale:macro" } else { "scale:meso" }.into(),
        provenance: if world {
            Provenance::Intrinsic(lineage.into())
        } else {
            Provenance::Born(lineage.into())
        },
        method: if world {
            Method::Inert
        } else {
            Method::Reactive
        },
        place,
        arrived: 0,
        visits: vec![],
        born: 0,
        alive: true,
        body_revision: 1,
        parts: BTreeMap::new(),
        traits: l.traits.clone(),
        accounts,
        skills: BTreeMap::new(),
        tenets: BTreeMap::new(),
        disposition: [0; 5],
    }
}

fn world_traits() -> WorldTraits {
    WorldTraits {
        shape: "shape:graph".into(),
        scale: "scale:macro".into(),
        base_unit_micrometres: 1000,
        static_traits: BTreeSet::new(),
        magic: BTreeMap::new(),
        canon: wing_glyphs::CanonSpec {
            version: 1,
            id: "world:canon".into(),
            revision: 1,
            glyphs: vec![wing_glyphs::GlyphDefinition {
                id: "glyph:cycle".into(),
                display: "↻".into(),
                effect: "effect:cycle".into(),
            }],
            variants: vec![],
            limits: Default::default(),
        },
        parent_world: None,
        neighbours: BTreeMap::new(),
    }
}
