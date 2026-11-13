// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mutualism as a founding rule: unset it changes nothing, set it round trips,
//! bad values and worlds are named, and the organ readings say what they say.

use super::*;
use crate::{
    bodied::Bodies,
    directing::Played,
    process::{IntakePort, ProcessId, Registry, Trade},
};

pub(super) fn founding(rule: Option<Mutualism>) -> Founding {
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
        mutualism: rule,
        ..Default::default()
    }
}

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("mutualism"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.mutualism.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("mutualism"));
    assert!(!a.rules.processes.keys().any(|k| k.starts_with("mutualism:")));
    assert!(!a.rules.conditions.iter().any(|k| k.starts_with("mutualism:")));
    assert!(a.sites.values().all(|s| s.conditions.keys().all(|k| !k.starts_with("mutualism:"))));
    assert!(!serde_json::to_string(&IntakePort::none()).unwrap().contains("trade"));
}

#[test]
fn it_round_trips_and_adds_the_turns_and_counts_and_nothing_else_to_a_founded_body() {
    let f = founding(Some(Mutualism::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let (plain, g) = (founding(None).generate().unwrap(), f.generate().unwrap());
    assert_eq!(g.rules.mutualism, Some(Mutualism::default()));
    // One producer (0) and one decomposer (2) in the five-line web: a grow, a nutrient turn, a sugar turn and a digestion.
    for id in ["mutualism:grow-0", "mutualism:nutrient-2-0", "mutualism:sugar-0-2", "mutualism:digest-2-0"] {
        assert!(g.rules.processes.contains_key(id), "{id}");
    }
    assert_eq!(g.rules.processes.len(), plain.rules.processes.len() + 4);
    assert!(g.sites.values().all(|s| s.conditions["mutualism:trust-2-0"] == 1));
    assert_eq!(g.population, plain.population, "the rule gives no body anything");
    assert!(g.rules.validate().is_ok());
}

#[test]
fn bad_values_and_worlds_are_refused_by_name() {
    let bad = |m: Mutualism| founding(Some(m)).generate().unwrap_err();
    assert!(bad(Mutualism { period: 0, ..Mutualism::default() }).contains("period"));
    assert!(bad(Mutualism { nutrient: 0, ..Mutualism::default() }).contains("nutrient"));
    assert!(bad(Mutualism { fee: 0, ..Mutualism::default() }).contains("fee"));
    assert!(bad(Mutualism { sugar: 0, ..Mutualism::default() }).contains("sugar"));
    assert!(bad(Mutualism { credit: 0, ..Mutualism::default() }).contains("credit"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Mutualism::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let bare = Founding { bodies: None, ..founding(Some(Mutualism::default())) };
    assert!(bare.generate().unwrap_err().contains("bodied"));
    assert!(Founding { ecology: false, ..founding(Some(Mutualism::default())) }.generate().is_err());
}

#[test]
fn an_organ_trades_only_while_its_tissue_is_held() {
    let g = founding(None).generate().unwrap();
    let mut e = g.population.groups.values().find(|g| g.entity.lineage == "lineage:0").unwrap().entity.as_ref().clone();
    assert!(!trades(&e, Trade::Gives) && !trades(&e, Trade::Takes));
    let id = *e.parts.keys().next().unwrap();
    let held = Registry::native().get(&ProcessId::new("mesocosm", "adhesion")).unwrap().reference();
    let part = e.part_mut(&id).unwrap();
    part.cells = [("function:adhesion".to_string(), 2)].into();
    part.functions = ["function:adhesion".to_string()].into();
    part.port = Some(IntakePort::none().trading(Trade::Takes).supported_by(held));
    assert!(trades(&e, Trade::Takes) && !trades(&e, Trade::Gives), "an organ that only takes");
    e.part_mut(&id).unwrap().cells.clear();
    assert!(!trades(&e, Trade::Takes), "no tissue, no organ");
}
