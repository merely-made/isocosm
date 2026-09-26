// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The clock: due processes over the groups that could match them (rulings
//! 258 and 259), and the unit a tick counts (rulings 256 and 257).

use isocosm::{
    Execution, Founding, Simulation, population::Population, rules::*, schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const ROUSED: &str = "test:roused";

/// One site, the world's body, then two cohorts of three: the first
/// lineage's members roused, the second's not. The one due process lets a
/// roused member rouse a member of the second lineage and practise.
fn rousing() -> Genesis {
    let mut g = Founding {
        seed: 4,
        sites: 1,
        population: 6,
        lineages: 2,
        cohort_size: 3,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let world = g.population.groups[&0].entity.clone();
    let mut body = g.population.get(1).unwrap().clone();
    body.accounts = BTreeMap::from([("world:soil".into(), 4)]);
    let mut population = Population::default();
    population.insert(world, 1).unwrap();
    for (lineage, roused) in [("lineage:0", true), ("lineage:1", false)] {
        let mut e = body.clone();
        e.lineage = lineage.into();
        e.traits = if roused {
            BTreeSet::from([ROUSED.into()])
        } else {
            BTreeSet::new()
        };
        population.insert(e, 3).unwrap();
    }
    g.population = population;
    g.rules.traits.insert(ROUSED.into());
    g.rules.processes.retain(|_, p| p.period.is_none());
    let rouse = Process {
        id: "test:rouse".into(),
        shape: Shape::Choice,
        requires: vec![
            Query::Alive(Binding::Actor),
            Query::Trait {
                who: Binding::Actor,
                key: ROUSED.into(),
            },
        ],
        commitments: vec![],
        effects: vec![
            Effect::Trait {
                who: Binding::Target,
                key: ROUSED.into(),
                present: true,
            },
            Effect::Practice {
                key: "skill:rousing".into(),
                amount: 1,
            },
        ],
        risk: None,
        target: Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: Some("lineage:1".into()),
        }),
        period: Some(1),
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    };
    g.rules.processes.insert(rouse.id.clone(), rouse);
    g
}

fn practised(sim: &Simulation, id: Id) -> u64 {
    let e = sim.state().population.get(id).unwrap();
    e.skills.get("skill:rousing").copied().unwrap_or(0)
}

#[test]
fn a_member_that_gains_a_required_trait_during_the_pass_is_still_visited() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(rousing(), mode).unwrap();
        let work = sim.advance(1).unwrap();
        // The first cohort each rouse member 4, splitting the second cohort.
        // Member 4 was not roused when the pass began, yet is visited, as a
        // scan over every group would visit it, and rouses member 5, who
        // acts later in the same pass; member 6 is never roused.
        let skills: Vec<u64> = (1..=6).map(|id| practised(&sim, id)).collect();
        assert_eq!(skills, [1, 1, 1, 1, 1, 0], "{mode:?}");
        // Five members ran; the unroused sixth and the world's body did not.
        assert_eq!((work.evaluations, work.accepted), (5, 5), "{mode:?}");
    }
}

#[test]
fn a_member_without_a_required_trait_is_not_evaluated() {
    let founding = Founding {
        seed: 5,
        sites: 2,
        population: 40,
        lineages: 3,
        cohort_size: 4,
        ..Default::default()
    };
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut g = founding.generate().unwrap();
        for group in g.population.groups.values_mut() {
            group.entity.traits.clear();
        }
        let mut sim = Simulation::new(g, mode).unwrap();
        let work = sim.advance(10).unwrap();
        // Every lineage's process requires its ability, which nobody now
        // carries, so only the weather runs: every third tick at each of
        // the two sites' world bodies.
        assert_eq!((work.evaluations, work.accepted), (6, 6), "{mode:?}");
    }
}

#[test]
fn a_budget_of_exactly_the_evaluations_that_run_is_enough() {
    let genesis = Founding {
        seed: 91,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut first = Simulation::new(genesis.clone(), mode).unwrap();
        let ran = first.advance(1).unwrap().evaluations as usize;
        assert!(ran > 0);
        let mut exact = genesis.clone();
        exact.rules.limits.events_per_advance = ran;
        let mut sim = Simulation::new(exact.clone(), mode).unwrap();
        sim.advance(1).unwrap();
        // The same world, apart from the budget in its rules.
        assert!(sim.state() == first.state(), "{mode:?}");
        exact.rules.limits.events_per_advance = ran - 1;
        let mut short = Simulation::new(exact, mode).unwrap();
        assert!(short.advance(1).is_err(), "{mode:?}");
    }
}

#[test]
fn the_clock_counts_a_minute_unless_the_world_states_its_unit() {
    let g = Founding::default().generate().unwrap();
    assert_eq!(g.rules.tick_microseconds, None);
    assert_eq!(g.rules.tick_microseconds(), 60_000_000);
    // Worlds without a unit serialize, and so hash, as before it existed.
    let json = serde_json::to_string(&g.rules).unwrap();
    assert!(!json.contains("tick_microseconds"));
    let back: Rules = serde_json::from_str(&json).unwrap();
    assert_eq!(back.revision(), g.rules.revision());
    // A stated unit keeps through bytes and is part of the rules' identity.
    let mut round = g.rules.clone();
    round.tick_microseconds = Some(6_000_000);
    let back: Rules = serde_json::from_str(&serde_json::to_string(&round).unwrap()).unwrap();
    assert_eq!(back, round);
    assert_eq!(back.tick_microseconds(), 6_000_000);
    assert_ne!(round.revision(), g.rules.revision());
    round.validate().unwrap();
    // A tick of no time is refused.
    round.tick_microseconds = Some(0);
    assert!(round.validate().is_err());
}
