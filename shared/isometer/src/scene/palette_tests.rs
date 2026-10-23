// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The material palette through the facade: a host names the table once and
//! every frame after it traces its terrain through it, unchanged.

use isometer_core::ground::{Ground, Terrain};
use isometer_lens::{Grade, TerrainPalette};

use super::{GroundTerrain, Scene, SceneFrame, SceneHost, TerrainSource};
use crate::bodies::SceneVolumes;
use crate::camera::SlabCamera;
use crate::volumes::DeclaredExtentVolumes;

/// Odd, so the centre pixel is exactly NDC `[0, 0]`.
const SIZE: u32 = 65;
const EXTENT: i32 = 12;
/// The board's own tile kind, as a brick material.
const TURF: u8 = 7;

struct Host;
impl SceneHost for Host {}

/// A flat field, so the centre pixel is a top face whose light term is the
/// shader's fixed one and the colour read back is the table's entry.
struct Flat;

impl Terrain for Flat {
    fn sea_level(&self, _extent: i32) -> i32 {
        1
    }

    fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
        3
    }
}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

/// One terrain-only frame under the board's own camera preset, with the grade
/// that leaves a traced colour arithmetically exact: no fog, no quantiser.
fn render(scene: &mut Scene, ground: &Ground) {
    let volumes = DeclaredExtentVolumes::from_documents(std::iter::empty(), 9);
    let terrain = GroundTerrain(ground);
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera: SlabCamera::dimetric_2_1([0.5, 4.0, 0.5], 6.0, 1.0, 64.0)
                    .expect("the preset frames"),
                bodies: &[],
                volumes: SceneVolumes::DeclaredSolid(&volumes),
                terrain: Some(&terrain as &dyn TerrainSource),
                dirty: &[],
                grade: Grade {
                    fog_start: 1.0,
                    ..Grade::clay()
                },
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )
        .expect("a terrain frame encodes");
    scene.queue.submit([encoder.finish()]);
}

fn centre(pixels: &[u8]) -> [u8; 3] {
    let at = ((SIZE / 2) as usize * SIZE as usize + (SIZE / 2) as usize) * 4;
    [pixels[at], pixels[at + 1], pixels[at + 2]]
}

/// The shader's top-face light: `0.38 + 0.62 * dot(normal, sun)` straight up.
fn top_face_light() -> f32 {
    let sun = [0.4f32, 0.8, 0.3];
    let length = (sun[0] * sun[0] + sun[1] * sun[1] + sun[2] * sun[2]).sqrt();
    0.38 + 0.62 * (sun[1] / length)
}

/// I1's facade done-condition: a host that names a table sees its own colour
/// on the cell, and taking the table away puts the fixed colours back.
#[test]
fn a_frame_with_a_palette_draws_a_fixture_cell_in_its_palette_colour() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping terrain palette facade receipt");
        return;
    };
    let ground = Ground::grow_with(&Flat, EXTENT, |_x, _z, _depth| TURF);
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();

    // Unknown, because the fixture's material is neither soil nor rock and no
    // table has been named yet.
    render(&mut scene, &ground);
    let (_, _, unknown) = scene.capture(|_, _, _| {}).expect("a capture reads back");
    let light = top_face_light();
    let expected = |colour: [f32; 3]| colour.map(|channel| (channel * light * 255.0).round() as u8);
    let near = |read: [u8; 3], want: [u8; 3], what: &str| {
        for channel in 0..3 {
            assert!(
                (read[channel] as i32 - want[channel] as i32).abs() <= 2,
                "{what}: channel {channel} read {}, expected {}",
                read[channel],
                want[channel]
            );
        }
    };
    near(
        centre(&unknown),
        expected([0.66, 0.20, 0.72]),
        "an unnamed material",
    );

    let turf = [0.24, 0.62, 0.30];
    scene.set_terrain_palette(Some(
        TerrainPalette::new([[0.66, 0.20, 0.72]]).with_material(TURF, turf),
    ));
    assert_eq!(
        scene.terrain_palette().map(TerrainPalette::len),
        Some(TURF as usize + 1)
    );
    render(&mut scene, &ground);
    let (_, _, painted) = scene.capture(|_, _, _| {}).expect("a capture reads back");
    near(centre(&painted), expected(turf), "the named tile kind");

    // The table is host state, so taking it away is a full return to the
    // tracer's own arithmetic rather than a frame-local override.
    scene.set_terrain_palette(None);
    render(&mut scene, &ground);
    let (_, _, cleared) = scene.capture(|_, _, _| {}).expect("a capture reads back");
    assert_eq!(cleared, unknown);
}


#[test]
fn coarse_terrain_keeps_native_projection_depth_and_query_distance() {
    struct Scaled<'a>(&'a Ground);
    impl TerrainSource for Scaled<'_> {
        fn cell_size(&self)->f32 {2.0}
        fn revision(&self)->u64 {0}
        fn brick_map(&self)->Result<isometer_lens::BrickMap,String> {GroundTerrain(self.0).brick_map()}
        fn refresh(&self,map:&mut isometer_lens::BrickMap,dirty:&[[i16;3]])
            ->Result<super::TerrainRefresh,String> {GroundTerrain(self.0).refresh(map,dirty)}
    }
    struct High;
    impl Terrain for High {
        fn sea_level(&self,_:i32)->i32 {2}
        fn surface(&self,_:i32,_:i32,_:i32)->i32 {7}
    }
    let Some((device,queue))=device() else {return};
    let fine=Ground::grow_with(&High,24,|_,_,_|TURF);
    let coarse=Ground::grow_with(&Flat,12,|_,_,_|TURF);
    let mut images=Vec::new(); let mut hits=Vec::new();
    let volumes=DeclaredExtentVolumes::from_documents(std::iter::empty(),9);
    let plain=GroundTerrain(&fine); let scaled=Scaled(&coarse);
    for source in [&plain as &dyn TerrainSource,&scaled as &dyn TerrainSource] {
        let mut scene=Scene::new(device.clone(),queue.clone(),SIZE,SIZE).unwrap();
        let mut encoder=device.create_command_encoder(&Default::default());
        scene.render(&mut encoder,SceneFrame {
            camera:SlabCamera::dimetric_2_1([0.5,8.0,0.5],6.0,1.0,64.0).unwrap(),
            bodies:&[],volumes:SceneVolumes::DeclaredSolid(&volumes),terrain:Some(source),dirty:&[],
            grade:Grade {fog_start:1.0,..Grade::clay()},terrain_appearance:None,
            body_budget:1,capsules:None,
        },&mut Host).unwrap();
        queue.submit([encoder.finish()]);
        images.push(scene.capture(|_,_,_|{}).unwrap().2);
        hits.push(scene.terrain_ray([0.5,12.,0.5],[0.,-1.,0.],32.).unwrap().unwrap());
    }
    assert_eq!(images[0],images[1],"terrain grain must preserve its native depth projection");
    // The shared DDA advances 0.0001 grid cells into its bounds. One map
    // begins at that bound, the other traverses air first, so allow that
    // sub-cell advance after conversion to native units.
    assert!((hits[0].distance-hits[1].distance).abs()<0.0005,"{hits:?}");
    for i in 0..3 {assert!((hits[0].point[i]-hits[1].point[i]).abs()<0.0005,"{hits:?}");}
    assert_eq!(hits[0].normal,hits[1].normal);
    assert_eq!(hits[0].material,hits[1].material);
}
