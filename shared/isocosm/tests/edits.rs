// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP4 in the sim (ruling 696): an edit is an asserted fact in the state and
//! the history, replayed by isometer; a run with one is assisted.

use isocosm::{
    Command, Execution, Founding, Session,
    map::{Grid, Layout},
};
use isometer_space::{Atlas, Edit, Op, Shape};

fn mapped(seed: u64) -> isocosm::simulation::Genesis {
    let grid = Grid::drawn(seed);
    Founding {
        seed,
        sites: grid.width * grid.height,
        map: Some(Layout::Grid(grid)),
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

fn carve(sim: &isocosm::Simulation) -> Edit {
    let atlas = sim.atlas().unwrap();
    let top = atlas.lift(0, 0, [0, 0]).unwrap().surface[0];
    Edit {
        op: Op::Carve,
        shape: Shape::Sphere {
            centre: [2, top, 2],
            radius: 2,
        },
    }
}

#[test]
fn an_edit_is_a_fact_isometer_replays_and_a_save_keeps() {
    let genesis = mapped(41);
    let mut session = Session::new(genesis.clone(), Execution::Individuals).unwrap();
    let before = session.sim.witness().digest();
    assert!(session.sim.lift(0, 0, [0, 0]).unwrap().exceptions.is_empty());
    let edit = carve(&session.sim);
    let moved = session.sim.moved(0, &edit).unwrap();
    assert!(!moved.is_empty(), "a carve at the surface moved nothing");
    session
        .command(Command::Edit {
            site: 0,
            edit: edit.clone(),
        })
        .unwrap();
    assert!(session.assisted());
    assert_eq!(session.sim.state().edits.len(), 1);
    assert_ne!(session.sim.witness().digest(), before);
    let chunk = session.sim.lift(0, 0, [0, 0]).unwrap();
    assert!(!chunk.exceptions.is_empty());

    let json = serde_json::to_vec(&session.save()).unwrap();
    let loaded = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(loaded.sim.lift(0, 0, [0, 0]).unwrap(), chunk);
}

#[test]
fn a_run_without_edits_hashes_as_before() {
    let session = Session::new(mapped(43), Execution::Individuals).unwrap();
    let json = serde_json::to_string(session.sim.state()).unwrap();
    assert!(!json.contains("\"edits\""));
    assert!(!session.assisted());
}

#[test]
fn edits_the_lift_cannot_read_are_refused() {
    let mut session = Session::new(mapped(47), Execution::Individuals).unwrap();
    let edit = carve(&session.sim);
    let fill_air = Edit {
        op: Op::Fill(0),
        ..edit.clone()
    };
    assert!(session.command(Command::Edit { site: 0, edit: fill_air }).is_err());
    assert!(session.command(Command::Edit { site: 9_999, edit }).is_err());
    let unmapped = Session::new(Founding::default().generate().unwrap(), Execution::Individuals);
    let mut unmapped = unmapped.unwrap();
    let flat = Edit {
        op: Op::Carve,
        shape: Shape::Box {
            min: [0; 3],
            max: [1; 3],
        },
    };
    assert!(unmapped.command(Command::Edit { site: 0, edit: flat }).is_err());
}
