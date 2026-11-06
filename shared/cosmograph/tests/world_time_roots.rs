// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1114: an ordinary world-time producer keeps its protected store in
//! native body parts, alongside the exposed tissue that grazing can remove.

use cosmograph::{
    Command, Execution, Founding, Session,
    bodied::Bodies,
    ecology::{Rates, Web},
    rules::{AccountKind, Rules},
    schema::{Entity, Id, PartId},
    simulation::{Genesis, Receipt},
};
use std::sync::Arc;

#[path = "world_time_roots/controls.rs"]
mod controls;

fn founding(seed: u64) -> Genesis {
    let grid = cosmograph::map::Grid {
        width: 2,
        height: 2,
        ..cosmograph::map::Grid::drawn(seed)
    };
    Founding {
        seed,
        sites: 4,
        population: 60,
        cohort_size: 1,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        web: Web::Diets,
        bodies: Some(Bodies::default()),
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

fn tissue(e: &Entity, rules: &Rules) -> String {
    e.parts
        .values()
        .flat_map(|part| part.matter.keys())
        .chain(e.accounts.keys())
        .find(|key| {
            matches!(rules.accounts.get(*key),
            Some(AccountKind::Matter { lineage, reserve: false, provision: false })
                if lineage != "world:ground")
        })
        .expect("the native producer carries its actual tissue account")
        .clone()
}

fn store(e: &Entity, id: PartId) -> bool {
    e.parts[&id]
        .cells
        .get("function:store")
        .copied()
        .unwrap_or(0)
        > 0
}

fn allocation(e: &Entity, rules: &Rules) -> (u64, u64) {
    let key = tissue(e, rules);
    e.parts
        .iter()
        .fold((0, 0), |(stored, exposed), (&id, part)| {
            let held = part.matter.get(&key).copied().unwrap_or(0);
            if store(e, id) {
                (stored + held, exposed)
            } else {
                (stored, exposed + held)
            }
        })
}

/// A controlled loss of tissue in the founding, before native conservation is
/// established. Every later change is an ordinary admitted action.
fn fixture_genesis(stored: u64, exposed: u64) -> (Genesis, Id, Id) {
    let mut g = founding(9);
    let plant = *g
        .population
        .groups
        .iter()
        .find(|(_, group)| {
            group.entity.lineage == "lineage:0"
                && g.started.saturating_sub(group.entity.born) >= 30 * 1440
        })
        .unwrap()
        .0;
    let grazer = *g
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.lineage == "lineage:1")
        .unwrap()
        .0;
    let source = g.population.get(plant).unwrap().clone();
    let key = tissue(&source, &g.rules);
    let roots: Vec<_> = source
        .parts
        .keys()
        .copied()
        .filter(|&id| store(&source, id))
        .collect();
    assert_eq!(roots.len(), 1, "one native root store");
    let foliage: Vec<_> = source
        .parts
        .iter()
        .filter(|(id, _)| !store(&source, **id))
        .map(|(&id, part)| (id, part.matter.get(&key).copied().unwrap_or(0)))
        .collect();
    let total: u64 = foliage.iter().map(|(_, n)| n).sum();
    assert_eq!(total, 40);
    let mut shares: Vec<_> = foliage
        .iter()
        .map(|&(id, n)| (id, n * exposed / total))
        .collect();
    let mut remaining = exposed - shares.iter().map(|(_, n)| n).sum::<u64>();
    for ((_, held), (_, capacity)) in shares.iter_mut().zip(&foliage) {
        let give = remaining.min(capacity.saturating_sub(*held));
        *held += give;
        remaining -= give;
    }
    assert_eq!(remaining, 0);
    let e = g.population.groups.get_mut(&plant).unwrap().entity_mut();
    e.accounts.remove(&key);
    for part in e.parts.values_mut() {
        Arc::make_mut(part).matter.remove(&key);
    }
    e.part_mut(&roots[0])
        .unwrap()
        .matter
        .insert(key.clone(), stored);
    for (id, held) in shares {
        e.part_mut(&id).unwrap().matter.insert(key.clone(), held);
    }
    let place = e.place;
    g.population
        .groups
        .get_mut(&grazer)
        .unwrap()
        .entity_mut()
        .place = place;
    (g, plant, grazer)
}

fn fixture(stored: u64, exposed: u64, mode: Execution) -> (Session, Id, Id) {
    let (g, plant, grazer) = fixture_genesis(stored, exposed);
    (Session::new(g, mode).unwrap(), plant, grazer)
}

fn act(s: &mut Session, actor: Id, target: Option<Id>, process: &str) -> Receipt {
    let before = s.sim.matter();
    let receipt: Receipt = serde_json::from_str(
        &s.command(Command::Act {
            actor,
            target,
            process: process.into(),
            cause: None,
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        (receipt.matter_before, receipt.matter_after),
        (before, before),
        "{receipt:?}"
    );
    assert_eq!(s.sim.matter(), before);
    receipt
}

fn at(s: &Session, plant: Id) -> (u64, u64) {
    allocation(
        s.sim.state().population.get(plant).unwrap(),
        &s.sim.genesis().rules,
    )
}

#[test]
fn world_time_producers_start_with_twenty_in_store_and_forty_exposed() {
    for seed in [9, 3, 7, 11, 5] {
        let g = founding(seed);
        let mut producers = 0;
        for group in g
            .population
            .groups
            .values()
            .filter(|group| group.entity.alive && group.entity.kingdom == "kingdom:flora")
        {
            let e = &group.entity;
            assert!(e.body.is_some(), "seed {seed}: producer is bodied");
            let key = tissue(e, &g.rules);
            let (mut stored, mut exposed) = (0, 0);
            for part in e.parts.values() {
                let held = part.matter.get(&key).copied().unwrap_or(0);
                if part.cells.get("function:store").copied().unwrap_or(0) > 0 {
                    stored += held;
                } else {
                    exposed += held;
                }
            }
            assert_eq!(
                (stored, exposed),
                (20, 40),
                "seed {seed}, {}: adult tissue is held in the root store and exposed parts",
                e.lineage
            );
            producers += group.count;
        }
        assert!(producers > 0, "seed {seed}: the draw contains producers");
    }
}

#[test]
fn root_regrowth_moves_two_into_denuded_parts_and_replays() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, plant, _) = fixture(20, 0, mode);
        let before = s.sim.matter();
        for n in 1..=10 {
            let r = act(&mut s, plant, None, "ecology:regrow-0");
            assert!(r.accepted(), "regrowth {n}, {mode:?}: {r:?}");
            assert_eq!(at(&s, plant), (20 - n * 2, n * 2));
        }
        let r = act(&mut s, plant, None, "ecology:regrow-0");
        assert!(!r.accepted(), "an empty root cannot create regrowth");
        assert_eq!(s.sim.matter(), before);
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
        let fork = s
            .fork_at(s.sim.state().tick, "branch:roots".into())
            .unwrap();
        assert_eq!(at(&fork, plant), at(&s, plant));
        assert_eq!(fork.sim.matter(), before);
    }
}

