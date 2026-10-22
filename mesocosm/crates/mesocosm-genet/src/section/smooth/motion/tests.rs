// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn fixture() -> (PlacedBody, SiteScene) {
    let genesis = isocosm::Founding {
        seed: 7,
        sites: 4,
        population: 8,
        map: Some(isocosm::map::Layout::Grid(isocosm::map::Grid {
            shape: "shape:plane".into(),
            width: 2,
            height: 2,
            side: 768,
            elevation: [16, 16],
            relief: [0, 0],
            sea_per_mille: 0,
        })),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let sim = isocosm::Simulation::new(genesis, isocosm::Execution::Individuals).unwrap();
    let volume = sim.volume(0, [350; 2], [418; 2], 16).unwrap();
    let document = BodyDocument::new(isometer::core::VolumeRef::from_tag(1), [1; 3]);
    let pose = embodiment::Grounding::new(&volume)
        .at(&document, 384, 384, isometer::core::Yaw::Zero)
        .unwrap();
    let body = PlacedBody {
        id: 6,
        document,
        at: volume.to_ground(pose.at),
        physical: Some(mesocosm_runtime::spatial::Pose {
            at: pose.at,
            yaw: pose.yaw,
        }),
        tint: [1.; 3],
        alive: true,
    };
    let bodies = vec![body.clone()];
    let scene = SiteScene {
        site: 0,
        ground: volume.ground().clone(),
        window: volume,
        played: Some(6),
        volumes: crate::section::body_volumes(&bodies).unwrap(),
        bodies,
    };
    (body, scene)
}

#[test]
fn admitted_adjacent_motion_interpolates_and_contacts_keep_the_native_frame() {
    let (mut body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    let start = motion.sample(&body, &scene, 0.02);
    assert!(!motion.contacts().is_empty());
    body.physical.as_mut().unwrap().at[0] += 1;
    body.at[0] += 1;
    scene.bodies[0] = body.clone();
    assert_eq!(motion.sample(&body, &scene, 0.), start);
    let middle = motion.sample(&body, &scene, 0.06);
    assert!(middle[0] > start[0] && middle[0] < body.at[0] as f32);
    for _ in 0..5 {
        motion.sample(&body, &scene, 0.06);
    }
    assert_eq!(motion.sample(&body, &scene, 0.), body.at.map(|v| v as f32));
    assert!(!motion.active());
}

#[test]
fn live_blocker_and_teleport_snap_instead_of_crossing_unchecked_space() {
    let (mut body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    motion.sample(&body, &scene, 0.02);
    body.physical.as_mut().unwrap().at[0] += 1;
    body.at[0] += 1;
    scene.bodies[0] = body.clone();
    motion.sample(&body, &scene, 0.02);
    assert!(motion.active());
    let mut blocker = body.clone();
    blocker.id = 7;
    scene.bodies.push(blocker);
    assert_eq!(
        motion.sample(&body, &scene, 0.02),
        body.at.map(|v| v as f32)
    );
    assert!(!motion.active());
    scene.bodies.pop();
    body.physical.as_mut().unwrap().at[0] += 8;
    body.at[0] += 8;
    scene.bodies[0] = body.clone();
    assert_eq!(
        motion.sample(&body, &scene, 0.02),
        body.at.map(|v| v as f32)
    );
    assert!(!motion.active());
}

#[test]
fn renderer_contacts_reject_water_and_keep_the_actual_floor_under_an_overhang() {
    let (body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    motion.sample(&body, &scene, 0.02);
    let p = *motion.contacts.values().next().unwrap();
    let below = [p[0].floor() as i32, p[1] as i32 - 1, p[2].floor() as i32];
    let solid = scene.ground.material(below).unwrap();
    assert_eq!(crate::section::smooth::height(&scene, p), Some(p[1]));
    scene
        .ground
        .write([(below, scene.window.materials.water)])
        .unwrap();
    assert_eq!(crate::section::smooth::height(&scene, p), None);
    scene
        .ground
        .write([(below, solid), ([below[0], below[1] + 6, below[2]], solid)])
        .unwrap();
    assert_eq!(crate::section::smooth::height(&scene, p), Some(p[1]));
}

#[test]
fn a_changed_terrain_coordinate_frame_is_not_a_walk() {
    let (mut body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    motion.sample(&body, &scene, 0.02);
    // The fixture is a flat plane. Recentring its window changes Ground
    // coordinates without moving the admitted native root by a single cell.
    scene.window.min[0] -= 2;
    scene.window.max[0] -= 2;
    body.at = scene.window.to_ground(body.physical.as_ref().unwrap().at);
    scene.bodies[0] = body.clone();
    assert_eq!(
        motion.sample(&body, &scene, 0.02),
        body.at.map(|v| v as f32)
    );
    assert!(!motion.active());
}
