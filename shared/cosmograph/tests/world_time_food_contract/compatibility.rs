// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::rules::PortFeeding;

#[test]
fn omitted_legacy_meal_definitions_and_stock_a1_keep_their_saved_meaning() {
    let json = serde_json::json!({
        "function": "function:intake",
        "sources": { "Only": ["lineage:0"] }
    });
    let legacy: PortFeeding = serde_json::from_value(json.clone()).unwrap();
    assert!(legacy.meal.is_none());
    assert_eq!(serde_json::to_value(&legacy).unwrap(), json);
    assert_eq!(cosmograph::digest(&legacy), cosmograph::digest(&json));
    for rates in [Rates::Stock, Rates::A1] {
        let g = Founding {
            seed: 9,
            sites: 1,
            population: 60,
            cohort_size: 1,
            lineages: 10,
            ecology: true,
            ecology_rates: rates,
            web: Web::Diets,
            bodies: Some(Bodies::default()),
            ..Founding::default()
        }
        .generate()
        .unwrap();
        assert!(!g.rules.accounts.contains_key(INGESTA));
        for p in g.rules.processes.values() {
            assert!(p.port_feeding.as_ref().is_none_or(|f| f.meal.is_none()));
            assert!(
                !p.requires
                    .iter()
                    .any(|q| matches!(q, Query::FoodPerHead { .. }))
            );
            if let Some(feeding) = &p.port_feeding {
                assert!(serde_json::to_value(feeding).unwrap().get("meal").is_none());
            }
        }
        let encoded = serde_json::to_vec(&g).unwrap();
        let old: Genesis = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(cosmograph::digest(&old), cosmograph::digest(&g));
        for mode in [Execution::Individuals, Execution::Grouped] {
            let s = Session::new(old.clone(), mode).unwrap();
            replay(&s);
        }
    }
}

#[test]
fn ingesta_cannot_borrow_current_or_inherited_tissue_but_accepts_foreign_matter() {
    use cosmograph::rules::AccountKind;
    for mode in [Execution::Individuals, Execution::Grouped] {
        for (owner, split, accepted) in [(1, false, false), (1, true, false), (2, false, true)] {
            let (mut g, eater, prey) = controlled(1, 0);
            stripped_plant(&mut g, prey, 3);
            g.rules.accounts.insert(
                INGESTA.into(),
                AccountKind::Matter {
                    lineage: format!("lineage:{owner}"),
                    reserve: false,
                    provision: false,
                },
            );
            let mut base = Session::new(g, mode).unwrap();
            if split {
                base.command(Command::Speciate {
                    founder: eater,
                    name: "ingesta-owner".into(),
                })
                .unwrap();
            }
            let total = base.sim.matter();
            let before = (books(&base, eater), books(&base, prey));
            let mut direct = base.clone();
            let r = act(&mut direct, eater, Some(prey), "ecology:feed-1");
            assert_eq!(r.accepted(), accepted, "owner{owner}, split{split}: {r:?}");
            if !accepted {
                assert!(
                    format!("{:?}", r.outcome).contains("ingesta cannot borrow"),
                    "the ownership guard causes refusal: {r:?}"
                );
            }
            let work = base.advance(1).unwrap();
            assert_eq!(work.accepted, u64::from(accepted), "{work:?}");
            for s in [&direct, &base] {
                if accepted {
                    assert_eq!(held(s, eater, "matter:1-0"), before.0["matter:1-0"] + 1);
                    assert_eq!(site_held(s, DETRITUS), 2);
                } else {
                    assert_eq!((books(s, eater), books(s, prey)), before);
                    assert_eq!(site_held(s, DETRITUS), 0);
                }
                assert_eq!(s.sim.matter(), total);
                replay(s);
            }
        }
    }
}

#[test]
fn a_food_query_cannot_reference_a_selectorless_manual_feed() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(1, 0);
        let generated = Session::new(g.clone(), mode).unwrap();
        assert_eq!(
            generated
                .sim
                .available_food(eater, "ecology:feed-1")
                .unwrap(),
            1
        );
        // The ordinary birth already references this feed. Remove that
        // reference before qualifying the otherwise valid manual grammar.
        g.rules.processes.remove("ecology:birth-1");
        let feed = g.rules.processes.get_mut("ecology:feed-1").unwrap();
        feed.target = None;
        feed.period = None;
        let mut manual = Session::new(g.clone(), mode)
            .expect("a partitioned feed may receive its target explicitly from a native action");
        let total = manual.sim.matter();
        let r = act(&mut manual, eater, Some(prey), "ecology:feed-1");
        assert!(r.accepted(), "manual food grammar remains usable: {r:?}");
        assert_eq!(manual.sim.matter(), total);
        replay(&manual);

        // A population query has no explicitly supplied target. Its feed
        // must therefore declare the selector whose population it reads.
        food_gate(&mut g, "ecology:feed-1", 1, 1);
        let error = match Session::new(g, mode) {
            Err(why) => why,
            Ok(_) => panic!("FoodPerHead admitted a reference to a selectorless manual feed"),
        };
        assert!(
            error.contains("food per head requires its feed to declare a target selector"),
            "{error}"
        );
    }
}
