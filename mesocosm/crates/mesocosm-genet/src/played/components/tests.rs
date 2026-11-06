// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::{
    Command, Execution, Founding, Session,
    bodied::Bodies,
    map::{Grid, Layout},
};

mod comparison;

fn native() -> Session {
    native_with_body_count(3)
}

fn native_with_body_count(body_count: u64) -> Session {
    let mut genesis = Founding {
        seed: 7,
        sites: 4,
        population: body_count * 8,
        cohort_size: 8,
        lineages: 2,
        bodies: Some(Bodies::default()),
        map: Some(Layout::Grid(Grid {
            shape: "shape:plane".into(),
            width: 2,
            height: 2,
            side: 128,
            elevation: [16, 16],
            relief: [0, 0],
            sea_per_mille: 0,
        })),
        ..Founding::default()
    }
    .generate()
    .unwrap();
    // Keep the generated native cohorts intact and co-locate their bodies in
    // the first valid grid site, so the scene exercises several supplied rows.
    for group in genesis.population.groups.values_mut() {
        if group.entity.body.is_some() {
            group.entity_mut().place = 0;
        }
    }
    Session::new(genesis, Execution::Grouped).unwrap()
}

fn scene(session: &Session) -> SiteScene {
    let scene = crate::played::scene::site_scene(&session.sim, None).unwrap();
    assert!(
        scene.bodies.len() >= 2,
        "the control needs several native bodies"
    );
    assert!(scene.bodies.iter().any(|body| !body.parts.is_empty()));
    scene
}

/// Compare every body field; float bits keep signed zero and exact fractions.
fn same_body(a: &PlacedBody, b: &PlacedBody) {
    assert_eq!(a.id, b.id);
    assert_eq!(a.document, b.document);
    assert_eq!(a.at, b.at);
    assert_eq!(a.physical, b.physical);
    assert_eq!(a.walks.len(), b.walks.len());
    for (a, b) in a.walks.iter().zip(&b.walks) {
        assert_eq!(a.poses, b.poses);
        assert_eq!(a.stride, b.stride);
    }
    assert_eq!(a.tint.map(f32::to_bits), b.tint.map(f32::to_bits));
    assert_eq!(a.parts, b.parts);
    assert_eq!(a.materials.len(), b.materials.len());
    for (a, b) in a.materials.iter().zip(&b.materials) {
        assert_eq!(
            (a.part, a.material, a.fraction.to_bits()),
            (b.part, b.material, b.fraction.to_bits())
        );
    }
    assert_eq!(a.palette, b.palette);
    assert_eq!(a.alive, b.alive);
}

fn same_roster(a: &[PlacedBody], b: &[PlacedBody]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        same_body(a, b);
    }
}

#[test]
fn native_scene_roundtrip_preserves_canonical_body_data_and_cohort_representation() {
    let session = native();
    let scene = scene(&session);
    let indexed = ComponentScene::capture(&scene);
    assert_eq!(indexed.len(), scene.bodies.len());
    assert_eq!(
        indexed.counts(),
        ComponentCounts {
            documents: scene.bodies.len(),
            placements: scene.bodies.len(),
            appearances: scene.bodies.len(),
        }
    );
    same_roster(&indexed.bodies().unwrap(), &scene.bodies);
    assert!(
        session
            .sim
            .state()
            .population
            .groups
            .values()
            .any(|group| group.count > 1)
    );
    assert!(indexed.len() < session.sim.state().population.count() as usize);
    for body in indexed.bodies().unwrap() {
        assert!(session.sim.state().population.groups.contains_key(&body.id));
    }
}

#[test]
fn scrambled_capture_has_canonical_order_and_duplicate_subjects_refuse() {
    let mut scene = scene(&native());
    let canonical = scene.bodies.clone();
    scene.bodies.reverse();
    let indexed = ComponentScene::capture(&scene);
    same_roster(&indexed.bodies().unwrap(), &canonical);
    scene.bodies.push(scene.bodies[0].clone());
    assert!(ComponentScene::capture(&scene).bodies().is_err());
}

#[test]
fn missing_or_differently_keyed_components_refuse_instead_of_omitting_subjects() {
    let scene = scene(&native());
    let indexed = ComponentScene::capture(&scene);
    let id = scene.bodies[0].id;
    for component in 0..3 {
        let mut broken = indexed.clone();
        match component {
            0 => assert!(broken.documents.remove(id)),
            1 => assert!(broken.placements.remove(id)),
            _ => assert!(broken.appearances.remove(id)),
        }
        assert!(broken.bodies().is_err());
    }
    // Equal cardinalities do not establish equal memberships.
    let mut broken = indexed.clone();
    let moved = broken.appearances.get(id).unwrap().clone();
    broken.appearances.remove(id);
    broken.appearances.set(u64::MAX, moved);
    assert_eq!(broken.counts(), indexed.counts());
    assert!(broken.bodies().is_err());
    same_roster(&indexed.bodies().unwrap(), &scene.bodies);
}

