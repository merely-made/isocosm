// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rot: what it lays, where the matter goes, and that a decomposer still eats.

use super::*;
use crate::{Execution, Session, bodied::Bodies, directing::Played, flows::{Holder, MadeBy}};

fn founding(rot: Option<Rot>) -> Founding {
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
        rot,
        ..Default::default()
    }
}

/// The founded world with every member of lineage 0 dead where it stands.
fn corpses(rot: Option<Rot>) -> Session {
    let mut g = founding(rot).generate().unwrap();
    for group in g.population.groups.values_mut().filter(|c| c.entity.lineage == "lineage:0") {
        group.entity_mut().alive = false;
    }
    Session::new(g, Execution::Individuals).unwrap()
}

#[test]
fn unset_rot_founds_and_hashes_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("rot"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.processes.keys().all(|id| !id.starts_with("ecology:rot")));
}

#[test]
fn rot_round_trips_and_lays_one_process_per_lineage() {
    let f = founding(Some(Rot { rate: [1, 20], ..Rot::default() }));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"rot\""));
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let g = f.generate().unwrap();
    let laid: Vec<_> = g.rules.processes.keys().filter(|id| id.starts_with("ecology:rot-")).collect();
    assert_eq!(laid.len(), f.lineages as usize);
    // Every other process is as it was founded without rot.
    let off = founding(None).generate().unwrap();
    for (id, p) in &off.rules.processes {
        assert_eq!(g.rules.processes.get(id), Some(p), "{id} unchanged");
    }
}

#[test]
fn a_rotting_share_is_at_least_one_unit() {
    let rot = Rot { rate: [1, 50], ..Rot::default() };
    assert_eq!((rot.gives(1), rot.gives(10), rot.gives(49), rot.gives(100), rot.gives(1000)), (1, 1, 1, 2, 20));
}

#[test]
fn decomposer_bites_lay_more_feedings_for_decomposers_alone() {
    let f = founding(Some(Rot { decomposer_bites: Some(3), ..Rot::default() }));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("decomposer_bites"));
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let g = f.generate().unwrap();
    let plain = founding(Some(Rot::default())).generate().unwrap();
    assert!(!serde_json::to_string(&Rot::default()).unwrap().contains("bites"), "unset is absent from the wire");
    // Lineage 2 is the decomposer of the three levels: it gets two more feedings, copies of its first.
    for n in 1..3 {
        let more = &g.rules.processes[&format!("ecology:feed-2-{n}")];
        let first = &g.rules.processes["ecology:feed-2"];
        assert_eq!(more.effects, first.effects);
        assert_eq!(more.target, first.target);
    }
    let added: Vec<_> = g.rules.processes.keys().filter(|id| !plain.rules.processes.contains_key(*id)).collect();
    assert_eq!(added, ["ecology:feed-2-1", "ecology:feed-2-2"]);
    for (id, p) in &plain.rules.processes {
        assert_eq!(g.rules.processes.get(id), Some(p), "{id} unchanged");
    }
    assert!(founding(Some(Rot { decomposer_bites: Some(0), ..Rot::default() })).generate().is_err());
}

#[test]
fn rot_names_what_it_cannot_found() {
    assert!(founding(Some(Rot { rate: [0, 5], ..Rot::default() })).generate().is_err());
    assert!(founding(Some(Rot { rate: [1, 0], ..Rot::default() })).generate().is_err());
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Rot::default())) };
    assert!(world_time.generate().is_err());
    let no_ecology = Founding { ecology: false, ..founding(Some(Rot::default())) };
    assert!(no_ecology.generate().is_err());
}

#[test]
fn a_dead_bodys_matter_reaches_the_soil_and_the_budget_stays_closed() {
    let mut s = corpses(Some(Rot { rate: [1, 2], ..Rot::default() }));
    let before = s.sim.budget();
    assert!(before.dead["lineage:0"] > 0);
    let (mut rotted, mut soil_in) = (0u64, 0u64);
    for _ in 0..20 {
        for f in s.advance_tick_with_flows().unwrap().flows {
            if f.made_by == MadeBy::Process("ecology:rot-0".into()) {
                assert!(matches!(f.from.0, Holder::Entity(_) | Holder::Part(..)), "from a body");
                assert!(f.from.1 == "matter:0-0" || f.from.1 == "world:soil", "{}", f.from.1);
                assert!(matches!(f.to.0, Holder::Site(_)));
                assert_eq!(f.to.1, "world:soil", "where producers draw");
                rotted += f.amount * f.count;
                soil_in += f.amount * f.count;
            }
        }
        assert_eq!(s.sim.budget().total(), before.total(), "no matter made or lost");
    }
    let after = s.sim.budget();
    assert!(rotted > 0 && soil_in == rotted);
    assert!(after.dead["lineage:0"] < before.dead["lineage:0"], "the dead gave matter up");
    assert!(after.sites.keys().all(|k| k == "world:soil"), "none left as a lineage's litter: {:?}", after.sites.keys());
}

#[test]
fn a_decomposer_still_eats_a_rotting_body() {
    let mut s = corpses(Some(Rot { rate: [1, 5], ..Rot::default() }));
    let (mut eaten, mut rotted) = (0u64, 0u64);
    for _ in 0..20 {
        for f in s.advance_tick_with_flows().unwrap().flows {
            let dead_lineage_0 = f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:0");
            match &f.made_by {
                MadeBy::Process(id) if id == "ecology:feed-2" && dead_lineage_0 => eaten += f.amount * f.count,
                MadeBy::Process(id) if id == "ecology:rot-0" => rotted += f.amount * f.count,
                _ => {},
            }
        }
    }
    assert!(rotted > 0, "the bodies rotted");
    assert!(eaten > 0, "the decomposer ate dead bodies while rot ran");
}

#[test]
fn rot_in_both_executions_is_the_same() {
    let hash = |mode| {
        let mut g = founding(Some(Rot { rate: [1, 5], ..Rot::default() })).generate().unwrap();
        for group in g.population.groups.values_mut().filter(|c| c.entity.lineage == "lineage:0") {
            group.entity_mut().alive = false;
        }
        let mut s = Session::new(g, mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}