#[test]
fn regrowth_runs_on_the_plants_own_240_minute_clock() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, plant, _) = fixture_genesis(20, 0);
        // Keep the actual generated regrowth and death definitions; other
        // organisms cannot graze this timing control while it waits.
        g.rules
            .processes
            .retain(|id, _| id == "ecology:regrow-0" || id == "ecology:death-0");
        g.population
            .groups
            .get_mut(&plant)
            .unwrap()
            .entity_mut()
            .born = g.started;
        let unit = g.rules.tick_microseconds();
        let period = cosmograph::rules::minutes_ticks(240, unit);
        assert_eq!(g.rules.processes["ecology:regrow-0"].period, Some(period));
        let mut s = Session::new(g, mode).unwrap();
        let before = s.sim.matter();
        s.advance(period - 1).unwrap();
        assert_eq!(
            at(&s, plant),
            (20, 0),
            "the body's first due tick has not arrived"
        );
        s.advance(1).unwrap();
        assert_eq!(at(&s, plant), (18, 2));
        assert_eq!(s.sim.matter(), before);
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn production_fills_exposed_tissue_before_the_root_store() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for (exposed, expected) in [(0, (10, 2)), (40, (12, 40))] {
            let (mut s, plant, _) = fixture(10, exposed, mode);
            let r = act(&mut s, plant, None, "ecology:produce-0");
            assert!(r.accepted(), "{mode:?}: {r:?}");
            assert_eq!(at(&s, plant), expected);
            assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
        }
    }
}