#[test]
fn removal_reinsertion_and_changed_components_are_isolated_from_other_snapshots() {
    let scene = scene(&native());
    let indexed = ComponentScene::capture(&scene);
    let mut edited = indexed.clone();
    let id = scene.bodies[0].id;
    assert!(std::ptr::eq(
        indexed.documents.get(id).unwrap(),
        edited.documents.get(id).unwrap()
    ));
    for remove in [0, 1, 2] {
        match remove {
            0 => assert!(edited.documents.remove(id)),
            1 => assert!(edited.placements.remove(id)),
            _ => assert!(edited.appearances.remove(id)),
        }
    }
    edited.supplied -= 1;
    let expected: Vec<_> = scene
        .bodies
        .iter()
        .filter(|body| body.id != id)
        .cloned()
        .collect();
    same_roster(&edited.bodies().unwrap(), &expected);
    let mut changed = scene.bodies[0].clone();
    changed.at[0] += 1;
    changed.tint = [0.125, 0.25, 0.5];
    changed.alive = !changed.alive;
    edited.insert(&changed);
    edited.supplied += 1;
    let restored = edited.bodies().unwrap();
    same_body(
        restored.iter().find(|body| body.id == id).unwrap(),
        &changed,
    );
    same_roster(&indexed.bodies().unwrap(), &scene.bodies);
    assert_eq!(edited.len(), indexed.len());
}

#[test]
fn physical_walk_and_appearance_fields_are_transported_without_reinterpretation() {
    let mut scene = scene(&native());
    let body = &mut scene.bodies[0];
    let root = body.document.root;
    // This is a transport control, not admission of a new physical trajectory.
    let from = cosmograph::spatial::Pose {
        at: [64, 17, 64],
        yaw: isometer::core::Yaw::Quarter,
        feet: std::collections::BTreeMap::from([(root, [1024, 0, 0])]),
    };
    let mut to = from.clone();
    to.at[0] += 1;
    body.physical = Some(to.clone());
    body.walks = vec![Walk {
        poses: [from, to],
        stride: Some(isometer::core::legs::Stride {
            stages: vec![isometer::core::legs::Stage {
                foot: root,
                root: [1024, 0, 0],
                lift: 1,
                target: Some([2048, 0, 0]),
                turn: 0,
            }],
            clearance: true,
            turn: 1,
        }),
    }];
    body.tint = [-0.0, 0.5, 1.0];
    let reconstructed = ComponentScene::capture(&scene).bodies().unwrap();
    same_roster(&reconstructed, &scene.bodies);
    scene.bodies.clear();
    let empty = ComponentScene::capture(&scene);
    assert!(empty.is_empty());
    assert_eq!(empty.counts(), ComponentCounts::default());
    assert!(empty.bodies().unwrap().is_empty());
}

#[test]
fn component_roundtrip_leaves_native_save_history_and_replay_unchanged() {
    let mut session = native();
    let id = *session
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, group)| group.entity.body.is_some())
        .unwrap()
        .0;
    session.command(Command::Inspect(id)).unwrap();
    session.advance(2).unwrap();
    let saved = session.save();
    let bytes = serde_json::to_vec(&saved).unwrap();
    let hash = session.sim.state_hash();
    let admitted = scene(&session);
    let indexed = ComponentScene::capture(&admitted);
    same_roster(&indexed.bodies().unwrap(), &admitted.bodies);
    assert_eq!(serde_json::to_vec(&session.save()).unwrap(), bytes);
    assert_eq!(session.sim.state_hash(), hash);
    assert_eq!(session.entries, saved.entries);

    let mut replay = Session::load(saved, Execution::Grouped).unwrap();
    let replay_scene = scene(&replay);
    same_roster(
        &ComponentScene::capture(&replay_scene).bodies().unwrap(),
        &admitted.bodies,
    );
    let fork = session
        .fork_at(session.sim.state().tick, "branch:component-scene".into())
        .unwrap();
    same_roster(
        &ComponentScene::capture(&scene(&fork)).bodies().unwrap(),
        &admitted.bodies,
    );
    assert_eq!(serde_json::to_vec(&session.save()).unwrap(), bytes);
    session.advance(2).unwrap();
    replay.advance(2).unwrap();
    assert_eq!(session.save(), replay.save());
}
