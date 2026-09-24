// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The query surface against real encoded frames, with no product world in
//! scope: hand-made body documents, declared-extent volumes, and a ground
//! grown from a two-number fixture terrain rather than any product's map.
//!
//! The capsule-fallback and cutaway arms of this surface need a host's own
//! policy to set up and stay in Mesocosm's `section/query_tests.rs`.

use isometer_core::ground::{Ground, Terrain};
use isometer_core::{BodyDocument, PartId, SpeciesId, VolumeRef};

use super::*;
use crate::bodies::Pose;
use crate::scene::{GroundTerrain, SceneFrame, SceneHost, TerrainSource};
use crate::volumes::DeclaredExtentVolumes;

const SIZE: u32 = 33;
/// The forward `CameraMode::Side` produces, and Eponym's level default.
const SIDE: [f32; 3] = [0.0, 0.0, -1.0];

/// Odd, so the centre pixel is exactly NDC `[0, 0]` and the pixel and clip
/// forms of a pick are the same question.
const ISO_SIZE: u32 = 65;
/// How far `Ground::grow` lays the fixture field on either horizontal axis.
const EXTENT: i32 = 12;

struct Host;
impl SceneHost for Host {}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

fn document(tag: u8) -> BodyDocument {
    BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(tag), 1_000, [1; 3])
}

fn camera() -> SlabCamera {
    SlabCamera {
        centre: [0.0; 3],
        forward: SIDE,
        half_height: 4.0,
        aspect: 1.0,
        depth: 32.0,
        cutaway: None,
    }
}

fn body<'a>(subject: SubjectKey, document: &'a BodyDocument, position: [f32; 3]) -> SceneBody<'a> {
    SceneBody {
        subject,
        document,
        pose: Pose {
            position,
            yaw_radians: 0.0,
        },
        scale: 1.0,
        grounded: false,
        tint: [1.0; 3],
        materials: &[],
        always_visible: false,
    }
}

/// One frame with no terrain: the isolated-preview branch, which still
/// completes a query receipt.
fn render(scene: &mut Scene, bodies: &[SceneBody<'_>], volumes: &DeclaredExtentVolumes) {
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera: camera(),
                bodies,
                volumes: SceneVolumes::DeclaredSolid(volumes),
                terrain: None,
                dirty: &[],
                grade: isometer_lens::Grade::retro(3),
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )
        .expect("a bodies-only frame encodes");
    scene.queue.submit([encoder.finish()]);
}

