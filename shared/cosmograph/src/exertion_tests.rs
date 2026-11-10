// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exertion: acts past a body's ease spend (case 1234).

use crate::{
    Execution, Simulation,
    climate_tests::{held, soil},
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};

/// A body-priced world with an act that does nothing, so that every cost is
/// exertion's own.
fn world() -> Genesis {
    let mut g = crate::probe::BodyFounding { seed: 3, ..Default::default() }.generate().unwrap().genesis;
    let mut pace = crate::generate::process("test:pace", Causation::Choice, vec![]);
    pace.period = Some(1);
    g.rules.processes.insert(pace.id.clone(), pace);
    g
}

fn rule(ease: u32, fit: u32, cost: u64) -> Exertion {
    Exertion { span: 24, ease, fit, cost }
}

fn paying(g: &Genesis, exertion: Exertion) -> Genesis {
    let mut g = g.clone();
    g.rules.exertion = Some(exertion);
    g
}

/// The first member of the first living cohort of `lineage`.
fn first(g: &Genesis, lineage: &str) -> Id {
    *g.population.groups.iter().find(|(_, c)| c.entity.lineage == lineage && c.entity.alive).unwrap().0
}

fn acts(sim: &mut Simulation, actor: Id, n: u32) {
    for _ in 0..n {
        assert_eq!(sim.execute(actor, None, "test:pace", None).outcome, Outcome::Accepted);
    }
}

fn spent(sim: &Simulation, actor: Id) -> Option<(Tick, u32)> {
    sim.state().population.get(actor).unwrap().spent
}

#[test]
fn unset_nothing_is_counted_charged_or_hashed() {
    let g = world();
    assert!(g.rules.exertion.is_none());
    let json = serde_json::to_string(&g.rules).unwrap();
    assert!(!json.contains("exertion"), "{json}");
    let mut set = g.rules.clone();
    set.exertion = Some(Exertion::default());
    assert_ne!(set.revision(), g.rules.revision());
    let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
    assert_eq!(*sim.genesis(), g, "an unset exertion leaves the genesis as it was");
    let (actor, total, ground) = (first(&g, "lineage:0"), sim.matter(), soil(&sim));
    acts(&mut sim, actor, 30);
    sim.advance(30).unwrap();
    assert!(sim.state().population.groups.values().all(|c| c.entity.spent.is_none()));
    let json = serde_json::to_string(sim.state()).unwrap();
    assert!(!json.contains("spent"), "no count reaches the wire");
    assert_eq!(sim.matter(), total);
    // The same act, with no exertion, moves nothing into the soil by itself.
    let mut again = Simulation::new(g, Execution::Individuals).unwrap();
    acts(&mut again, actor, 30);
    assert_eq!(soil(&again), ground);
}

#[test]
fn a_body_all_of_contracting_cells_is_at_ease_by_its_share() {
    let x = rule(4, 8, 1);
    assert_eq!(ease(&x, 0, 10), 4);
    assert_eq!(ease(&x, 10, 10), 12);
    assert_eq!(ease(&x, 5, 10), 8);
    assert_eq!(ease(&x, 0, 0), 4, "no cells, no share");
    assert_eq!(ease(&x, 3, 10), 6, "whole acts, rounded down");
    assert_eq!(ease(&x, 99, 10), 12, "never more than all");
    let d = Exertion::default();
    assert_eq!((d.span, d.ease, d.fit, d.cost), (24, 4, 8, 1));
    assert!(rule(0, 0, 0).validate().is_ok());
    assert!(Exertion { span: 0, ..d }.validate().is_err());
}

#[test]
fn a_span_of_no_ticks_is_refused() {
    let mut g = world();
    g.rules.exertion = Some(Exertion { span: 0, ..Exertion::default() });
    assert!(Simulation::new(g, Execution::Individuals).is_err());
}

/// Matter an act past ease puts in the soil, against a control that costs
/// nothing, for `n` acts by `actor`.
fn extra(g: &Genesis, x: Exertion, actor: Id, n: u32) -> u128 {
    let control = Exertion { cost: 0, ..x.clone() };
    let mut paid = Simulation::new(paying(g, x), Execution::Individuals).unwrap();
    let mut free = Simulation::new(paying(g, control), Execution::Individuals).unwrap();
    let total = paid.matter();
    acts(&mut paid, actor, n);
    acts(&mut free, actor, n);
    assert_eq!(paid.matter(), total, "matter is conserved");
    assert_eq!(spent(&paid, actor), Some((0, n)));
    assert_eq!(spent(&paid, actor), spent(&free, actor));
    soil(&paid) - soil(&free)
}

#[test]
fn acts_past_the_ease_pay_each_into_the_soil() {
    let g = world();
    let actor = first(&g, "lineage:0");
    for (n, ease, cost) in [(1u32, 3u32, 2u64), (3, 3, 2), (4, 3, 2), (9, 3, 5), (6, 0, 1)] {
        let x = rule(ease, 0, cost);
        let owed = u128::from(cost) * u128::from(n.saturating_sub(ease));
        assert_eq!(extra(&g, x, actor, n), owed, "{n} acts at ease {ease}");
    }
}

#[test]
fn the_count_resets_at_the_next_span() {
    let g = paying(&world(), rule(2, 0, 3));
    let actor = first(&g, "lineage:0");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    acts(&mut sim, actor, 5);
    assert_eq!(spent(&sim, actor), Some((0, 5)));
    let ground = soil(&sim);
    sim.state.tick += 23;
    acts(&mut sim, actor, 1);
    assert_eq!(spent(&sim, actor), Some((0, 6)), "still the same span");
    assert_eq!(soil(&sim) - ground, 3);
    sim.state.tick += 1;
    let ground = soil(&sim);
    acts(&mut sim, actor, 2);
    assert_eq!(spent(&sim, actor), Some((1, 2)), "a new span counts afresh");
    assert_eq!(soil(&sim), ground, "two acts are within the ease");
    acts(&mut sim, actor, 1);
    assert_eq!(soil(&sim) - ground, 3);
}

