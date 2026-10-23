// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::world;
use isocosm::{
    bodied::{self, Bodies},
    harm,
    probe::{BodyFounding, HarmFounding, readings},
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Case {
    seed: u64,
    index: u64,
    old_traits: BTreeSet<String>,
    old_clutch: u32,
    old_anamorphic: bool,
    heal: bool,
}

#[derive(Deserialize)]
struct Expected {
    rows: Vec<Case>,
}

fn cases() -> Vec<Case> {
    serde_json::from_str::<Expected>(include_str!("life_prechange.json"))
        .unwrap()
        .rows
}

#[test]
fn ordinary_heal_catalogue_declares_a_trait() {
    let declared: BTreeSet<_> = bodied::TRAITS.into_iter().collect();
    assert_eq!(declared.len(), 10);
    assert!(declared.contains(harm::HEAL));
    assert!(declared.contains(harm::FRAGMENT));
    assert!(!isocosm::rules::default_functions().contains_key("function:heal"));
}

#[test]
fn ordinary_heal_draw_matches_fixed_prechange_data() {
    let expected = cases();
    assert_eq!(expected.len(), 18);
    assert!(expected.iter().any(|case| case.heal));
    assert!(expected.iter().any(|case| !case.heal));
    for case in expected {
        let (mut traits, clutch, anamorphic) =
            bodied::life(case.seed, case.index, &Bodies::default());
        assert_eq!(
            traits.contains(harm::HEAL),
            case.heal,
            "548: HEAL follows its own draw; seed {}, index {}",
            case.seed,
            case.index,
        );
        traits.remove(harm::HEAL);
        assert_eq!(traits, case.old_traits, "existing life traits");
        assert_eq!(clutch, case.old_clutch, "existing clutch");
        assert_eq!(anamorphic, case.old_anamorphic, "existing anamorphy");
    }
}

#[test]
fn ordinary_founders_inherit_the_seeded_trait() {
    let expected = cases();
    let mut seen = [false; 2];
    for seed in [0, 1, 3, 5, 2026101001] {
        let g = world(seed);
        let mut bare_founding = super::founding(seed, Bodies::default());
        bare_founding.bodies = None;
        let bare = bare_founding.generate().unwrap();
        assert!(g.rules.traits.contains(harm::HEAL));
        for (index, (key, lineage)) in g.lineages.iter().enumerate() {
            let bodied = matches!(lineage.kingdom.as_str(), "kingdom:flora" | "kingdom:fauna");
            let heal = bodied && expected.iter().find(|case| {
                case.seed == seed && case.index == index as u64
            }).expect("fixed founder expectation").heal;
            assert_eq!(lineage.traits.contains(harm::HEAL), heal);
            assert_eq!(lineage.development.is_some(), bodied);
            if bodied {
                seen[usize::from(heal)] = true;
            }
            for (first, group) in g.population.groups.iter().filter(|(_, c)| &c.entity.lineage == key) {
                assert_eq!(group.entity.traits.contains(harm::HEAL), heal);
                if !bodied {
                    let original = &bare.population.groups[first];
                    assert_eq!(group.count, original.count, "bare cohort count");
                    assert_eq!(group.entity.accounts, original.entity.accounts, "bare accounts");
                    assert!(group.entity.body.is_none());
                    assert!(group.entity.soma.is_empty());
                    assert!(group.entity.systems.is_empty());
                    if lineage.kingdom == "kingdom:world" {
                        assert!(group.entity.parts.is_empty(), "agentless world has no parts");
                    } else {
                        assert_eq!(group.entity.parts, std::collections::BTreeMap::from([(
                            isocosm::schema::PartId(0),
                            isocosm::schema::Part {
                                traits: lineage.traits.clone(),
                                ..Default::default()
                            }
                            .into(),
                        )]), "bare Part0 retains only its lineage traits");
                    }
                }
            }
        }
    }
    assert_eq!(seen, [true, true], "both assigned and absent founders");
}

fn has_growth(value: &Value) -> bool {
    match value {
        Value::Object(fields) => fields.iter().any(|(key, value)| {
            matches!(key.as_str(), "Heal" | "Grow") || has_growth(value)
        }),
        Value::Array(values) => values.iter().any(has_growth),
        _ => false,
    }
}

#[test]
fn ordinary_heal_adds_no_generic_scheduler() {
    for seed in [0, 1, 3, 5, 2026101001] {
        let g = world(seed);
        assert!(g.rules.traits.contains(harm::HEAL));
        let processes = serde_json::to_value(&g.rules.processes).unwrap();
        assert!(!has_growth(&processes), "trait assignment installs no process");
    }
}

#[test]
fn authored_harm_columns_preserve_absent_and_present_counts() {
    for seed in [1, 2] {
        for (harm, carried) in [
            (None, None),
            (Some(HarmFounding {
                heals: 0, fragments: 0, ..Default::default()
            }), Some(false)),
            (Some(HarmFounding {
                heals: 1000, fragments: 1000, ..Default::default()
            }), Some(true)),
        ] {
            let w = BodyFounding { seed, harm, ..Default::default() }
                .generate().unwrap();
            let derived = readings::derive(&w);
            let members: Vec<_> = w.genesis.population.groups.values()
                .filter(|c| c.entity.kingdom != "kingdom:world")
                .map(|c| (c.entity.as_ref(), c.count)).collect();
            let values = readings::evaluate(
                &derived, &w, &members, &w.genesis.sites, 0,
                readings::inspect(&members, 10), &Default::default(),
            ).unwrap();
            for identity in [harm::HEAL, harm::FRAGMENT] {
                let column = derived.iter()
                    .position(|r| r.key == format!("alive:{identity}"));
                match carried {
                    None => assert!(column.is_none(), "undeclared optional column"),
                    Some(present) => {
                        let i = column.expect("declared optional column");
                        let actual: u64 = members.iter()
                            .filter(|(e, _)| e.alive && e.traits.contains(identity))
                            .map(|(_, count)| count).sum();
                        assert_eq!(values[i], actual, "actual Alive count");
                        assert_eq!(actual > 0, present, "authored present/absent fixture");
                    },
                }
            }
        }
    }
}