#[test]
fn a_pick_addresses_the_nearer_body_and_expires_with_its_frame() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer query receipt");
        return;
    };
    let (near, far) = (document(70), document(71));
    let volumes = DeclaredExtentVolumes::from_documents([&near, &far], 9);
    // The camera looks down -z, so the larger z is the nearer surface.
    let bodies = [
        body(SubjectKey(4), &near, [0.0, 0.0, 3.0]),
        body(SubjectKey(9), &far, [0.0, 0.0, -3.0]),
    ];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();

    assert_eq!(scene.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    render(&mut scene, &bodies, &volumes);

    let hit = scene.pick_pixel([SIZE / 2, SIZE / 2]).unwrap().unwrap();
    assert_eq!(hit.address.subject, SubjectKey(4));
    assert!(!hit.tied);
    assert!((hit.point[2] - 4.0).abs() < 1e-4, "the near face at z=4");
    assert_eq!(scene.pick_ndc([0.0; 2]).unwrap(), Some(hit));
    assert!(scene.validate_pick(hit, &bodies[0], SceneVolumes::DeclaredSolid(&volumes)));
    // The address belongs to its own subject and to no other.
    assert!(!scene.validate_pick(hit, &bodies[1], SceneVolumes::DeclaredSolid(&volumes)));

    assert_eq!(
        scene.pick_ndc([f32::NAN, 0.0]),
        Err(BodyPickError::InvalidCoordinates)
    );
    assert_eq!(
        scene.pick_pixel([SIZE, 0]),
        Err(BodyPickError::InvalidCoordinates)
    );

    // A second encode mints a new generation, so the old receipt is spent
    // even though the address it carries is still current.
    render(&mut scene, &bodies, &volumes);
    let current = scene.pick_ndc([0.0; 2]).unwrap().unwrap();
    assert_eq!(current.address, hit.address);
    assert_ne!(current.frame, hit.frame);
    assert!(!scene.validate_pick(hit, &bodies[0], SceneVolumes::DeclaredSolid(&volumes)));

    // Every visual change a host owns rather than a frame drops the receipt.
    scene.resize(SIZE, SIZE + 2);
    assert_eq!(scene.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    assert_eq!(scene.select_part(SubjectKey(4), None, false), None);
}

/// Plan §7 condition 4: resolving the projector's identity question by
/// narrowing a subject to 32 bits would be caught here rather than in a
/// product.
#[test]
fn a_subject_key_beyond_u32_survives_a_pick_and_its_queries() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping wide-subject query receipt");
        return;
    };
    let wide = SubjectKey(u64::from(u32::MAX) + 7);
    let document = document(72);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(wide, &document, [0.0; 3])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);

    let hit = scene.pick_pixel([SIZE / 2, SIZE / 2]).unwrap().unwrap();
    assert_eq!(hit.address.subject, wide);
    assert_eq!(hit.address.part, PartId(0));
    assert_eq!(
        scene.select_part(wide, None, false).map(|a| a.subject),
        Some(wide)
    );
    assert_eq!(
        scene.select_part(SubjectKey(hit.address.subject.0 as u32 as u64), None, false),
        None
    );
    let anchors = scene
        .glyph_anchors(
            &bodies[0],
            SceneVolumes::DeclaredSolid(&volumes),
            Some(hit.address),
        )
        .unwrap();
    assert_eq!(anchors[0].selection.subject, wide);
}

/// The camera pair and the pick agree about which pixel a world point is on,
/// and `part_bounds` reports the drawn geometry rather than the whole body.
#[test]
fn the_camera_pixel_of_a_part_is_the_pixel_that_picks_it() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping camera/pick agreement receipt");
        return;
    };
    let document = document(73);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(SubjectKey(2), &document, [2.0, 1.0, 0.0])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);

    let (min, max) = scene.part_bounds(SubjectKey(2), PartId(0)).unwrap();
    assert_eq!(min, [1.0, 0.0, -1.0]);
    assert_eq!(max, [3.0, 2.0, 1.0]);
    assert_eq!(scene.part_bounds(SubjectKey(2), PartId(3)), None);
    assert_eq!(scene.part_bounds(SubjectKey(5), PartId(0)), None);

    let centre = [0, 1, 2].map(|axis| (min[axis] + max[axis]) * 0.5);
    let pixel = camera().pixel_of(centre, [SIZE, SIZE]).unwrap();
    let hit = scene.pick_pixel(pixel).unwrap().unwrap();
    assert_eq!(hit.address.subject, SubjectKey(2));
    // A world point outside the frame has no pixel and no hit.
    assert_eq!(camera().pixel_of([40.0, 0.0, 0.0], [SIZE, SIZE]), None);
}

/// The focus emphasis a host sets is presentation, and it spends the receipt
/// because the next draw paints different pixels.
#[test]
fn setting_focus_spends_the_query_receipt_only_when_it_changes() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping focus receipt");
        return;
    };
    let document = document(74);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(SubjectKey(1), &document, [0.0; 3])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);
    let generation = scene.query_generation().unwrap();

    scene.set_body_focus(None, None);
    assert_eq!(scene.query_generation(), Some(generation));

    let address = scene.select_part(SubjectKey(1), None, false).unwrap();
    scene.set_body_focus(Some(SubjectKey(1)), Some(address));
    assert_eq!(scene.query_generation(), None);
}

/// A flat field at `low`, optionally carrying one square ridge at `high`
/// over the inclusive column range `ridge`. Two numbers and a box, so a test
/// can say exactly what stands under a pixel without a product's world.
struct Fixture {
    low: i32,
    high: i32,
    ridge: Option<[i32; 2]>,
}

impl Terrain for Fixture {
    fn sea_level(&self, _extent: i32) -> i32 {
        0
    }

    fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
        match self.ridge {
            Some([min, max]) if (min..=max).contains(&x) && (min..=max).contains(&z) => self.high,
            _ => self.low,
        }
    }
}

fn field() -> Ground {
    Ground::grow(
        &Fixture {
            low: 2,
            high: 2,
            ridge: None,
        },
        EXTENT,
    )
}

fn ridged() -> Ground {
    Ground::grow(
        &Fixture {
            low: 2,
            high: 20,
            ridge: Some([4, 8]),
        },
        EXTENT,
    )
}

/// The board's locked lens, looking down onto the fixture field. A pitched
/// camera is what makes a terrain pick land on a top face, which is the hit
/// a host turns into a tile.
fn iso_camera() -> SlabCamera {
    SlabCamera::dimetric_2_1([0.5, 4.0, 0.5], 6.0, 1.0, 64.0).expect("the preset frames")
}

/// One frame under [`iso_camera`], with the ground bound or deliberately off.
fn iso_render(
    scene: &mut Scene,
    bodies: &[SceneBody<'_>],
    volumes: &DeclaredExtentVolumes,
    ground: Option<&Ground>,
) {
    let terrain = ground.map(GroundTerrain);
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera: iso_camera(),
                bodies,
                volumes: SceneVolumes::DeclaredSolid(volumes),
                terrain: terrain.as_ref().map(|source| source as &dyn TerrainSource),
                dirty: &[],
                grade: isometer_lens::Grade::retro(3),
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )
        .expect("a terrain frame encodes");
    scene.queue.submit([encoder.finish()]);
}

/// I2's first done-condition: open ground answers with the cell under the
/// pixel, at the fixture's own surface height for that column.
#[test]
fn a_pick_over_open_ground_names_the_cell_under_the_pixel() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping open-ground terrain pick receipt");
        return;
    };
    let ground = field();
    let document = document(80);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    iso_render(&mut scene, &[], &volumes, Some(&ground));

    let Some(Pick::Terrain(hit)) = scene.pick([0.0; 2]).unwrap() else {
        panic!("open ground answers with terrain");
    };
    assert_eq!(hit.normal, [0.0, 1.0, 0.0], "a flat field is hit on its top");
    assert_eq!(
        ground.surface(hit.voxel[0], hit.voxel[2]),
        Some(hit.voxel[1]),
        "the cell is the column's surface"
    );
    assert!((hit.point[1] - (hit.voxel[1] + 1) as f32).abs() < 1e-3);
    assert!(hit.distance > 0.0);
    assert_ne!(hit.material, 0, "a hit cell is never air");
    // The carried cell is the one the ray entered, not a re-floored float.
    let stepped = [0, 1, 2].map(|i| (hit.point[i] - hit.normal[i] * 0.001).floor() as i32);
    assert_eq!(stepped, hit.voxel);
    assert_eq!(hit.brick, hit.voxel.map(|v| v.div_euclid(8) as i16));

    // Eponym's narrower query is untouched: no body, so no pick.
    assert_eq!(scene.pick_ndc([0.0; 2]).unwrap(), None);
    assert_eq!(
        scene.pick_at_pixel([ISO_SIZE / 2, ISO_SIZE / 2]).unwrap(),
        Some(Pick::Terrain(hit)),
        "the pixel form asks the centre the same question"
    );
}

/// I2's second done-condition: a pick over a body is still the body, and it
/// is exactly the body `pick_ndc` names.
#[test]
fn a_pick_over_a_body_answers_the_body_pick_ndc_answers() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping body-over-terrain pick receipt");
        return;
    };
    let ground = field();
    let document = document(81);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let at = [0.5, 6.0, 0.5];
    let bodies = [body(SubjectKey(11), &document, at)];
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    iso_render(&mut scene, &bodies, &volumes, Some(&ground));

    let pixel = iso_camera()
        .pixel_of(at, [ISO_SIZE, ISO_SIZE])
        .expect("the body is on screen");
    let hit = scene
        .pick_pixel(pixel)
        .unwrap()
        .expect("a body above the field occludes it");
    assert_eq!(hit.address.subject, SubjectKey(11));
    assert_eq!(scene.pick_at_pixel(pixel).unwrap(), Some(Pick::Body(hit)));
}

