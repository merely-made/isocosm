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

/// Mesocosm's bodies: the default with the probe's hazard (704), and rest.
fn mesocosm() -> Bodies {
    Bodies {
        hazard: harm::hazard::RATE,
        wound: harm::hazard::CELLS,
        ..Bodies::default()
    }
}

/// Bodies nothing wounds.
fn calm() -> Bodies {
    Bodies::default()
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
    act("test:mend", Effect::Heal { into: "world:soil".into(), rest: false })
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
    let (s, _) = world(mesocosm(), vec![], false);
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
    assert!(rules.processes.contains_key(harm::rest::ID), "and so does rest");
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

fn lost(s: &Session, c: Id) -> usize {
    s.sim.state().population.get(c).unwrap().parts.values().map(|p| p.lost.len()).sum()
}

fn held(s: &Session, c: Id) -> u64 {
    crate::anatomy::held(s.sim.state().population.get(c).unwrap(), &s.sim.genesis().rules, "matter:1-0")
}

fn soil(s: &Session, c: Id) -> u64 {
    let site = s.sim.state().population.get(c).unwrap().place;
    s.sim.state().sites[&site].accounts.get("world:soil").copied().unwrap_or(0)
}

fn run(s: &mut Session, c: Id, process: &str) {
    s.command(Command::Act { actor: c, target: None, process: process.into(), cause: None }).unwrap();
}

/// What restoring the critter's next lost cell costs, in full and at rest.
fn prices(s: &Session, c: Id) -> Option<(u64, u64)> {
    let e = s.sim.state().population.get(c).unwrap();
    let (id, _) = healing::next(e)?;
    let mass = crate::anatomy::cell_mass(e.extent(id), s.sim.genesis().rules.body());
    Some((mass, mass.div_ceil(4).max(1)))
}

#[test]
fn rest_restores_the_nearest_cell_for_a_quarter_of_its_mass_and_serves_a_wound() {
    let (mut s, c) = world(calm(), vec![harm::rest::process("world:soil")], true);
    wound(&mut s, c);
    assert_eq!(s.sim.mood(c), Some(-10), "only the wound presses");
    // It serves the wound as paid healing does, and wins the slot.
    assert_eq!(s.sim.consider(c, false).options["body:rest"].0, 1100);
    assert_eq!(s.sim.choose(c).process.as_deref(), Some("body:rest"));
    let (id, cell) = healing::next(s.sim.state().population.get(c).unwrap()).unwrap();
    let (_, price) = prices(&s, c).unwrap();
    let before = (lost(&s, c), held(&s, c), soil(&s, c));
    let matter = s.sim.matter();
    run(&mut s, c, "body:rest");
    assert!(!s.sim.state().population.get(c).unwrap().parts[&id].lost.contains(&cell), "the nearest recorded cell");
    assert_eq!((lost(&s, c), held(&s, c), soil(&s, c)), (before.0 - 1, before.1 - price, before.2 + price));
    assert_eq!(s.sim.matter(), matter, "the payment goes to the soil");
    // It rests while it can pay for a cell, one a time, and no longer.
    let mut rests = 1;
    while s.sim.consider(c, false).options.contains_key("body:rest") {
        run(&mut s, c, "body:rest");
        rests += 1;
        assert!(rests < 200, "rest ends");
    }
    assert!(prices(&s, c).is_none_or(|(_, quarter)| quarter + 1 > held(&s, c)), "for want of a cell or its price and a milligram");
}

#[test]
fn rest_needs_no_healing_trait_and_never_more_than_a_cell() {
    for trait_present in [true, false] {
        let (mut s, c) = world(calm(), vec![harm::rest::process("world:soil"), mend()], true);
        let traits = &mut s.sim.state.lineages.get_mut("lineage:1").unwrap().traits;
        if trait_present {
            traits.insert(harm::HEAL.into());
        } else {
            traits.remove(harm::HEAL);
        }
        wound(&mut s, c);
        let before = lost(&s, c);
        let (full, quarter) = prices(&s, c).unwrap();
        assert!(quarter < held(&s, c), "it can pay a quarter and keep a milligram");
        run(&mut s, c, "body:rest");
        assert_eq!(lost(&s, c), before - 1, "rest heals one cell, trait or none ({trait_present})");
        if !trait_present {
            run(&mut s, c, "test:mend");
            assert_eq!(lost(&s, c), before - 1, "a lineage without the trait does not pay to heal");
        } else if full <= held(&s, c) {
            run(&mut s, c, "test:mend");
            assert!(lost(&s, c) < before - 1, "paid healing, which may heal many, runs on");
        }
    }
}

#[test]
fn rest_never_pays_a_bodys_last_milligram() {
    // A body at nought dies (`ecology:death`), so it needs the price and one more.
    for (spare, rests) in [(0, false), (1, true)] {
        let (mut s, c) = world(calm(), vec![harm::rest::process("world:soil")], true);
        wound(&mut s, c);
        let (_, price) = prices(&s, c).unwrap();
        let rules = s.sim.genesis().rules.clone();
        let drop = held(&s, c) - (price + spare);
        crate::anatomy::take(s.sim.state.population.lift(c).unwrap(), &rules, "matter:1-0", drop).unwrap().unwrap();
        // The matter goes to the soil, so the world's total stands.
        let site = s.sim.state().population.get(c).unwrap().place;
        *s.sim.state.sites.get_mut(&site).unwrap().accounts.entry("matter:1-0".into()).or_default() += drop;
        assert_eq!(held(&s, c), price + spare);
        assert_eq!(s.sim.consider(c, false).options.contains_key("body:rest"), rests, "holding {} for a price of {price}", price + spare);
        if rests {
            run(&mut s, c, "body:rest");
            assert_eq!(held(&s, c), 1, "it keeps a milligram");
        }
    }
}
