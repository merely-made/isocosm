// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Section consumer receipts: actual encoded body poses joined to its terrain.

use super::*;
use mesocosm_core::{BodyDocument, BodyPhenotype, OrganismId, PartId, VolumeRef};
use mesocosm_mesh::VolumeMap;
use mesocosm_render::{RenderError, Renderer};

#[path = "picking_pixels.rs"]
mod pixels;

fn world() -> (World, OrganismId, OrganismId) {
    let mut world = World::new(41, 40);
    let controlled = world.controlled_id().unwrap();
    let other = world
        .organisms
        .iter()
        .find(|o| o.id != controlled)
        .unwrap()
        .id;
    world
        .organisms
        .retain(|o| o.id == controlled || o.id == other);
    for organism in &mut world.organisms {
        organism.phenotype = BodyPhenotype::seed(BodyDocument::new(
            organism.species,
            VolumeRef::from_tag(if organism.id == controlled { 250 } else { 251 }),
            1_000,
            [1; 3],
        ));
        organism.position = [0, 200, if organism.id == controlled { -3 } else { 3 }];
    }
    (world, controlled, other)
}

fn section(ground: &Ground, half: f32) -> Option<(Renderer, Section)> {
    let renderer = match Renderer::headless(65, 65) {
        Ok(renderer) => renderer,
        Err(RenderError::NoAdapter) => {
            eprintln!("no adapter; skipping section visible-part receipt");
            return None;
        },
        Err(error) => panic!("headless renderer failed: {error:?}"),
    };
    let section = Section::new(
        renderer.device().clone(),
        renderer.queue().clone(),
        65,
        65,
        wgpu::TextureFormat::Rgba8Unorm,
        ground,
        Framing::new(half, CameraMode::Side),
    )
    .unwrap();
    Some((renderer, section))
}

fn render(
    section: &mut Section,
    world: &World,
    volumes: &VolumeMap,
    ground: &Ground,
    centre: [f32; 3],
) -> Result<(), String> {
    let mut encoder = section.device.create_command_encoder(&Default::default());
    section.render(
        &mut encoder,
        SectionFrame {
            world,
            volumes,
            ground,
            dirty: &[],
            centre,
            pose: None,
            roster: &[],
        },
    )?;
    section.queue.submit([encoder.finish()]);
    Ok(())
}

#[test]
fn visible_body_identity_uses_the_drawn_pose_and_expires_with_its_frame() {
    let (world, _, nearer) = world();
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(world.ground(), 4.0) else {
        return;
    };
    let centre = [0.25, 200.25, 0.0];
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    let selection = section.pick_pixel([32, 32]).unwrap().unwrap().selection;
    // Independently projected open rectangle: world x=-1..1, y=199..201,
    // half-height 4, centre [.25,200.25], 65 pixels. No edge passes through
    // a pixel centre; leading/top/trailing and their outside neighbours count.
    pixels::assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        world.ground(),
        centre,
        selection,
        Some([22, 38, 26, 42]),
    );
    let hit = section.pick_pixel([32, 32]).unwrap().unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), Some(hit));
    assert_eq!(
        hit.selection.organism, nearer,
        "nearest surface beats controlled-body priority"
    );
    assert!(!hit.tied);
    assert!(section.validate_pick(hit, &world, &volumes));
    let hash = mesocosm_core::snapshot::state_hash(&world);

    section
        .set_body_yaw(nearer, core::f32::consts::FRAC_PI_4)
        .unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    assert!(section.validate_selection(hit.selection, &world, &volumes));
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    assert_eq!(
        section.body_stats().mesh_upload_bytes,
        0,
        "pose changes retain static geometry"
    );
    assert!(!section.validate_pick(hit, &world, &volumes));
    let turned = section.pick_ndc([0.0; 2]).unwrap().unwrap();
    assert_eq!(turned.selection, hit.selection);
    assert_ne!(turned.point, hit.point);
    assert_eq!(mesocosm_core::snapshot::state_hash(&world), hash);
    assert!(section.set_body_yaw(nearer, f32::NAN).is_err());
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), Some(turned));
    assert_eq!(
        section.pick_ndc([f32::NAN, 0.0]),
        Err(BodyPickError::InvalidCoordinates)
    );
    assert_eq!(
        section.pick_pixel([65, 0]),
        Err(BodyPickError::InvalidCoordinates)
    );

    section.resize(67, 65);
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    section.set_mode(CameraMode::Across);
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    assert!(
        render(
            &mut section,
            &world,
            &volumes,
            world.ground(),
            [f32::NAN; 3]
        )
        .is_err()
    );
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
}

