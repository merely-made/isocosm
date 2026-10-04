// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Feeding (ruling 287): what eating takes, the holdings a body is read by,
//! and the weighted draw that chooses the prey.

use isocosm::{
    Execution, Founding, Simulation, population::Population, rules::*, schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const EAT: &str = "test:eat";
const EATER: &str = "test:eater";
const GUT: &str = "test:gut";

/// A count of members of a lineage, each holding a ledger.
type Group<'a> = (u64, &'a str, &'a [(&'a str, u64)]);

/// One site, its world's body, then `groups` at that site in identity
/// order, each a count of members of a lineage holding a ledger; lineage
/// 0's members are eaters. The rules declare `test:a`, `test:b` and
/// `test:c` matter of lineage 1 and the gut matter of lineage 0, and
/// `processes` are the only due ones.
fn table(groups: &[Group], processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 8,
        sites: 1,
        population: 2,
        lineages: 2,
        cohort_size: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let world = g.population.groups[&0].entity.clone();
    let mut body = g.population.get(1).unwrap().clone();
    body.traits.clear();
    let mut population = Population::default();
    population.insert(world, 1).unwrap();
    for (count, lineage, ledger) in groups {
        let mut e = body.clone();
        e.lineage = (*lineage).into();
        e.accounts = ledger.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        if *lineage == "lineage:0" {
            e.traits.insert(EATER.into());
        }
        population.insert(e, *count).unwrap();
    }
    g.population = population;
    g.rules.traits.insert(EATER.into());
    for (key, lineage) in [
        ("test:a", "lineage:1"),
        ("test:b", "lineage:1"),
        ("test:c", "lineage:1"),
        (GUT, "lineage:0"),
    ] {
        let kind = AccountKind::Matter {
            lineage: lineage.into(),
            reserve: false,
            provision: false,
        };
        g.rules.accounts.insert(key.into(), kind);
    }
    g.rules.processes.retain(|_, p| p.period.is_none());
    for p in processes {
        g.rules.processes.insert(p.id.clone(), p);
    }
    g
}

fn due(id: &str, requires: Vec<Query>, effects: Vec<Effect>, target: Option<Target>) -> Process {
    Process {
        id: id.into(),
        causation: Causation::Choice,
        requires,
        commitments: vec![],
        effects,
        risk: None,
        target,
        period: Some(1),
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

/// An eater eats `amount` of a living member of `among` holding at least
/// `at_least`.
fn eat(amount: u64, at_least: u64, among: &[&str]) -> Process {
    due(
        EAT,
        vec![
            Query::Alive(Binding::Actor),
            Query::Trait {
                who: Binding::Actor,
                key: EATER.into(),
            },
            Query::Holds {
                who: Binding::Target,
                at_least,
            },
        ],
        vec![Effect::Eat {
            from: Binding::Target,
            amount: amount.into(),
            into: GUT.into(),
            of: vec![],
            whole: false,
        }],
        Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: None,
            among: among.iter().map(|l| l.to_string()).collect(),
            weighted: true,
        }),
    )
}

fn ledger(sim: &Simulation, id: Id) -> Ledger {
    sim.state().population.get(id).unwrap().accounts.clone()
}

