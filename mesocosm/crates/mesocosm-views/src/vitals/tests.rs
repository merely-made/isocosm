// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The panel's wound and mood readings (the 2026-10-18 note to ruling 227).

use super::*;
use cosmograph::{Execution, Session, harm, history::Command, schema::PartId};
use mesocosm_runtime::Founded;

/// A generated world's played critter, in its session.
fn played() -> (Session, Id) {
    let founded = Founded::generated(3, 60).unwrap();
    let s = Session::new(founded.genesis, Execution::Individuals).unwrap();
    let critter = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method == cosmograph::schema::Method::Deliberative)
        .map(|(first, _)| *first)
        .expect("a generated world has a played lineage");
    (s, critter)
}

fn reading(s: &Session, critter: Id) -> Vitals {
    vitals_of(&s.sim, Some(critter), 1_000, None, None)
}

fn row<'a>(rows: &'a [(&'static str, String)], key: &str) -> Option<&'a str> {
    rows.iter().find(|(k, _)| *k == key).map(|(_, v)| v.as_str())
}

#[test]
fn the_panel_states_wounds_and_mood_beside_what_the_critter_holds() {
    let (mut s, critter) = played();
    let whole = reading(&s, critter);
    assert_eq!(whole.wound, Some(0));
    assert_eq!(whole.mood, s.sim.mood(critter));
    let rows = vital_rows(&whole);
    assert_eq!(row(&rows, "wounded"), Some("0 of 100"));
    assert!(row(&rows, "mood").is_some() && row(&rows, "holds").is_some());

    // Cut its limbs down to a cell each until a quarter of its cells are gone.
    let e = s.sim.state().population.get(critter).unwrap();
    let limbs: Vec<PartId> = e.living().map(|(id, _)| id).filter(|id| e.parent_of(*id).is_some()).collect();
    for id in limbs {
        let e = s.sim.state().population.get(critter).unwrap();
        if harm::wound_share(e) >= 25 {
            break;
        }
        let left = cosmograph::anatomy::living_cells(e.extent(id), &e.parts[&id]);
        if left > 1 {
            s.command(Command::Wound { entity: critter, part: Some(id), cells: left - 1 }).unwrap();
        }
    }
    let hurt = reading(&s, critter);
    let share = hurt.wound.unwrap();
    assert!(share >= 25, "{share}");
    assert!(hurt.mood.unwrap() <= whole.mood.unwrap() - 10, "a wound weighs on mood");
    let rows = vital_rows(&hurt);
    assert_eq!(row(&rows, "wounded"), Some(format!("{share} of 100").as_str()));
    assert_eq!(row(&rows, "mood"), Some(hurt.mood.unwrap().to_string().as_str()));
}

#[test]
fn a_dead_or_absent_critter_states_its_state_alone() {
    let (mut s, critter) = played();
    // Losing every cell of the root kills a body.
    let e = s.sim.state().population.get(critter).unwrap();
    let root = e.living().map(|(id, _)| id).find(|id| e.parent_of(*id).is_none()).unwrap();
    let cells = cosmograph::anatomy::living_cells(e.extent(root), &e.parts[&root]);
    s.command(Command::Wound { entity: critter, part: Some(root), cells }).unwrap();
    assert!(!s.sim.state().population.get(critter).unwrap().alive);
    for dead in [reading(&s, critter), vitals_of(&s.sim, None, 0, None, None)] {
        assert_eq!((dead.wound, dead.mood), (None, None));
        assert_eq!(vital_rows(&dead), [("state", "dead".to_owned())]);
    }
}
