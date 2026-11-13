// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Tough plant matter and dead matter in classes at the meal: a grazer
//! without grinding that bites tough tissue is harmed and the matter still
//! moves; with grinding it is not; soft tissue and a world without the rule
//! are as before; a dead-stock root eats every class, a limited mouth only
//! its own; wood rots slowest; the budget stays closed. The played A1 world
//! founds no jaw or class mouth, so lineage 3 (grazer of lineage 0) and
//! lineage 2 (the decomposer) are given them.

use super::categories_tests::founding;
use super::*;
use crate::{
    Execution, Session,
    flows::{Holder, MadeBy},
    process::{DeadClass, IntakePort, Tolerance},
    simulation::Genesis,
};

const RULE: Categories = Categories { harm: 2, wood_rot_every: 1 };

/// What one run saw.
#[derive(Default)]
struct Seen {
    /// Matter the eater's meals took, by prey lineage and whether from a tough part.
    took: std::collections::BTreeMap<(String, bool), u64>,
    harms: Vec<crate::flows::Harm>,
    closed: bool,
}

impl Seen {
    fn of(&self, lineage: &str) -> u64 {
        self.took.iter().filter(|((l, _), _)| l == lineage).map(|(_, n)| n).sum()
    }

    fn tough(&self, lineage: &str) -> u64 {
        self.took.get(&(lineage.to_owned(), true)).copied().unwrap_or(0)
    }
}

/// Every lineage 0 body's matter moved into one part of the tissue named
/// (tough, or soft), so a bite lands on what the test says; none leaves
/// the founded spread.
fn tissue(mut g: Genesis, tough: Option<bool>) -> Genesis {
    let Some(want) = tough else { return g };
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:0") {
        let e = group.entity_mut();
        let id = *e.parts.keys().find(|id| super::tough(e, **id) == want).unwrap();
        let mut all = crate::schema::Ledger::new();
        for part in e.parts.values_mut() {
            for (k, n) in std::mem::take(&mut std::sync::Arc::make_mut(part).matter) {
                *all.entry(k).or_default() += n;
            }
        }
        e.part_mut(&id).unwrap().matter = all;
    }
    g
}

/// Everything of `lines` at one site, so the eater meets what it eats.
fn gather(mut g: Genesis, lines: &[&str]) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        if lines.contains(&group.entity.lineage.as_str()) {
            group.entity_mut().place = site;
        }
    }
    g
}

fn die(mut g: Genesis, lines: &[&str]) -> Genesis {
    for group in g.population.groups.values_mut().filter(|g| lines.contains(&g.entity.lineage.as_str())) {
        let e = group.entity_mut();
        e.alive = false;
        e.died = Some(0);
    }
    g
}

/// `lineage`'s every mouth replaced by `port`, and its feed's sources worked out again.
fn mouth(mut g: Genesis, lineage: &str, port: impl Fn(IntakePort) -> IntakePort) -> Genesis {
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == lineage) {
        let e = group.entity_mut();
        let ports: Vec<_> = e.parts.iter().filter_map(|(id, p)| p.port.map(|p| (*id, p))).collect();
        for (id, old) in ports {
            e.part_mut(&id).unwrap().port = Some(port(old));
        }
    }
    super::spoilage::refeed(&mut g).unwrap();
    g
}

fn run(g: Genesis, eater: &str, ticks: u64) -> Seen {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let feed = format!("ecology:feed-{}", eater.trim_start_matches("lineage:"));
    let mut seen = Seen { closed: true, ..Seen::default() };
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            if f.made_by != MadeBy::Process(feed.clone()) || !matches!(f.to.0, Holder::Entity(_) | Holder::Part(..)) {
                continue;
            }
            let Some(kind) = &f.from_kind else { continue };
            let tough = match f.from.0 {
                Holder::Part(body, part) => s.sim.state().population.get(body).is_some_and(|e| tough(e, part)),
                _ => false,
            };
            *seen.took.entry((kind.lineage.clone(), tough)).or_default() += f.amount * f.count;
        }
        seen.harms.extend(s.sim.harms().iter().filter(|h| h.made_by == feed).cloned());
        seen.closed &= s.sim.budget().total() == total;
    }
    seen
}

fn world(rule: Option<Categories>, t: Option<bool>) -> Genesis {
    gather(tissue(founding(rule).generate().unwrap(), t), &["lineage:0", "lineage:3"])
}

