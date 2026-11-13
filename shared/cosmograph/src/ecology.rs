// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A second generated law family, using the ordinary interpreter. Places are
//! explicitly well-mixed compartments here; no physical contact is inferred.
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use std::collections::BTreeMap;

mod admission;
mod budget;
mod categories;
mod detritus;
mod dormant;
mod guard;
mod joint;
mod limits;
mod mutualism;
mod rot;
mod spoilage;
mod validate;
mod evidence;
mod run;
mod validated_run;
mod independence;
pub(crate) mod rates;
pub mod mass_law;
mod web;
pub mod world_time;
#[cfg(test)]
mod web_tests;
#[cfg(test)]
mod limits_tests;
#[cfg(test)]
mod rot_tests;
#[cfg(test)]
mod spoilage_tests;
#[cfg(test)]
mod spoilage_meal_tests;
#[cfg(test)]
mod categories_tests;
#[cfg(test)]
mod categories_meal_tests;
#[cfg(test)]
mod detritus_tests;
#[cfg(test)]
mod mutualism_tests;
#[cfg(test)]
mod mutualism_flow_tests;
#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod guard_meal_tests;
pub(crate) use web::Role;
pub use web::Web;
pub use independence::*;
pub use rates::Rates;
pub use mass_law::MassLaw;
pub use limits::Limits;
pub use rot::{DECOMPOSER_BITES, Rot};
pub use categories::{Categories, tough};
pub use detritus::{DUNG, Detritus, left_by, takes};
pub use guard::{Guard, Ward};
pub use mutualism::{Mutualism, trades};
pub use spoilage::{Flesh, Spoilage};
pub use budget::Budget;
pub use joint::*;
pub use run::{run_joint, run_joint_validated};
pub use validated_run::{AdmittedJointRun, ValidatedJointRun};
pub(crate) use evidence::JointCapture;
pub(crate) use admission::validate_material_provenance;

/// The account a producer captures (890). Energy is not matter (879): tissue
/// is never relabelled as fuel.
const SUN: &str = "sim:energy";
/// A site's light credit per tick at full sun, and the most it holds (rulings
/// 982 and 983). The credit is scaled by the sun's intensity at the tick
/// (`light::at`), so it is nothing at night; the store holds a night's light,
/// half a day of full credit, so producers draw a reserve down while it is
/// dark. 500 is about the burst the seed-9 A1 producers drew at one site in
/// one tick, so light limits when they crowd and not otherwise. In bodied
/// ecologies the sun process's presence reseeds which part receives new
/// tissue, so their trajectories differ (wing ruling 929).
pub(crate) const LIGHT: u64 = 500;
const STORE: u64 = 720 * LIGHT;
/// The trophic levels and the kingdom each is generated in, by `Role::level`.
const LEVELS: [&str; 3] = crate::directing::found::LEVELS;
/// The turn group of the producers that draw on one soil.
const SOIL_TURNS: &str = "turn:world:soil";
const KINGDOMS: [&str; 3] = ["kingdom:flora", "kingdom:fauna", "kingdom:myco"];

/// The sun: each site gains light while it holds less than its store.
fn sunlight() -> Process {
    let mut sun = process(
        "ecology:sunlight",
        Causation::Agentless,
        vec![Effect::Sunlight {
            who: Binding::Place,
            account: SUN.into(),
            full: LIGHT,
        }],
    );
    sun.requires = vec![Query::Below {
        who: Binding::Place,
        key: SUN.into(),
        amount: STORE,
    }];
    sun.period = Some(1);
    sun.priority = -2;
    sun
}

