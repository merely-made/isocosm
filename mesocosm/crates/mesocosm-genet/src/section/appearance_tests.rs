// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::section::{BodyMode, BodyPickError, CameraMode, Framing, SectionFrame};
use isometer_mesh::VolumeMap;
use isometer_render::{RenderError, Renderer};
use mesocosm_core::{Founding, World};

fn render(section: &mut Section, world: &World, volumes: &VolumeMap, centre: [f32; 3]) -> Vec<u8> {
    let mut encoder = section.device.create_command_encoder(&Default::default());
    section
        .render(
            &mut encoder,
            SectionFrame {
                world,
                volumes,
                ground: world.ground(),
                dirty: &[],
                centre,
                pose: None,
                roster: &[],
            },
        )
        .unwrap();
    section.queue.submit([encoder.finish()]);
    section
        .capture(|_, _, _| {})
        .expect("fresh tint readback")
        .2
}

fn cached(section: &Section) {
    let stats = section.body_stats();
    assert_eq!(stats.fallback_bodies, 0);
    assert!(
        stats.material_parts > 0,
        "the fixture draws phenotype materials"
    );
    assert!(
        stats.secretory_parts > 0,
        "the expressed process mark is present"
    );
    assert_eq!(
        stats.mesh_builds, 0,
        "appearance cannot rebuild static meshes"
    );
    assert_eq!(
        stats.mesh_upload_bytes, 0,
        "appearance cannot upload static meshes"
    );
}

#[test]
fn tint_changes_material_pixels_and_instances_while_geometry_stays_cached() {
    let renderer = match Renderer::headless(128, 128) {
        Ok(renderer) => renderer,
        Err(RenderError::NoAdapter) => {
            eprintln!("no adapter; skipping Section tint pixel/cache receipt");
            return;
        },
        Err(error) => panic!("tint receipt renderer failed: {error:?}"),
    };
    let founding = Founding::SpacedRoster;
    let mut world = World::expression_practice(7, founding, founding.palette()).unwrap();
    let subject = world.controlled_id().unwrap();
    let preview = world
        .preview_expression(world.discoveries()[0].condition)
        .unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == subject)
        .unwrap()
        .phenotype = preview.phenotype;
    let organism = world.controlled().unwrap();
    let volumes = crate::fixture::volumes_for(&world);
    let hash = mesocosm_core::state_hash(&world);
    let material_facts = crate::section::materials::project(&organism.phenotype, world.ruleset());
    assert!(!material_facts.is_empty());
    let mut section = Section::new(
        renderer.device().clone(),
        renderer.queue().clone(),
        128,
        128,
        wgpu::TextureFormat::Rgba8Unorm,
        world.ground(),
        Framing::new(28.0, CameraMode::Oblique),
    )
    .unwrap();
    section.configure_bodies(BodyMode::Voxels, 1);
    let (min, max) = section
        .presentation_bounds(organism, &volumes)
        .unwrap()
        .unwrap();
    let centre = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5);
    let span = (0..3).map(|i| max[i] - min[i]).fold(1.0, f32::max);
    section.set_half_height(span);
    section.set_body_preview(true, span * 4.0);
    let initial = render(&mut section, &world, &volumes, centre);
    assert!(section.body_stats().mesh_builds > 0);
    assert!(section.body_stats().mesh_upload_bytes > 0);
    assert!(initial.chunks_exact(4).any(|pixel| pixel[..3] != [0; 3]));

    let tint = [0.9, 0.15, 0.15];
    assert_eq!(section.set_body_tint(subject, Some(tint)), Ok(true));
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    let red = render(&mut section, &world, &volumes, centre);
    cached(&section);
    assert!(section.body_stats().instance_upload_bytes > 0);
    assert_ne!(
        red, initial,
        "instance tint visibly changes a material body"
    );
    let generation = section.scene.query_generation().unwrap();

    assert_eq!(section.set_body_tint(subject, Some(tint)), Ok(false));
    assert_eq!(section.scene.query_generation().unwrap(), generation);
    for invalid in [
        [f32::NAN, 0.0, 0.0],
        [0.0, f32::INFINITY, 0.0],
        [0.0, 0.0, f32::NEG_INFINITY],
        [-0.001, 0.0, 0.0],
        [0.0, 1.001, 0.0],
    ] {
        assert_eq!(
            section.set_body_tint(subject, Some(invalid)),
            Err(LiveBodyError::InvalidBody)
        );
        assert_eq!(section.body_tint(subject), Some(tint));
        assert_eq!(section.scene.query_generation().unwrap(), generation);
    }
    assert_eq!(render(&mut section, &world, &volumes, centre), red);
    cached(&section);
    assert_eq!(
        section.body_stats().instance_upload_bytes,
        0,
        "identical tint reuses instances"
    );

    assert_eq!(
        section.set_body_tint(subject, Some([0.0, 0.0, 1.0])),
        Ok(true)
    );
    let blue = render(&mut section, &world, &volumes, centre);
    cached(&section);
    assert!(section.body_stats().instance_upload_bytes > 0);
    let changed = red
        .chunks_exact(4)
        .zip(blue.chunks_exact(4))
        .filter(|(before, after)| before[..3] != after[..3])
        .count();
    assert!(
        changed > 32,
        "a visible area must change, got {changed} pixels"
    );

    assert_eq!(section.set_body_tint(subject, None), Ok(true));
    assert_eq!(section.body_tint(subject), None);
    assert_eq!(render(&mut section, &world, &volumes, centre), initial);
    cached(&section);
    assert_eq!(section.set_body_tint(subject, None), Ok(false));
    assert_eq!(mesocosm_core::state_hash(&world), hash);
    assert_eq!(
        crate::section::materials::project(&organism.phenotype, world.ruleset()),
        material_facts,
        "presentation tint cannot change material facts"
    );
}
