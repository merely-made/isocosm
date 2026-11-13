// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Spoilage as a founding rule: unset it changes nothing, set it round
//! trips, bad values are named, a corpse ages by the tick it died, and a
//! body's death is remembered only where the rule is set.

use super::*;
use crate::{Execution, Session, bodied::Bodies, directing::Played, process::{IntakePort, NisKind, Tolerance}};

pub(super) fn founding(spoilage: Option<Spoilage>) -> Founding {
    let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(5) };
    Founding {
        seed: 5,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(crate::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies::default()),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        web: Web::Diets,
        lineages: 5,
        spoilage,
        ..Default::default()
    }
}

#[test]
fn unset_spoilage_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("spoilage"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.spoilage.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("spoilage"));
    assert!(a.population.groups.values().all(|g| g.entity.died.is_none()));
    assert!(a.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.tolerance_bits() == 0)));
}

#[test]
fn spoilage_round_trips_and_gives_the_founded_mouths_their_tolerances() {
    let f = founding(Some(Spoilage::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"spoilage\""));
    assert!(!json.contains("decomposer_flesh_toxic"), "the default is not written");
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let off = Spoilage { decomposer_flesh_toxic: false, ..Spoilage::default() };
    let json = serde_json::to_string(&off).unwrap();
    assert!(json.contains("decomposer_flesh_toxic"));
    assert_eq!(serde_json::from_str::<Spoilage>(&json).unwrap(), off);
    let g = f.generate().unwrap();
    assert_eq!(g.rules.spoilage, Some(Spoilage::default()));
    // The bulk plant mouth keeps no tolerance: nothing in the played A1 world has a jaw or a root.
    let tolerant = g.population.groups.values().flat_map(|g| g.entity.parts.values()).filter_map(|p| p.port).filter(|p| p.tolerance_bits() != 0).count();
    assert_eq!(tolerant, 0);
    // The founded jaw and root, by the rule.
    let rule = Spoilage::default();
    let jaw = rule.founded(IntakePort::live(NisKind::Consumer));
    assert!(jaw.tolerates(Tolerance::AgedFlesh) && !jaw.tolerates(Tolerance::ToxicFlesh) && !jaw.tolerates(Tolerance::DecomposerFlesh));
    let root = rule.founded(IntakePort::deadstock());
    assert!(root.tolerates(Tolerance::AgedFlesh) && root.tolerates(Tolerance::ToxicFlesh) && root.tolerates(Tolerance::DecomposerFlesh));
    let bulk = rule.founded(IntakePort::live(NisKind::Producer));
    assert_eq!(bulk.tolerance_bits(), 0);
}

#[test]
fn spoilage_names_what_it_cannot_found() {
    let bad = |s: Spoilage| founding(Some(s)).generate().unwrap_err();
    assert!(bad(Spoilage { aged_after: 0, ..Spoilage::default() }).contains("aged_after"));
    assert!(bad(Spoilage { aged_after: 50, toxic_after: 50, ..Spoilage::default() }).contains("toxic_after"));
    assert!(bad(Spoilage { harm: 0, ..Spoilage::default() }).contains("harm"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Spoilage::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let no_ecology = Founding { ecology: false, ..founding(Some(Spoilage::default())) };
    assert!(no_ecology.generate().unwrap_err().contains("ecology"));
}

#[test]
fn a_corpse_is_fresh_then_aged_then_toxic_at_the_configured_ticks() {
    let rule = Spoilage { aged_after: 10, toxic_after: 25, ..Spoilage::default() };
    let class = |now| rule.class(Some(100), now);
    assert_eq!((class(100), class(109)), (Flesh::Fresh, Flesh::Fresh));
    assert_eq!((class(110), class(124)), (Flesh::Aged, Flesh::Aged));
    assert_eq!((class(125), class(10_000)), (Flesh::Toxic, Flesh::Toxic));
    assert_eq!(rule.class(None, 10_000), Flesh::Fresh, "an unknown death reads fresh");
    // By a body: the living are fresh whatever their `died`.
    let g = founding(Some(rule)).generate().unwrap();
    let mut e = g.population.groups.values().next().unwrap().entity.as_ref().clone();
    assert_eq!(rule.of(&e, 500), Flesh::Fresh);
    e.alive = false;
    e.died = Some(100);
    assert_eq!(rule.of(&e, 105), Flesh::Fresh);
    assert_eq!(rule.of(&e, 115), Flesh::Aged);
    assert_eq!(rule.of(&e, 130), Flesh::Toxic);
}

#[test]
fn a_death_is_remembered_only_where_the_rule_is_set() {
    let run = |spoilage| {
        let mut f = founding(spoilage);
        f.bodies = Some(Bodies { hazard: crate::harm::hazard::RATE, wound: crate::harm::hazard::CELLS, ..Bodies::default() });
        let mut s = Session::new(f.generate().unwrap(), Execution::Individuals).unwrap();
        s.advance(120).unwrap();
        let now = s.sim.state().tick;
        let dead: Vec<_> = s.sim.state().population.groups.values().filter(|g| !g.entity.alive).map(|g| g.entity.died).collect();
        (dead, now)
    };
    let (with, now) = run(Some(Spoilage::default()));
    assert!(!with.is_empty(), "some bodies died in 120 ticks");
    assert!(with.iter().filter(|d| d.is_some_and(|t| t <= now)).count() * 2 >= with.len(), "most of the dead have a death tick");
    let (without, _) = run(None);
    assert!(without.iter().all(Option::is_none));
}