#[test]
fn nearer_presented_terrain_occludes_bodies_but_cutaway_and_isolation_remove_it() {
    use mesocosm_core::places::Places;
    let ground = Ground::grow(&Places::grown(4_242, 4, 4), 4);
    assert!(ground.solid([1, 1, 4]), "independent near-terrain fixture");
    let (mut world, controlled, _) = world();
    world.organisms.retain(|o| o.id == controlled);
    let organism = &mut world.organisms[0];
    organism.phenotype = BodyPhenotype::seed(BodyDocument::new(
        organism.species,
        VolumeRef::from_tag(250),
        1_000,
        [4; 3],
    ));
    organism.position = [1, 1, 0];
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(&ground, 4.0) else {
        return;
    };
    let mut habitat = world.terrarium_habitat();
    habitat.bounds.min = [-10, 0, -10];
    habitat.bounds.max = [10, 10, 10];
    habitat.chamber = [1, 1, 0];
    let centre = [1.25, 1.25, 0.0];
    section.configure_terrarium(&habitat, 0.0, Cutaway::Never, [1, 1, 0]);
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), None);
    assert_ne!(section.map.material_at([1, 1, 4]), 0);
    let selection = section.select_part(controlled, None, false).unwrap();
    let hidden = pixels::assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        &ground,
        centre,
        selection,
        None,
    );
    assert!(
        !hidden[32 * 65 + 32],
        "nearer terrain also wins the rendered pixel"
    );

    section.set_body_preview(true, 60.0);
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert_eq!(
        section
            .pick_ndc([0.0; 2])
            .unwrap()
            .unwrap()
            .selection
            .organism,
        controlled
    );
    let exposed = pixels::assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        &ground,
        centre,
        selection,
        None,
    );
    assert!(
        exposed[32 * 65 + 32],
        "isolated view removes the terrain occluder"
    );
    let dimensions = (section.map.pointer_extent(), section.map.atlas_extent());
    section.configure_terrarium(&habitat, 0.0, Cutaway::Always, [1, 1, 0]);
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert!(
        section.terrain_upload_pending,
        "isolated render cannot acknowledge terrain upload"
    );
    assert_eq!(
        (section.map.pointer_extent(), section.map.atlas_extent()),
        dimensions,
        "retained bedrock keeps atlas dimensions fixed while cutaway materials change"
    );
    section.set_body_preview(false, 60.0);
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert!(
        !section.terrain_upload_pending,
        "terrain return consumed the pending full upload"
    );
    let resumed = pixels::assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        &ground,
        centre,
        selection,
        None,
    );
    assert!(
        resumed[32 * 65 + 32],
        "leaving isolation uploads the pending cutaway material map"
    );
    section.configure_terrarium(&habitat, 0.0, Cutaway::Never, [1, 1, 0]);
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), None);

    let hash = mesocosm_core::snapshot::state_hash(&world);
    section.configure_terrarium(&habitat, 0.0, Cutaway::Always, [1, 1, 0]);
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    assert_eq!(
        section.map.material_at([1, 1, 4]),
        0,
        "query uses the cutaway map"
    );
    assert!(
        ground.solid([1, 1, 4]),
        "authoritative terrain remains intact"
    );
    assert_eq!(
        section
            .pick_ndc([0.0; 2])
            .unwrap()
            .unwrap()
            .selection
            .organism,
        controlled
    );
    let exposed = pixels::assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        &ground,
        centre,
        selection,
        None,
    );
    assert!(
        exposed[32 * 65 + 32],
        "cutaway removes the rendered occluder too"
    );
    assert_eq!(mesocosm_core::snapshot::state_hash(&world), hash);

    // A different habitat with the same pitch/reveal must apply even when
    // Ground's revision did not change.
    let old = section.pick_ndc([0.0; 2]).unwrap().unwrap();
    habitat.chamber[2] = 100;
    habitat.bounds.min[0] = 3;
    section.configure_terrarium(&habitat, 0.0, Cutaway::Always, [1, 1, 0]);
    assert_eq!(section.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    assert!(!section.validate_pick(old, &world, &volumes));
    render(&mut section, &world, &volumes, &ground, centre).unwrap();
    let Some(wing_scene::Cutaway::Bounds { min, .. }) = section.view(centre).cutaway else {
        panic!("a configured terrarium supplies a bounds cutaway");
    };
    assert_eq!(min[0], 3.0);
    assert_eq!(
        section.pick_ndc([0.0; 2]).unwrap(),
        None,
        "new habitat bounds clip the body"
    );
    assert!(
        section.map.material_at([3, 1, 4]) != 0,
        "new chamber restores previously hidden terrain"
    );
}