#[test]
fn a_grazer_without_grinding_is_harmed_by_tough_tissue_and_the_matter_still_moves() {
    let seen = run(world(Some(RULE), Some(true)), "lineage:3", 40);
    assert!(seen.of("lineage:0") > 0, "the grazer ate the plant");
    assert!(!seen.harms.is_empty(), "and paid for it");
    assert!(seen.harms.iter().all(|h| h.tough && h.cells == 2 && h.lineage == "lineage:0"));
    assert!(seen.closed, "no matter made or lost");
}

#[test]
fn a_grazer_with_grinding_is_not_harmed() {
    let g = mouth(world(Some(RULE), Some(true)), "lineage:3", |p| p.tolerating(Tolerance::ToughPlant));
    let seen = run(g, "lineage:3", 40);
    assert!(seen.of("lineage:0") > 0 && seen.harms.is_empty());
    assert!(seen.closed);
}

#[test]
fn soft_tissue_is_eaten_as_before() {
    let seen = run(world(Some(RULE), Some(false)), "lineage:3", 40);
    assert!(seen.of("lineage:0") > 0 && seen.harms.is_empty());
    assert_eq!(seen.tough("lineage:0"), 0);
    assert!(seen.closed);
    // The same meals, bite for bite, with the rule unset.
    let plain = run(world(None, Some(false)), "lineage:3", 40);
    assert_eq!(plain.took, seen.took);
}

#[test]
fn with_the_rule_unset_tough_tissue_is_eaten_unharmed_as_today() {
    let seen = run(world(None, Some(true)), "lineage:3", 40);
    assert!(seen.of("lineage:0") > 0, "the grazer eats it");
    assert!(seen.harms.is_empty() && seen.closed);
    let founded = run(world(None, None), "lineage:3", 40);
    assert!(founded.of("lineage:0") > 0 && founded.harms.is_empty());
}

#[test]
fn a_founded_plant_has_both_tissues_and_the_grazer_meets_the_tough() {
    let seen = run(world(Some(RULE), None), "lineage:3", 60);
    assert!(seen.of("lineage:0") > 0 && seen.closed);
    assert!(seen.tough("lineage:0") > 0, "its bites reach the stalk");
    assert!(seen.harms.iter().all(|h| h.tough));
}

/// Lineage 3 (a bodied grazer standing in for a decomposer, whose lineage the
/// founded worlds leave bare) with a root for the dead limited as `port`
/// makes it, over the dead of lineage 0 (plant) and lineage 4 (animal).
fn feast(port: impl Fn(IntakePort) -> IntakePort) -> Seen {
    let g = die(founding(Some(RULE)).generate().unwrap(), &["lineage:0", "lineage:4"]);
    let root = |old: IntakePort| port(IntakePort::deadstock().supported_by(old.support().unwrap()));
    run(gather(mouth(g, "lineage:3", root), &["lineage:0", "lineage:3", "lineage:4"]), "lineage:3", 60)
}

#[test]
fn a_dead_stock_root_eats_every_class() {
    let seen = feast(|p| p);
    assert!(seen.of("lineage:4") > 0, "carrion");
    assert!(seen.of("lineage:0") > seen.tough("lineage:0"), "litter");
    assert!(seen.tough("lineage:0") > 0, "wood");
    assert!(seen.harms.is_empty(), "the root grinds");
    assert!(seen.closed);
}

#[test]
fn a_mouth_limited_to_litter_eats_the_soft_dead_plant_alone() {
    let seen = feast(|p| p.taking(DeadClass::Litter));
    assert!(seen.of("lineage:0") > 0);
    assert_eq!((seen.tough("lineage:0"), seen.of("lineage:4")), (0, 0));
    assert!(seen.closed);
}

#[test]
fn a_mouth_limited_to_wood_eats_the_tough_dead_plant_alone() {
    let seen = feast(|p| p.taking(DeadClass::Wood));
    assert!(seen.tough("lineage:0") > 0);
    assert_eq!((seen.of("lineage:0") - seen.tough("lineage:0"), seen.of("lineage:4")), (0, 0));
    assert!(seen.closed);
}

#[test]
fn a_mouth_limited_to_carrion_eats_the_dead_animal_alone() {
    let seen = feast(|p| p.taking(DeadClass::Carrion));
    assert!(seen.of("lineage:4") > 0 && seen.of("lineage:0") == 0);
    assert!(seen.closed);
}

