// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The GPU ownership mask. Fresh Section readbacks: selection changes only
//! the named part's pixels, which is a colour-owner claim independent of the
//! CPU surface query it is asserted against.
//!
//! Fixtures come from the parent, `query_tests.rs`.

use super::*;

pub(super) fn assert_selection_mask(
    section: &mut Section,
    world: &World,
    volumes: &VolumeMap,
    ground: &Ground,
    centre: [f32; 3],
    selection: BodySelection,
    expected_rectangle: Option<[u32; 4]>,
) -> Vec<bool> {
    section.set_body_focus(None, None);
    render(section, world, volumes, ground, centre).unwrap();
    let (width, height, before) = section
        .capture(|_, _, _| {})
        .expect("fresh full-frame readback");
    section.set_body_focus(None, Some(selection));
    render(section, world, volumes, ground, centre).unwrap();
    let (_, _, after) = section
        .capture(|_, _, _| {})
        .expect("fresh highlighted readback");
    let mut changed = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            let painted = before[offset..offset + 3] != after[offset..offset + 3];
            let queried = section
                .pick_pixel([x, y])
                .unwrap()
                .is_some_and(|hit| hit.selection == selection);
            assert_eq!(
                painted, queried,
                "GPU/CPU part ownership at pixel [{x},{y}]"
            );
            if let Some([left, right, top, bottom]) = expected_rectangle {
                let expected = (left..=right).contains(&x) && (top..=bottom).contains(&y);
                assert_eq!(
                    painted, expected,
                    "independent interior/edge/outside mask at [{x},{y}]"
                );
            }
            changed.push(painted);
        }
    }
    section.set_body_focus(None, None);
    render(section, world, volumes, ground, centre).unwrap();
    changed
}

#[test]
fn noncentral_pivot_and_quarter_turned_attachment_are_not_culled_by_core_bounds() {
    let (mut world, controlled, other) = world();
    let mut attached = None;
    for organism in &mut world.organisms {
        if organism.id == controlled {
            organism.position = [100, 200, 0];
            continue;
        }
        organism.position = [8, 200, 0];
        let mut body = organism.body().clone();
        let part = body
            .attach(
                VolumeRef::from_tag(252),
                1_000,
                [1, 1, 5],
                mesocosm_core::Attachment {
                    parent: PartId(0),
                    offset: [0; 3],
                    yaw: mesocosm_core::Yaw::Quarter,
                },
                mesocosm_core::Provenance::founding(),
            )
            .unwrap();
        body.parts[part.0 as usize].pivot = [1, 1, 10];
        assert_eq!(
            body.aabb().min[0] + organism.position[0],
            7,
            "core box is wholly outside this viewport"
        );
        // Actual attached volume [0,2]x[0,2]x[0,10], pivot [1,1,10],
        // +90 yaw at x=8 gives world x=-2..8, y=199..201, z=-1..1.
        organism.phenotype = BodyPhenotype::seed(body);
        attached = Some(part);
    }
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(world.ground(), 2.0) else {
        return;
    };
    let centre = [0.25, 200.25, 0.0];
    render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
    let hit = section.pick_pixel([32, 32]).unwrap().unwrap();
    assert_eq!(
        (hit.selection.organism, hit.selection.part),
        (other, attached.unwrap())
    );
    assert_selection_mask(
        &mut section,
        &world,
        &volumes,
        world.ground(),
        centre,
        hit.selection,
        Some([0, 64, 20, 52]),
    );
}

#[test]
fn pitched_front_and_far_wall_pixels_share_the_query_interval() {
    let (mut world, controlled, _) = world();
    world.organisms.retain(|o| o.id == controlled);
    let organism = &mut world.organisms[0];
    organism.phenotype = BodyPhenotype::seed(BodyDocument::new(
        organism.species,
        VolumeRef::from_tag(250),
        1_000,
        [4; 3],
    ));
    let volumes = crate::fixture::volumes_for(&world);
    let Some((_renderer, mut section)) = section(world.ground(), 5.0) else {
        return;
    };
    let habitat = world.terrarium_habitat();
    section.set_mode(CameraMode::TerrariumEast);
    section.configure_terrarium(&habitat, 45.0, Cutaway::Never, [0; 3]);
    section.set_body_preview(true, 16.0);
    let centre = [0.0, 200.0, 0.0];
    // At x=+8 and -8 the two standing walls cut these 2-world-unit cubes.
    // Pixel [32,16] projects through both admitted interiors, away from a
    // polygon boundary. Old screen-up rays missed the front-wall body.
    for position in [[8, 210, 0], [-8, 194, 0]] {
        world.organisms[0].position = position;
        render(&mut section, &world, &volumes, world.ground(), centre).unwrap();
        let hit = section.pick_pixel([32, 16]).unwrap().unwrap();
        assert_eq!(hit.selection.organism, controlled);
        let mask = assert_selection_mask(
            &mut section,
            &world,
            &volumes,
            world.ground(),
            centre,
            hit.selection,
            None,
        );
        assert!(
            mask[16 * 65 + 32],
            "standing-wall interior pixel {position:?}"
        );
        assert!(!mask[60 * 65 + 32], "outside pixel {position:?}");
    }
}
