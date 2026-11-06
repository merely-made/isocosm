// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn tiny(actors: u64, plants: u64, gut: u64) -> (Genesis, Id, Id) {
    let (mut g, eater, prey) = controlled(1, 0);
    stripped_plant(&mut g, prey, 1);
    g.population
        .lift(eater)
        .unwrap()
        .accounts
        .insert(DETRITUS.into(), 7);
    put_ingesta(&mut g, eater, 1, gut);
    let (animal, plant) = (
        g.population.get(eater).unwrap().clone(),
        g.population.get(prey).unwrap().clone(),
    );
    g.population = Population::default();
    let eater = g.population.insert(animal, actors).unwrap();
    let prey = g.population.insert(plant, plants).unwrap();
    (g, eater, prey)
}

#[test]
fn three_one_unit_bites_digest_one_keep_two_waste_with_real_saved_residue() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey) = tiny(1, 3, 0);
        let mut direct = Session::new(g.clone(), mode).unwrap();
        let mut scheduled = Session::new(g, mode).unwrap();
        let total = direct.sim.matter();
        let initial = held(&direct, eater, "matter:1-0");
        for (step, gut, kept, waste) in [(0, 1, 0, 0), (1, 2, 0, 0), (2, 0, 1, 2)] {
            let receipt = act(&mut direct, eater, Some(prey + step), "ecology:feed-1");
            assert!(receipt.accepted(), "{mode:?}: {receipt:?}");
            let work = scheduled.advance(1).unwrap();
            assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
            for s in [&direct, &scheduled] {
                assert_eq!(held(s, eater, INGESTA), gut);
                assert_eq!(held(s, eater, "matter:1-0"), initial + kept);
                assert_eq!(site_held(s, DETRITUS), waste);
                assert_eq!(
                    held(s, eater, DETRITUS),
                    7,
                    "old waste never funds a new meal"
                );
                assert_eq!(
                    s.sim
                        .state()
                        .population
                        .get(eater)
                        .unwrap()
                        .accounts
                        .get(INGESTA)
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(s.sim.matter(), total);
                replay(s);
            }
        }
        for id in prey..prey + 3 {
            assert_eq!(
                held(&direct, id, "matter:0-0"),
                20,
                "the protected store remains"
            );
            assert_eq!(held(&scheduled, id, "matter:0-0"), 20);
        }
    }
}

#[test]
fn two_zero_quotas_cannot_digest_a_previous_incomplete_bolus() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey) = tiny(2, 1, 2);
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        let before: Vec<_> = [eater, eater + 1, prey]
            .into_iter()
            .map(|id| (id, books(&s, id)))
            .collect();
        let work = s.advance(1).unwrap();
        assert_eq!(
            work.evaluations, 2,
            "both eaters reach the shared pass: {work:?}"
        );
        for (id, was) in before {
            assert_eq!(books(&s, id), was);
        }
        assert_eq!(site_held(&s, DETRITUS), 0);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}

#[test]
fn death_and_native_rot_recover_the_pending_matter_without_spawning_or_losing_it() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = tiny(1, 1, 2);
        auxiliary(
            &mut g,
            "ecology:feed-1",
            "test:rot",
            vec![Effect::Rot {
                who: Binding::Actor,
                amount: u64::MAX.into(),
            }],
        );
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        s.command(Command::Kill { entity: eater }).unwrap();
        assert_eq!(
            held(&s, eater, INGESTA),
            2,
            "death does not discard gut contents"
        );
        let r = act(&mut s, eater, None, "test:rot");
        assert!(r.accepted(), "{r:?}");
        assert_eq!(held(&s, eater, INGESTA), 0);
        assert_eq!(
            site_held(&s, INGESTA),
            2,
            "rot preserves the actual matter account"
        );
        assert_eq!(held(&s, prey, "matter:0-0"), 21);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}

#[test]
fn intake_capacity_bounds_only_the_final_residue_and_refuses_atomically() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        for amount in [1, 3] {
            let (mut g, eater, prey) = controlled(1, 0);
            stripped_plant(&mut g, prey, amount);
            let function = g.rules.processes["ecology:feed-1"]
                .port_feeding
                .as_ref()
                .unwrap()
                .function
                .clone();
            let body_rules = g.rules.body();
            let e = g.population.lift(eater).unwrap();
            let intake = intake_id(e, &function);
            let ceiling = anatomy::ceiling(e.extent(intake), body_rules);
            let existing: u64 = e.parts[&intake].matter.values().sum();
            assert!(ceiling > existing);
            e.part_mut(&intake)
                .unwrap()
                .matter
                .insert("matter:8-0".into(), ceiling - existing);
            let mut direct = Session::new(g.clone(), mode).unwrap();
            let mut scheduled = Session::new(g, mode).unwrap();
            let total = direct.sim.matter();
            let before = books(&direct, eater);
            let food_before = books(&direct, prey);
            let r = act(&mut direct, eater, Some(prey), "ecology:feed-1");
            let work = scheduled.advance(1).unwrap();
            assert_eq!(r.accepted(), amount == 3, "{r:?}");
            assert_eq!(work.accepted, u64::from(amount == 3), "{work:?}");
            for s in [&direct, &scheduled] {
                if amount == 1 {
                    assert_eq!(books(s, eater), before);
                    assert_eq!(books(s, prey), food_before);
                    assert_eq!(site_held(s, DETRITUS), 0);
                } else {
                    assert_eq!(held(s, eater, "matter:1-0"), before["matter:1-0"] + 1);
                    assert_eq!(site_held(s, DETRITUS), 2);
                }
                assert_eq!(held(s, eater, INGESTA), 0);
                assert_eq!(s.sim.matter(), total);
                replay(s);
            }
        }
    }
}
