// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use mesocosm_mesh::VolumeMap;
use mesocosm_render::{RenderError, Renderer};

use super::bodies::{HostBodies, key};
use super::*;
use isometer::{BodyLayer, PartAddress, SceneVolumes};

/// The layer as `Section` builds it, with Mesocosm's half beside it.
fn layer() -> Option<(Renderer, BodyLayer, HostBodies)> {
    match Renderer::headless(16, 16) {
        Ok(renderer) => {
            let mut bodies = BodyLayer::new(renderer.device(), 16, 16, SLAB_DEPTH);
            bodies.budget = DEFAULT_BODY_BUDGET;
            Some((renderer, bodies, HostBodies::new()))
        },
        Err(RenderError::NoAdapter) => None,
        Err(error) => panic!("headless renderer failed: {error:?}"),
    }
}

fn prepare(
    layer: &mut BodyLayer,
    host: &mut HostBodies,
    world: &World,
    volumes: &mesocosm_mesh::VolumeMap,
) {
    let none: Vec<Vec<mesocosm_render::PartMaterial>> =
        world.organisms.iter().map(|_| Vec::new()).collect();
    let controlled = world.controlled_id();
    let scene: Vec<_> = world
        .organisms
        .iter()
        .zip(&none)
        .map(|(organism, materials)| host.scene_body(organism, materials, controlled))
        .collect();
    host.fallback.clear();
    host.played_fallback = None;
    layer.prepare(
        &scene,
        SceneVolumes::Voxels(volumes),
        super::view::slab_camera(CameraMode::Side, [0.0, 20.0, 0.0], 256.0, 1.0).window(),
        |body, stats| host.add_fallback(body, stats),
    );
}

/// The `Section::validate_selection` path, with the world lookup the adapter
/// does before the scene sees the body.
fn validate(
    layer: &mut BodyLayer,
    host: &HostBodies,
    world: &World,
    volumes: &mesocosm_mesh::VolumeMap,
    address: PartAddress,
) -> bool {
    let id = super::bodies::organism_of(address.subject);
    let Some(organism) = world.organisms.iter().find(|organism| organism.id == id) else {
        return false;
    };
    let body = host.scene_body(organism, &[], None);
    layer.validate_address(address, &body, SceneVolumes::Voxels(volumes))
}

#[test]
fn preview_depth_and_bounds_are_temporary() {
    let Some((renderer, _, _)) = layer() else {
        return;
    };
    let world = World::new(7, 3);
    let mut section = Section::new(
        renderer.device().clone(),
        renderer.queue().clone(),
        16,
        16,
        wgpu::TextureFormat::Rgba8Unorm,
        world.ground(),
        Framing::new(28.0, CameraMode::TerrariumEast),
    )
    .unwrap();
    for habitat in [None, Some(framed_habitat(&world))] {
        if let Some(habitat) = habitat {
            section.configure_terrarium(&habitat, 12.0, Cutaway::Always, [0, 0, 0]);
        }
        let centre = [0.0, 20.0, 0.0];
        let original = section.view(centre);
        section.set_body_preview(true, 80.0);
        let preview = section.view(centre);
        assert_eq!(preview.depth, 80.0);
        assert_eq!(preview.clip().bounds, None);
        assert!(preview.window().half[2] >= 40.0);
        section.set_body_preview(false, 80.0);
        let restored = section.view(centre);
        assert_eq!(restored.depth, original.depth);
        assert_eq!(restored.cutaway, original.cutaway);
        assert_eq!(restored.clip_from_world(), original.clip_from_world());
    }
}

#[test]
fn selection_carries_owner_and_expires_after_severing() {
    let Some((_renderer, mut layer, mut host)) = layer() else {
        eprintln!("no adapter; skipping inspection identity receipt");
        return;
    };
    let mut world = World::new(41, 40);
    let volumes = crate::fixture::volumes_for(&world);
    prepare(&mut layer, &mut host, &world, &volumes);
    let subject = world.controlled_id().expect("played organism");
    let root = layer
        .select_part(key(subject), None, false)
        .expect("drawn root part");
    let selected = layer
        .select_part(key(subject), Some(root), false)
        .expect("drawn non-root part");
    assert_ne!(selected.part, PartId(0));
    assert!(validate(&mut layer, &host, &world, &volumes, selected));

    let other = world
        .organisms
        .iter()
        .find(|organism| organism.id != subject)
        .expect("another organism")
        .id;
    let other_selection = layer
        .select_part(key(other), Some(root), false)
        .expect("other drawn body");
    assert_eq!(other_selection.part, root.part);
    assert_ne!(other_selection.subject, root.subject);

    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == subject)
        .expect("selected organism")
        .position[0] += 5;
    assert!(validate(&mut layer, &host, &world, &volumes, selected));

    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == subject)
        .expect("selected organism")
        .phenotype
        .sever(selected.part);
    assert!(!validate(&mut layer, &host, &world, &volumes, selected));
}

#[test]
fn failed_projection_never_offers_a_part() {
    let Some((_renderer, mut layer, mut host)) = layer() else {
        eprintln!("no adapter; skipping inspection fallback receipt");
        return;
    };
    let world = World::new(42, 40);
    prepare(&mut layer, &mut host, &world, &VolumeMap::new());
    let subject = world.controlled_id().expect("played organism");
    assert_eq!(layer.select_part(key(subject), None, false), None);
}
