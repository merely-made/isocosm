// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn terrain_volume(paged: bool) -> isometer::space::volume::Volume {
    terrain_with_edits(paged, &[])
}

fn terrain_with_edits(
    paged: bool,
    edits: &[(u64, isometer::space::edit::Edit)],
) -> isometer::space::volume::Volume {
    let genesis = cosmograph::Founding {
        seed: 7,
        sites: 4,
        population: 8,
        map: Some(cosmograph::map::Layout::Grid(cosmograph::map::Grid {
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
    let sim = cosmograph::Simulation::new(genesis, cosmograph::Execution::Individuals).unwrap();
    if paged {
        isometer::space::volume::Volume::paged(
            &sim.atlas().unwrap(),
            0,
            [350; 2],
            [418; 2],
            16,
            edits,
        )
        .unwrap()
    } else {
        isometer::space::volume::Volume::lift(
            &sim.atlas().unwrap(),
            0,
            [350; 2],
            [418; 2],
            16,
            edits,
        )
        .unwrap()
    }
}

fn fixture() -> (PlacedBody, SiteScene) {
    let volume = terrain_volume(false);
    let document = BodyDocument::new(isometer::core::VolumeRef::from_tag(1), [1; 3]);
    let pose = embodiment::Grounding::new(&volume)
        .at(&document, 384, 384, isometer::core::Yaw::Zero)
        .unwrap();
    let body = PlacedBody {
        id: 6,
        document,
        at: volume.to_ground(pose.at),
        walks: vec![],
        physical: Some(pose.clone()),
        tint: [1.; 3],
        materials: vec![],
        alive: true,
    };
    let bodies = vec![body.clone()];
    let scene = SiteScene {
        walking: None,
        site: 0,
        ground: volume.ground().clone(),
        window: volume,
        played: Some(6),
        volumes: crate::section::body_volumes(&bodies).unwrap(),
        bodies,
    };
    (body, scene)
}

fn jointed_fixture() -> (PlacedBody, SiteScene, Pose, isometer::core::legs::Stride) {
    jointed_motion(false)
}
fn jointed_motion(pivot: bool) -> (PlacedBody, SiteScene, Pose, isometer::core::legs::Stride) {
    let mut founding = mesocosm_runtime::Founded::generated(7, 60)
        .unwrap()
        .genesis
        .founding
        .unwrap();
    let b = founding.bodies.as_mut().unwrap();
    b.roster = true;
    b.founding = Some("SpacedRoster".into());
    b.proportions = true;
    let genesis = founding.generate().unwrap();
    let volume = terrain_volume(false);
    let rules = embodiment::Walking::default();
    for doc in genesis
        .population
        .groups
        .values()
        .filter_map(|g| g.entity.body.as_ref())
    {
        if isometer::core::legs::chains(doc).len() < 4 {
            continue;
        }
        let mut ground = embodiment::Grounding::new(&volume)
            .with_articulation(true)
            .with_walking(Some(rules));
        let Ok(from) = ground.find(doc, [384, 0, 384], Default::default(), 12, 2, 64, &[]) else {
            continue;
        };
        let directions = if pivot {
            vec![
                (0, 0, from.yaw.compose(isometer::core::Yaw::Quarter)),
                (0, 0, from.yaw.compose(isometer::core::Yaw::ThreeQuarter)),
            ]
        } else {
            vec![
                (1, 0, from.yaw),
                (-1, 0, from.yaw),
                (0, 1, from.yaw),
                (0, -1, from.yaw),
            ]
        };
        for (dx, dz, yaw) in directions {
            let Ok(to) = ground.at(doc, from.at[0] + dx, from.at[2] + dz, yaw) else {
                continue;
            };
            let stride = match embodiment::plan_stride(&volume, doc, &from, &to, rules, &[]) {
                Ok(stride) => stride,
                Err(why) => {
                    if pivot {
                        eprintln!(
                            "generated pivot with {} parts and {} feet toward {yaw:?}: {why}",
                            doc.living().count(),
                            from.feet.len()
                        );
                    }
                    continue;
                },
            };
            let body = PlacedBody {
                id: 6,
                document: doc.clone(),
                at: volume.to_ground(from.at),
                physical: Some(from),
                walks: vec![],
                tint: [1.; 3],
                materials: vec![],
                alive: true,
            };
            let bodies = vec![body.clone()];
            let scene = SiteScene {
                site: 0,
                ground: volume.ground().clone(),
                window: volume.clone(),
                volumes: crate::section::body_volumes(&bodies).unwrap(),
                bodies,
                played: Some(6),
                walking: Some(rules),
            };
            return (body, scene, to, stride);
        }
    }
    panic!("jointed roster must supply a supported moving fixture");
}

#[test]
fn displayed_pivot_uses_the_accepted_orientation_and_supported_stride() {
    let (mut body, mut scene, to, stride) = jointed_motion(true);
    let from = body.physical.clone().unwrap();
    let mut motion = Motion::new(&body, &scene);
    motion.sample_revision(&body, &scene, 0., 1);
    body.physical = Some(to.clone());
    body.at = scene.window.to_ground(to.at);
    body.walks = vec![Walk {
        poses: [from.clone(), to.clone()],
        stride: Some(stride.clone()),
    }];
    scene.bodies = vec![body.clone()];
    let mut pivoted = false;
    for _ in 0..160 {
        motion.sample_revision(&body, &scene, 0.025, 2);
        if motion.age < motion.duration() {
            let phase = ((motion.age / motion.duration()) * stride.samples() as f32).floor() as i64
                * isometer::core::legs::UNIT
                / stride.samples();
            let frame = embodiment::stride_sample(&from, &to, &stride, phase).unwrap();
            let yaw = isometer::core::legs::Rotation::at(from.yaw, frame.turn).radians();
            assert!((motion.yaw_radians() - yaw).abs() < 0.00001);
            pivoted |= motion.pivoting();
        }
    }
    assert!(
        pivoted,
        "an admitted turn must not snap to its final facing"
    );
    assert!(!motion.active());
}

#[test]
fn displayed_root_follows_the_exact_native_weight_transfer() {
    let (mut body, mut scene, to, stride) = jointed_fixture();
    let from = body.physical.clone().unwrap();
    let mut motion = Motion::new(&body, &scene);
    motion.sample_revision(&body, &scene, 0., 1);
    body.physical = Some(to.clone());
    body.at = scene.window.to_ground(to.at);
    body.walks = vec![Walk {
        poses: [from.clone(), to.clone()],
        stride: Some(stride.clone()),
    }];
    scene.bodies = vec![body.clone()];
    let mut shifted = false;
    for _ in 0..80 {
        let at = motion.sample_revision(&body, &scene, 0.05, 2);
        let phase = ((motion.age / motion.duration()).clamp(0., 1.) * stride.samples() as f32)
            .floor() as i64
            * isometer::core::legs::UNIT
            / stride.samples();
        let frame = embodiment::stride_sample(&from, &to, &stride, phase).unwrap();
        let zero = scene.window.to_ground([0; 3]);
        let expected = [0, 1, 2]
            .map(|k| zero[k] as f32 + frame.root[k] as f32 / isometer::core::legs::UNIT as f32);
        assert!((0..3).all(|k| (at[k] - expected[k]).abs() < 0.002));
        shifted |= motion.shifting();
    }
    assert!(shifted, "native body transfer must be visible");
    assert!(!motion.active());
}

#[test]
fn a_missing_weight_transfer_witness_cannot_start_a_display_gait() {
    let (mut body, mut scene, to, _) = jointed_fixture();
    let from = body.physical.clone().unwrap();
    let mut motion = Motion::new(&body, &scene);
    motion.sample_revision(&body, &scene, 0., 1);
    body.physical = Some(to.clone());
    body.at = scene.window.to_ground(to.at);
    body.walks = vec![[from, to].into()];
    scene.bodies = vec![body.clone()];
    let at = motion.sample_revision(&body, &scene, 0.05, 2);
    assert_eq!(at, body.at.map(|v| v as f32));
    assert!(!motion.active());
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
    let v = &scene.window;
    let e = i64::from(v.ground().extent());
    let at = [
        i64::from(below[0]) + v.min[0] + e,
        i64::from(below[1]) + v.datum,
        i64::from(below[2]) + v.min[1] + e,
    ];
    let fill = |min: [i64; 3], material| {
        (
            0,
            isometer::space::edit::Edit {
                op: isometer::space::edit::Op::Fill(material),
                shape: isometer::space::edit::Shape::Box {
                    min,
                    max: min.map(|v| v + 1),
                },
            },
        )
    };
    scene.window = terrain_with_edits(false, &[fill(at, scene.window.materials.water)]);
    scene.ground = scene.window.ground().clone();
    assert_eq!(crate::section::smooth::height(&scene, p), None);
    scene.window = terrain_with_edits(
        false,
        &[fill(at, solid), fill([at[0], at[1] + 6, at[2]], solid)],
    );
    scene.ground = scene.window.ground().clone();
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

#[test]
fn a_missed_picture_uses_admitted_steps_and_never_invents_a_distant_path() {
    let (mut body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    let start = motion.sample(&body, &scene, 0.02);
    let from = motion.pose.clone().unwrap();
    let mut middle = from.clone();
    middle.at[0] += 1;
    let mut end = middle.clone();
    end.at[0] += 1;
    body.walks = vec![
        [from.clone(), middle.clone()].into(),
        [middle.clone(), end.clone()].into(),
    ];
    body.physical.as_mut().unwrap().at = end.at;
    body.at = scene.window.to_ground(end.at);
    scene.bodies[0] = body.clone();
    assert_eq!(motion.sample(&body, &scene, 0.), start);
    assert!(motion.active());
    let moving = motion.sample(&body, &scene, 0.06);
    assert!(moving[0] > start[0] && moving[0] < body.at[0] as f32);
    for _ in 0..12 {
        motion.sample(&body, &scene, 0.06);
    }
    assert_eq!(motion.sample(&body, &scene, 0.), body.at.map(|v| v as f32));
    assert!(!motion.active());
    // A missing edge is an unavailable route, not permission to fill the gap.
    assert!(trail(&[[from.clone(), middle].into()], &from, &end).is_none());
}

#[test]
fn renderer_contacts_read_the_paged_volume_instead_of_its_empty_projection() {
    let (body, mut scene) = fixture();
    let mut motion = Motion::new(&body, &scene);
    motion.sample(&body, &scene, 0.0);
    let contact = *motion.contacts().values().next().unwrap();
    assert_eq!(super::super::height(&scene, contact), Some(contact[1]));
    scene.window = terrain_volume(true);
    scene.ground = scene.window.ground().clone();
    assert_eq!(scene.ground.brick_count(), 0);
    assert_eq!(super::super::height(&scene, contact), Some(contact[1]));
}
