// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A second generated law family, using the ordinary interpreter. Places are
//! explicitly well-mixed compartments here; no physical contact is inferred.
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use std::collections::BTreeMap;

pub(crate) fn configure(founding: &Founding, g: &mut Genesis) -> Result<()> {
    g.rules
        .processes
        .retain(|id, _| !id.starts_with("process:cycle-"));
    for (&id, site) in &mut g.sites {
        let supply = founding
            .population
            .checked_mul(founding.stock_max)
            .ok_or("soil supply overflow")?
            / u64::from(founding.sites);
        site.accounts.insert(
            "world:soil".into(),
            supply + crate::draw(founding.seed, "soil", &[id]) % founding.stock_max,
        );
    }
    for group in g.population.groups.values_mut() {
        let e = &mut group.entity;
        if e.kingdom == "kingdom:world" {
            continue;
        }
        let i = e
            .lineage
            .strip_prefix("lineage:")
            .unwrap()
            .parse::<u32>()
            .map_err(|e| e.to_string())?;
        e.accounts
            .retain(|id, _| id == &format!("matter:{i}-0") || id == "world:soil");
        // Birth thresholds are above initial bodies, so reproduction must be fed.
        e.accounts
            .insert(format!("matter:{i}-0"), founding.stock_min + 2);
        let role = ["life:producer", "life:consumer", "life:decomposer"][i as usize % 3];
        e.kingdom = ["kingdom:flora", "kingdom:fauna", "kingdom:myco"][i as usize % 3].into();
        e.traits.insert(role.into());
        g.rules.traits.insert(role.into());
    }
    for i in 0..founding.lineages {
        g.rules
            .traits
            .insert(["life:producer", "life:consumer", "life:decomposer"][i as usize % 3].into());
        let body = format!("matter:{i}-0");
        let own = format!("ability:cycle-{i}");
        let lineage = format!("lineage:{i}");
        let mut upkeep = process(
            &format!("ecology:upkeep-{i}"),
            Shape::Choice,
            vec![
                Effect::Transform {
                    who: Binding::Actor,
                    take: BTreeMap::from([(body.clone(), 1)]),
                    give: BTreeMap::from([("world:soil".into(), 1)]),
                },
                Effect::Transfer {
                    from: Binding::Actor,
                    to: Binding::Place,
                    account: "world:soil".into(),
                    amount: 1,
                },
            ],
        );
        upkeep.requires.push(Query::Trait {
            who: Binding::Actor,
            key: own.clone(),
        });
        upkeep.period = Some(5);
        upkeep.priority = 20;
        g.rules.processes.insert(upkeep.id.clone(), upkeep);
        let mut death = process(
            &format!("ecology:death-{i}"),
            Shape::Transition,
            vec![Effect::Death],
        );
        death.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Below {
                who: Binding::Actor,
                key: body.clone(),
                amount: 1,
            },
        ]);
        death.period = Some(1);
        death.priority = 30;
        death.note = true;
        g.rules.processes.insert(death.id.clone(), death);
        let mut age = process(
            &format!("ecology:age-{i}"),
            Shape::Transition,
            vec![Effect::Death],
        );
        age.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Age {
                at_least: 20 + crate::draw(founding.seed, "lifespan", &[u64::from(i)]) % 41,
            },
        ]);
        age.period = Some(1);
        age.priority = 31;
        age.note = true;
        g.rules.processes.insert(age.id.clone(), age);
        let child_mass = founding.stock_min.max(2);
        let mut birth = process(
            &format!("ecology:birth-{i}"),
            Shape::Transition,
            vec![Effect::Birth {
                provision: BTreeMap::from([(body.clone(), child_mass)]),
            }],
        );
        birth.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: own.clone(),
            },
            Query::Account {
                who: Binding::Actor,
                key: body.clone(),
                at_least: (founding.stock_min + 2) * 3,
            },
        ]);
        birth.period = Some(7);
        birth.priority = 40;
        birth.note = true;
        g.rules.processes.insert(birth.id.clone(), birth);
        if i % 3 == 0 {
            let mut grow = process(
                &format!("ecology:produce-{i}"),
                Shape::Choice,
                vec![
                    Effect::Transfer {
                        from: Binding::Place,
                        to: Binding::Actor,
                        account: "world:soil".into(),
                        amount: 2,
                    },
                    Effect::Transform {
                        who: Binding::Actor,
                        take: BTreeMap::from([("world:soil".into(), 2)]),
                        give: BTreeMap::from([(body, 2)]),
                    },
                ],
            );
            grow.requires.extend([
                Query::Trait {
                    who: Binding::Actor,
                    key: own,
                },
                Query::Condition {
                    key: "world:habitable".into(),
                    at_least: 1,
                },
            ]);
            grow.period = Some(2);
            grow.priority = 0;
            g.rules.processes.insert(grow.id.clone(), grow);
        } else {
            for j in 0..founding.lineages {
                if i == j || (i % 3 == 1 && j % 3 != 0) {
                    continue;
                }
                let food = format!("matter:{j}-0");
                let mut feed = process(
                    &format!("ecology:feed-{i}-{j}"),
                    Shape::Choice,
                    vec![
                        Effect::Transfer {
                            from: Binding::Target,
                            to: Binding::Actor,
                            account: food.clone(),
                            amount: 1,
                        },
                        Effect::Transform {
                            who: Binding::Actor,
                            take: BTreeMap::from([(food.clone(), 1)]),
                            give: BTreeMap::from([(body.clone(), 1)]),
                        },
                    ],
                );
                feed.requires.extend([
                    Query::Trait {
                        who: Binding::Actor,
                        key: own.clone(),
                    },
                    Query::Account {
                        who: Binding::Target,
                        key: food,
                        at_least: 1,
                    },
                ]);
                feed.target = Some(Target {
                    same_place: true,
                    alive: Some(i % 3 == 1),
                    lineage: Some(format!("lineage:{j}")),
                });
                feed.period = Some(3);
                feed.priority = 10;
                g.rules.processes.insert(feed.id.clone(), feed);
            }
        }
        g.lineages
            .get_mut(&lineage)
            .unwrap()
            .traits
            .insert(["life:producer", "life:consumer", "life:decomposer"][i as usize % 3].into());
        g.lineages.get_mut(&lineage).unwrap().kingdom =
            ["kingdom:flora", "kingdom:fauna", "kingdom:myco"][i as usize % 3].into();
    }
    Ok(())
}
