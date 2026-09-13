// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent pixel coverage receipt against real voxel raster depth.
use super::*;
use crate::section::{CameraMode, view::View};
use mesocosm_core::{VolumeRef, effect_experiment::Glyph};
use mesocosm_lens::FRAME_FORMAT;
use mesocosm_mesh::{BodyMesh, Volume};
use mesocosm_render::{LiveBody, LiveBodyRenderer};

const WIDTH: u32 = 128;

fn read(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<[u8; 4]> {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("glyph depth receipt readback"),
        size: u64::from(WIDTH * WIDTH * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(WIDTH),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: WIDTH,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("GPU readback");
    let mapped = slice.get_mapped_range().expect("mapped glyph receipt");
    let pixels = mapped
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect();
    drop(mapped);
    buffer.unmap();
    pixels
}

fn clear(
    encoder: &mut wgpu::CommandEncoder,
    color: Option<&wgpu::TextureView>,
    depth: &wgpu::TextureView,
) {
    let attachment = color.map(|view| wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    });
    let colors = attachment.map(|a| [Some(a)]);
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("glyph receipt clear"),
        color_attachments: colors.as_ref().map_or(&[], |v| v.as_slice()),
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

#[test]
fn glyph_stroke_edges_share_voxel_depth_and_camera() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("glyph depth receipt requires a GPU adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
        .expect("glyph depth receipt requires a GPU device");
    let texture = |format, usage, label| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: WIDTH,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    let color = texture(
        FRAME_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        "glyph receipt color",
    );
    let depth = texture(
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
        "glyph receipt depth",
    );
    let color_view = color.create_view(&Default::default());
    let depth_view = depth.create_view(&Default::default());
    let mesh = BodyMesh::single(VolumeRef::from_tag(91), &Volume::solid([4, 4, 4], 7));
    let mut bodies = LiveBodyRenderer::new(&device, FRAME_FORMAT, 8);
    let mut glyphs = GlyphLayer::new(&device);
    for mode in [CameraMode::Side, CameraMode::Oblique] {
        let view = View {
            mode,
            centre: [0.0; 3],
            half: 7.0,
            aspect: 1.0,
            depth: 32.0,
            pitch: None,
            bounds: None,
        };
        let forward = view.basis()[2];
        let mut render = |body: bool, distance: Option<f32>, ignore_occluder: bool| {
            let mut encoder = device.create_command_encoder(&Default::default());
            clear(&mut encoder, Some(&color_view), &depth_view);
            if body {
                bodies
                    .draw(
                        &device,
                        &queue,
                        &mut encoder,
                        &color_view,
                        &depth_view,
                        view.matrix(),
                        Some(view.clip()),
                        &[LiveBody {
                            mesh: &mesh,
                            materials: &[],
                            origin: [-2.0; 3],
                            scale: 1.0,
                            yaw_radians: 0.0,
                            tint: [1.0; 3],
                            focused: false,
                            selected_part: None,
                        }],
                    )
                    .expect("real voxel occluder");
            }
            if let Some(distance) = distance {
                if ignore_occluder {
                    clear(&mut encoder, None, &depth_view);
                }
                glyphs
                    .set(vec![SpatialGlyph {
                        centre: forward.map(|v| v * distance),
                        size: 8.0,
                        angle: 0.0,
                        glyph: Glyph::Slashes,
                        color: [1.0, 0.0, 1.0, 1.0],
                    }])
                    .expect("valid fixture glyph");
                glyphs.draw(&queue, &mut encoder, &color_view, &depth_view, view);
            }
            queue.submit([encoder.finish()]);
            read(&device, &queue, &color)
        };
        let body = render(true, None, false);
        let isolated = render(false, Some(5.0), false);
        let behind = render(true, Some(5.0), false);
        let ahead = render(true, Some(-5.0), false);
        let no_occluder_depth = render(true, Some(5.0), true);
        let mut hidden = 0;
        let mut exposed = 0;
        let mut front_mismatch = 0;
        for i in 0..body.len() {
            let is_body = body[i][..3] != [0, 0, 0];
            let is_glyph = isolated[i][..3] != [0, 0, 0];
            if is_glyph && is_body {
                hidden += 1;
                assert_eq!(
                    behind[i], body[i],
                    "{mode:?}: rear stroke leaks through body at pixel {i}"
                );
                assert_eq!(
                    no_occluder_depth[i], isolated[i],
                    "{mode:?}: depth-cleared control must reveal stroke at pixel {i}"
                );
            } else if is_glyph {
                exposed += 1;
                assert_eq!(
                    behind[i], isolated[i],
                    "{mode:?}: exposed stroke edge disappeared at pixel {i}"
                );
            }
            if ahead[i] != no_occluder_depth[i] {
                front_mismatch += 1;
            }
        }
        assert!(
            hidden > 20 && exposed > 20,
            "{mode:?}: fixture must cover body and stroke edges ({hidden}/{exposed})"
        );
        // Translation along the view axis leaves projection unchanged. Allow
        // two boundary pixels for floating-point raster ties in oblique view.
        assert!(
            front_mismatch <= 2,
            "{mode:?}: front glyph differs from depth-cleared control at {front_mismatch} pixels"
        );
        eprintln!(
            "glyph depth {mode:?}: hidden={hidden}, exposed={exposed}, front_difference={front_mismatch}"
        );
    }
}
