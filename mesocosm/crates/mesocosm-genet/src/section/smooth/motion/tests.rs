// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn terrain_volume(paged:bool) -> isometer::space::volume::Volume {
    terrain_with_edits(paged,&[])
}

fn terrain_with_edits(paged:bool,edits:&[(u64,isometer::space::edit::Edit)])
    -> isometer::space::volume::Volume {
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
    if paged {
        isometer::space::volume::Volume::paged(&sim.atlas().unwrap(),0,[350;2],[418;2],16,edits).unwrap()
    } else {isometer::space::volume::Volume::lift(&sim.atlas().unwrap(),0,[350;2],[418;2],16,edits).unwrap()}
}

fn fixture() -> (PlacedBody, SiteScene) {
    let volume=terrain_volume(false);
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
    let v=&scene.window;
    let e=i64::from(v.ground().extent());
    let at=[i64::from(below[0])+v.min[0]+e,i64::from(below[1])+v.datum,
        i64::from(below[2])+v.min[1]+e];
    let fill=|min:[i64;3],material| (0,isometer::space::edit::Edit {
        op:isometer::space::edit::Op::Fill(material),
        shape:isometer::space::edit::Shape::Box {min,max:min.map(|v|v+1)} });
    scene.window=terrain_with_edits(false,&[fill(at,scene.window.materials.water)]);
    scene.ground=scene.window.ground().clone();
    assert_eq!(crate::section::smooth::height(&scene, p), None);
    scene.window=terrain_with_edits(false,&[fill(at,solid),fill([at[0],at[1]+6,at[2]],solid)]);
    scene.ground=scene.window.ground().clone();
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
    let mut middle = from.clone(); middle.at[0] += 1;
    let mut end = middle.clone(); end.at[0] += 1;
    body.walks = vec![[from.clone(),middle.clone()],[middle.clone(),end.clone()]];
    body.physical.as_mut().unwrap().at = end.at;
    body.at = scene.window.to_ground(end.at); scene.bodies[0] = body.clone();
    assert_eq!(motion.sample(&body,&scene,0.), start);
    assert!(motion.active());
    let moving = motion.sample(&body,&scene,0.06);
    assert!(moving[0] > start[0] && moving[0] < body.at[0] as f32);
    for _ in 0..12 { motion.sample(&body,&scene,0.06); }
    assert_eq!(motion.sample(&body,&scene,0.), body.at.map(|v|v as f32));
    assert!(!motion.active());
    // A missing edge is an unavailable route, not permission to fill the gap.
    assert!(trail(&[[from.clone(),middle]], &from, &end).is_none());
}


#[test]
fn renderer_contacts_read_the_paged_volume_instead_of_its_empty_projection() {
    let (body,mut scene)=fixture();
    let mut motion=Motion::new(&body,&scene);
    motion.sample(&body,&scene,0.0);
    let contact=*motion.contacts().values().next().unwrap();
    assert_eq!(super::super::height(&scene,contact),Some(contact[1]));
    scene.window=terrain_volume(true);
    scene.ground=scene.window.ground().clone();
    assert_eq!(scene.ground.brick_count(),0);
    assert_eq!(super::super::height(&scene,contact),Some(contact[1]));
}
