// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 454: a pass reads the world as it began, and ground its takers
//! would take more of than it holds is shared out among them, each the same
//! fraction of its take, floored, the remainder staying where it was.

use isocosm::{
    Execution, Founding, Simulation, population::Population, rules::*, schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

const TAKER: &str = "test:taker";
const WANT: &str = "test:want";
const GUT: &str = "test:gut";

/// One site holding `soil`, its world's body, then members at that site in
/// identity order, each a lineage and a ledger; lineage 0's members take.
fn world(soil: u64, members: &[(&str, &[(&str, u64)])], processes: Vec<Process>) -> Genesis {
    let mut g = Founding {
        seed: 6,
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
    for (lineage, ledger) in members {
        let mut e = body.clone();
        e.lineage = (*lineage).into();
        e.accounts = ledger.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        if *lineage == "lineage:0" {
            e.traits.insert(TAKER.into());
        }
        population.insert(e, 1).unwrap();
    }
    g.population = population;
    g.sites.get_mut(&0).unwrap().accounts = BTreeMap::from([("world:soil".into(), soil)]);
    g.rules.traits.insert(TAKER.into());
    g.rules.accounts.insert(WANT.into(), AccountKind::Energy);
    for (key, lineage) in [
        ("test:a", "lineage:1"),
        ("test:b", "lineage:1"),
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
        requires: [
            vec![Query::Trait {
                who: Binding::Actor,
                key: TAKER.into(),
            }],
            requires,
        ]
        .concat(),
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

/// Each taker takes as much soil as its want says.
fn take() -> Process {
    let want = Amount::Computed(Expr::Read(Reading::Account {
        who: Binding::Actor,
        key: WANT.into(),
    }));
    due(
        "test:take",
        vec![],
        vec![Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: "world:soil".into(),
            amount: want,
        }],
        None,
    )
}

fn soil(sim: &Simulation, id: Id) -> u64 {
    let e = sim.state().population.get(id).unwrap();
    e.accounts.get("world:soil").copied().unwrap_or(0)
}

fn site_soil(sim: &Simulation) -> u64 {
    sim.state().sites[&0].accounts["world:soil"]
}

/// Takers wanting `wants`, in identity order, from `held`: what each got
/// and what the site kept.
fn taken(held: u64, wants: &[u64]) -> (Vec<u64>, u64) {
    let ledgers: Vec<[(&str, u64); 1]> = wants.iter().map(|w| [(WANT, *w)]).collect();
    let members: Vec<(&str, &[(&str, u64)])> =
        ledgers.iter().map(|l| ("lineage:0", &l[..])).collect();
    let mut results = vec![];
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut sim = Simulation::new(world(held, &members, vec![take()]), mode).unwrap();
        let before = sim.matter();
        sim.advance(1).unwrap();
        assert_eq!(sim.matter(), before, "{mode:?}");
        let got = (1..=wants.len() as Id).map(|id| soil(&sim, id)).collect();
        results.push((got, site_soil(&sim)));
    }
    assert_eq!(results[0], results[1], "both modes share alike");
    results.remove(0)
}

#[test]
fn a_short_site_gives_each_taker_the_same_fraction_floored() {
    // Twelve asked of ten: each gets two and a half, floored, and the two
    // left over stay in the ground.
    assert_eq!(taken(10, &[3, 3, 3, 3]), (vec![2, 2, 2, 2], 2));
    // Takes of three and one: each gets three quarters of its take,
    // floored, so the small takers get nothing.
    assert_eq!(taken(6, &[3, 3, 1, 1]), (vec![2, 2, 0, 0], 2));
    // The control: a site holding enough gives every take whole, the last
    // taker included, as identity order once did only for the first.
    assert_eq!(taken(12, &[3, 3, 3, 3]), (vec![3, 3, 3, 3], 0));
    assert_eq!(taken(20, &[3, 3, 1, 1]), (vec![3, 3, 1, 1], 12));
}

#[test]
fn identity_order_decides_no_share() {
    let (forward, kept) = taken(7, &[5, 4, 2, 1]);
    let (backward, kept_too) = taken(7, &[1, 2, 4, 5]);
    // The same takes in the other order get the same shares.
    let reversed: Vec<u64> = backward.into_iter().rev().collect();
    assert_eq!((forward, kept), (reversed, kept_too));
}

/// Hunters of lineage 0 biting `bite` of one weighted prey of lineage 1.
fn hunt(bite: u64) -> Process {
    due(
        "test:hunt",
        vec![Query::Holds {
            who: Binding::Target,
            at_least: 1,
        }],
        vec![Effect::Eat {
            from: Binding::Target,
            amount: bite.into(),
            into: GUT.into(),
            of: vec![],
        }],
        Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: None,
            among: BTreeSet::from(["lineage:1".into()]),
            weighted: true,
        }),
    )
}

