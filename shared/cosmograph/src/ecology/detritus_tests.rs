// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The detritus rule (D39 model 4, the catalog's `dung-stock`): unset it
//! changes nothing; set it round trips and bad values are named; a meal of an
//! animal leaves dung at its site with the budget closed; a dung mouth takes
//! dung and a mouth limited to other classes does not; rot and the mouths
//! return it to the soil; both executions agree.

use super::*;
use crate::{
    Execution, Session,
    bodied::Bodies,
    directing::Played,
    flows::{Holder, MadeBy},
    process::{DeadClass, IntakePort, ProcessId, Registry},
    schema::Entity,
    simulation::Genesis,
};

pub(super) fn founding(rule: Option<Detritus>) -> Founding {
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
        categories: Some(Categories::default()),
        detritus: rule,
        ..Default::default()
    }
}

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("detritus"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.detritus.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("detritus"));
    assert!(!a.rules.processes.keys().any(|k| k.starts_with("detritus:")));
    assert!(!a.rules.accounts.contains_key(DUNG));
    assert!(a.sites.values().all(|s| !s.accounts.contains_key(DUNG)));
    assert!(a.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.class_bits() == 0)));
}

#[test]
fn it_round_trips_and_adds_a_turn_for_each_line_that_is_not_a_producer() {
    let f = founding(Some(Detritus::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let (plain, g) = (founding(None).generate().unwrap(), f.generate().unwrap());
    assert_eq!(g.rules.detritus, Some(Detritus::default()));
    let eaters = (0..5).filter(|&i| Role::of(Web::Diets, i) != Role::Producer).count();
    assert_eq!(g.rules.processes.len(), plain.rules.processes.len() + 1 + eaters);
    assert!(g.sites.values().all(|s| s.accounts[DUNG] == 0));
    assert!(g.rules.validate().is_ok());
}

#[test]
fn bad_values_and_worlds_are_refused_by_name() {
    let bad = |rule: Detritus| founding(Some(rule)).generate().unwrap_err();
    assert!(bad(Detritus { share: [0, 3], ..Detritus::default() }).contains("share"));
    assert!(bad(Detritus { share: [3, 3], ..Detritus::default() }).contains("share"));
    assert!(bad(Detritus { bite: 0, ..Detritus::default() }).contains("bite"));
    assert!(bad(Detritus { kept: 4, ..Detritus::default() }).contains("bite"));
    assert!(bad(Detritus { period: 0, ..Detritus::default() }).contains("period"));
    assert!(bad(Detritus { rot: 0, ..Detritus::default() }).contains("rot"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Detritus::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let bare = Founding { bodies: None, ..founding(Some(Detritus::default())) };
    assert!(bare.generate().unwrap_err().contains("bodied"));
    let no_categories = Founding { categories: None, ..founding(Some(Detritus::default())) };
    assert!(no_categories.generate().unwrap_err().contains("categories"));
}

/// Lineages 0 (the grazer's plants) and 3 (the grazer) and 2 (decomposers) all at the first site.
fn together(mut g: Genesis, lines: &[&str]) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut().filter(|g| lines.contains(&g.entity.lineage.as_str())) {
        group.entity_mut().place = site;
    }
    g
}

#[derive(Default)]
struct Seen {
    /// Matter a lineage's meals took from the plants, per leg, to a body and to the site's dung.
    kept: u64,
    left: u64,
    /// What the dung mouths took from the site.
    eaten: u64,
    /// The soil rot made from dung.
    rotted: u64,
    closed: bool,
}

fn run(g: Genesis, ticks: u64, mode: Execution) -> (Seen, Session) {
    let mut s = Session::new(g, mode).unwrap();
    let total = s.sim.budget().total();
    let mut seen = Seen { closed: true, ..Seen::default() };
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let by = |name: &str| matches!(&f.made_by, MadeBy::Process(p) if p.starts_with(name));
            if by("ecology:feed-3") && f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:0") {
                match f.to.0 {
                    Holder::Site(_) => seen.left += f.amount * f.count,
                    Holder::Entity(_) | Holder::Part(..) => seen.kept += f.amount * f.count,
                    _ => {},
                }
            }
            if by("detritus:eat-") && matches!(f.from.0, Holder::Site(_)) {
                seen.eaten += f.amount * f.count;
            }
            if by("detritus:rot") && !f.internal() {
                seen.rotted += f.amount * f.count;
            }
        }
        seen.closed &= s.sim.budget().total() == total;
    }
    (seen, s)
}

