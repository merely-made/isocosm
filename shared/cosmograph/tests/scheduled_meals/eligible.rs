// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::process::{IntakePort, NisKind, Registry};
use cosmograph::rules::{Binding, Causation, Effect, FeedingSources, Query};

const READY: &str = "test:ready-for-meal";

fn act(s: &mut Session, actor: Id, target: Option<Id>, process: &str) -> Receipt {
    serde_json::from_str(
        &s.command(Command::Act {
            actor,
            target,
            process: process.into(),
            cause: None,
        })
        .unwrap(),
    )
    .unwrap()
}

/// Retain the old compound-meal grammar as an authored compatibility control.
/// Fresh bodied Diets founding now uses one partitioned meal instead.
pub(super) fn authored_compound_meal(
    g: &mut Genesis,
    eater: Id,
    feed_key: &str,
    kept: u64,
    dropped: u64,
) {
    let function = g.rules.processes[feed_key]
        .port_feeding
        .as_ref()
        .unwrap()
        .function
        .clone();
    let e = g.population.lift(eater).unwrap();
    let registry = Registry::native();
    let (part, original) = e
        .living()
        .find_map(|(id, part)| {
            let port = part.port?;
            let addressed = match port.support() {
                Some(support) => registry
                    .resolve(support)
                    .is_some_and(|def| format!("function:{}", def.id.name) == function),
                None => part.functions.contains(&function),
            };
            addressed.then_some((id, port))
        })
        .expect("the actual feed addresses a declared living intake part");
    let port = IntakePort::live(NisKind::Producer);
    e.part_mut(&part).unwrap().port = Some(match original.support() {
        Some(support) => port.supported_by(support),
        None => port,
    });
    let feed = g.rules.processes.get_mut(feed_key).unwrap();
    let body = match &feed.effects[0] {
        Effect::Eat { into, .. } => into.clone(),
        _ => panic!("the native feed must begin with its retained tissue meal"),
    };
    let feeding = feed.port_feeding.as_mut().unwrap();
    feeding.sources = FeedingSources::Only(["lineage:0".into()].into());
    feeding.meal = None;
    feed.target.as_mut().unwrap().alive = Some(true);
    feed.requires.retain(|q| {
        !matches!(
            q,
            Query::Holds {
                who: Binding::Target,
                ..
            }
        )
    });
    feed.requires.push(Query::Holds {
        who: Binding::Target,
        at_least: kept + dropped,
    });
    feed.effects = vec![
        Effect::Eat {
            from: Binding::Target,
            amount: kept.into(),
            into: body,
            of: vec![],
            whole: false,
        },
        Effect::Eat {
            from: Binding::Target,
            amount: dropped.into(),
            into: DETRITUS.into(),
            of: vec![],
            whole: false,
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: DETRITUS.into(),
            amount: dropped.into(),
        },
    ];
}

fn hunter(lineage: u8) -> (Genesis, Id, Id, String, String) {
    let mut g = Founding {
        seed: 31,
        sites: 1,
        population: 60,
        cohort_size: 1,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        web: Web::Diets,
        bodies: Some(Bodies::default()),
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let member = |line: &str| {
        *g.population
            .groups
            .iter()
            .find(|(_, group)| group.entity.lineage == line)
            .unwrap()
            .0
    };
    let (eater, prey) = (member(&format!("lineage:{lineage}")), member("lineage:0"));
    g.population
        .groups
        .retain(|id, _| *id == eater || *id == prey);
    g.population.lift(eater).unwrap().place = 0;
    g.population.lift(prey).unwrap().place = 0;
    let feed_key = format!("ecology:feed-{lineage}");
    g.rules.processes.retain(|id, _| id == &feed_key);
    let (kept, dropped) = if lineage == 3 { (3, 6) } else { (1, 2) };
    authored_compound_meal(&mut g, eater, &feed_key, kept, dropped);
    let feed = g.rules.processes.get_mut(&feed_key).unwrap();
    feed.period = Some(1);
    let mut ready = feed.clone();
    ready.id = READY.into();
    ready.causation = Causation::Transition;
    ready.period = None;
    ready.target = None;
    ready.port_feeding = None;
    ready.risk = None;
    ready.requires = vec![Query::Alive(Binding::Actor)];
    ready.effects = vec![Effect::Trait {
        who: Binding::Actor,
        key: format!("ability:cycle-{lineage}"),
        present: true,
    }];
    g.rules.processes.insert(ready.id.clone(), ready);
    (g, eater, prey, feed_key, format!("matter:{lineage}-0"))
}

fn balances(s: &Session, eater: Id, prey: Id, body: &str) -> (u64, u64, u64) {
    (
        held(s, prey, PREY),
        held(s, eater, body),
        s.sim.state().sites[&0].accounts[DETRITUS],
    )
}

fn native_meal(lineage: u8, expected: Meal) {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey, feed, body) = hunter(lineage);
        let mut setup = Session::new(g, mode).unwrap();
        assert_eq!(held(&setup, prey, "world:soil"), 4);
        assert_eq!(held(&setup, prey, PREY), 60);
        // Body geometry and nominal meal sizes remain exact. Idempotent admitted
        // setup acts move only the native action number until its ordinary
        // bite draw permits the complete authored meal.
        let mut admitted = None;
        for _ in 0..512 {
            let before = balances(&setup, eater, prey, &body);
            let mut candidate = setup.clone();
            let r = act(&mut candidate, eater, Some(prey), &feed);
            if r.accepted() && change(before, balances(&candidate, eater, prey, &body)) == expected
            {
                admitted = Some((setup.clone(), candidate));
                break;
            }
            assert!(act(&mut setup, eater, None, READY).accepted());
        }
        let (mut scheduled, direct) =
            admitted.expect("the bounded native draws include a full meal");
        let before = balances(&scheduled, eater, prey, &body);
        let total = scheduled.sim.matter();
        assert_eq!(direct.sim.matter(), total);
        assert_eq!(held(&direct, prey, "world:soil"), 4);
        let work = scheduled.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
        assert_eq!(
            change(before, balances(&scheduled, eater, prey, &body)),
            expected,
            "only port-eligible source accounts may receive a meal quota"
        );
        assert_eq!(held(&scheduled, prey, "world:soil"), 4);
        let e = scheduled.sim.state().population.get(prey).unwrap();
        assert_eq!(
            e.parts[&e.body.as_ref().unwrap().root].matter.get(PREY),
            Some(&20)
        );
        assert_eq!(scheduled.sim.matter(), total);
        let saved = scheduled.save();
        for replay in [Execution::Individuals, Execution::Grouped] {
            assert_eq!(Session::load(saved.clone(), replay).unwrap().save(), saved);
        }
    }
}

#[test]
fn an_authored_compound_predator_meal_does_not_budget_ineligible_loose_soil() {
    native_meal(
        3,
        Meal {
            taken: 9,
            kept: 3,
            dropped: 6,
        },
    );
}

#[test]
fn an_authored_compound_grazer_meal_schedules_three_one_two() {
    native_meal(
        1,
        Meal {
            taken: 3,
            kept: 1,
            dropped: 2,
        },
    );
}
