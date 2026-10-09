// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    meaning::value,
    population::Population,
    probe::{
        Crowd, MealLog, Meals, PredatorFounding, ProbeFounding, ProbeWorld, Variant, run_exact,
    },
    schema::*,
};
use std::collections::BTreeMap;

const PREY: &str = "matter:0-0";
const BODY: &str = "matter:2-0";

/// One site with hungry hunters of lineage 2 holding `hunters`, biting
/// `bite` at a time, and prey of lineage 0 holding `prey`, one member each;
/// the hunt is the only process and nothing is contested.
fn table(prey: &[u64], hunters: &[u64], bite: u64) -> ProbeWorld {
    let mut w = ProbeFounding {
        seed: 5,
        ticks: 1,
        sites: [1, 1],
        predators: Some(PredatorFounding {
            hunters: [2, 2],
            bite: [bite, bite],
            appetite: [3, 3],
            fat: [0, 0],
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

/// What each prey member holds after, most first, each hunter's body, and
/// the hunt's meals.
type Outcome = (Vec<u64>, Vec<u64>, Meals);

fn outcome<'a>(members: impl Iterator<Item = (&'a Entity, u64)>, meals: &MealLog) -> Outcome {
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
    let meals = meals.get("probe:hunt-2").copied().unwrap_or_default();
    (prey, hunters, meals)
}

fn exact(w: &ProbeWorld, dynamics: u64) -> Outcome {
    let run = run_exact(w, dynamics, true).unwrap();
    let groups = run.sim.state().population.groups.values();
    outcome(groups.map(|g| (&g.entity, g.count)), &run.meals)
}

fn crowd(w: &ProbeWorld, dynamics: u64) -> crate::Result<Outcome> {
    let crowd = Crowd::new(w, dynamics, Variant::Histogram)?.run()?;
    Ok(outcome(
        crowd.bins.iter().map(|(e, &n)| (e, n)),
        &crowd.meals,
    ))
}

fn meals(count: u64, held: u128) -> Meals {
    Meals { count, held }
}

#[test]
fn the_crowd_draws_prey_as_the_core_does() {
    // Weights 3, 3 and 1, each hunter drawing from the pass's start (ruling
    // 454): a three twice in 18/49, the two threes in 18/49, a three and
    // the one in 12/49, and the one twice in 1/49, which two bites of one
    // share out to nothing each, the one staying.
    let w = table(&[3, 3, 1], &[0, 0], 1);
    let want = [
        (vec![3, 1, 1], 18.0, vec![1, 1], meals(2, 6)),
        (vec![2, 2, 1], 18.0, vec![1, 1], meals(2, 6)),
        (vec![3, 2, 0], 12.0, vec![1, 1], meals(2, 4)),
        (vec![3, 3, 1], 1.0, vec![0, 0], meals(2, 2)),
    ];
    let seeds = 4900u64;
    for arm in ["exact", "crowd"] {
        let mut seen: BTreeMap<Vec<u64>, u64> = BTreeMap::new();
        for dynamics in 0..seeds {
            let (prey, hunters, fed) = match arm {
                "exact" => exact(&w, dynamics),
                _ => crowd(&w, dynamics).unwrap(),
            };
            let ended = want.iter().find(|w| w.0 == prey).expect("a possible end");
            assert_eq!((&hunters, fed), (&ended.2, ended.3), "{arm} {prey:?}");
            *seen.entry(prey).or_default() += 1;
        }
        assert_eq!(seen.len(), want.len(), "{arm}: {seen:?}");
        for (prey, in_49, _, _) in &want {
            let share = seen[prey] as f64 / seeds as f64;
            let expected = in_49 / 49.0;
            assert!((share - expected).abs() < 0.025, "{arm} {prey:?}: {share}");
        }
    }
}

#[test]
fn a_prey_too_small_for_its_hunters_is_shared_alike_in_both_runners() {
    // One bite for two hunters: each gets half of it, floored, so neither
    // eats, the prey keeps its one, and nothing is refused, whatever the
    // hunters hold.
    for hunters in [[0, 0], [0, 1]] {
        let w = table(&[1, 0], &hunters, 1);
        let shared = (vec![1, 0], hunters.to_vec(), meals(2, 2));
        assert_eq!(exact(&w, 1), shared, "{hunters:?}");
        assert_eq!(crowd(&w, 1).unwrap(), shared, "{hunters:?}");
        let counted = Crowd::new(&w, 1, Variant::Histogram).unwrap().run();
        assert_eq!(counted.unwrap().shortfalls, 1);
    }
    // Two bites of two from a prey of three: three quarters each, floored,
    // one each, the prey keeping one.
    let w = table(&[3], &[0, 0], 2);
    let shared = (vec![1], vec![1, 1], meals(2, 6));
    assert_eq!(exact(&w, 1), shared);
    assert_eq!(crowd(&w, 1).unwrap(), shared);
    // Nothing to eat: no hunter eats, and none is short.
    let bare = table(&[0, 0], &[0, 1], 1);
    let none = (vec![0, 0], vec![0, 1], Meals::default());
    assert_eq!(exact(&bare, 1), none);
    assert_eq!(crowd(&bare, 1).unwrap(), none);
    let counted = Crowd::new(&bare, 1, Variant::Histogram).unwrap().run();
    assert_eq!(counted.unwrap().shortfalls, 0);
}

/// One producer of a single frond holding `held`, one grazer with its stores
/// full and the graze the only process: its bite takes the frond whole where
/// it would take all of it and the crossing allows (516), and bites it
/// otherwise.
fn grazing(seed: u64, held: u64) -> ProbeWorld {
    use crate::{
        development::{Soma, develop},
        probe::BodyFounding,
    };
    let mut w = BodyFounding {
        seed,
        ticks: 1,
        sites: [1, 1],
        ..Default::default()
    }
    .generate()
    .unwrap();
    let g = &mut w.genesis;
    g.rules.processes.retain(|id, _| id == "body:graze-1");
    let rules = g.rules.clone();
    let find = |lineage: &str| {
        let groups = g.population.groups.values();
        let mut found = groups.map(|c| &c.entity).filter(|e| e.lineage == lineage);
        found.next().unwrap().clone()
    };
    let (world, mut frond, mut grazer) =
        (find("world:ground"), find("lineage:0"), find("lineage:1"));
    let d = g.lineages["lineage:0"].development.clone().unwrap();
    let soma = Soma {
        segments: vec![1],
        absent: vec![],
        seed: 0,
    };
    frond.parts = develop(&rules, &d, &soma).unwrap();
    frond.soma = vec![1];
    frond.parts.get_mut(&0).unwrap().matter = BTreeMap::from([("tissue:0".into(), held)]);
    let room = crate::anatomy::room(&grazer, &rules, "reserve:1");
    crate::anatomy::give(&mut grazer, &rules, "reserve:1", room)
        .unwrap()
        .unwrap();
    let mut population = Population::default();
    population.insert(world, 1).unwrap();
    population.insert(frond, 1).unwrap();
    population.insert(grazer, 1).unwrap();
    g.population = population;
    w
}

/// Both runners graze a frond alike, taken whole or bitten: the same states
/// and the same site, across worlds whose drawn domains let the frond land
/// as it was, expressing nothing, or not at all.
#[test]
fn a_whole_meal_lands_alike_in_crowd_and_core() {
    use crate::probe::readings::{crowd_members, exact_members};
    // Members per state, however each runner groups them.
    let states = |members: Vec<(&Entity, u64)>| {
        let mut v: std::collections::BTreeMap<Entity, u64> = Default::default();
        for (e, n) in members {
            *v.entry(crate::probe::aggregate::normalize(e.clone()))
                .or_default() += n;
        }
        v.into_iter().collect::<Vec<_>>()
    };
    let (mut whole, mut bitten) = (0, 0);
    for seed in 0..12 {
        for held in [1, 400] {
            let w = grazing(seed, held);
            let exact = run_exact(&w, 3, true).unwrap();
            let crowd = Crowd::new(&w, 3, Variant::Histogram)
                .unwrap()
                .run()
                .unwrap();
            let a = states(exact_members(&exact));
            assert_eq!(
                a,
                states(crowd_members(&crowd)),
                "seed {seed}, frond {held}"
            );
            assert_eq!(exact.sim.state().sites, crowd.sites, "seed {seed}");
            let grazer = a.iter().find(|(e, _)| e.lineage == "lineage:1").unwrap();
            let took = grazer
                .0
                .parts
                .values()
                .any(|p| p.matter.contains_key("tissue:0"));
            let prey = a.iter().find(|(e, _)| e.lineage == "lineage:0").unwrap();
            let left = prey.0.parts.get(&0).map(|p| value(&p.matter, "tissue:0"));
            whole += usize::from(took);
            bitten += usize::from(left.is_some_and(|t| t < held));
        }
    }
    assert!(whole > 0 && bitten > 0, "{whole} whole, {bitten} bitten");
}
