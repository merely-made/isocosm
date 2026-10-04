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
    let states = |members: Vec<(&Entity, u64)>| {
        let mut v: Vec<(Entity, u64)> = members
            .into_iter()
            .map(|(e, n)| (normalize(e.clone()), n))
            .collect();
        v.sort();
        v
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