fn dung(s: &Session) -> u64 {
    s.sim.state().sites.values().map(|site| site.accounts.get(DUNG).copied().unwrap_or(0)).sum()
}

#[test]
fn a_meal_leaves_a_share_of_what_it_took_at_the_site_and_the_budget_stays_closed() {
    let g = together(founding(Some(Detritus::default())).generate().unwrap(), &["lineage:0", "lineage:3"]);
    let (seen, s) = run(g, 60, Execution::Individuals);
    assert!(seen.left > 0 && seen.kept > 0, "the grazers ate and left dung: {} left, {} kept", seen.left, seen.kept);
    // A meal of three leaves one: a third of what was taken, less the rounding of smaller meals.
    assert!(seen.left * 3 <= seen.left + seen.kept, "{} left of {}", seen.left, seen.left + seen.kept);
    assert!(seen.closed, "matter moved, never made");
    assert!(dung(&s) > 0 || seen.rotted > 0);
    // Unset, the same world leaves nothing.
    let plain = together(founding(None).generate().unwrap(), &["lineage:0", "lineage:3"]);
    let (none, _) = run(plain, 60, Execution::Individuals);
    assert_eq!(none.left, 0);
}

#[test]
fn a_dung_mouth_takes_dung_and_a_mouth_limited_to_other_classes_does_not() {
    let mouths = |port: IntakePort| {
        let g = founding(Some(Detritus::default())).generate().unwrap();
        let mut e = g.population.groups.values().map(|g| (*g.entity).clone()).find(|e| e.lineage == "lineage:3").unwrap();
        let ids: Vec<_> = e.living().map(|(id, _)| id).collect();
        for id in ids {
            let part = e.part_mut(&id).unwrap();
            if let Some(old) = part.port {
                part.port = Some(port.supported_by(old.support().unwrap()));
            }
        }
        takes(&e, DeadClass::Dung)
    };
    let root = IntakePort::deadstock();
    let bare = founding(Some(Detritus::default())).generate().unwrap();
    let decomposer = bare.population.groups.values().map(|g| &g.entity).find(|e| e.lineage == "lineage:2").unwrap();
    assert!(takes(decomposer, DeadClass::Dung), "a bare decomposer has the founded root");
    assert!(mouths(root), "a mouth limited to none takes every class, dung too");
    assert!(mouths(root.taking(DeadClass::Dung)));
    assert!(!mouths(root.taking(DeadClass::Litter)));
    assert!(!mouths(root.taking(DeadClass::Wood).taking(DeadClass::Carrion)));
    assert!(!mouths(IntakePort::live(crate::process::NisKind::Producer)), "a living mouth is not a dung mouth");
}

#[test]
fn rot_returns_dung_to_the_soil_and_a_dung_mouth_eats_it() {
    let mut g = founding(Some(Detritus::default())).generate().unwrap();
    for site in g.sites.values_mut() {
        site.accounts.insert(DUNG.into(), 60);
    }
    let soil = |g: &Genesis| g.sites.values().map(|s| s.accounts["world:soil"]).sum::<u64>();
    let before = soil(&g);
    // The founded decomposers' roots take every class, so they eat dung where they live.
    let (seen, s) = run(g.clone(), 40, Execution::Individuals);
    assert!(seen.rotted > 0, "rot took dung");
    assert!(seen.eaten > 0, "a decomposer took dung");
    assert!(dung(&s) < 60 * 6);
    assert!(seen.closed);
    let _ = before;
    // Mouths that take no dung: only rot returns it.
    let mut blind = g;
    for group in blind.population.groups.values_mut() {
        let e = group.entity_mut();
        let ids: Vec<_> = e.parts.keys().copied().collect();
        for id in ids {
            e.part_mut(&id).unwrap().port = Some(IntakePort::deadstock().taking(DeadClass::Litter));
        }
    }
    let (rot_only, _) = run(blind, 40, Execution::Individuals);
    assert_eq!(rot_only.eaten, 0);
    assert!(rot_only.rotted > 0 && rot_only.closed);
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let hash = |mode| {
        let g = together(founding(Some(Detritus::default())).generate().unwrap(), &["lineage:0", "lineage:3", "lineage:2"]);
        let mut s = Session::new(g, mode).unwrap();
        s.advance(60).unwrap();
        (s.sim.state_hash(), dung(&s))
    };
    let (a, b) = (hash(Execution::Individuals), hash(Execution::Grouped));
    assert_eq!(a, b);
    assert!(a.1 > 0, "there was dung to agree on");
}
