// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A generated world's played lineage (rulings 682 and 683): its members
//! deliberate, the world's mind gives it a need, and directing's rules give
//! each trophic level its slot.

use crate::{Founding, Result, rules::*, schema::*, simulation::Genesis};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The lineage a founding gives a mind, by index, and how many sites'
/// worth of each trophic level, as founded, a region's slot holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Played {
    pub lineage: u32,
    pub region_sites: u32,
}

/// The share of its cells, out of a hundred, a body loses before its wounds
/// weigh on its mood.
const WOUNDED: u64 = 25;

/// The trophic levels generated ecologies name (ruling 39).
pub const LEVELS: [&str; 3] = ["life:producer", "life:consumer", "life:decomposer"];

pub(crate) fn found(f: &Founding, g: &mut Genesis, played: Played) -> Result<()> {
    if played.lineage >= f.lineages || played.region_sites == 0 {
        return Err("the played lineage or its region is outside the founding".into());
    }
    let i = played.lineage;
    let lineage = format!("lineage:{i}");
    for group in g.population.groups.values_mut() {
        if group.entity.lineage == lineage {
            group.entity_mut().method = Method::Deliberative;
        }
    }
    let rules = &mut g.rules;
    let strain: Key = "sim:strain".into();
    rules
        .accounts
        .entry(strain.clone())
        .or_insert(AccountKind::Strain);
    // Every line is hungry below what its bodies are founded with; mood
    // drives migration (753), and the played line's choice (683).
    let hunger = |j: u32| Need {
        traits: BTreeSet::from([format!("ability:cycle-{j}")]),
        query: Query::Below {
            who: Binding::Actor,
            key: format!("matter:{j}-0"),
            amount: f.stock_min + 2,
        },
        weight: -10,
    };
    let mind = rules.mind.get_or_insert_with(|| Mind {
        strain,
        needs: vec![],
        bearing: 100,
        bearing_traits: BTreeMap::new(),
        rise: 0,
        rise_traits: BTreeMap::new(),
        stake: 0,
        rate: None,
    });
    mind.needs.extend((0..f.lineages).map(hunger));
    // A body that has lost a quarter of its cells is wounded (the 2026-10-18
    // note to ruling 227); paid healing serves it, and eating does not.
    mind.needs.push(Need {
        traits: BTreeSet::new(),
        query: Query::Wounded {
            who: Binding::Actor,
            at_least: WOUNDED,
        },
        weight: -10,
    });
    // Strain builds while mood stays low and eases when it recovers, as a
    // probe mind's does, from the same ranges and draws.
    let pick = |domain: &str, r: [u64; 2]| r[0] + crate::draw(f.seed, domain, &[0]) % (r[1] - r[0] + 1);
    let (low_mood, build, bleed) =
        (pick("mind-low-mood", Mind::LOW_MOOD), pick("mind-build", Mind::BUILD), pick("mind-bleed", Mind::BLEED));
    // Where mood reads only the body, strain is a rate worked out when read
    // (E7); otherwise its two processes keep it each tick.
    if mind.bodily() {
        mind.rate = Some(Rate { low_mood, build, bleed });
    } else {
        let keeping = mind.strain_processes(low_mood, build, bleed);
        rules
            .processes
            .extend(keeping.into_iter().map(|p| (p.id.clone(), p)));
    }
    let mut slots = BTreeMap::new();
    for level in LEVELS.into_iter().filter(|l| rules.traits.contains(*l)) {
        let founded: u128 = g
            .population
            .groups
            .values()
            .filter(|c| c.entity.traits.contains(level))
            .map(|c| {
                crate::meaning::mass(&crate::anatomy::books(&c.entity), rules) * u128::from(c.count)
            })
            .sum();
        let slot = (founded * u128::from(played.region_sites)).div_ceil(u128::from(f.sites.max(1)));
        slots.insert(level.into(), slot.max(1));
    }
    // Each kind decides at its own pace (D19), lineages cycling producer,
    // consumer, decomposer as the ecology founds them.
    for i in 0..f.lineages {
        if let Some(l) = g.lineages.get_mut(&format!("lineage:{i}")) {
            l.decides = Some(crate::ecology::rates::DECIDES[i as usize % 3]);
        }
    }
    migrate(g)?;
    let rules = &mut g.rules;
    rules.directing = Some(Directing {
        slots,
        movement_ties: MovementTies::Seeded,
        ..Directing::default()
    });
    Ok(())
}

/// Members migrate (753): a move process per route, open to a member at
/// the route's site while its mood is low. A site names itself by the
/// condition `site:<id>`, so a move asks where its actor is.
fn migrate(g: &mut Genesis) -> Result<()> {
    let mut moves = Vec::new();
    for (&id, site) in &mut g.sites {
        let here: Key = format!("site:{id}");
        site.conditions.insert(here.clone(), 1);
        g.rules.conditions.insert(here.clone());
        for route in &site.routes {
            let mut p = crate::generate::process(
                &format!("move:{id}-{}", route.to),
                Causation::Choice,
                vec![Effect::Move {
                    destination: route.to,
                }],
            );
            p.requires.extend([
                Query::Condition {
                    key: here.clone(),
                    at_least: 1,
                },
                Query::MoodBelow { amount: 0 },
            ]);
            p.period = Some(route.travel.max(1));
            moves.push(p);
        }
    }
    if g.rules.processes.len() + moves.len() > g.rules.limits.processes {
        return Err("a move per route exceeds the world's process limit".into());
    }
    g.rules
        .processes
        .extend(moves.into_iter().map(|p| (p.id.clone(), p)));
    Ok(())
}
