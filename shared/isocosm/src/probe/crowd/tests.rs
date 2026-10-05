// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    population::Population,
    probe::{BodyFounding, Crowd, ProbeWorld, Variant, aggregate::normalize, readings, run_exact},
    schema::*,
};

/// One site of a producer, its parts and provision full, and two grazers,
/// every provision full, one
/// grazer's stores full and the other's empty, and only the births running;
/// with no variance and no absences every child's soma is its recipe's.
fn breeding(seed: u64) -> ProbeWorld {
    let mut w = BodyFounding {
        seed,
        ticks: 1,
        sites: [1, 1],
        variance: [0, 0],
        absence: [0, 0],
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &mut w.genesis;
    g.rules
        .processes
        .retain(|id, _| id.starts_with("body:bear") || id.starts_with("body:bud"));
    let rules = g.rules.clone();
    let find = |lineage: &str| {
        let groups = g.population.groups.values();
        let mut found = groups.map(|c| &c.entity).filter(|e| e.lineage == lineage);
        found.next().unwrap().clone()
    };
    let fill = |e: &mut Entity, key: &str| {
        let room = crate::anatomy::room(e, &rules, key);
        crate::anatomy::give(e, &rules, key, room).unwrap().unwrap();
    };
    let (world, mut frond, mut fed) = (find("world:ground"), find("lineage:0"), find("lineage:1"));
    fill(&mut frond, "tissue:0");
    fill(&mut frond, "provision:0");
    fill(&mut fed, "provision:1");
    let hungry = fed.clone();
    fill(&mut fed, "reserve:1");
    let mut population = Population::default();
    for e in [world, frond, fed, hungry] {
        population.insert(e, 1).unwrap();
    }
    g.population = population;
    w
}

/// Both runners make the same children of a bud, a brood and a clutch.
#[test]
fn births_come_alike_in_crowd_and_core() {
    // Members per state, however each runner groups them.
    let states = |members: Vec<(&Entity, u64)>| {
        let mut v: std::collections::BTreeMap<Entity, u64> = Default::default();
        for (e, n) in members {
            *v.entry(normalize(e.clone())).or_default() += n;
        }
        v.into_iter().collect::<Vec<_>>()
    };
    let (mut buds, mut broods, mut eggs) = (0, 0, 0);
    for seed in 0..8 {
        let w = breeding(seed);
        let exact = run_exact(&w, 3, true).unwrap();
        let crowd = Crowd::new(&w, 3, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let a = states(readings::exact_members(&exact));
        assert_eq!(a, states(readings::crowd_members(&crowd)), "seed {seed}");
        for (e, n) in &a {
            if e.born == 1 {
                match (e.lineage.as_str(), e.parts.len()) {
                    ("lineage:0", _) => buds += n,
                    (_, 1) => eggs += n,
                    _ => broods += n,
                }
            }
        }
    }
    assert!(
        buds > 0 && broods > 0 && eggs > 0,
        "{buds} buds, {broods} broods, {eggs} eggs"
    );
}

/// One site of a producer of one frond holding plenty and two alike fed,
/// iteroparous grazers, their tissue, stores and provision full: each
/// broods, grazes its provision full again and nurses its own unweaned
/// young, not the other's, who grazes too until weaned (554).
fn nursing(seed: u64) -> ProbeWorld {
    use crate::development::{Soma, develop};
    let mut w = BodyFounding {
        seed,
        ticks: 4,
        sites: [1, 1],
        variance: [0, 0],
        absence: [0, 0],
        semelparous: 0,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &mut w.genesis;
    let kept = ["body:graze-1", "body:bear-1", "body:nurse-1", "body:wean-1"];
    g.rules
        .processes
        .retain(|id, _| kept.contains(&id.as_str()));
    let rules = g.rules.clone();
    let find = |lineage: &str| {
        let groups = g.population.groups.values();
        let mut found = groups.map(|c| &c.entity).filter(|e| e.lineage == lineage);
        found.next().unwrap().clone()
    };
    let (world, mut frond, mut parent) =
        (find("world:ground"), find("lineage:0"), find("lineage:1"));
    let d = g.lineages["lineage:0"].development.clone().unwrap();
    let soma = Soma {
        segments: vec![1],
        absent: vec![],
        seed: 0,
    };
    frond.parts = develop(&rules, &d, &soma).unwrap();
    frond.soma = vec![1];
    frond.parts.get_mut(&0).unwrap().matter =
        std::collections::BTreeMap::from([("tissue:0".into(), 10_000)]);
    for key in ["tissue:1", "reserve:1", "provision:1"] {
        let room = crate::anatomy::room(&parent, &rules, key);
        crate::anatomy::give(&mut parent, &rules, key, room)
            .unwrap()
            .unwrap();
    }
    let mut population = Population::default();
    for (e, n) in [(world, 1), (frond, 1), (parent, 2)] {
        population.insert(e, n).unwrap();
    }
    g.population = population;
    w
}

/// Both runners nurse alike: the crowd holds the parent and its unweaned
/// young as kin, relation kept, and returns them to its counts once the
/// young is weaned, ending in the same states as the core; milk flows.
#[test]
fn milk_flows_alike_in_crowd_and_core() {
    let states = |members: Vec<(&Entity, u64)>| {
        let mut v: std::collections::BTreeMap<Entity, u64> = Default::default();
        for (e, n) in members {
            *v.entry(normalize(e.clone())).or_default() += n;
        }
        v.into_iter().collect::<Vec<_>>()
    };
    let mut milk = 0;
    for seed in 0..6 {
        let w = nursing(seed);
        let exact = run_exact(&w, 3, true).unwrap();
        let crowd = Crowd::new(&w, 3, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let a = states(readings::exact_members(&exact));
        assert_eq!(a, states(readings::crowd_members(&crowd)), "seed {seed}");
        let mut genesis = w.genesis.clone();
        genesis.dynamics = Some(3);
        let mut sim = crate::Simulation::new(genesis, crate::Execution::Individuals).unwrap();
        sim.watch("body:nurse-1");
        sim.advance(w.ticks).unwrap();
        milk += sim.take_watched().len();
    }
    assert!(milk > 0, "no milk flowed");
}

/// A crowd bite takes what its part held of its share (459): a share of
/// 8 mg landing on a root frond holding 3, with a living frond beyond it so
/// it cannot be taken whole, takes 3.
#[test]
fn a_crowd_bite_takes_what_its_part_held() {
    use crate::{
        development::{Soma, develop},
        probe::aggregate::{self, Took},
    };
    use std::collections::BTreeMap;
    let w = BodyFounding {
        seed: 1,
        sites: [1, 1],
        variance: [0, 0],
        absence: [0, 0],
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &w.genesis;
    let rules = &g.rules;
    let find = |lineage: &str| {
        let groups = g.population.groups.values();
        let mut found = groups.map(|c| &c.entity).filter(|e| e.lineage == lineage);
        found.next().unwrap().clone()
    };
    let (mut prey, mut grazer) = (find("lineage:0"), find("lineage:1"));
    let d = g.lineages["lineage:0"].development.clone().unwrap();
    let soma = Soma {
        segments: vec![2],
        absent: vec![],
        seed: 0,
    };
    prey.parts = develop(rules, &d, &soma).unwrap();
    for (part, held) in [(0, 3), (1, 10)] {
        prey.parts.get_mut(&part).unwrap().matter = BTreeMap::from([("tissue:0".into(), held)]);
    }
    let room = crate::anatomy::room(&grazer, rules, "reserve:1");
    crate::anatomy::give(&mut grazer, rules, "reserve:1", room)
        .unwrap()
        .unwrap();
    let site = g.sites[&grazer.place].clone();
    let portion = BTreeMap::from([("tissue:0".to_string(), 8)]);
    let draws = BTreeMap::new();
    let a = aggregate::Act {
        start: &site,
        count: 1,
        tick: 1,
        rules,
        lineages: Some(&g.lineages),
        draws: &draws,
        meal: Some((&prey, &portion, Some(0))),
    };
    let p = &rules.processes["body:graze-1"];
    let acted = aggregate::act(p, &grazer, &mut site.clone(), aggregate::Run::Free, a)
        .unwrap()
        .expect("the meal is accepted");
    let Some(Took::Bite(taken)) = acted.took else {
        panic!("a bite, not a part whole: {:?}", acted.took);
    };
    assert_eq!(taken, BTreeMap::from([("tissue:0".to_string(), 3)]));
}
