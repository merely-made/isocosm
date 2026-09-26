// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    meaning::value,
    population::Population,
    probe::{Crowd, PredatorFounding, ProbeFounding, ProbeWorld, Variant, run_exact},
    schema::*,
};
use std::collections::BTreeMap;

const PREY: &str = "matter:0-0";
const BODY: &str = "matter:2-0";

/// One site with hungry hunters of lineage 2 holding `hunters`, biting one
/// at a time, and prey of lineage 0 holding `prey`, one member each; the
/// hunt is the only process and nothing is contested.
fn table(prey: &[u64], hunters: &[u64]) -> ProbeWorld {
    let mut w = ProbeFounding {
        seed: 5,
        ticks: 1,
        sites: [1, 1],
        predators: Some(PredatorFounding {
            hunters: [2, 2],
            bite: [1, 1],
            appetite: [3, 3],
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &mut w.genesis;
    g.rules.competitions.clear();
    g.rules.processes.retain(|id, _| id == "probe:hunt-2");
    let find = |lineage: &str| {
        let groups = g.population.groups.values();
        let mut found = groups.map(|c| &c.entity).filter(|e| e.lineage == lineage);
        found.next().unwrap().clone()
    };
    let (world, member, hunter) = (find("world:ground"), find("lineage:0"), find("lineage:2"));
    let holding = |e: &Entity, key: &str, held: u64| {
        let mut e = e.clone();
        e.accounts = BTreeMap::from([(key.to_string(), held)]);
        e
    };
    let mut population = Population::default();
    population.insert(world, 1).unwrap();
    for &held in prey {
        population.insert(holding(&member, PREY, held), 1).unwrap();
    }
    for &held in hunters {
        population.insert(holding(&hunter, BODY, held), 1).unwrap();
    }
    g.population = population;
    w
}

/// What each prey member holds after, most first, and each hunter's body.
fn outcome<'a>(members: impl Iterator<Item = (&'a Entity, u64)>) -> (Vec<u64>, Vec<u64>) {
    let (mut prey, mut hunters) = (vec![], vec![]);
    for (e, n) in members {
        let (list, key) = match e.lineage.as_str() {
            "lineage:0" => (&mut prey, PREY),
            "lineage:2" => (&mut hunters, BODY),
            _ => continue,
        };
        list.extend((0..n).map(|_| value(&e.accounts, key)));
    }
    prey.sort_unstable_by(|a, b| b.cmp(a));
    hunters.sort_unstable();
    (prey, hunters)
}

fn exact(w: &ProbeWorld, dynamics: u64) -> (Vec<u64>, Vec<u64>) {
    let run = run_exact(w, dynamics, true).unwrap();
    let groups = run.sim.state().population.groups.values();
    outcome(groups.map(|g| (&g.entity, g.count)))
}

fn crowd(w: &ProbeWorld, dynamics: u64) -> crate::Result<(Vec<u64>, Vec<u64>)> {
    let crowd = Crowd::new(w, dynamics, Variant::Histogram)?.run()?;
    Ok(outcome(crowd.bins.iter().map(|(e, &n)| (e, n))))
}

#[test]
fn the_crowd_draws_prey_as_the_core_does() {
    // Weights 3, 3 and 1. The first meal takes from a three six times in
    // seven; after it, the second takes from the untouched three, the
    // eaten one or the one in the ratio 3:2:1, and after a meal of the one
    // it takes from a three. So the prey end as 2, 2, 1 three times in
    // seven, as 3, 1, 1 twice, and as 3, 2, 0 twice.
    let w = table(&[3, 3, 1], &[0, 0]);
    let want = [
        (vec![2, 2, 1], 3.0),
        (vec![3, 1, 1], 2.0),
        (vec![3, 2, 0], 2.0),
    ];
    let seeds = 3500u64;
    for arm in ["exact", "crowd"] {
        let mut seen: BTreeMap<Vec<u64>, u64> = BTreeMap::new();
        for dynamics in 0..seeds {
            let (prey, hunters) = match arm {
                "exact" => exact(&w, dynamics),
                _ => crowd(&w, dynamics).unwrap(),
            };
            // Every hunter eats a bite.
            assert_eq!(hunters, [1, 1], "{arm}");
            *seen.entry(prey).or_default() += 1;
        }
        assert_eq!(seen.len(), want.len(), "{arm}: {seen:?}");
        for (prey, sevenths) in &want {
            let share = seen[prey] as f64 / seeds as f64;
            let expected = sevenths / 7.0;
            assert!((share - expected).abs() < 0.03, "{arm} {prey:?}: {share}");
        }
    }
}

#[test]
fn prey_running_out_part_way_feeds_alike_hunters_and_refuses_unlike_ones() {
    // One meal for two hunters: the core feeds the first in identity order.
    // Alike, either leaves the same crowd.
    let alike = table(&[1, 0], &[0, 0]);
    let fed = (vec![0, 0], vec![0, 1]);
    assert_eq!(exact(&alike, 1), fed);
    assert_eq!(crowd(&alike, 1).unwrap(), fed);
    let counted = Crowd::new(&alike, 1, Variant::Histogram).unwrap().run();
    assert_eq!(counted.unwrap().shortfalls, 1);
    // Unlike, the first is the one holding nothing, which the crowd cannot
    // know.
    let unlike = table(&[1, 0], &[0, 1]);
    assert_eq!(exact(&unlike, 1), (vec![0, 0], vec![1, 1]));
    let refused = crowd(&unlike, 1).unwrap_err();
    assert!(refused.contains("prey run out part way"), "{refused}");
    // Nothing to eat: no hunter eats, whatever their order.
    let bare = table(&[0, 0], &[0, 1]);
    let none = (vec![0, 0], vec![0, 1]);
    assert_eq!(exact(&bare, 1), none);
    assert_eq!(crowd(&bare, 1).unwrap(), none);
    let counted = Crowd::new(&bare, 1, Variant::Histogram).unwrap().run();
    assert_eq!(counted.unwrap().shortfalls, 0);
}
