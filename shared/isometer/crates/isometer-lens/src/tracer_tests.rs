// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use isocosm::legacy::mesocosm::places::{Ground, Places};

use crate::{
    BrickChange, BrickFrameInput, BrickMap, BrickProjectionRevision, BrickRevision,
    BrickTraceError, BrickTracer, CritterPose, Flight, Grade, LeasedAtlas, critter::Capsule,
};

mod atlas;
mod grade;
mod palette;
mod roster;
mod unlit;

fn ground() -> Ground {
    let grown = Places::grown(4_242, 4, 64);
    Ground::grow(&grown, 64)
}

fn flight(ground: &Ground) -> Flight {
    let top = ground.surface(4, 4).expect("ground column");
    Flight {
        eye: [4.5, top as f32 + 14.0, 4.5],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    }
}

#[test]
fn a_carved_ground_updates_only_its_brick_slots_in_the_live_tracer() {
    let mut ground = ground();
    let mut map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let before = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("initial brick frame");
    assert!(before.diagnostics.brick_upload_bytes > 0);
    assert_eq!(before.diagnostics.trace_passes, 1);

    let top = ground.surface(4, 4).expect("ground column");
    assert!(ground.carve([4, top, 4], 2) > 0);
    let dirty = ground.drain_dirty();
    let slots = map
        .refresh(&ground, dirty)
        .expect("a carve preserves the brick map shape");
    assert!(!slots.is_empty());
    let after = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .changed(BrickChange::Slots(&slots)),
        )
        .expect("updated brick frame");

    assert_eq!(
        after.diagnostics.brick_upload_bytes,
        slots.len() as u64 * (8 * 8 * 8 + size_of::<u32>()) as u64,
        "a carve uploads just its changed brick slots"
    );
    assert_ne!(
        before.pixels, after.pixels,
        "the opened material is visible"
    );
}

#[test]
fn a_steady_brick_frame_has_no_upload_churn() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::retro(3);
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade);
    let first = tracer.capture(input).expect("first brick frame");
    let steady = tracer.capture(input).expect("steady brick frame");

    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
    assert_eq!(steady.diagnostics.uniform_upload_bytes, 0);
    assert_eq!(steady.diagnostics.resource_creations, 0);
    assert_eq!(steady.pixels, first.pixels);
}

#[test]
fn an_equal_extent_projection_republishes_without_recreating_textures() {
    let ground = ground();
    let mut keys = ground.keys();
    let first_key = keys.next().expect("ground has a first brick");
    let second_key = keys
        .find(|key| *key != first_key)
        .expect("ground has a second brick");
    let first_map = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(4), [first_key])
        .expect("first one-brick projection");
    let second_map = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(5), [second_key])
        .expect("second one-brick projection");
    assert_eq!(first_map.pointer_extent(), second_map.pointer_extent());
    assert_eq!(first_map.atlas_extent(), second_map.atlas_extent());
    assert_ne!(first_map.origin(), second_map.origin());

    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping projection replacement receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    tracer
        .capture(BrickFrameInput::new(&first_map, revision, &camera, &grade))
        .expect("first projection");
    let replaced = tracer
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("equal-sized replacement projection");

    assert!(replaced.diagnostics.projection_replaced);
    assert!(!replaced.diagnostics.map_recreated);
    assert_eq!(replaced.diagnostics.resource_creations, 0);
    assert_eq!(replaced.diagnostics.bind_group_rebuilds, 0);
    assert_eq!(
        replaced.diagnostics.brick_upload_bytes,
        size_of_val(second_map.pointers()) as u64 + second_map.atlas().len() as u64
    );
    let mut control = BrickTracer::headless(64, 64).expect("control tracer");
    let control_frame = control
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("fresh second projection");
    assert_eq!(
        replaced.pixels, control_frame.pixels,
        "equal-sized replacement did not publish the selected page"
    );

    let steady = tracer
        .capture(BrickFrameInput::new(&second_map, revision, &camera, &grade))
        .expect("steady replacement projection");
    assert!(!steady.diagnostics.projection_replaced);
    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
}

#[test]
fn an_unknown_changed_slot_is_refused_before_the_cache_stamp_advances() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping invalid-slot receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let revision = BrickRevision(ground.revision());
    tracer
        .capture(BrickFrameInput::new(&map, revision, &camera, &grade))
        .expect("baseline frame");

    let next = BrickRevision(revision.0 + 1);
    let invalid = [u32::MAX];
    assert!(matches!(
        tracer.capture(
            BrickFrameInput::new(&map, next, &camera, &grade).changed(BrickChange::Slots(&invalid))
        ),
        Err(crate::BrickTraceError::UnknownBrickSlot(u32::MAX))
    ));

    let admitted = tracer
        .capture(BrickFrameInput::new(&map, next, &camera, &grade))
        .expect("full publication after refusal");
    assert!(
        admitted.diagnostics.brick_upload_bytes > 0,
        "the refused slot change advanced the resident cache stamp"
    );
}