/// I2's third done-condition: a ridge between the camera and a body is the
/// answer, and the bodies-only query declines rather than picking through it.
#[test]
fn a_body_behind_a_ridge_picks_the_ridge_and_pick_ndc_declines() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping ridge occlusion pick receipt");
        return;
    };
    let ground = ridged();
    let document = document(82);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let at = [0.5, 6.0, 0.5];
    let bodies = [body(SubjectKey(12), &document, at)];
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    let pixel = iso_camera()
        .pixel_of(at, [ISO_SIZE, ISO_SIZE])
        .expect("the body is on screen");

    // The positive control: with the ridge unbound the body is right there.
    iso_render(&mut scene, &bodies, &volumes, None);
    let exposed = scene
        .pick_pixel(pixel)
        .unwrap()
        .expect("nothing occludes the body yet");
    assert_eq!(exposed.address.subject, SubjectKey(12));

    iso_render(&mut scene, &bodies, &volumes, Some(&ground));
    assert_eq!(
        scene.pick_pixel(pixel).unwrap(),
        None,
        "the bodies-only pick refuses to reach through the ridge"
    );
    let Some(Pick::Terrain(hit)) = scene.pick_at_pixel(pixel).unwrap() else {
        panic!("the ridge is what the pixel shows");
    };
    assert!(
        (4..=8).contains(&hit.voxel[0]) && (4..=8).contains(&hit.voxel[2]),
        "{:?} is a ridge column",
        hit.voxel
    );
    assert!(hit.voxel[1] > 2, "the hit stands above the field");
    assert!(
        hit.distance < exposed.distance,
        "the ridge is nearer than the body it hides"
    );
}

/// I2's fourth done-condition: the frame's own terrain flag decides, not the
/// map the scene still holds from an earlier frame.
#[test]
fn a_frame_with_terrain_off_never_answers_with_ground() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping terrain-off pick receipt");
        return;
    };
    let ground = ridged();
    let document = document(83);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let at = [0.5, 6.0, 0.5];
    let bodies = [body(SubjectKey(13), &document, at)];
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    let pixel = iso_camera()
        .pixel_of(at, [ISO_SIZE, ISO_SIZE])
        .expect("the body is on screen");

    iso_render(&mut scene, &bodies, &volumes, Some(&ground));
    assert!(matches!(
        scene.pick_at_pixel(pixel).unwrap(),
        Some(Pick::Terrain(_))
    ));

    // Same scene, same map in hand, terrain off for this frame.
    iso_render(&mut scene, &bodies, &volumes, None);
    let hit = scene
        .pick_pixel(pixel)
        .unwrap()
        .expect("the body draws without the ridge");
    assert_eq!(scene.pick_at_pixel(pixel).unwrap(), Some(Pick::Body(hit)));
    // A pixel with neither body nor drawn ground is simply empty.
    assert_eq!(scene.pick([-0.95, -0.95]).unwrap(), None);
}

/// I2's fifth done-condition: the wider pick expires on exactly the terms
/// the narrower one does.
#[test]
fn a_pick_before_any_frame_is_not_ready() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping terrain pick readiness receipt");
        return;
    };
    let ground = field();
    let document = document(84);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();

    assert_eq!(scene.pick([0.0; 2]), Err(BodyPickError::NotReady));
    assert_eq!(scene.pick_at_pixel([0, 0]), Err(BodyPickError::NotReady));
    assert_eq!(
        scene.pick_at_pixel([ISO_SIZE, 0]),
        Err(BodyPickError::InvalidCoordinates)
    );
    assert_eq!(
        scene.pick([0.0, f32::NAN]),
        Err(BodyPickError::InvalidCoordinates)
    );

    iso_render(&mut scene, &[], &volumes, Some(&ground));
    assert!(scene.pick([0.0; 2]).unwrap().is_some());
    scene.invalidate_query();
    assert_eq!(scene.pick([0.0; 2]), Err(BodyPickError::NotReady));
}