fn of(entries: &[(&str, u64)]) -> Ledger {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// The eater is member 1 and the prey member 2; returns the prey's ledger
/// and the eater's gut after one meal of `amount`.
fn meal(prey: &[(&str, u64)], amount: u64) -> (Ledger, u64) {
    let groups: &[Group] = &[(1, "lineage:0", &[]), (1, "lineage:1", prey)];
    let genesis = table(groups, vec![eat(amount, 1, &["lineage:1"])]);
    let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
    let before = sim.matter();
    let r = sim.execute(1, Some(2), EAT, None);
    assert_eq!(r.outcome, isocosm::simulation::Outcome::Accepted);
    // What is eaten moves and is never made or lost.
    assert_eq!((r.matter_before, r.matter_after), (before, before));
    assert_eq!(sim.matter(), before);
    let gut = ledger(&sim, 1).get(GUT).copied().unwrap_or(0);
    (ledger(&sim, 2), gut)
}

#[test]
fn eating_takes_from_every_matter_account_in_proportion() {
    // Shares of 2.5, 1.5 and 1: the unit left over goes to the larger
    // remainders, tied here, so to the first key. Energy is not matter.
    let prey = [
        ("test:a", 5),
        ("test:b", 3),
        ("test:c", 2),
        ("sim:energy", 7),
    ];
    let left = of(&[
        ("test:a", 2),
        ("test:b", 2),
        ("test:c", 1),
        ("sim:energy", 7),
    ]);
    assert_eq!(meal(&prey, 5), (left, 5));
    // Shares of 0.5, 1 and 0.5: the tie between the first and last key goes
    // to the first.
    let prey = [("test:a", 1), ("test:b", 2), ("test:c", 1)];
    let left = of(&[("test:a", 0), ("test:b", 1), ("test:c", 1)]);
    assert_eq!(meal(&prey, 2), (left, 2));
    // Shares of 0.3, 0.6 and 0.1 of one unit: the largest remainder takes it.
    let prey = [("test:a", 3), ("test:b", 6), ("test:c", 1)];
    let left = of(&[("test:a", 3), ("test:b", 5), ("test:c", 1)]);
    assert_eq!(meal(&prey, 1), (left, 1));
    // A body holding less than the meal is eaten whole.
    let prey = [("test:a", 2), ("test:c", 1)];
    let left = of(&[("test:a", 0), ("test:c", 0)]);
    assert_eq!(meal(&prey, 10), (left, 3));
}

#[test]
fn holdings_count_every_matter_account_and_nothing_else() {
    let groups: &[Group] = &[
        (1, "lineage:0", &[]),
        (
            1,
            "lineage:1",
            &[("test:a", 1), ("test:b", 1), ("sim:energy", 9)],
        ),
        (1, "lineage:1", &[("test:a", 2), ("test:c", 1)]),
    ];
    let genesis = table(groups, vec![eat(1, 3, &["lineage:1"])]);
    let mut sim = Simulation::new(genesis, Execution::Individuals).unwrap();
    let r = sim.execute(1, Some(2), EAT, None);
    assert!(matches!(
        r.outcome,
        isocosm::simulation::Outcome::Blocked(_)
    ));
    let r = sim.execute(1, Some(3), EAT, None);
    assert_eq!(r.outcome, isocosm::simulation::Outcome::Accepted);
}

#[test]
fn a_body_holding_too_little_is_not_run_for() {
    let spend = due(
        "test:spend",
        vec![Query::Holds {
            who: Binding::Actor,
            at_least: 2,
        }],
        vec![Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: GUT.into(),
            amount: 1.into(),
        }],
        None,
    );
    let groups: &[Group] = &[(3, "lineage:0", &[(GUT, 3), ("sim:energy", 9)])];
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(table(groups, vec![spend.clone()]), mode).unwrap();
        let members: Vec<u64> = (0..4)
            .map(|_| sim.advance(1).unwrap().represented)
            .collect();
        // Two ticks spend the gut down to one; the energy never counts.
        assert_eq!(members, [3, 3, 0, 0], "{mode:?}");
    }
}

fn meal_of(p: &mut Process) -> (&mut Binding, &mut Amount, &mut Key) {
    match &mut p.effects[0] {
        Effect::Eat {
            from, amount, into, ..
        } => (from, amount, into),
        _ => unreachable!("the process eats"),
    }
}

#[test]
fn rules_refuse_a_meal_of_nothing_of_a_place_or_into_what_is_not_matter() {
    let groups: &[Group] = &[(1, "lineage:0", &[])];
    let good = table(groups, vec![eat(1, 1, &["lineage:1"])]);
    good.validate().unwrap();
    let refused = |change: &dyn Fn(&mut Process)| {
        let mut g = good.clone();
        change(g.rules.processes.get_mut(EAT).unwrap());
        assert!(g.validate().is_err());
    };
    refused(&|p| *meal_of(p).1 = 0.into());
    refused(&|p| *meal_of(p).0 = Binding::Place);
    refused(&|p| *meal_of(p).0 = Binding::Actor);
    refused(&|p| *meal_of(p).2 = "sim:energy".into());
    refused(&|p| *meal_of(p).2 = "test:absent".into());
    // Every lineage a selector draws among must exist.
    refused(&|p| {
        let among = &mut p.target.as_mut().unwrap().among;
        among.insert("lineage:9".into());
    });
}