fn ledger(sim: &Simulation, id: Id) -> Ledger {
    let mut l = sim.state().population.get(id).unwrap().accounts.clone();
    l.retain(|_, v| *v > 0);
    l
}

fn of(entries: &[(&str, u64)]) -> Ledger {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn a_prey_too_small_for_its_eaters_gives_each_the_same_fraction() {
    let prey: &[(&str, u64)] = &[("test:a", 6), ("test:b", 4)];
    let hunters = [("lineage:0", &[][..]); 3];
    let members: Vec<(&str, &[(&str, u64)])> = hunters
        .iter()
        .copied()
        .chain([("lineage:1", prey)])
        .collect();
    let mut sim =
        Simulation::new(world(0, &members, vec![hunt(4)]), Execution::Individuals).unwrap();
    let before = sim.matter();
    sim.advance(1).unwrap();
    assert_eq!(sim.matter(), before);
    // Twelve asked of ten: each hunter gets three, and the prey gives nine
    // as one share of its two accounts, keeping the one left over.
    for id in 1..=3 {
        assert_eq!(ledger(&sim, id), of(&[(GUT, 3)]), "hunter {id}");
    }
    assert_eq!(ledger(&sim, 4), of(&[("test:a", 1)]));
    // The control: a lone hunter takes its whole bite.
    let alone: Vec<(&str, &[(&str, u64)])> = vec![("lineage:0", &[]), ("lineage:1", prey)];
    let mut sim = Simulation::new(world(0, &alone, vec![hunt(4)]), Execution::Individuals).unwrap();
    sim.advance(1).unwrap();
    assert_eq!(ledger(&sim, 1), of(&[(GUT, 4)]));
    assert_eq!(ledger(&sim, 2), of(&[("test:a", 4), ("test:b", 2)]));
}

/// Givers of lineage 0 giving `gift` of their gut to one weighted member of
/// lineage 1: a draw that takes nothing, so its pass is not planned.
fn give(gift: u64) -> Process {
    let mut p = hunt(1);
    p.id = "test:give".into();
    p.effects = vec![Effect::Transfer {
        from: Binding::Actor,
        to: Binding::Target,
        account: GUT.into(),
        amount: gift.into(),
    }];
    p
}

/// Each actor's draw with every actor of `process` acting, and alone.
fn draws(process: Process, gut: u64) -> BTreeSet<Option<Id>> {
    // Six actors, then members of uneven holdings, so an act moves the
    // weights a later actor would draw by.
    let prey: [&[(&str, u64)]; 4] = [
        &[("test:a", 9)],
        &[("test:a", 2)],
        &[("test:b", 5)],
        &[("test:a", 1), ("test:b", 1)],
    ];
    let actor: &[(&str, u64)] = &[(GUT, gut)];
    let mut members: Vec<(&str, &[(&str, u64)])> = vec![("lineage:0", actor); 6];
    members.extend(prey.iter().map(|p| ("lineage:1", *p)));
    let mut targets = BTreeSet::new();
    let id = process.id.clone();
    for seed in 0..24 {
        let mut all = world(0, &members, vec![process.clone()]);
        all.dynamics = Some(seed);
        let mut sim = Simulation::new(all.clone(), Execution::Individuals).unwrap();
        sim.watch(&id);
        sim.advance(1).unwrap();
        let drawn: BTreeMap<Id, Option<Id>> = sim
            .take_watched()
            .into_iter()
            .map(|w| (w.actor, w.target))
            .collect();
        assert_eq!(drawn.len(), 6, "seed {seed}");
        for one in 1..=6 {
            // The same actor alone, the others kept but not acting.
            let mut alone = all.clone();
            for (&first, group) in alone.population.groups.iter_mut() {
                if first != one && group.entity.lineage == "lineage:0" {
                    group.entity.traits.remove(TAKER);
                }
            }
            let mut sim = Simulation::new(alone, Execution::Individuals).unwrap();
            sim.watch(&id);
            sim.advance(1).unwrap();
            let watched = sim.take_watched();
            assert_eq!(watched[0].target, drawn[&one], "seed {seed}, actor {one}");
            targets.insert(drawn[&one]);
        }
    }
    targets
}

#[test]
fn every_draw_of_a_pass_reads_the_pass_start() {
    // Hunters, planned before anything is eaten, and givers, drawing at
    // their turn after earlier gifts, draw as each would alone. The
    // control: the draws differ, so agreeing is not agreeing on one.
    assert!(draws(hunt(3), 0).len() > 2);
    assert!(draws(give(40), 40).len() > 2);
}