#[test]
fn a_body_with_more_contracting_cells_pays_for_fewer_acts() {
    let g = world();
    let (plant, grazer) = (first(&g, "lineage:0"), first(&g, "lineage:1"));
    let cells = |id: Id| -> [u64; 2] {
        let e = g.population.get(id).unwrap();
        let sum = |f: &dyn Fn(&str) -> bool| -> u64 {
            e.living().flat_map(|(_, p)| p.cells.iter()).filter(|(k, _)| f(k)).map(|(_, n)| u64::from(*n)).sum()
        };
        [sum(&|k| k == "function:contract"), sum(&|_| true)]
    };
    let [none, all] = cells(plant);
    let [some, whole] = cells(grazer);
    assert_eq!(none, 0);
    assert!(some > 0 && some < whole && all > 0);
    let x = rule(2, 11, 2);
    let at_ease = |id: Id| u128::from(ease(&x, cells(id)[0], cells(id)[1]));
    assert!(at_ease(grazer) > at_ease(plant));
    let n = 12;
    for id in [plant, grazer] {
        let owed = 2 * (u128::from(n) - at_ease(id).min(u128::from(n)));
        assert_eq!(extra(&g, x.clone(), id, n), owed);
    }
    let pays = |id: Id| extra(&g, x.clone(), id, n);
    assert!(pays(grazer) < pays(plant));
}

#[test]
fn a_nudged_act_counts_but_upkeep_and_other_processes_do_not() {
    let g = paying(&world(), rule(0, 0, 1));
    let actor = first(&g, "lineage:0");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    // The lineage's own upkeep is Choice, but ordinary activity is paid inside it.
    let upkeep = sim.genesis().rules.processes.keys().find(|k| k.starts_with("body:upkeep-0")).unwrap().clone();
    assert_eq!(sim.execute(actor, None, &upkeep, None).outcome, Outcome::Accepted);
    assert_eq!(spent(&sim, actor), None);
    // An agentless or transition process never counts either.
    let transition = sim.genesis().rules.processes.values()
        .find(|p| p.causation == Causation::Transition && p.id.starts_with("body:")).map(|p| p.id.clone());
    if let Some(id) = transition {
        let _ = sim.execute(actor, None, &id, None);
        assert_eq!(spent(&sim, actor), None);
    }
    acts(&mut sim, actor, 1);
    assert_eq!(spent(&sim, actor), Some((0, 1)));
}

#[test]
fn a_lineage_with_no_upkeep_pays_nothing() {
    let mut g = paying(&world(), rule(0, 0, 4));
    g.rules.processes.retain(|k, _| crate::climate::upkeep_of(k).is_none());
    let actor = first(&g, "lineage:0");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let ground = soil(&sim);
    acts(&mut sim, actor, 3);
    assert_eq!(spent(&sim, actor), Some((0, 3)));
    assert_eq!(soil(&sim), ground);
}

#[test]
fn a_body_that_cannot_pay_gives_what_it_has_and_still_acts() {
    let g = paying(&world(), rule(0, 0, 1_000_000));
    let actor = first(&g, "lineage:0");
    let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
    let rules = sim.genesis().rules.clone();
    let (had, ground, total) = (held(&rules, sim.state().population.get(actor).unwrap()), soil(&sim), sim.matter());
    assert!(had > 0);
    acts(&mut sim, actor, 1);
    assert_eq!(soil(&sim) - ground, had, "it gives all it holds");
    assert_eq!(held(&rules, sim.state().population.get(actor).unwrap()), 0);
    assert_eq!(sim.matter(), total);
    // Empty-handed, it still acts and pays nothing more.
    acts(&mut sim, actor, 2);
    assert_eq!(spent(&sim, actor), Some((0, 3)));
    assert_eq!(soil(&sim) - ground, had);
}

#[test]
fn the_spend_appears_in_the_flows() {
    let g = paying(&world(), rule(0, 0, 2));
    let actor = first(&g, "lineage:0");
    let mut sim = Simulation::new(g, Execution::Individuals).unwrap();
    let rules = sim.genesis().rules.clone();
    let before = held(&rules, sim.state().population.get(actor).unwrap());
    acts(&mut sim, actor, 1);
    let after = held(&rules, sim.state().population.get(actor).unwrap());
    assert_eq!(before - after, 2);
}

#[test]
fn a_run_with_the_rule_set_replays_to_the_same_hash() {
    let g = paying(&world(), rule(1, 1, 1));
    let run = |g: &Genesis, mode| {
        let mut sim = Simulation::new(g.clone(), mode).unwrap();
        let total = sim.matter();
        sim.advance(60).unwrap();
        assert_eq!(sim.matter(), total);
        (sim.state_hash(), sim.state().population.groups.values().filter(|c| c.entity.spent.is_some()).count())
    };
    let (hash, counted) = run(&g, Execution::Individuals);
    assert!(counted > 0, "the rule bites in the run");
    assert_eq!(run(&g, Execution::Individuals).0, hash);
    let again: Genesis = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
    assert_eq!(run(&again, Execution::Individuals).0, hash);
    run(&g, Execution::Grouped);
    // The rule moves the world from one without it.
    assert_ne!(run(&world(), Execution::Individuals).0, hash);
}