#[test]
fn growth_and_regrowth_repair_the_most_wounded_exposed_part() {
    for process in ["ecology:produce-0", "ecology:regrow-0"] {
        let (mut g, plant, _) = fixture_genesis(20, 40);
        let e = g.population.get(plant).unwrap();
        let key = tissue(e, &g.rules);
        let victim = e
            .parts
            .iter()
            .filter(|(id, _)| !store(e, **id))
            .max_by_key(|(_, p)| p.matter.get(&key).copied().unwrap_or(0))
            .map(|(&id, _)| id)
            .unwrap();
        let others: std::collections::BTreeMap<_, _> = e
            .parts
            .iter()
            .filter(|(id, _)| !store(e, **id) && **id != victim)
            .map(|(&id, p)| (id, p.matter.get(&key).copied().unwrap_or(0)))
            .collect();
        g.population
            .groups
            .get_mut(&plant)
            .unwrap()
            .entity_mut()
            .part_mut(&victim)
            .unwrap()
            .matter
            .insert(key.clone(), 0);
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        assert!(act(&mut s, plant, None, process).accepted());
        let e = s.sim.state().population.get(plant).unwrap();
        assert_eq!(
            e.parts[&victim].matter.get(&key).copied().unwrap_or(0),
            2,
            "{process}"
        );
        for (id, held) in others {
            assert_eq!(
                e.parts[&id].matter.get(&key).copied().unwrap_or(0),
                held,
                "{process}, {id:?}"
            );
        }
    }
}

#[test]
fn maintenance_and_one_native_seed_draw_only_on_the_root() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, plant, _) = fixture(20, 40, mode);
        assert!(act(&mut s, plant, None, "ecology:upkeep-0").accepted());
        assert_eq!(at(&s, plant), (19, 40));

        let (mut s, plant, _) = fixture(30, 40, mode);
        let existing: std::collections::BTreeSet<_> =
            s.sim.state().population.groups.keys().copied().collect();
        let r = act(&mut s, plant, None, "ecology:birth-0");
        assert!(r.accepted(), "{mode:?}: {r:?}");
        assert_eq!(at(&s, plant), (20, 40));
        let children: Vec<_> = s
            .sim
            .state()
            .population
            .groups
            .iter()
            .filter(|(id, group)| !existing.contains(id) && group.entity.lineage == "lineage:0")
            .map(|(_, group)| group)
            .collect();
        assert_eq!(
            children.iter().map(|g| g.count).sum::<u64>(),
            1,
            "one native seed"
        );
        let child = &children[0].entity;
        assert!(child.body.is_some());
        let (stored, exposed) = allocation(child, &s.sim.genesis().rules);
        assert_eq!(
            stored + exposed,
            10,
            "the child's matter is what its parent paid"
        );
        assert!(stored > 0, "a new plant retains its native root store");
        let child = *s
            .sim
            .state()
            .population
            .groups
            .iter()
            .find(|(id, group)| !existing.contains(id) && group.entity.lineage == "lineage:0")
            .unwrap()
            .0;
        let before = at(&s, child);
        let r = act(&mut s, child, None, "ecology:produce-0");
        assert!(
            r.accepted(),
            "a newly seeded body can photosynthesize: {r:?}"
        );
        let after = at(&s, child);
        assert_eq!(after.0 + after.1, before.0 + before.1 + 2);
        assert!(after.1 > before.1, "the seed's exposed parts can grow");
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn a_plant_dies_only_when_every_part_is_empty() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for (stored, exposed, dies) in [(20, 0, false), (0, 20, false), (0, 0, true)] {
            let (mut s, plant, _) = fixture(stored, exposed, mode);
            let r = act(&mut s, plant, None, "ecology:death-0");
            assert_eq!(r.accepted(), dies, "{stored}/{exposed}, {mode:?}: {r:?}");
            assert_eq!(s.sim.state().population.get(plant).unwrap().alive, !dies);
        }
    }
}