#[test]
fn with_the_rule_unset_a_class_limit_is_not_read() {
    let g = die(founding(None).generate().unwrap(), &["lineage:0", "lineage:4"]);
    let root = |old: IntakePort| IntakePort::deadstock().supported_by(old.support().unwrap()).taking(DeadClass::Litter);
    let g = gather(mouth(g, "lineage:3", root), &["lineage:0", "lineage:3", "lineage:4"]);
    let seen = run(g, "lineage:3", 60);
    assert!(seen.of("lineage:4") > 0 && seen.of("lineage:0") > 0, "the mouth takes all dead stock, as before");
}

#[test]
fn with_spoilage_also_set_carrion_ages_and_harms_a_jaw_as_before() {
    let rule = Spoilage { aged_after: 1, toxic_after: 2, harm: 2, decomposer_flesh_toxic: true };
    let both = Founding { spoilage: Some(rule), ..founding(Some(RULE)) };
    let g = die(both.generate().unwrap(), &["lineage:4"]);
    let g = mouth(g, "lineage:3", |old| rule.founded(IntakePort::live(crate::process::NisKind::Consumer).supported_by(old.support().unwrap())));
    let seen = run(gather(g, &["lineage:3", "lineage:4"]), "lineage:3", 40);
    assert!(seen.of("lineage:4") > 0, "the jaw ate carrion");
    let carrion: Vec<_> = seen.harms.iter().filter(|h| h.lineage == "lineage:4").collect();
    assert!(!carrion.is_empty() && carrion.iter().all(|h| !h.tough && h.flesh == Flesh::Toxic), "which had aged to toxin");
    assert!(seen.closed);
}

/// What the dead plants, their matter all in `tissue`, still hold in tough and
/// in soft parts after `ticks` of rot.
fn rotted(every: u64, ticks: u64, tissue_of: bool) -> (u64, u64, bool) {
    let f = Founding { rot: Some(Rot::default()), ..founding(Some(Categories { wood_rot_every: every, ..RULE })) };
    let g = die(tissue(f.generate().unwrap(), Some(tissue_of)), &["lineage:0"]);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let mut closed = true;
    for _ in 0..ticks {
        s.advance(1).unwrap();
        closed &= s.sim.budget().total() == total;
    }
    let (mut hard, mut soft) = (0, 0);
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.lineage == "lineage:0") {
        soft += g.entity.accounts.values().sum::<u64>() * g.count;
        for (id, p) in &g.entity.parts {
            let held: u64 = p.matter.values().sum::<u64>() * g.count;
            if tough(&g.entity, *id) { hard += held } else { soft += held }
        }
    }
    (hard, soft, closed)
}

#[test]
fn wood_rots_slowest_and_the_matter_stays_closed() {
    let (hard1, soft_of1, closed1) = rotted(1, 30, true);
    let (hard9, _, closed9) = rotted(9, 30, true);
    assert!(closed1 && closed9, "rot moves matter and makes none");
    assert!(hard9 > hard1, "wood rots slower when it rests: {hard9} against {hard1}");
    // Litter is unaffected by the wood's rest.
    let (_, soft1, _) = rotted(1, 30, false);
    let (hard, soft9, _) = rotted(9, 30, false);
    assert_eq!(soft1, soft9);
    assert_eq!(hard, 0, "a plant of soft tissue holds no wood");
    // At one the rule is rot's single rate.
    let f = Founding { rot: Some(Rot::default()), ..founding(None) };
    let g = die(tissue(f.generate().unwrap(), Some(true)), &["lineage:0"]);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    s.advance(30).unwrap();
    let left: u64 = s.sim.state().population.groups.values().filter(|g| g.entity.lineage == "lineage:0")
        .map(|g| crate::anatomy::books(&g.entity).values().sum::<u64>() * g.count).sum();
    assert_eq!(left, hard1 + soft_of1);
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let hash = |mode| {
        let mut s = Session::new(world(Some(RULE), Some(true)), mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
    let rotting = |mode| {
        let f = Founding { rot: Some(Rot::default()), ..founding(Some(Categories { wood_rot_every: 4, ..RULE })) };
        let mut s = Session::new(die(f.generate().unwrap(), &["lineage:0"]), mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(rotting(Execution::Individuals), rotting(Execution::Grouped));
}