#[test]
fn selectors_without_a_draw_serialize_as_before() {
    let plain = Target {
        same_place: true,
        alive: Some(true),
        lineage: None,
        among: BTreeSet::new(),
        weighted: false,
    };
    let json = serde_json::to_string(&plain).unwrap();
    assert_eq!(json, r#"{"same_place":true,"alive":true,"lineage":null}"#);
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(back, plain);
}

/// An eater at member 1, then prey of lineage 1: one holding one, one
/// holding three, and a group of two holding one each, so members 2 to 5
/// weigh 1, 3, 1 and 1.
fn prey() -> Genesis {
    let groups: &[Group] = &[
        (1, "lineage:0", &[]),
        (1, "lineage:1", &[("test:a", 1)]),
        (1, "lineage:1", &[("test:a", 2), ("test:b", 1)]),
        (2, "lineage:1", &[("test:c", 1)]),
    ];
    table(groups, vec![eat(1, 1, &["lineage:1"])])
}

const WEIGHTS: [(Id, u128); 4] = [(2, 1), (3, 3), (4, 1), (5, 1)];

/// The member eaten in the first tick under `seed`, as the draw is keyed:
/// by the process, the tick and the actor.
fn expected(seed: u64) -> Id {
    let total: u128 = WEIGHTS.iter().map(|w| w.1).sum();
    let x = isocosm::draw(seed, &format!("target:{EAT}"), &[1, 1]);
    let mut pick = (u128::from(x) * total) >> 64;
    for (id, weight) in WEIGHTS {
        if pick < weight {
            return id;
        }
        pick -= weight;
    }
    unreachable!("the pick falls within the total")
}

fn eaten(sim: &Simulation, before: &Genesis) -> Id {
    let changed: Vec<Id> = (2..=5)
        .filter(|&id| sim.state().population.get(id) != before.population.get(id))
        .collect();
    assert_eq!(changed.len(), 1);
    changed[0]
}

#[test]
fn abundant_prey_are_eaten_more_by_a_draw_keyed_by_the_act() {
    let base = prey();
    let seeds = 2000;
    let mut counts: BTreeMap<Id, u64> = BTreeMap::new();
    for seed in 0..seeds {
        let mut genesis = base.clone();
        genesis.dynamics = Some(seed);
        let mut members = BTreeSet::new();
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut sim = Simulation::new(genesis.clone(), mode).unwrap();
            sim.advance(1).unwrap();
            members.insert(eaten(&sim, &base));
        }
        // The same member in both modes, the one the key draws.
        assert_eq!(members, BTreeSet::from([expected(seed)]), "seed {seed}");
        *counts.entry(expected(seed)).or_default() += 1;
    }
    // Each member is eaten in proportion to what it holds.
    for (id, weight) in WEIGHTS {
        let want = seeds as f64 * weight as f64 / 6.0;
        let got = counts[&id] as f64;
        assert!(
            (got - want).abs() < 0.15 * want,
            "member {id}: {got} of {want}"
        );
    }
}

#[test]
fn a_watched_process_shows_each_meal_and_what_its_prey_held() {
    let base = prey();
    for seed in 0..40 {
        let mut genesis = base.clone();
        genesis.dynamics = Some(seed);
        let mut plain = Simulation::new(genesis.clone(), Execution::Grouped).unwrap();
        plain.advance(1).unwrap();
        assert!(plain.take_watched().is_empty());
        let mut sim = Simulation::new(genesis, Execution::Grouped).unwrap();
        sim.watch(EAT);
        sim.advance(1).unwrap();
        // Watching changes nothing the world does.
        assert_eq!(sim.state_hash(), plain.state_hash(), "seed {seed}");
        let target = eaten(&sim, &base);
        let held = WEIGHTS.iter().find(|w| w.0 == target).unwrap().1;
        let act = isocosm::watch::Watched {
            tick: 1,
            process: EAT.into(),
            actor: 1,
            target: Some(target),
            count: 1,
            target_matter: held,
        };
        assert_eq!(sim.take_watched(), [act], "seed {seed}");
        assert!(sim.take_watched().is_empty());
    }
    // An advance refused part way takes its acts back with its changes.
    let mut short = base.clone();
    short.rules.limits.events_per_advance = 1;
    let mut sim = Simulation::new(short, Execution::Individuals).unwrap();
    sim.watch(EAT);
    sim.advance(1).unwrap();
    assert_eq!(sim.take_watched().len(), 1);
    assert!(sim.advance(2).is_err());
    assert!(sim.take_watched().is_empty());
}