#[test]
fn repeated_native_grazing_wounds_exposed_parts_without_eating_the_root() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, plant, grazer) = fixture(20, 40, mode);
        let mut meals = 0;
        for _ in 0..100 {
            // Rent keeps this real grazer below satiety without discarding its
            // meals or bypassing the body's native accounting.
            act(&mut s, grazer, None, "ecology:upkeep-1");
            if act(&mut s, grazer, Some(plant), "ecology:feed-1").accepted() {
                meals += 1;
            }
            assert_eq!(
                at(&s, plant).0,
                20,
                "every ordinary bite leaves the root alone"
            );
            assert!(s.sim.state().population.get(plant).unwrap().alive);
        }
        assert!(meals > 0, "the control must actually graze");
        assert!(at(&s, plant).1 < 40, "grazing removes exposed tissue");
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn a_denuded_living_plant_never_offers_its_root_as_a_fallback_meal() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, plant, grazer) = fixture(20, 0, mode);
        let r = act(&mut s, grazer, Some(plant), "ecology:feed-1");
        assert!(
            !r.accepted(),
            "a grazing mouth cannot reach the protected root: {r:?}"
        );
        assert_eq!(at(&s, plant), (20, 0));
        assert!(s.sim.state().population.get(plant).unwrap().alive);
        assert!(act(&mut s, plant, None, "ecology:produce-0").accepted());
        assert_eq!(
            at(&s, plant),
            (20, 2),
            "the root-only plant can produce new exposed tissue"
        );
    }
}

#[test]
fn a_speciated_plant_keeps_root_routing_and_native_seed_growth() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut s, plant, _) = fixture(20, 0, mode);
        let line = s
            .command(Command::Speciate {
                founder: plant,
                name: "shoots".into(),
            })
            .unwrap();
        assert_eq!(line, "lineage:shoots");
        assert!(act(&mut s, plant, None, "ecology:regrow-0").accepted());
        assert_eq!(at(&s, plant), (18, 2));
        assert!(act(&mut s, plant, None, "ecology:produce-0").accepted());
        assert_eq!(at(&s, plant), (18, 4));
        assert!(act(&mut s, plant, None, "ecology:upkeep-0").accepted());
        assert_eq!(at(&s, plant), (17, 4));
        for _ in 0..17 {
            assert!(act(&mut s, plant, None, "ecology:upkeep-0").accepted());
        }
        assert_eq!(at(&s, plant), (0, 4));
        assert!(
            !act(&mut s, plant, None, "ecology:death-0").accepted(),
            "a fork with exposed tissue survives an empty root"
        );
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());

        let (mut s, plant, _) = fixture(30, 40, mode);
        let original: std::collections::BTreeSet<_> =
            s.sim.state().population.groups.keys().copied().collect();
        s.command(Command::Speciate {
            founder: plant,
            name: "shoots".into(),
        })
        .unwrap();
        let r = act(&mut s, plant, None, "ecology:birth-0");
        assert!(r.accepted(), "a native lineage fork can seed: {r:?}");
        assert_eq!(at(&s, plant), (20, 40));
        let (&child, _) = s
            .sim
            .state()
            .population
            .groups
            .iter()
            .find(|(id, group)| !original.contains(id) && group.entity.lineage == "lineage:shoots")
            .unwrap();
        let before = at(&s, child);
        assert!(act(&mut s, child, None, "ecology:produce-0").accepted());
        assert_eq!(at(&s, child).0 + at(&s, child).1, before.0 + before.1 + 2);
        assert_eq!(Session::load(s.save(), mode).unwrap().save(), s.save());
    }
}

#[test]
fn world_time_founders_keep_the_small_single_seeded_plant_bounds() {
    for seed in [9, 3, 7, 11, 5] {
        let g = founding(seed);
        assert!(
            g.founding
                .as_ref()
                .unwrap()
                .bodies
                .as_ref()
                .unwrap()
                .sensing
        );
        for group in g
            .population
            .groups
            .values()
            .filter(|group| group.entity.kingdom == "kingdom:flora")
        {
            let e = &group.entity;
            let exposed = e.parts.keys().filter(|&&id| !store(e, id)).count();
            assert!(
                (5..=17).contains(&exposed),
                "seed {seed}: {exposed} exposed parts"
            );
            assert_eq!(
                g.lineages[&e.lineage].development.as_ref().unwrap().clutch,
                1
            );
        }
    }
}
