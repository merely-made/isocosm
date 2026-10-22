// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::borrowed_section::{Presentation, tests::fixture};

#[test]
#[ignore = "explicit H6 actual-device picking comparison"]
fn viewport_pixels_match_the_original_section_and_refuse_an_unpresented_revision() {
    let (session, participant, _, window, site) = fixture();
    let handle = SceneHandle::default();
    handle
        .update(
            &session,
            participant,
            window,
            isocosm::directing::readings::Mode::Survival,
            site,
            0,
        )
        .unwrap();
    let native = handle.snapshot().unwrap();
    let size = [192, 144];
    let camera = native.camera(size[0] as f32 / size[1] as f32);
    let presentation = Presentation {
        mode: section::CameraMode::default(),
        depth: None,
        forward: camera.forward,
        centre: camera.centre,
        half_height: camera.half_height,
        bodies: BodyMode::Voxels,
        budget: section::DEFAULT_BODY_BUDGET,
        light: 1.0,
        sky: section::SkyBlock::fixed(),
        terrain: section::TerrainStyle::default(),
        terrain_y: native
            .scene
            .played
            .and_then(|id| native.scene.at(id))
            .map_or(camera.centre[1], |at| at[1] as f32),
        focus: None,
        selection: None,
    };
    handle
        .present(&native.scene, 7, presentation.clone())
        .unwrap();
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions::default()).unwrap();
    let host = section::host_device::from_renderer(core.renderer());
    let source = Source::new(host.clone(), handle.clone());
    let mut leaf = SectionProducer {
        handle: handle.clone(),
        inner: Some(SceneProducer::new(source)),
    };
    let request = FrameRequest {
        device: core.device(),
        queue: core.queue(),
        size,
        aspect: camera.aspect,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    leaf.inner.as_mut().unwrap().render_scene(&request).unwrap();
    assert!(handle.last_error().is_none(), "{:?}", handle.last_error());
    let selected = leaf
        .select_part(native.played().unwrap(), None, false)
        .expect("the presented played body has an exact part");
    assert!(leaf.validate_selection(selected));

    // The original loop's Section, framing, pose and roster on the same device.
    let mut old = Section::new(
        &host,
        size[0],
        size[1],
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &native.scene.ground,
        Framing::default(),
    )
    .unwrap();
    old.configure_bodies(BodyMode::Voxels, section::DEFAULT_BODY_BUDGET);
    old.set_half_height(camera.half_height);
    old.set_body_light(presentation.light);
    old.set_sky(presentation.sky);
    old.configure_terrain(presentation.terrain, camera.centre, presentation.terrain_y);
    let pose = section::pose_of(&native.scene, 1.0).map(|(pose, _)| pose);
    let roster = section::roster_of(&native.scene, old.slab_window(camera.centre), 1.0);
    let target = core.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("H6 original loop comparison"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let mut encoder = core.device().create_command_encoder(&Default::default());
    old.draw(
        &mut encoder,
        &target.create_view(&Default::default()),
        SectionFrame {
            scene: &native.scene,
            dirty: &[],
            centre: camera.centre,
            pose: pose.as_ref(),
            roster: &roster,
        },
    )
    .unwrap();
    core.queue().submit([encoder.finish()]);
    let before = std::time::Instant::now();
    let mut counts = [0u32; 3];
    for y in (0..size[1]).step_by(3) {
        for x in (0..size[0]).step_by(3) {
            let mut expected = old.pick_at_pixel([x, y]).unwrap();
            // Half-size logical leaf at 2x native scale, including pixel centres.
            let (picked, revision) = leaf
                .pick(
                    ((x as f64 + 0.5) / 2.0, (y as f64 + 0.5) / 2.0),
                    (size[0] as f64 / 2.0, size[1] as f64 / 2.0),
                )
                .unwrap();
            assert_eq!(revision, 7);
            if let (Some(isometer::Pick::Body(want)), Some(isometer::Pick::Body(got))) =
                (&mut expected, picked)
            {
                assert_ne!(
                    want.frame, got.frame,
                    "separate encoded frames retain separate freshness tokens"
                );
                want.frame = got.frame;
            }
            assert_eq!(picked, expected, "pixel {x},{y}");
            counts[match picked {
                Some(isometer::Pick::Body(_)) => 0,
                Some(isometer::Pick::Terrain(_)) => 1,
                None => 2,
            }] += 1;
        }
    }
    assert!(
        counts[0] > 0 && counts[1] > 0 && counts[2] > 0,
        "{counts:?}"
    );
    assert!(leaf.pick((-1.0, 2.0), (96.0, 72.0)).is_err());
    handle.present(&native.scene, 8, presentation).unwrap();
    assert!(
        !leaf.validate_selection(selected),
        "selection expires with the shown snapshot"
    );
    assert!(
        leaf.pick((48.0, 36.0), (96.0, 72.0))
            .unwrap_err()
            .contains("before presentation")
    );
    eprintln!(
        "H6_PICK samples={} body={} terrain={} empty={} comparison_us={}",
        counts.iter().sum::<u32>(),
        counts[0],
        counts[1],
        counts[2],
        before.elapsed().as_micros()
    );
}