#[test]
fn a_nearer_sdf_body_composes_in_front_of_ground() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick/body composition receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let terrain = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("terrain frame");
    let top = ground.surface(4, 4).expect("ground column") as f32;
    let pose = CritterPose::from_capsules(
        vec![Capsule {
            a: [4.5, top + 7.0, 4.5],
            ra: 1.4,
            b: [4.5, top + 5.0, 4.5],
            rb: 1.1,
        }],
        [[4.5, top + 6.6, 4.5, 0.18], [4.5, top + 6.2, 4.5, 0.18]],
        [0.15, 0.86, 0.32],
    );
    let composed = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .with_pose(&pose),
        )
        .expect("composed frame");

    assert_eq!(composed.diagnostics.brick_upload_bytes, 0);
    assert!(composed.diagnostics.uniform_upload_bytes > 0);
    assert_ne!(
        terrain.pixels, composed.pixels,
        "body wins its nearer pixels"
    );
}

/// The depth join, proven without a raster engine: a caller-owned depth
/// texture pre-cleared to a chosen plane stands in for a raster tenant's
/// stored depth, and the traced pass must lose exactly where that plane is
/// nearer than the traced surface's own clip depth.
#[test]
fn the_depth_join_settles_pixels_between_tracer_and_raster_depth() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("no adapter; skipping depth join receipt");
        return;
    };
    let Ok((device, queue)) = pollster::block_on(adapter.request_device(&Default::default()))
    else {
        eprintln!("no device; skipping depth join receipt");
        return;
    };
    let (width, height) = (96u32, 64u32);
    let mut tracer = BrickTracer::with_device(device.clone(), queue.clone(), width, height);
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let camera = flight(&ground);
    let grade = Grade::clay();

    let colour = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth join colour"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::FRAME_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let colour_view = colour.create_view(&Default::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth join depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&Default::default());

    // The stand-in raster: sentinel red everywhere, depth at a chosen value.
    let clear = |depth_value: f32| {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stand-in raster clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &colour_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(depth_value),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        drop(pass);
        queue.submit([encoder.finish()]);
    };
    let sentinel_pixels = |label: &str| -> usize {
        let padded = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("depth join readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            colour.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect(label);
        let data = slice.get_mapped_range().expect("map range");
        let mut sentinels = 0;
        for row in 0..height {
            let start = (row * padded) as usize;
            for pixel in data[start..start + (width * 4) as usize].chunks_exact(4) {
                if pixel[..3] == [255, 0, 0] {
                    sentinels += 1;
                }
            }
        }
        sentinels
    };
    // Depth as a world-z ramp: clip.z = (z - 4.6) / 1.2, clip.w = 1. The
    // flight camera's rays leave eye z 4.5 and descend roughly 14 voxels
    // with forward z slopes of 0.009 to 0.093, so hits land near z 4.6 at
    // the frame's bottom and z 5.8 at its top. Against a raster plane at
    // 0.5 the trace then wins below z 5.2 and loses above it, and the
    // gentle ramp keeps both classes present under relief variation.
    // Columns are WGSL mat4x4 columns.
    let z_plane = [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0 / 1.2, 0.0],
        [0.0, 0.0, -4.6 / 1.2, 1.0],
    ];
    let input = |clip: [[f32; 4]; 4]| {
        BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
            .with_clip_from_world(clip)
    };
    let trace = |tracer: &mut BrickTracer, clip: [[f32; 4]; 4]| {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        tracer
            .encode_with_depth(&mut encoder, &colour_view, &depth_view, input(clip))
            .expect("depth join frame");
        queue.submit([encoder.finish()]);
    };
    let total = (width * height) as usize;

    // A frame input without the matrix must refuse rather than write
    // identity depth.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let refused = tracer.encode_with_depth(
        &mut encoder,
        &colour_view,
        &depth_view,
        BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade),
    );
    assert_eq!(refused, Err(BrickTraceError::MissingClipFromWorld));
    drop(encoder);

    // Constant traced depth on either side of the raster plane first: the
    // whole frame must change hands, both ways.
    let constant_depth = |depth: f32| {
        [
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, depth, 1.0],
        ]
    };
    clear(0.5);
    trace(&mut tracer, constant_depth(0.0));
    assert_eq!(sentinel_pixels("nearer trace"), 0);

    clear(0.5);
    trace(&mut tracer, constant_depth(1.0));
    assert_eq!(sentinel_pixels("farther trace"), total);

    // Raster mid-plane: the trace wins exactly where its surface sits
    // before world z 4.55, and only there.
    clear(0.5);
    trace(&mut tracer, z_plane);
    let split = sentinel_pixels("mid raster");
    assert!(
        split > 0 && split < total,
        "the mid-plane raster must occlude some traced pixels and lose others, kept {split} of {total}"
    );
}