#[test]
fn a_long_rotating_body_survives_viewport_and_front_slab_edge_culling() {
    let (mut world, controlled, other) = world();
    for organism in &mut world.organisms {
        if organism.id == controlled {
            organism.position = [100, 200, 0];
        } else {
            organism.phenotype = BodyPhenotype::seed(BodyDocument::new(
                organism.species,
                VolumeRef::from_tag(251),
                1_000,
                [1, 1, 10],
            ));
            organism.position = [7, 200, 8];
        }
    }
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(world.ground(), 2.0) else {
        return;
    };
    let centre = [0.25, 200.25, 0.0];
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), None);
    section
        .set_body_yaw(other, core::f32::consts::FRAC_PI_2)
        .unwrap();
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    let hit = section.pick_ndc([0.0; 2]).unwrap().unwrap();
    assert_eq!(hit.selection.organism, other);
    assert!(
        (hit.point[2] - 7.0).abs() < 1e-4,
        "clipped front reveals the actual back face"
    );
    assert_eq!(section.body_stats().voxel_bodies, 2);
}

#[test]
fn a_queried_part_expires_after_severing_and_fallbacks_never_offer_false_visibility() {
    let (mut world, controlled, _) = world();
    world.organisms.retain(|o| o.id == controlled);
    let organism = &mut world.organisms[0];
    organism.position = [0, 200, 0];
    let mut body = organism.body().clone();
    let part = body
        .attach(
            VolumeRef::from_tag(252),
            1_000,
            [1; 3],
            mesocosm_core::Attachment {
                parent: PartId(0),
                offset: [4, 0, 0],
                yaw: mesocosm_core::Yaw::Quarter,
            },
            mesocosm_core::Provenance::founding(),
        )
        .unwrap();
    organism.phenotype = BodyPhenotype::seed(body);
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(world.ground(), 4.0) else {
        return;
    };
    let centre = [4.25, 200.25, 0.0];
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    let hit = section.pick_ndc([0.0; 2]).unwrap().unwrap();
    assert_eq!(hit.selection.part, part);
    world.organisms[0].phenotype.sever(part);
    assert!(!section.validate_pick(hit, &world, &volumes));
    assert!(!section.validate_selection(hit.selection, &world, &volumes));
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    assert_eq!(section.pick_ndc([0.0; 2]).unwrap(), None);
    render(
        &mut section,
        &world,
        &VolumeMap::new(),
        world.ground(),
        centre,
    )
    .unwrap();
    assert_eq!(
        section.pick_ndc([0.0; 2]),
        Err(BodyPickError::CapsuleFallback)
    );
}
