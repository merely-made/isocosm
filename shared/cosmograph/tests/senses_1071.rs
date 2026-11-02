// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1071, doctrine Bodies 15: what is not inert senses, behind
//! `Bodies::sensing`, on for new foundings and off where a saved founding
//! does not name it.

use cosmograph::{Execution, Founding, Session, bodied::Bodies, map::{Grid, Layout}, rules::Query,
    senses::{SMELL, form, Form}};

fn founding(seed: u64, sensing: bool) -> Founding {
    let grid = Grid { width: 3, height: 2, ..Grid::drawn(seed) };
    Founding { seed, sites: 6, map: Some(Layout::Grid(grid)), ecology: true,
        ecology_rates: cosmograph::ecology::Rates::A1,
        bodies: Some(Bodies { sensing, ..Bodies::default() }),
        played: Some(cosmograph::directing::Played { lineage: 1, region_sites: 2 }), ..Default::default() }
}

#[test]
fn a_new_founding_senses_and_an_unnamed_one_does_not() {
    assert!(Bodies::default().sensing);
    let json = serde_json::to_value(Bodies { sensing: false, smell: [0, 0], ..Bodies::default() }).unwrap();
    assert!(json.get("sensing").is_none() && json.get("smell").is_none(), "an unnamed flag reads as off");
    let old: Bodies = serde_json::from_value(json).unwrap();
    assert!(!old.sensing);

    let g = founding(9, true).generate().unwrap();
    let senses = g.rules.senses.as_ref().expect("a sensing world");
    for (key, l) in &g.lineages {
        let animal = l.kingdom == "kingdom:fauna";
        assert_eq!(senses.smell.contains_key(key), animal, "{key}");
        if let Some(r) = senses.smell.get(key) {
            assert!((SMELL[0]..=SMELL[1]).contains(r), "{key} smells {r}");
        }
    }
    let off = founding(9, false).generate().unwrap();
    assert!(off.rules.senses.is_none());
}

#[test]
fn plants_grow_on_what_they_sense_only_when_the_world_senses() {
    let sensed = |g: &cosmograph::simulation::Genesis| g.rules.processes.iter()
        .filter(|(id, _)| id.starts_with("ecology:produce-"))
        .all(|(_, p)| p.requires.iter().any(|q| matches!(q, Query::Sensed { .. })));
    assert!(sensed(&founding(9, true).generate().unwrap()));
    let off = founding(9, false).generate().unwrap();
    assert!(off.rules.processes.iter().filter(|(id, _)| id.starts_with("ecology:produce-"))
        .all(|(_, p)| !p.requires.iter().any(|q| matches!(q, Query::Sensed { .. }))));
}

#[test]
fn each_form_reads_its_own_surroundings() {
    let s = Session::new(founding(9, true).generate().unwrap(), Execution::Grouped).unwrap();
    let sim = &s.sim;
    let mut read = std::collections::BTreeMap::new();
    for (&id, g) in &sim.state().population.groups {
        let Some(f) = form(&g.entity.kingdom) else { continue };
        let readings: &[&str] = match f {
            Form::Animal => &["sense:smell"],
            Form::Plant => &["sense:light", "sense:soil", "sense:crowding", "sense:light-share", "sense:soil-share"],
            Form::Territory => &["sense:here", "sense:downhill"],
            Form::World => &["sense:births", "sense:deaths"],
        };
        for r in readings {
            if let Some(v) = sim.sensed(id, r) {
                read.entry(format!("{f:?} {r}")).or_insert(v);
            }
        }
        // A form reads nothing of another's.
        if f != Form::Plant { assert_eq!(sim.sensed(id, "sense:light"), None); }
    }
    eprintln!("{read:?}");
    for key in ["Animal sense:smell", "Plant sense:crowding", "Plant sense:light-share"] {
        assert!(read.contains_key(key), "{key} read: {read:?}");
    }
    // The world reads its last tick's activity; at founding there is none.
    assert!(read.get("World sense:births").is_none(), "no tick before the first");
    let off = Session::new(founding(9, false).generate().unwrap(), Execution::Grouped).unwrap();
    assert!(off.sim.state().population.groups.keys().all(|id| off.sim.sensed(*id, "sense:smell").is_none()));
}

#[test]
fn an_unknown_reading_is_refused() {
    let mut g = founding(9, true).generate().unwrap();
    let p = g.rules.processes.values_mut().find(|p| p.id.starts_with("ecology:produce-")).unwrap();
    p.requires.push(Query::Sensed { reading: "sense:psychic".into(), at_least: 1 });
    assert!(g.validate().unwrap_err().contains("unknown sense reading"));
}