pub(crate) fn configure(founding: &Founding, g: &mut Genesis) -> Result<()> {
    g.rules
        .processes
        .retain(|id, _| !id.starts_with("process:cycle-"));
    for (&id, site) in &mut g.sites {
        let supply = founding
            .population
            .checked_mul(founding.stock_max)
            .ok_or("soil supply overflow")?
            / u64::from(founding.sites);
        site.accounts.insert(
            "world:soil".into(),
            supply + crate::draw(founding.seed, "soil", &[id]) % founding.stock_max,
        );
        // A full store from the first tick: acts before the sun's first pass work.
        site.accounts.insert(SUN.into(), STORE);
    }
    g.rules.processes.insert("ecology:sunlight".into(), sunlight());
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        if e.kingdom == "kingdom:world" {
            continue;
        }
        let i = e
            .lineage
            .strip_prefix("lineage:")
            .unwrap()
            .parse::<u32>()
            .map_err(|e| e.to_string())?;
        e.accounts
            .retain(|id, _| id == &format!("matter:{i}-0") || id == "world:soil");
        // Birth thresholds are above initial bodies, so reproduction must be fed.
        e.accounts
            .insert(format!("matter:{i}-0"), founding.stock_min + 2);
        let level = Role::of(founding.web, i).level();
        let role = LEVELS[level];
        e.kingdom = KINGDOMS[level].into();
        e.traits.insert(role.into());
        g.rules.traits.insert(role.into());
    }
    let rates = founding.ecology_rates;
    // Life history is stated in minutes and read at the world's unit (1072).
    let unit = g.rules.tick_microseconds();
    let minutes = |m: u64| crate::rules::minutes_ticks(m, unit);
    let producers = (0..founding.lineages).filter(|&i| Role::of(founding.web, i) == Role::Producer).count();
    let sensing = founding.bodies.as_ref().is_some_and(|b| b.sensing);
    for i in 0..founding.lineages {
        let role = Role::of(founding.web, i);
        let consumer = role.consumer();
        g.rules.traits.insert(LEVELS[role.level()].into());
        let body = format!("matter:{i}-0");
        let own = format!("ability:cycle-{i}");
        let lineage = format!("lineage:{i}");
        // Upkeep and growth are rates, not choices (D16, case 1076): a played
        // lineage's decision period paces what it chooses, while strain and
        // growth run on the body's own clock for every body, so a body's
        // birth tick and identity never decide whether it pays or grows.
        let mut upkeep = process(
            &format!("ecology:upkeep-{i}"),
            Causation::Transition,
            vec![
                Effect::Transform {
                    who: Binding::Actor,
                    take: BTreeMap::from([(body.clone(), 1)]),
                    give: BTreeMap::from([("world:soil".into(), 1)]),
                    conversion: None,
                },
                Effect::Transfer {
                    from: Binding::Actor,
                    to: Binding::Place,
                    account: "world:soil".into(),
                    amount: 1.into(),
                },
            ],
        );
        upkeep.requires.push(Query::Trait {
            who: Binding::Actor,
            key: own.clone(),
        });
        // A predator pays upkeep twice as often as a grazer and breeds at
        // half its pace: the cost of a meal that runs.
        let predator = role == Role::Predator;
        let upkeep_every = rates.upkeep_period(role);
        upkeep.period = Some(if predator { (upkeep_every / 2).max(1) } else { upkeep_every });
        upkeep.priority = 20;
        g.rules.processes.insert(upkeep.id.clone(), upkeep);
        let mut death = process(
            &format!("ecology:death-{i}"),
            Causation::Transition,
            vec![Effect::Death],
        );
        death.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Below {
                who: Binding::Actor,
                key: body.clone(),
                amount: 1,
            },
        ]);
        death.period = Some(1);
        death.priority = 30;
        death.note = true;
        g.rules.processes.insert(death.id.clone(), death);
        let mut age = process(
            &format!("ecology:age-{i}"),
            Causation::Transition,
            vec![Effect::Death],
        );
        let lifespan = rates.lifespan(crate::draw(founding.seed, "lifespan", &[u64::from(i)]));
        age.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Age {
                at_least: minutes(lifespan),
            },
        ]);
        age.period = Some(1);
        age.priority = 31;
        age.note = true;
        // An individual's life is its own (906): from maturity a hazard can take
        // it, and the lineage's drawn lifespan stays the ceiling. Where the
        // ceiling is not above maturity the hazard could never act, so it is
        // not declared.
        if lifespan > rates::MATURE {
            let mut hazard = age.clone();
            hazard.id = format!("ecology:hazard-{i}");
            hazard.priority += 1;
            for q in &mut hazard.requires {
                if let Query::Age { at_least } = q {
                    *at_least = minutes(rates::MATURE);
                }
            }
            hazard.effects.clear();
            hazard.risk = Some(Risk {
                per_million: crate::rules::per_minute_per_tick(rates::HAZARD_PER_MILLION, unit),
                effects: vec![Effect::Death],
                aging: None,
            });
            g.rules.processes.insert(hazard.id.clone(), hazard);
        }
        g.rules.processes.insert(age.id.clone(), age);
        let child_mass = founding.stock_min.max(2);
        let mut birth = process(
            &format!("ecology:birth-{i}"),
            Causation::Transition,
            vec![Effect::Birth {
                provision: BTreeMap::from([(body.clone(), child_mass)]),
                scatter: None,
            }],
        );
        birth.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Account {
                who: Binding::Actor,
                key: body.clone(),
                at_least: rates.birth_gate(founding.stock_min, consumer),
            },
        ]);
        birth.period = Some(minutes(rates.birth_period(consumer) * if predator { 2 } else { 1 }));
        birth.priority = 40;
        birth.note = true;
        g.rules.processes.insert(birth.id.clone(), birth);
        if role == Role::Producer {
            let mut grow = process(
                &format!("ecology:produce-{i}"),
                Causation::Transition,
                vec![
                    // Capture (890): the producer takes the site's light, then
                    // spends it with the soil it synthesizes into tissue.
                    Effect::Transfer {
                        from: Binding::Place,
                        to: Binding::Actor,
                        account: SUN.into(),
                        amount: 1.into(),
                    },
                    Effect::Transfer {
                        from: Binding::Place,
                        to: Binding::Actor,
                        account: "world:soil".into(),
                        amount: 2.into(),
                    },
                    Effect::Transform {
                        who: Binding::Actor,
                        take: BTreeMap::from([("world:soil".into(), 2)]),
                        give: BTreeMap::from([(body, 2)]),
                        conversion: None,
                    },
                    // The light is spent in the synthesis, in an effect of
                    // its own so the matter transform stays matter-only.
                    Effect::Transform {
                        who: Binding::Actor,
                        take: BTreeMap::from([(SUN.into(), 1)]),
                        give: BTreeMap::new(),
                        conversion: None,
                    },
                ],
            );
            grow.requires.extend([
                Query::Trait {
                    who: Binding::Actor,
                    key: own,
                },
                Query::Condition {
                    key: "world:habitable".into(),
                    at_least: 1,
                },
            ]);
            if sensing {
                // A plant grows on what it senses (1071): its share of its
                // site's light and soil, crowded by the plants beside it,
                // must cover what one act captures and synthesizes.
                grow.requires.extend([
                    Query::Sensed { reading: "sense:light-share".into(), at_least: 1 },
                    Query::Sensed { reading: "sense:soil-share".into(), at_least: 2 },
                ]);
            }
            grow.period = Some(minutes(2));
            grow.priority = 0;
            // Producers that share ground take turns at it: no lineage's
            // index decides who eats first. A lone producer has none to share with.
            if producers > 1 {
                grow.turn = Some(SOIL_TURNS.into());
            }
            g.rules.processes.insert(grow.id.clone(), grow);
        } else {
            // One feeding process per consumer (ruling 287), choosing among
            // its prey by a draw weighted by what each holds: consumers eat
            // the living of the roles their diet admits, decomposers the dead
            // of every other lineage.
            let prey = (0..founding.lineages)
                .filter(|&j| i != j && role.eats(Role::of(founding.web, j)))
                .map(|j| format!("lineage:{j}"))
                .collect();
            let mut feed = process(
                &format!("ecology:feed-{i}"),
                Causation::Choice,
                vec![Effect::Eat {
                    from: Binding::Target,
                    amount: Amount::Fixed(rates.meal(consumer)),
                    into: body.clone(),
                    of: vec![],
                    whole: false,
                }],
            );
            feed.requires.extend([
                Query::Trait {
                    who: Binding::Actor,
                    key: own.clone(),
                },
                Query::Holds {
                    who: Binding::Target,
                    at_least: 1,
                },
            ]);
            feed.target = Some(Target {
                same_place: true,
                alive: Some(consumer),
                lineage: None,
                among: prey,
                weighted: true,
            });
            feed.period = Some(rates.feed_period(role));
            feed.priority = rates.feed_priority(role);
            g.rules.processes.insert(feed.id.clone(), feed);
        }
        g.lineages
            .get_mut(&lineage)
            .unwrap()
            .traits
            .insert(LEVELS[role.level()].into());
        g.lineages.get_mut(&lineage).unwrap().kingdom = KINGDOMS[role.level()].into();
    }
    if rates == Rates::WorldTime {
        world_time::configure(founding, g)?;
    }
    Ok(())
}

/// What lays over the configured and embodied ecology: the mass law, then
/// the flat rate sets' dormancy, which copies the scaled active rates, then
/// the consumers' limits, which gate the feed and birth processes as scaled,
/// then rot, whose processes gate nothing and copy nothing, then spoilage,
/// which reads the mouths the rest founded, then the categories, which read
/// the parts a body has, then the guard, whose rent rides beside upkeep, then mutualism, whose turns read
/// the organs the repertoire lowers, then the detritus web, whose dung turns read mouths, then the repertoire, which
/// lowers catalog parts into kinds beside them.
pub(crate) fn finish(founding: &Founding, g: &mut Genesis) -> Result<()> {
    mass_law::apply(founding, g)?;
    if founding.ecology_rates != Rates::WorldTime {
        dormant::flat(founding, g)?;
    }
    limits::apply(founding, g)?;
    rot::apply(founding, g)?;
    spoilage::apply(founding, g)?;
    categories::apply(founding, g)?;
    guard::apply(founding, g)?;
    mutualism::apply(founding, g)?;
    detritus::apply(founding, g)?;
    crate::repertoire::apply(founding, g)?;
    crate::inherit::apply(founding, g)?;
    Ok(())
}
