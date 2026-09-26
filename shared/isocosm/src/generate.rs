// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, population::Population, rules::*, schema::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The declared, versioned generator domain, saved with every bench receipt.
/// Generated topology, metabolic wiring, stocks and cadence all vary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Founding {
    pub seed: u64,
    pub sites: u32,
    pub population: u64,
    pub lineages: u32,
    pub cohort_size: u64,
    pub stock_min: u64,
    pub stock_max: u64,
    pub period_max: u64,
    pub base_unit_micrometres: u64,
    pub ecology: bool,
}

impl Default for Founding {
    fn default() -> Self {
        Self {
            seed: 1,
            sites: 6,
            population: 256,
            lineages: 4,
            cohort_size: 32,
            stock_min: 8,
            stock_max: 64,
            period_max: 5,
            base_unit_micrometres: 1000,
            ecology: false,
        }
    }
}

fn set(values: &[&str]) -> BTreeSet<Key> {
    values.iter().map(|s| s.to_string()).collect()
}
pub(crate) fn process(id: &str, shape: Shape, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        shape,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: set(&["sim:matter", "sim:nonnegative"]),
        note: false,
    }
}

impl Founding {
    pub fn generate(&self) -> Result<Genesis> {
        if self.sites == 0
            || self.sites > 256
            || self.lineages == 0
            || self.lineages > 32
            || self.population == 0
            || self.population > 1_000_000
            || self.cohort_size == 0
            || self.stock_min == 0
            || self.stock_max < self.stock_min
            || self.stock_max > 1_000_000
            || self.period_max == 0
            || self.period_max > 1000
            || self.base_unit_micrometres == 0
        {
            return Err("founding parameters outside declared generator domain".into());
        }
        let random = |domain: &str, i| crate::draw(self.seed, domain, &[i]);
        let mut lineages = BTreeMap::from([(
            "world:ground".into(),
            Lineage {
                parent: None,
                revision: 1,
                traits: BTreeSet::new(),
                kingdom: "kingdom:world".into(),
            },
        )]);
        let mut accounts = BTreeMap::from([
            (
                "world:soil".into(),
                AccountKind::Matter {
                    lineage: "world:ground".into(),
                },
            ),
            ("sim:energy".into(), AccountKind::Energy),
            ("sim:attention".into(), AccountKind::Attention),
            ("sim:time".into(), AccountKind::Time),
            ("sim:obligation".into(), AccountKind::Obligation),
        ]);
        let mut processes = BTreeMap::new();
        let mut traits = BTreeSet::new();
        let kingdoms = [
            "kingdom:flora",
            "kingdom:fauna",
            "kingdom:myco",
            "kingdom:micro",
        ];
        for i in 0..self.lineages {
            let lineage = format!("lineage:{i}");
            let ability = format!("ability:cycle-{i}");
            traits.insert(ability.clone());
            lineages.insert(
                lineage.clone(),
                Lineage {
                    parent: None,
                    revision: 1,
                    traits: BTreeSet::from([ability.clone()]),
                    kingdom: kingdoms[i as usize % kingdoms.len()].into(),
                },
            );
            let n = 2 + random("compartments", u64::from(i)) % 3;
            for j in 0..n {
                accounts.insert(
                    format!("matter:{i}-{j}"),
                    AccountKind::Matter {
                        lineage: lineage.clone(),
                    },
                );
            }
            for j in 0..n {
                let source = format!("matter:{i}-{j}");
                let destination = format!("matter:{i}-{}", (j + 1) % n);
                let amount = 1 + random("rate", u64::from(i) * 8 + j) % 5;
                let id = format!("process:cycle-{i}-{j}");
                let mut p = process(
                    &id,
                    Shape::Choice,
                    vec![Effect::Transform {
                        who: Binding::Actor,
                        take: BTreeMap::from([(source.clone(), amount)]),
                        give: BTreeMap::from([(destination, amount)]),
                    }],
                );
                p.requires.extend([
                    Query::Trait {
                        who: Binding::Actor,
                        key: ability.clone(),
                    },
                    Query::Account {
                        who: Binding::Actor,
                        key: source,
                        at_least: amount,
                    },
                    Query::Condition {
                        key: "world:habitable".into(),
                        at_least: 1,
                    },
                ]);
                p.period = Some(1 + random("period", u64::from(i) * 8 + j) % self.period_max);
                processes.insert(id, p);
            }
        }
        let mut mark = process(
            "sim:remember",
            Shape::Choice,
            vec![Effect::Note {
                kind: "sim:observed".into(),
                text: "Observed at the specimen bench.".into(),
                lifetime: None,
            }],
        );
        mark.note = true;
        processes.insert(mark.id.clone(), mark);
        let mut donate = process(
            "sim:give",
            Shape::Choice,
            vec![Effect::Transfer {
                from: Binding::Actor,
                to: Binding::Target,
                account: "world:soil".into(),
                amount: 1,
            }],
        );
        donate.note = true;
        processes.insert(donate.id.clone(), donate);
        let mut birth = process(
            "sim:birth",
            Shape::Transition,
            vec![Effect::Birth {
                provision: BTreeMap::from([("world:soil".into(), 2)]),
            }],
        );
        birth.note = true;
        processes.insert(birth.id.clone(), birth);
        let mut death = process("sim:death", Shape::Transition, vec![Effect::Death]);
        death.note = true;
        processes.insert(death.id.clone(), death);
        let polity = process(
            "sim:found-polity",
            Shape::Transition,
            vec![Effect::FoundPolity {
                governance: "governance:consent".into(),
                focus: set(&["sim:give"]),
                support: "world:soil".into(),
            }],
        );
        processes.insert(polity.id.clone(), polity);
        let mut reckon = process(
            "sim:reckon",
            Shape::Transition,
            vec![Effect::Record {
                axis: "feat:reserve".into(),
                account: "world:soil".into(),
            }],
        );
        reckon.note = true;
        processes.insert(reckon.id.clone(), reckon);
        let mut weather = process(
            "world:weather",
            Shape::Agentless,
            vec![Effect::Condition {
                key: "world:weather".into(),
                delta: 1,
            }],
        );
        weather.requires.clear();
        weather.period = Some(3);
        weather.priority = -1;
        processes.insert(weather.id.clone(), weather);
        let rules = Rules {
            version: crate::VERSION,
            accounts,
            conditions: set(&["world:habitable", "world:weather"]),
            traits,
            relations: set(&["sim:parent", "sim:member", "sim:owns"]),
            note_kinds: set(&["sim:act", "sim:observed", "sim:discover"]),
            processes,
            field: FieldPolicy {
                strength: 1_000_000,
                decay_per_tick: 10_000,
                legend_floor: 250_000,
            },
            limits: Limits {
                entities: 1_000_001 + u64::from(self.sites),
                ..Limits::default()
            },
            epoch_ticks: 32,
            collection_buffer: 16,
            competitions: BTreeMap::new(),
            similitude: None,
        };
        let mut sites = BTreeMap::new();
        for i in 0..u64::from(self.sites) {
            let mut neighbours = BTreeSet::new();
            if self.sites > 1 {
                neighbours.insert((i + 1) % u64::from(self.sites));
            }
            if self.sites > 2 {
                let extra = random("route", i) % u64::from(self.sites);
                if extra != i {
                    neighbours.insert(extra);
                }
            }
            sites.insert(
                i,
                Site {
                    terrain_seed: random("terrain", i),
                    accounts: BTreeMap::new(),
                    conditions: BTreeMap::from([
                        ("world:habitable".into(), 1),
                        ("world:weather".into(), 0),
                    ]),
                    routes: neighbours
                        .into_iter()
                        .map(|to| Route {
                            to,
                            travel: 1 + random("distance", i * 256 + to) % 4,
                            transmission: 500_000
                                + (random("transmission", i * 256 + to) % 500_001) as u32,
                        })
                        .collect(),
                },
            );
        }
        let mut population = Population::default();
        // The world has a body at each site for its agentless processes.
        for i in 0..u64::from(self.sites) {
            population.insert(
                Entity {
                    lineage: "world:ground".into(),
                    kingdom: "kingdom:world".into(),
                    scale: "scale:macro".into(),
                    provenance: Provenance::Intrinsic("world:ground".into()),
                    method: Method::Inert,
                    place: i,
                    arrived: 0,
                    visits: vec![],
                    born: 0,
                    alive: true,
                    body_revision: 1,
                    parts: BTreeMap::new(),
                    traits: BTreeSet::new(),
                    accounts: BTreeMap::new(),
                    skills: BTreeMap::new(),
                    tenets: BTreeMap::new(),
                    disposition: [0; 5],
                },
                1,
            )?;
        }
        let mut remaining = self.population;
        let mut group = 0;
        while remaining > 0 {
            let i = random("lineage", group) % u64::from(self.lineages);
            let lineage = format!("lineage:{i}");
            let n = 2 + random("compartments", i) % 3;
            let mut accounts = BTreeMap::from([("world:soil".into(), 4)]);
            for j in 0..n {
                accounts.insert(
                    format!("matter:{i}-{j}"),
                    self.stock_min
                        + random("stock", group * 8 + j) % (self.stock_max - self.stock_min + 1),
                );
            }
            let count = self.cohort_size.min(remaining);
            population.insert(
                Entity {
                    lineage: lineage.clone(),
                    kingdom: lineages[&lineage].kingdom.clone(),
                    scale: "scale:meso".into(),
                    provenance: Provenance::Born(lineage.clone()),
                    method: Method::Reactive,
                    place: random("habitat", group) % u64::from(self.sites),
                    arrived: 0,
                    visits: vec![],
                    born: 0,
                    alive: true,
                    body_revision: 1,
                    parts: BTreeMap::from([(
                        0,
                        Part {
                            parent: None,
                            traits: lineages[&lineage].traits.clone(),
                            severed: false,
                        },
                    )]),
                    traits: lineages[&lineage].traits.clone(),
                    accounts,
                    skills: BTreeMap::new(),
                    tenets: BTreeMap::new(),
                    disposition: [0; 5],
                },
                count,
            )?;
            remaining -= count;
            group += 1;
        }
        let mut genesis = Genesis {
            version: crate::VERSION,
            seed: self.seed,
            dynamics: None,
            founding: Some(self.clone()),
            rules,
            lineages,
            sites,
            population,
            world: WorldTraits {
                shape: "shape:graph".into(),
                scale: "scale:macro".into(),
                base_unit_micrometres: self.base_unit_micrometres,
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
            },
        };
        if self.ecology {
            crate::ecology::configure(self, &mut genesis)?;
        }
        genesis.validate()?;
        Ok(genesis)
    }
}
