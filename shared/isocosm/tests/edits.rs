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
            by: None,
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
    assert!(session.command(Command::Edit { site: 0, edit: fill_air, by: None }).is_err());
    assert!(session.command(Command::Edit { site: 9_999, edit, by: None }).is_err());
    let unmapped = Session::new(Founding::default().generate().unwrap(), Execution::Individuals);
    let mut unmapped = unmapped.unwrap();
    let flat = Edit {
        op: Op::Carve,
        shape: Shape::Box {
            min: [0; 3],
            max: [1; 3],
        },
    };
    assert!(unmapped.command(Command::Edit { site: 0, edit: flat, by: None }).is_err());
}

/// A mapped world whose voxel materials carry a density and move through
/// a matter account of their own, `world:earth`, beside the edible soil.
fn weighed(seed: u64) -> isocosm::simulation::Genesis {
    let mut g = mapped(seed);
    let soil = g.rules.accounts["world:soil"].clone();
    g.rules.accounts.insert("world:earth".into(), soil);
    for m in g.world.materials.iter_mut().filter(|m| m.key != "world:air") {
        m.density = Some(3);
        m.account = Some("world:earth".into());
    }
    g
}

/// A living member, and a carve at the surface of its own site.
fn digger(sim: &isocosm::Simulation) -> (u64, u64, Edit) {
    let (&id, cohort) = sim.state().population.groups.iter().find(|(_, c)| c.entity.alive).unwrap();
    let site = cohort.entity.place;
    let top = sim.atlas().unwrap().lift(site, 0, [0, 0]).unwrap().surface[0];
    let edit = Edit {
        op: Op::Carve,
        shape: Shape::Sphere { centre: [2, top, 2], radius: 2 },
    };
    (id, site, edit)
}

#[test]
fn a_members_carve_credits_its_ledger_by_density_and_conserves() {
    let mut session = Session::new(weighed(51), Execution::Individuals).unwrap();
    let (actor, site, edit) = digger(&session.sim);
    let cells: i64 = session.sim.moved(site, &edit).unwrap().values().sum();
    assert!(cells > 0);
    let held = |s: &Session| s.sim.state().population.get(actor).unwrap().accounts.get("world:earth").copied().unwrap_or(0);
    let (before, matter) = (held(&session), session.sim.matter());
    session.command(Command::Edit { site, edit: edit.clone(), by: Some(actor) }).unwrap();
    assert_eq!(held(&session), before + 3 * cells as u64);
    assert_eq!(session.sim.matter(), matter + 3 * cells as u128);
    assert!(!session.assisted(), "a member's edit is no dev act");
    assert_eq!(session.sim.state().population.groups[&actor].count, 1);
    // What it dug it can build with; more than it holds is refused whole.
    let top = session.sim.atlas().unwrap().lift(site, 0, [0, 0]).unwrap().surface[16 * 32 + 16];
    let wall = |h: i64| Edit {
        op: Op::Fill(3),
        shape: Shape::Box { min: [16, top + 1, 16], max: [17, top + 1 + h, 17] },
    };
    let fill = |h: i64| Command::Edit { site, edit: wall(h), by: Some(actor) };
    assert!(session.command(fill(cells + 1)).is_err());
    session.command(fill(1)).unwrap();
    assert_eq!(held(&session), before + 3 * (cells as u64 - 1));
}

#[test]
fn a_material_without_a_density_is_the_dev_sources_alone() {
    let mut session = Session::new(mapped(53), Execution::Individuals).unwrap();
    let (actor, site, edit) = digger(&session.sim);
    let by = Some(actor);
    assert!(session.command(Command::Edit { site, edit: edit.clone(), by }).is_err());
    session.command(Command::Edit { site, edit, by: None }).unwrap();
    let mut g = weighed(53);
    g.world.materials[3].account = None;
    assert!(g.validate().is_err(), "a density without its account");
}

#[test]
fn a_member_names_its_patch_only_in_its_own_site() {
    let mut session = Session::new(mapped(57), Execution::Individuals).unwrap();
    let (actor, site, _) = digger(&session.sim);
    let patch = isometer_space::places::PlaceId { site, cell: [2, 30, 2] };
    let elsewhere = isometer_space::places::PlaceId { site: site + 1, ..patch };
    assert!(session.command(Command::Patch { entity: actor, patch: Some(elsewhere) }).is_err());
    session.command(Command::Patch { entity: actor, patch: Some(patch) }).unwrap();
    let state = session.sim.state();
    assert_eq!(state.population.get(actor).unwrap().patch, Some(patch));
    assert_eq!(state.population.groups[&actor].count, 1, "split out of its cohort");
    let json = serde_json::to_vec(&session.save()).unwrap();
    let loaded = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(loaded.sim.state().population.get(actor).unwrap().patch, Some(patch));
}
