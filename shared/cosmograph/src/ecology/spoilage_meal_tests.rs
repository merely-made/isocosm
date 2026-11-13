// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A jaw that bites what it does not tolerate is harmed, and the meal still
//! happens with the budget closed; one that tolerates it is not; without the
//! rule the same meal is refused. The played A1 world founds no jaw, so the
//! tests give lineage 3 (the predator of a web of diets), and the played lineage 1, a participant, is no prey for it,
//! so the dead are lineage 4 one.

use super::spoilage_tests::founding;
use super::*;
use crate::{
    Execution, Session,
    flows::{Holder, MadeBy},
    process::{IntakePort, NisKind, Tolerance},
    simulation::Genesis,
};

const RULE: Spoilage = Spoilage { aged_after: 1, toxic_after: 2, harm: 2, decomposer_flesh_toxic: true };

/// Lineage 3 with a jaw (living animal matter) tolerating `extra`, and the
/// lineage `dead` (if any) lying dead since tick 0. Lineage 3 eats only the `prey` lines.
fn world(rule: Option<Spoilage>, extra: &[Tolerance], dead: Option<&str>, prey: &[&str]) -> Genesis {
    let mut g = founding(rule).generate().unwrap();
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        if e.lineage == "lineage:3" {
            let ports: Vec<_> = e.parts.iter().filter_map(|(id, p)| p.port.map(|p| (*id, p))).collect();
            for (id, old) in ports {
                let mut jaw = IntakePort::live(NisKind::Consumer).supported_by(old.support().unwrap());
                if let Some(rule) = rule {
                    jaw = rule.founded(jaw);
                }
                for t in extra {
                    jaw = jaw.tolerating(*t);
                }
                e.part_mut(&id).unwrap().port = Some(jaw);
            }
        }
        if dead == Some(e.lineage.as_str()) {
            e.alive = false;
            e.died = Some(0);
        }
    }
    spoilage::refeed(&mut g).unwrap();
    // Narrowed to the one line under test, if the mouth admits it, so the
    // jaw's choice is not left to what else lives at its site.
    let feed = g.rules.processes.get_mut("ecology:feed-3").unwrap();
    if let Some(crate::rules::PortFeeding { sources: crate::rules::FeedingSources::Only(lines), .. }) = &mut feed.port_feeding {
        lines.retain(|l| prey.contains(&l.as_str()));
    }
    g
}

/// Everything of `lines` living at one site, so the jaw meets what it eats.
fn gather(mut g: Genesis, lines: &[&str]) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        if lines.contains(&group.entity.lineage.as_str()) {
            group.entity_mut().place = site;
        }
    }
    g
}

struct Eaten {
    /// Matter lineage 3's meals took from each lineage, by flesh.
    meals: u64,
    harms: Vec<crate::flows::Harm>,
    closed: bool,
}

fn run(g: Genesis, prey: &str, ticks: u64) -> Eaten {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let mut out = Eaten { meals: 0, harms: vec![], closed: true };
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let by_jaw = f.made_by == MadeBy::Process("ecology:feed-3".into());
            if by_jaw && matches!(f.to.0, Holder::Entity(_) | Holder::Part(..)) && f.from_kind.as_ref().is_some_and(|k| k.lineage == prey) {
                out.meals += f.amount * f.count;
            }
        }
        out.harms.extend(s.sim.harms().iter().filter(|h| h.made_by == "ecology:feed-3" && h.lineage == prey).cloned());
        out.closed &= s.sim.budget().total() == total;
    }
    out
}

#[test]
fn a_jaw_eating_toxic_flesh_is_harmed_and_the_matter_still_moves() {
    let eaten = run(world(Some(RULE), &[], Some("lineage:4"), &["lineage:4"]), "lineage:4", 40);
    assert!(eaten.meals > 0, "the jaw ate dead lineage:4");
    assert!(!eaten.harms.is_empty(), "and paid for it");
    assert!(eaten.harms.iter().all(|h| h.flesh == Flesh::Toxic && h.cells == 2));
    assert!(eaten.closed, "no matter made or lost");
}

#[test]
fn a_jaw_that_tolerates_toxic_flesh_is_not_harmed() {
    let eaten = run(world(Some(RULE), &[Tolerance::ToxicFlesh], Some("lineage:4"), &["lineage:4"]), "lineage:4", 40);
    assert!(eaten.meals > 0);
    assert!(eaten.harms.is_empty());
    assert!(eaten.closed);
}

#[test]
fn a_harmed_eater_loses_cells() {
    let g = world(Some(RULE), &[], Some("lineage:4"), &["lineage:4"]);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    for _ in 0..40 {
        let _ = s.advance_tick_with_flows().unwrap();
        if let Some(h) = s.sim.harms().first() {
            let eater = s.sim.state().population.get(h.eater).unwrap();
            assert!(crate::harm::wound_share(eater) > 0 || !eater.alive, "the wound is on the eater");
            return;
        }
    }
    panic!("no bite harmed in 40 ticks");
}

#[test]
fn a_jaw_eating_a_living_decomposer_is_harmed_unless_it_tolerates_it() {
    let harmed = run(gather(world(Some(RULE), &[], None, &["lineage:2", "lineage:4"]), &["lineage:2", "lineage:3"]), "lineage:2", 100);
    assert!(harmed.meals > 0, "the predator bit a living decomposer");
    // Mostly the living; a decomposer that died in the run is toxic flesh too.
    assert!(harmed.harms.iter().any(|h| h.flesh == Flesh::Fresh), "the living were bitten and paid for it");
    assert!(harmed.closed);
    let freely = run(gather(world(Some(RULE), &[Tolerance::DecomposerFlesh], None, &["lineage:2", "lineage:4"]), &["lineage:2", "lineage:3"]), "lineage:2", 100);
    // Tolerating the living flesh is not tolerating the toxic flesh of the dead.
    assert!(freely.meals > 0 && freely.harms.iter().all(|h| h.flesh == Flesh::Toxic));
    assert!(freely.closed);
}

#[test]
fn a_fresh_corpse_does_not_harm_a_jaw() {
    let fresh = Spoilage { aged_after: 5_000, toxic_after: 10_000, ..RULE };
    let eaten = run(world(Some(fresh), &[], Some("lineage:4"), &["lineage:4"]), "lineage:4", 40);
    assert!(eaten.meals > 0 && eaten.harms.is_empty());
}

#[test]
fn with_the_rule_unset_the_same_meals_are_refused() {
    for (dead, prey) in [(Some("lineage:4"), "lineage:4"), (None, "lineage:2")] {
        let eaten = run(world(None, &[], dead, &[prey, "lineage:4"]), prey, 40);
        assert_eq!((eaten.meals, eaten.harms.len()), (0, 0), "{prey} is no food for the jaw");
        assert!(eaten.closed);
    }
}

#[test]
fn spoilage_in_both_executions_is_the_same() {
    let hash = |mode| {
        let mut s = Session::new(world(Some(RULE), &[], Some("lineage:4"), &["lineage:4"]), mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}



