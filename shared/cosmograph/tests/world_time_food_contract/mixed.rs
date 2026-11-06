// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::geometry::{Body, Sketch};
use std::collections::BTreeMap;

#[test]
fn a_mixed_part_meal_uses_that_parts_stock_in_direct_and_scheduled_execution() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(3, 8);
        g.population.lift(prey).unwrap().embody(Body::sketch([
            (
                0,
                Sketch {
                    half_extent: [4, 4, 4],
                    part: Part {
                        matter: BTreeMap::from([("matter:8-0".into(), 100)]),
                        ..Part::default()
                    },
                    ..Sketch::default()
                },
            ),
            (
                1,
                Sketch {
                    parent: Some(0),
                    half_extent: [4, 4, 4],
                    offset: [0, 16, 0],
                    part: Part {
                        matter: BTreeMap::from([
                            ("matter:8-0".into(), 1),
                            ("matter:6-0".into(), 5),
                        ]),
                        ..Part::default()
                    },
                    ..Sketch::default()
                },
            ),
        ]));
        auxiliary(
            &mut g,
            "ecology:feed-3",
            "test:ready",
            vec![Effect::Trait {
                who: Binding::Actor,
                key: "ability:cycle-3".into(),
                present: true,
            }],
        );
        let mut setup = Session::new(g, mode).unwrap();
        let total = setup.sim.matter();
        let mut selected = None;
        for _ in 0..1024 {
            // A native idempotent action advances the ordinary bite draw;
            // the test never chooses a part for the implementation.
            let mut direct = setup.clone();
            let r = act(&mut direct, eater, Some(prey), "ecology:feed-3");
            let target = direct.sim.state().population.get(prey).unwrap();
            if r.accepted()
                && target.parts[&PartId(0)].matter["matter:8-0"] == 100
                && target.parts[&PartId(1)].matter.values().all(|n| *n == 0)
            {
                selected = Some((setup.clone(), direct));
                break;
            }
            assert!(act(&mut setup, eater, None, "test:ready").accepted());
        }
        let (mut scheduled, direct) =
            selected.expect("a native draw takes the six-unit mixed part");
        let before = held(&scheduled, eater, "matter:3-0");
        let work = scheduled.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
        for id in [eater, prey] {
            assert_eq!(books(&scheduled, id), books(&direct, id));
        }
        for s in [&direct, &scheduled] {
            assert_eq!(held(s, prey, "matter:8-0"), 100);
            assert_eq!(held(s, prey, "matter:6-0"), 0);
            assert_eq!(held(s, prey, "world:soil"), 4, "the loose soil is rejected");
            assert_eq!(held(s, eater, "matter:3-0") - before, 2);
            assert_eq!(site_held(s, DETRITUS), 4);
            assert_eq!(held(s, eater, INGESTA), 0);
            assert_eq!(s.sim.matter(), total);
            replay(s);
        }
    }
}

#[test]
fn an_actual_incorporated_producer_part_is_food_on_a_speciated_animal_holder() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut g = founding();
        let (eater, donor, holder) = (member_id(&g, 1), member_id(&g, 0), member_id(&g, 8));
        g.population
            .groups
            .retain(|id, _| [eater, donor, holder].contains(id));
        for id in [eater, donor, holder] {
            g.population.lift(id).unwrap().place = 0;
        }
        g.rules.processes.retain(|key, _| key == "ecology:feed-1");
        g.rules.processes.get_mut("ecology:feed-1").unwrap().period = None;
        let plant = g.population.get(donor).unwrap();
        let allocation = anatomy::root_store::allocation(plant, &g.rules, "matter:0-0").unwrap();
        let leaf = *allocation
            .exposed
            .keys()
            .find(|id| plant.children_of(**id).is_empty())
            .unwrap();
        for id in g
            .population
            .get(donor)
            .unwrap()
            .parts
            .keys()
            .copied()
            .collect::<Vec<_>>()
        {
            g.population
                .lift(donor)
                .unwrap()
                .part_mut(&id)
                .unwrap()
                .matter
                .clear();
        }
        let plant = g.population.lift(donor).unwrap();
        plant
            .part_mut(&allocation.store)
            .unwrap()
            .matter
            .insert("matter:0-0".into(), 20);
        plant
            .part_mut(&leaf)
            .unwrap()
            .matter
            .insert("matter:0-0".into(), 3);
        // One free native host box makes the placement deterministic. The
        // foreign leaf itself is moved by an admitted whole-part meal.
        g.population.lift(holder).unwrap().embody(Body::sketch([(
            0,
            Sketch {
                half_extent: [4, 4, 4],
                part: Part {
                    matter: BTreeMap::from([("matter:8-0".into(), 15)]),
                    ..Part::default()
                },
                ..Sketch::default()
            },
        )]));
        for line in ["lineage:0", "lineage:8"] {
            g.lineages
                .get_mut(line)
                .unwrap()
                .development
                .as_mut()
                .unwrap()
                .domain = 0;
        }
        auxiliary(
            &mut g,
            "ecology:feed-1",
            "test:incorporate",
            vec![Effect::Eat {
                from: Binding::Target,
                amount: 3.into(),
                into: "matter:8-0".into(),
                of: vec!["matter:0-0".into()],
                whole: true,
            }],
        );
        let target = g.rules.processes["ecology:feed-1"].target.clone();
        g.rules
            .processes
            .get_mut("test:incorporate")
            .unwrap()
            .target = target;
        let gate = food_gate(&mut g, "ecology:feed-1", 1, 1);
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 1);
        let r = act(&mut s, holder, Some(donor), "test:incorporate");
        assert!(r.accepted(), "{r:?}");
        let carrier = s.sim.state().population.get(holder).unwrap();
        assert_eq!(
            carrier.parts.len(),
            2,
            "the leaf was incorporated, not digested"
        );
        assert_eq!(held(&s, holder, "matter:0-0"), 3);
        assert!(
            !s.sim
                .state()
                .population
                .get(donor)
                .unwrap()
                .parts
                .contains_key(&leaf)
        );
        assert_eq!(
            s.sim.available_food(eater, "ecology:feed-1").unwrap(),
            1,
            "only the new holder supplies exposed producer tissue"
        );
        s.command(Command::Speciate {
            founder: holder,
            name: "leaf-carrier".into(),
        })
        .unwrap();
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 1);
        assert!(act(&mut s, eater, None, &gate).accepted());
        let before = held(&s, eater, "matter:1-0");
        let r = act(&mut s, eater, Some(holder), "ecology:feed-1");
        assert!(
            r.accepted(),
            "holder lineage never substitutes for offered provenance: {r:?}"
        );
        assert_eq!(held(&s, holder, "matter:0-0"), 0);
        assert_eq!(held(&s, eater, "matter:1-0") - before, 1);
        assert_eq!(site_held(&s, DETRITUS), 2);
        assert_eq!(s.sim.available_food(eater, "ecology:feed-1").unwrap(), 0);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}
