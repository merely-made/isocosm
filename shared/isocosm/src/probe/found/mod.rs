// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The probe's declared domain. Every quantity a world's rules hold is drawn
//! from its seed within the stated ranges, the fight's and the mind's
//! included. Food regrows from site soil; upkeep and fight costs return body
//! to it, so matter cycles. The lineage count is fixed so every draw reads
//! alike.

mod mind;

pub use mind::{MindFounding, STRAIN};

use super::{Competition, Competitor, ProbeWorld, Similitude};
use crate::{
    Result, generate::process, population::Population, rules::*, schema::*, simulation::Genesis,
};
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
    pub ration: [u64; 2],
    /// A member is hungry while its body is below this.
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
    /// Food regrown per tick at a site, per mille of the site's founders.
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

fn account(key: &str, at_least: u64) -> Query {
    Query::Account {
        who: Binding::Actor,
        key: key.into(),
        at_least,
    }
}

fn spend(body: &str) -> Vec<Effect> {
    vec![
        Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::from([(body.into(), 1)]),
            give: BTreeMap::from([("world:soil".into(), 1)]),
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: "world:soil".into(),
            amount: 1,
        },
    ]
}

/// Strain taken in a fight; nothing when the draw gave none.
fn strain(amount: u64) -> Vec<Effect> {
    if amount == 0 {
        return vec![];
    }
    vec![Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([(STRAIN.into(), amount)]),
    }]
}

fn feed(body: &str, amount: u64) -> Vec<Effect> {
    vec![
        Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: "world:food".into(),
            amount,
        },
        Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::from([("world:food".into(), amount)]),
            give: BTreeMap::from([(body.into(), amount)]),
        },
    ]
}

impl ProbeFounding {
    fn pick(&self, domain: &str, i: u64, range: [u64; 2]) -> u64 {
        range[0] + crate::draw(self.seed, domain, &[i]) % (range[1] - range[0] + 1)
    }

