// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Wounds feed mood in played worlds (the 2026-10-18 note to ruling 227): the
//! wound need, paid healing serving it where eating does not, and strain
//! building while the mood stays low and easing as it recovers.

use super::Played;
use crate::{
    Execution, Founding, Session, bodied::Bodies, harm, harm::healing, history::Command, rules::*,
    schema::*,
};
use std::collections::BTreeMap;

const STRAIN: &str = "sim:strain";

fn calm() -> Bodies {
    Bodies {
        hazard: [0, 0],
        wound: [0, 0],
        ..Bodies::default()
    }
}

/// A played world, with `extra` processes, and with only its wound need
/// where `only_wounds`, so that no hunger moves the mood.
fn world(bodies: Bodies, extra: Vec<Process>, only_wounds: bool) -> (Session, Id) {
    let mut genesis = Founding {
        seed: 3,
        sites: 1,
        population: 30,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(bodies),
        played: Some(Played {
            lineage: 1,
            region_sites: 1,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    genesis
        .rules
        .processes
        .extend(extra.into_iter().map(|p| (p.id.clone(), p)));
    if only_wounds {
        let mind = genesis.rules.mind.as_mut().unwrap();
        mind.needs.retain(|n| matches!(n.query, Query::Wounded { .. }));
    }
    let s = Session::new(genesis, Execution::Individuals).unwrap();
    let c = *s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method == Method::Deliberative)
        .unwrap()
        .0;
    (s, c)
}

fn act(id: &str, effect: Effect) -> Process {
    let mut p = crate::generate::process(id, Causation::Choice, vec![effect]);
    p.period = Some(1);
    p
}

fn mend() -> Process {
    act("test:mend", Effect::Heal { into: "world:soil".into() })
}

fn eat() -> Process {
    let give = BTreeMap::from([("matter:1-0".to_string(), 1)]);
    act("test:eat", Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::from([("world:soil".to_string(), 1)]),
        give,
        conversion: None,
    })
}

fn share(s: &Session, c: Id) -> u64 {
    harm::wound_share(s.sim.state().population.get(c).unwrap())
}

/// Takes cells from the critter's limbs, never its root, until a quarter of
/// its cells are lost.
fn wound(s: &mut Session, c: Id) {
    let e = s.sim.state().population.get(c).unwrap();
    let limbs: Vec<PartId> = e.living().map(|(id, _)| id).filter(|id| e.parent_of(*id).is_some()).collect();
    for id in limbs {
        let e = s.sim.state().population.get(c).unwrap();
        if share(s, c) >= 25 {
            break;
        }
        let left = crate::anatomy::living_cells(e.extent(id), &e.parts[&id]);
        if left > 1 {
            s.command(Command::Wound { entity: c, part: Some(id), cells: left - 1 }).unwrap();
        }
    }
    assert!(share(s, c) >= 25, "the critter's limbs hold a quarter of its cells");
}

fn strain(s: &Session, c: Id) -> u64 {
    s.sim.state().population.get(c).unwrap().accounts.get(STRAIN).copied().unwrap_or(0)
}

#[test]
fn a_played_world_has_a_wound_need_after_its_hunger_and_strain_processes() {
    let (s, _) = world(Bodies::default(), vec![], false);
    let rules = &s.sim.genesis().rules;
    let mind = rules.mind.as_ref().unwrap();
    assert_eq!(mind.needs.len(), 3 + 1, "a hunger a lineage, then the wound");
    let wound = mind.needs.last().unwrap();
    assert_eq!((wound.weight, wound.traits.is_empty()), (-10, true));
    assert_eq!(wound.query, Query::Wounded { who: Binding::Actor, at_least: 25 });
    assert_eq!(rules.accounts[&mind.strain], AccountKind::Strain);
    assert_eq!(mind.strain, STRAIN);
    let build = &rules.processes["mind:strain"];
    let relief = &rules.processes["mind:relief"];
    assert!(build.requires.iter().any(|q| matches!(q, Query::MoodBelow { .. })));
    assert!(relief.requires.iter().any(|q| matches!(q, Query::Mood { .. })));
    assert!(rules.processes.contains_key(harm::hazard::ID), "the hazard joins the played world");
}

#[test]
fn healing_serves_a_wound_and_a_meal_does_not() {
    let (mut s, c) = world(calm(), vec![mend(), eat()], true);
    wound(&mut s, c);
    assert_eq!(s.sim.mood(c), Some(-10), "only the wound presses");
    let options = s.sim.consider(c, false).options;
    // The need's weight, 10, at low mood's 110 per cent, a hundred times.
    assert_eq!(options["test:mend"].0, 1100);
    assert_eq!(options["test:eat"].0, 0, "a meal does not heal a wound");
    assert_eq!(s.sim.choose(c).process.as_deref(), Some("test:mend"));
    // Unwounded, nothing presses and neither act serves a need.
    let (s, c) = world(calm(), vec![mend(), eat()], true);
    let options = s.sim.consider(c, false).options;
    assert_eq!((options["test:mend"].0, options["test:eat"].0), (0, 0));
}

#[test]
fn strain_builds_while_wounds_hold_the_mood_low_and_eases_as_they_heal() {
    let (mut s, c) = world(calm(), vec![], true);
    wound(&mut s, c);
    assert_eq!(strain(&s, c), 0);
    s.advance(6).unwrap();
    let built = strain(&s, c);
    assert!(built >= 6, "each tick of low mood builds some: {built}");
    while let Some((id, cell)) = healing::next(s.sim.state().population.get(c).unwrap()) {
        healing::restore(s.sim.state.population.lift(c).unwrap(), id, cell).unwrap();
    }
    assert_eq!((share(&s, c), s.sim.mood(c)), (0, Some(0)));
    s.advance(4).unwrap();
    assert!(strain(&s, c) < built, "relief bleeds it: {} from {built}", strain(&s, c));
}