#[test]
fn feeding_draws_the_same_members_however_equal_prey_are_grouped() {
    // Eaters of lineage 0 eat each other as well as lineage 1, so an eater
    // is sometimes drawn from within its own group; a bulk process keeps
    // the grouped runner's groups whole while the individual one splits.
    let rest = due(
        "test:rest",
        vec![Query::Alive(Binding::Actor)],
        vec![Effect::Practice {
            key: "skill:rest".into(),
            amount: 1.into(),
        }],
        None,
    );
    let groups: &[Group] = &[
        (4, "lineage:0", &[(GUT, 2)]),
        (5, "lineage:1", &[("test:a", 3), ("test:b", 1)]),
        (3, "lineage:1", &[("test:c", 2)]),
    ];
    let mut base = table(groups, vec![eat(1, 1, &["lineage:0", "lineage:1"]), rest]);
    let held = |sim: &Simulation, ids: std::ops::RangeInclusive<Id>| -> u64 {
        ids.flat_map(|id| ledger(sim, id).into_values()).sum()
    };
    for seed in 0..4 {
        base.dynamics = Some(seed);
        let mut individuals = Simulation::new(base.clone(), Execution::Individuals).unwrap();
        let mut grouped = Simulation::new(base.clone(), Execution::Grouped).unwrap();
        for tick in 1..=12 {
            individuals.advance(1).unwrap();
            grouped.advance(1).unwrap();
            let (a, b) = (individuals.state_hash(), grouped.state_hash());
            assert_eq!(a, b, "seed {seed}, tick {tick}");
        }
        let groups = |sim: &Simulation| sim.state().population.groups.len();
        assert!(groups(&grouped) < groups(&individuals), "seed {seed}");
        // The eaters hold more than the eight they began with.
        assert!(held(&grouped, 1..=4) > 8, "seed {seed}");
    }
}

#[test]
fn every_consumer_feeds_through_one_process_drawing_among_its_prey() {
    let g = Founding {
        seed: 3,
        ecology: true,
        population: 60,
        lineages: 6,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let feeds: BTreeMap<&str, &Target> = g
        .rules
        .processes
        .values()
        .filter(|p| p.id.starts_with("ecology:feed-"))
        .map(|p| (p.id.as_str(), p.target.as_ref().unwrap()))
        .collect();
    let lineages =
        |ids: &[u32]| -> BTreeSet<Key> { ids.iter().map(|i| format!("lineage:{i}")).collect() };
    // Consumers eat living producers; decomposers the dead of every other
    // lineage.
    let want = [
        ("ecology:feed-1", true, lineages(&[0, 3])),
        ("ecology:feed-2", false, lineages(&[0, 1, 3, 4, 5])),
        ("ecology:feed-4", true, lineages(&[0, 3])),
        ("ecology:feed-5", false, lineages(&[0, 1, 2, 3, 4])),
    ];
    assert_eq!(feeds.len(), want.len());
    for (id, alive, among) in want {
        let t = feeds[id];
        assert!(t.weighted && t.same_place && t.lineage.is_none(), "{id}");
        assert_eq!((t.alive, &t.among), (Some(alive), &among), "{id}");
    }
}

#[test]
fn feeding_ecologies_run_alike_grouped_and_individually() {
    for seed in 0..3 {
        let f = Founding {
            seed,
            ecology: true,
            population: 60,
            cohort_size: 6,
            sites: 2,
            lineages: 6,
            ..Default::default()
        };
        let result = isocosm::aggregate::compare(&f, 48).unwrap();
        assert_eq!(result.individuals.accepted, result.grouped.accepted);
    }
}
