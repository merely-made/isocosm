// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The clock: due processes over the groups that could match them (rulings
//! 258, 259, 285 and 286), advances limited by work and not ticks (284), and
//! the unit a tick counts (rulings 256 and 257).

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
        causation: Causation::Choice,
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
                amount: 1.into(),
            },
        ],
        risk: None,
        target: Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: Some("lineage:1".into()),
            among: BTreeSet::new(),
            weighted: false,
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
fn a_member_that_gains_a_required_trait_during_a_pass_acts_from_the_next() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(rousing(), mode).unwrap();
        let work = sim.advance(1).unwrap();
        // The first cohort each rouse member 4, the first of the second
        // cohort as the pass began. A pass reads the world as it began
        // (ruling 454), so member 4, roused part way, does not act in it.
        let skills: Vec<u64> = (1..=6).map(|id| practised(&sim, id)).collect();
        assert_eq!(skills, [1, 1, 1, 0, 0, 0], "{mode:?}");
        assert_eq!((work.evaluations, work.accepted), (3, 3), "{mode:?}");
        // The next pass began with member 4 roused: it rouses member 5,
        // passing over itself.
        let work = sim.advance(1).unwrap();
        let skills: Vec<u64> = (1..=6).map(|id| practised(&sim, id)).collect();
        assert_eq!(skills, [2, 2, 2, 1, 0, 0], "{mode:?}");
        assert_eq!((work.evaluations, work.accepted), (4, 4), "{mode:?}");
        let roused = |id| {
            sim.state()
                .population
                .get(id)
                .unwrap()
                .traits
                .contains(ROUSED)
        };
        assert_eq!((roused(5), roused(6)), (true, false), "{mode:?}");
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

fn budgeted() -> Genesis {
    Founding {
        seed: 91,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

#[test]
fn a_budget_counts_the_members_that_run_and_means_the_same_in_both_modes() {
    let genesis = budgeted();
    let mut members = BTreeSet::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut first = Simulation::new(genesis.clone(), mode).unwrap();
        let work = first.advance(3).unwrap();
        members.insert(work.represented);
        let ran = work.represented as usize;
        assert!(ran > 0);
        let mut exact = genesis.clone();
        exact.rules.limits.events_per_advance = ran;
        let mut sim = Simulation::new(exact.clone(), mode).unwrap();
        sim.advance(3).unwrap();
        // The same world, apart from the budget in its rules.
        assert!(sim.state() == first.state(), "{mode:?}");
        exact.rules.limits.events_per_advance = ran - 1;
        let mut short = Simulation::new(exact, mode).unwrap();
        assert!(short.advance(3).is_err(), "{mode:?}");
    }
    assert_eq!(members.len(), 1, "the modes count the same members");
}

#[test]
fn an_advance_spans_any_idle_time_its_work_allows() {
    let mut genesis = budgeted();
    // Ruling 284 retired the tick limit; nothing reads it now.
    genesis.rules.limits.advance_ticks = 10;
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut long = Simulation::new(genesis.clone(), mode).unwrap();
        long.advance(50).unwrap();
        let mut steps = Simulation::new(genesis.clone(), mode).unwrap();
        for _ in 0..5 {
            steps.advance(10).unwrap();
        }
        assert_eq!(long.state_hash(), steps.state_hash(), "{mode:?}");
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

/// One site, the world's body, then one cohort of three born at tick zero
/// holding four of soil, with `processes` as the only due ones.
fn cohort(processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 6,
        sites: 1,
        population: 3,
        lineages: 1,
        cohort_size: 3,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.processes.retain(|_, p| p.period.is_none());
    for p in processes {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g
}

fn due(id: &str, priority: i32, requires: Vec<Query>, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires,
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: Some(1),
        priority,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

fn practise(key: &str) -> Effect {
    Effect::Practice {
        key: key.into(),
        amount: 1.into(),
    }
}

fn skill(sim: &Simulation, id: Id, key: &str) -> u64 {
    let e = sim.state().population.get(id).unwrap();
    e.skills.get(key).copied().unwrap_or(0)
}

#[test]
fn a_group_is_visited_when_it_comes_of_age_and_not_before() {
    let grown = due(
        "test:grown",
        0,
        vec![Query::Alive(Binding::Actor), Query::Age { at_least: 5 }],
        vec![practise("skill:grown")],
    );
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(cohort(vec![grown.clone()]), mode).unwrap();
        // Too young for four ticks: nothing is evaluated, not even blocked.
        assert_eq!(sim.advance(4).unwrap().represented, 0, "{mode:?}");
        let work = sim.advance(1).unwrap();
        assert_eq!((work.represented, work.accepted), (3, 3), "{mode:?}");
        assert!((1..=3).all(|id| skill(&sim, id, "skill:grown") == 1));
    }
}

#[test]
fn a_reserve_running_out_is_seen_in_the_same_tick() {
    let soil = "world:soil";
    let drain = due(
        "test:drain",
        0,
        vec![Query::Account {
            who: Binding::Actor,
            key: soil.into(),
            at_least: 1,
        }],
        vec![Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: soil.into(),
            amount: 1.into(),
        }],
    );
    let starve = due(
        "test:starve",
        1,
        vec![Query::Below {
            who: Binding::Actor,
            key: soil.into(),
            amount: 1,
        }],
        vec![practise("skill:starved")],
    );
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(cohort(vec![drain.clone(), starve.clone()]), mode).unwrap();
        let members: Vec<u64> = (0..6)
            .map(|_| sim.advance(1).unwrap().represented)
            .collect();
        // Four ticks drain the soil, the fourth leaving none, when starving
        // runs for the first time, after the drain in the same tick; from
        // then on only starving runs.
        assert_eq!(members, [3, 3, 3, 6, 3, 3], "{mode:?}");
        assert!((1..=3).all(|id| skill(&sim, id, "skill:starved") == 3));
    }
}
