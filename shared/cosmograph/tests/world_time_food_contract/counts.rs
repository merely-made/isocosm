// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn full_native_breeder_counts_food_without_appetite_and_does_not_copy_its_gut() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(1, 0);
        put_tissue(g.population.lift(eater).unwrap(), "matter:1-0", 18);
        put_ingesta(&mut g, eater, 1, 2);
        g.population.lift(eater).unwrap().born = g.started - 365 * 1440;
        let (animal, plant) = (
            g.population.get(eater).unwrap().clone(),
            g.population.get(prey).unwrap().clone(),
        );
        g.population = Population::default();
        let eater = g.population.insert(animal, 1).unwrap();
        let prey = g.population.insert(plant, 3).unwrap();
        assert!(
            g.rules.processes["ecology:birth-1"]
                .requires
                .iter()
                .any(|q| matches!(q, Query::FoodPerHead { feed, .. } if feed == "ecology:feed-1"))
        );
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 3);
        let full = books(&s, eater);
        assert!(
            !act(&mut s, eater, Some(prey), "ecology:feed-1").accepted(),
            "a full animal has no appetite"
        );
        assert_eq!(books(&s, eater), full);
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 3);
        let child = s.sim.state().population.next_id;
        let birth = act(&mut s, eater, None, "ecology:birth-1");
        assert!(
            birth.accepted(),
            "food remains available to a full breeder: {birth:?}"
        );
        assert_eq!(held(&s, eater, "matter:1-0"), 12);
        assert_eq!(held(&s, eater, INGESTA), 2);
        assert_eq!(held(&s, child, "matter:1-0"), 6);
        assert_eq!(
            held(&s, child, INGESTA),
            0,
            "the child receives paid tissue, not a cloned bolus"
        );
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}

#[test]
fn food_counts_exclude_one_self_from_a_cohort_and_follow_real_speciation() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for count in [1, 3, 5] {
            let (mut g, eater, _) = controlled(3, 8);
            put_ingesta(&mut g, eater, 3, 2);
            let animal = g.population.get(eater).unwrap().clone();
            g.population = Population::default();
            let eater = g.population.insert(animal, count).unwrap();
            let gate = food_gate(&mut g, "ecology:feed-3", 2, 1);
            let mut s = Session::new(g, mode).unwrap();
            for member in eater..eater + count {
                assert_eq!(
                    s.sim.available_food(member, "ecology:feed-3").unwrap(),
                    count - 1
                );
            }
            assert!(
                !act(&mut s, eater, None, &gate).accepted(),
                "own heads are the denominator before splitting"
            );
            if count == 1 {
                continue;
            }
            s.command(Command::Speciate {
                founder: eater,
                name: format!("food-{count}"),
            })
            .unwrap();
            assert_eq!(
                s.sim.available_food(eater, "ecology:feed-3").unwrap(),
                count - 1
            );
            assert_eq!(
                held(&s, eater, INGESTA),
                2,
                "pending matter survives the actual split"
            );
            assert!(
                act(&mut s, eater, None, &gate).accepted(),
                "the fork's denominator is its one current member"
            );
            let before = held(&s, eater, "matter:3-0");
            let r = act(&mut s, eater, Some(eater + 1), "ecology:feed-3");
            assert!(
                r.accepted(),
                "an inherited native meal still admits its ancestor's tissue: {r:?}"
            );
            let e = s.sim.state().population.get(eater).unwrap();
            assert_eq!(
                e.accounts.get("matter:3-0").copied().unwrap_or(0),
                0,
                "retained ancestral tissue stays in physical parts"
            );
            assert!(anatomy::held(e, &s.sim.genesis().rules, "matter:3-0") > before);
            assert!(held(&s, eater, INGESTA) < 3);
            replay(&s);
        }
    }
}

#[test]
fn roots_only_and_dead_living_food_supply_no_food_to_the_query_or_meal() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for dead in [false, true] {
            let (mut g, eater, prey) = controlled(1, 0);
            if dead {
                g.population.lift(prey).unwrap().alive = false;
            } else {
                stripped_plant(&mut g, prey, 0);
            }
            let gate = food_gate(&mut g, "ecology:feed-1", 1, 1);
            let mut s = Session::new(g, mode).unwrap();
            let total = s.sim.matter();
            assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 0);
            assert!(!act(&mut s, eater, None, &gate).accepted());
            assert!(!act(&mut s, eater, Some(prey), "ecology:feed-1").accepted());
            assert_eq!(s.sim.matter(), total);
            replay(&s);
        }
    }
}

#[test]
fn a_declared_deadstock_port_counts_and_digests_only_after_the_offering_dies() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(1, 0);
        stripped_plant(&mut g, prey, 3);
        let function = g.rules.processes["ecology:feed-1"]
            .port_feeding
            .as_ref()
            .unwrap()
            .function
            .clone();
        let e = g.population.lift(eater).unwrap();
        let id = intake_id(e, &function);
        let support = e.parts[&id].port.unwrap().support().unwrap();
        e.part_mut(&id).unwrap().port =
            Some(cosmograph::process::IntakePort::deadstock().supported_by(support));
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 0);
        assert!(!act(&mut s, eater, Some(prey), "ecology:feed-1").accepted());
        s.command(Command::Kill { entity: prey }).unwrap();
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 1);
        let before = held(&s, eater, "matter:1-0");
        let r = act(&mut s, eater, Some(prey), "ecology:feed-1");
        assert!(r.accepted(), "{r:?}");
        assert_eq!(held(&s, prey, "matter:0-0"), 20);
        assert_eq!(held(&s, eater, "matter:1-0"), before + 1);
        assert_eq!(site_held(&s, DETRITUS), 2);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}

#[test]
fn scheduled_births_read_the_same_food_and_heads_for_the_whole_pass() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(1, 0);
        let started = g.started;
        let e = g.population.lift(eater).unwrap();
        put_tissue(e, "matter:1-0", 18);
        e.born = started - 365 * 1440;
        let (animal, plant) = (
            g.population.get(eater).unwrap().clone(),
            g.population.get(prey).unwrap().clone(),
        );
        g.population = Population::default();
        g.population.insert(animal, 2).unwrap();
        g.population.insert(plant, 5).unwrap();
        g.rules.processes.get_mut("ecology:feed-1").unwrap().period = None;
        g.rules.processes.get_mut("ecology:birth-1").unwrap().period = Some(1);
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        let work = s.advance(1).unwrap();
        assert_eq!(
            (work.accepted, work.blocked),
            (2, 0),
            "both full parents read five foods per two heads: {work:?}"
        );
        let heads: u64 = s
            .sim
            .state()
            .population
            .groups
            .values()
            .filter(|g| g.entity.alive && g.entity.lineage == "lineage:1")
            .map(|g| g.count)
            .sum();
        assert_eq!(
            heads, 4,
            "the second parent does not count the first parent's new child"
        );
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}
