// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Execution, Founding, rules::*};

fn session() -> Session {
    let mut g = Founding {
        seed: 7,
        sites: 1,
        population: 80,
        cohort_size: 8,
        lineages: 3,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.functions = default_functions();
    g.rules.shapes = default_shapes();
    let digestive = default_systems().remove("system:digestive").unwrap();
    for c in g.population.groups.values_mut() {
        let e = &mut c.entity;
        if e.lineage == "lineage:1" {
            e.parts.get_mut(&PartId(0)).unwrap().functions =
                BTreeSet::from(["function:intake".into(), "function:store".into()]);
            e.systems
                .insert("system:digestive".into(), digestive.clone());
            c.count = 1;
        }
    }
    // Rebuild identities after shrinking the cohorts.
    let mut population = crate::population::Population::default();
    for c in g.population.groups.values() {
        population.insert(c.entity.clone(), c.count).unwrap();
    }
    g.population = population;
    Session::new(g, Execution::Individuals).unwrap()
}

#[test]
fn complexity_counts_living_expression_and_realization_once() {
    let mut s = session();
    assert_eq!(complexity(&s, "lineage:1"), 3);
    let id = *s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, c)| c.entity.lineage == "lineage:1")
        .unwrap()
        .0;
    let mut duplicate = s.sim.state().population.get(id).unwrap().clone();
    duplicate
        .parts
        .get_mut(&PartId(0))
        .unwrap()
        .functions
        .insert("foreign:function".into());
    duplicate.systems.insert(
        "system:respiratory".into(),
        default_systems().remove("system:respiratory").unwrap(),
    );
    s.sim.state.population.insert(duplicate, 50).unwrap();
    assert_eq!(
        complexity(&s, "lineage:1"),
        3,
        "members and foreign keys add nothing"
    );
    let mut varied = s.sim.state().population.get(id).unwrap().clone();
    varied.parts.get_mut(&PartId(0)).unwrap().functions =
        BTreeSet::from(["function:secrete".into()]);
    varied.systems.insert(
        "system:glandular".into(),
        default_systems().remove("system:glandular").unwrap(),
    );
    let mut cut = varied.clone();
    cut.parts.get_mut(&PartId(0)).unwrap().functions = BTreeSet::from(["function:respire".into()]);
    let mut doc = crate::geometry::document(&crate::geometry::Frame::default());
    doc.parts[0].severed = true;
    cut.body = Some(doc);
    s.sim.state.population.insert(cut, 1).unwrap();
    assert_eq!(complexity(&s, "lineage:1"), 3, "tombstones express nothing");
    s.sim.state.population.insert(varied, 1).unwrap();
    assert_eq!(
        complexity(&s, "lineage:1"),
        5,
        "varied members contribute distinct names"
    );
    for c in s.sim.state.population.groups.values_mut() {
        if c.entity.lineage == "lineage:1" {
            c.entity.alive = false;
        }
    }
    assert_eq!(
        complexity(&s, "lineage:1"),
        0,
        "dead members realize no system"
    );
}

#[test]
fn boundary_uses_complexity_before_members_and_key_breaks_ties() {
    let mut s = session();
    assert!(standing(&s, "lineage:0").1 > standing(&s, "lineage:1").1);
    let offer = |_: &Session, _: &str| {
        vec![Candidate {
            name: "candidate:identical".into(),
            commands: vec![],
        }]
    };
    let turns = adapt(&mut s, None, &offer, 0).unwrap();
    assert_eq!(
        turns.iter().map(|t| t.lineage.as_str()).collect::<Vec<_>>(),
        ["lineage:1", "lineage:0", "lineage:2", "world:ground"]
    );
    let turns = adapt(&mut s, Some("lineage:1"), &offer, 0).unwrap();
    assert!(!turns.iter().any(|t| t.lineage == "lineage:1"));
    for c in s.sim.state.population.groups.values_mut() {
        if c.entity.lineage == "lineage:2" {
            c.entity.alive = false;
        }
    }
    let turns = adapt(&mut s, None, &offer, 0).unwrap();
    assert!(!turns.iter().any(|t| t.lineage == "lineage:2"));
}