    pub fn generate(&self) -> Result<ProbeWorld> {
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
        if ranges.iter().any(|r| r[0] > r[1])
            || !self.mind.valid()
            || self.sites[0] == 0
            || self.ration[0] < 2
            || self.hunger[0] == 0
            || self.cost[0] == 0
            || self.upset[1] > 1000
            || self.lineages == 0
            || self.cohort == 0
            || self.ticks == 0
            || self.bound_per_mille > 1000
        {
            return Err("probe founding outside its declared domain".into());
        }
        let sites = self.pick("probe-sites", 0, self.sites);
        let ration = self.pick("probe-ration", 0, self.ration);
        let hunger = self.pick("probe-hunger", 0, self.hunger);
        let margin = self.pick("probe-margin", 0, self.margin);
        let cost = self.pick("probe-cost", 0, self.cost);
        let upset = self.pick("probe-upset", 0, self.upset) as u32;
        let advantage = self.pick("probe-advantage", 0, self.advantage);
        let spend_strain = self.pick("probe-spend-strain", 0, self.spend_strain);
        let per_site: Vec<u64> = (0..self.lineages)
            .map(|i| self.pick("probe-members", u64::from(i), self.members))
            .collect();
        let founders: u64 = per_site.iter().sum();
        let regrowth = (self.pick("probe-regrowth", 0, self.regrowth) * founders).div_ceil(1000);
        let mut accounts = BTreeMap::from([
            (
                "world:soil".into(),
                AccountKind::Matter {
                    lineage: "world:ground".into(),
                },
            ),
            (
                "world:food".into(),
                AccountKind::Matter {
                    lineage: "world:ground".into(),
                },
            ),
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
        let mut kinds = Vec::new();
        let mut regrow = process(
            "probe:regrow",
            Shape::Agentless,
            vec![Effect::Transform {
                who: Binding::Place,
                take: BTreeMap::from([("world:soil".into(), regrowth)]),
                give: BTreeMap::from([("world:food".into(), regrowth)]),
            }],
        );
        regrow.requires.push(Query::Account {
            who: Binding::Place,
            key: "world:soil".into(),
            at_least: regrowth,
        });
        regrow.period = Some(1);
        regrow.priority = -1;
        processes.insert(regrow.id.clone(), regrow);
        for i in 0..self.lineages {
            let identity = format!("ability:probe-{i}");
            let body = format!("matter:{i}-0");
            let lineage = format!("lineage:{i}");
            let leaning =
                if crate::draw(self.seed, "probe-leaning", &[u64::from(i)]).is_multiple_of(2) {
                    "leaning:contest"
                } else {
                    "leaning:scramble"
                };
            accounts.insert(
                body.clone(),
                AccountKind::Matter {
                    lineage: lineage.clone(),
                },
            );
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
            let own = Query::Trait {
                who: Binding::Actor,
                key: identity.clone(),
            };
            let mut upkeep = process(&format!("probe:upkeep-{i}"), Shape::Choice, spend(&body));
            upkeep.requires.extend([own.clone(), account(&body, 1)]);
            upkeep.period = Some(1);
            let mut starve = process(
                &format!("probe:starve-{i}"),
                Shape::Transition,
                vec![Effect::Death],
            );
            starve.requires.extend([
                own.clone(),
                Query::Below {
                    who: Binding::Actor,
                    key: body.clone(),
                    amount: 1,
                },
            ]);
            starve.period = Some(1);
            starve.priority = 1;
            let mut eat = process(
                &format!("probe:eat-{i}"),
                Shape::Choice,
                feed(&body, ration),
            );
            eat.requires.push(own.clone());
            let mut share = process(
                &format!("probe:share-{i}"),
                Shape::Choice,
                feed(&body, ration / 2),
            );
            share.requires.push(own.clone());
            let mut fought = spend(&body);
            fought.extend(strain(spend_strain));
            let mut spent = process(&format!("probe:spend-{i}"), Shape::Choice, fought);
            spent.requires.extend([own, account(&body, 1)]);
            kinds.push(Competitor {
                identity,
                body: body.clone(),
                hungry: Query::Below {
                    who: Binding::Actor,
                    key: body,
                    amount: hunger,
                },
                eat: eat.id.clone(),
                share: share.id.clone(),
                spend: spent.id.clone(),
            });
            for p in [upkeep, starve, eat, share, spent] {
                processes.insert(p.id.clone(), p);
            }
        }
        let minds: Vec<mind::Kind> = kinds
            .iter()
            .map(|k| mind::Kind {
                identity: &k.identity,
                body: &k.body,
            })
            .collect();
        let (mind, keeping) = self.mind.draw(self.seed, &minds, hunger);
        let round_strain = self.pick("probe-round-strain", 0, self.round_strain);
        let round = process("probe:round", Shape::Choice, strain(round_strain));
        for p in keeping.into_iter().chain([round]) {
            processes.insert(p.id.clone(), p);
        }
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
            competitions: BTreeMap::from([(
                "probe:feeding".into(),
                Competition {
                    food: "world:food".into(),
                    ration,
                    contest: "leaning:contest".into(),
                    margin,
                    cost,
                    round: "probe:round".into(),
                    upset,
                    advantage,
                    kinds,
                },
            )]),
            similitude: Some(Similitude {
                default_bound: self.bound_per_mille,
                bounds: BTreeMap::new(),
            }),
            mind: Some(mind),
        };
        let mut site_map = BTreeMap::new();
        let mut population = Population::default();
        for s in 0..sites {
            site_map.insert(
                s,
                Site {
                    terrain_seed: crate::draw(self.seed, "probe-terrain", &[s]),
                    conditions: BTreeMap::new(),
                    accounts: BTreeMap::from([
                        (
                            "world:soil".into(),
                            regrowth * self.pick("probe-soil", s, [2, 4]),
                        ),
                        (
                            "world:food".into(),
                            self.pick("probe-food", s, [0, regrowth]),
                        ),
                    ]),
                    routes: vec![],
                },
            );
            population.insert(member(&lineages, "world:ground", s, BTreeMap::new()), 1)?;
        }
        let mut cohort = 0;
        for s in 0..sites {
            for (i, &n) in per_site.iter().enumerate() {
                let (lineage, body) = (format!("lineage:{i}"), format!("matter:{i}-0"));
                let mut left = n;
                while left > 0 {
                    let count = left.min(self.cohort);
                    let reserve = self.pick("probe-body", cohort, [1, hunger + ration]);
                    let ledger = BTreeMap::from([(body.clone(), reserve)]);
                    population.insert(member(&lineages, &lineage, s, ledger), count)?;
                    left -= count;
                    cohort += 1;
                }
            }
        }
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
