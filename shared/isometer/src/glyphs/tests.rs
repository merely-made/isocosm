// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independent pixel coverage receipt against real voxel raster depth.
//!
//! Carried over from Mesocosm's `section/glyphs/tests.rs` with the product's
//! `CameraMode` replaced by the bare forward vectors those presets produce,
//! exactly as the camera suite does.
use super::*;
use crate::camera::Cutaway;
use isometer_core::VolumeRef;
use mesocosm_lens::FRAME_FORMAT;
use mesocosm_mesh::{BodyMesh, Volume};
use mesocosm_render::{LiveBody, LiveBodyRenderer};

const WIDTH: u32 = 128;
const SLAB_DEPTH: f32 = 32.0;
const OBLIQUE_DEGREES: f32 = 20.0;

/// The forward `CameraMode::Side` produces.
const SIDE: [f32; 3] = [0.0, 0.0, -1.0];

/// The forward `CameraMode::Oblique` produces: yawed off `-z`, then pitched
/// down, both by the same angle.
fn oblique() -> [f32; 3] {
    let (yaw, pitch) = (OBLIQUE_DEGREES.to_radians(), OBLIQUE_DEGREES.to_radians());
    [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
}

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
    for glyph in Stroke::ALL.iter().copied() {
        for orientation in [
            GlyphOrientation::CameraFacing,
            GlyphOrientation::WorldPlane {
                right: [
                    std::f32::consts::FRAC_1_SQRT_2,
                    0.0,
                    std::f32::consts::FRAC_1_SQRT_2,
                ],
                up: [0.0, 1.0, 0.0],
            },
        ] {
            for (mode, look) in [("side", SIDE), ("oblique", oblique())] {
                let view = SlabCamera {
                    centre: [0.0; 3],
                    forward: look,
                    half_height: 7.0,
                    aspect: 1.0,
                    depth: SLAB_DEPTH,
                    cutaway: None,
                };
                let forward = view.basis()[2];
                let mut render = |body: bool,
                                  distance: Option<f32>,
                                  ignore_occluder: bool,
                                  bounds: Option<([f32; 3], [f32; 3])>,
                                  clip_depth: Option<f32>| {
                    let view = SlabCamera {
                        cutaway: bounds.map(|(min, max)| Cutaway::Bounds { min, max }),
                        depth: clip_depth.unwrap_or(view.depth),
                        ..view
                    };
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
                                view.clip_from_world(),
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
                                glyph,
                                orientation,
                                color: [1.0, 0.0, 1.0, 1.0],
                            }])
                            .expect("valid fixture glyph");
                        glyphs.draw(&queue, &mut encoder, &color_view, &depth_view, view);
                    }
                    queue.submit([encoder.finish()]);
                    read(&device, &queue, &color)
                };
                let body = render(true, None, false, None, None);
                let isolated = render(false, Some(8.0), false, None, None);
                let behind = render(true, Some(8.0), false, None, None);
                let ahead = render(true, Some(-8.0), false, None, None);
                let no_occluder_depth = render(true, Some(8.0), true, None, None);
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
                            "{mode}: rear stroke leaks through body at pixel {i}"
                        );
                        assert_eq!(
                            no_occluder_depth[i], isolated[i],
                            "{mode}: depth-cleared control must reveal stroke at pixel {i}"
                        );
                    } else if is_glyph {
                        exposed += 1;
                        assert_eq!(
                            behind[i], isolated[i],
                            "{mode}: exposed stroke edge disappeared at pixel {i}"
                        );
                    }
                    if ahead[i] != no_occluder_depth[i] {
                        front_mismatch += 1;
                    }
                }
                assert!(
                    hidden > 20 && exposed > 20,
                    "{mode}: fixture must cover body and stroke edges ({hidden}/{exposed})"
                );
                // Translation along the view axis leaves projection unchanged. Allow
                // two boundary pixels for floating-point raster ties in oblique view.
                assert!(
                    front_mismatch <= 2,
                    "{mode}: front glyph differs from depth-cleared control at {front_mismatch} pixels"
                );
                eprintln!(
                    "glyph depth {glyph:?}/{orientation:?}/{mode}: hidden={hidden}, exposed={exposed}, front_difference={front_mismatch}"
                );
                if mode == "side"
                    && glyph == Stroke::Slashes
                    && matches!(orientation, GlyphOrientation::WorldPlane { .. })
                {
                    let full = render(false, Some(0.0), false, None, None);
                    let cut = render(
                        false,
                        Some(0.0),
                        false,
                        Some(([0.0, -100.0, -100.0], [100.0; 3])),
                        None,
                    );
                    let slab = render(false, Some(0.0), false, None, Some(1.5));
                    let mut kept = 0;
                    let mut removed = 0;
                    let mut slab_kept = 0;
                    let mut slab_removed = 0;
                    for i in 0..full.len() {
                        let column = i % WIDTH as usize;
                        if column < WIDTH as usize / 2 {
                            assert_eq!(
                                &cut[i][..3],
                                &[0, 0, 0],
                                "world bound leaks left stroke pixel {i}"
                            );
                            removed += usize::from(full[i][..3] != [0, 0, 0]);
                        } else {
                            assert_eq!(cut[i], full[i], "world bound drops right stroke pixel {i}");
                            kept += usize::from(full[i][..3] != [0, 0, 0]);
                        }
                        // Side projection maps pixel centres to world x directly.
                        // The authored 45-degree plane has z=x, so slab |z|<=.75
                        // independently predicts exactly which stroke pixels remain.
                        let world_x = ((column as f32 + 0.5) / WIDTH as f32 * 2.0 - 1.0) * 7.0;
                        if world_x.abs() <= 0.75 {
                            assert_eq!(slab[i], full[i], "slab drops interior stroke pixel {i}");
                            slab_kept += usize::from(full[i][..3] != [0, 0, 0]);
                        } else {
                            assert_eq!(
                                &slab[i][..3],
                                &[0, 0, 0],
                                "slab leaks edge stroke pixel {i}"
                            );
                            slab_removed += usize::from(full[i][..3] != [0, 0, 0]);
                        }
                    }
                    assert!(
                        kept > 20 && removed > 20 && slab_kept > 20 && slab_removed > 20,
                        "clipped fixtures must straddle boundaries"
                    );
                }
            }
        }
    }
}

#[test]
fn world_plane_is_camera_independent_and_invalid_lists_do_not_replace_it() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
        .expect("glyph validation GPU");
    let (device, _) = pollster::block_on(adapter.request_device(&Default::default()))
        .expect("glyph validation device");
    let valid = SpatialGlyph {
        centre: [1.0, 2.0, 3.0],
        size: 2.0,
        angle: 0.3,
        glyph: Stroke::Quotes,
        color: [1.0, 0.0, 1.0, 1.0],
        orientation: GlyphOrientation::WorldPlane {
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        },
    };
    let mut layer = GlyphLayer::new(&device);
    let admitted = vec![valid; MAX_SPATIAL_GLYPHS];
    layer.set(admitted.clone()).unwrap();
    assert!(layer.set(vec![valid; MAX_SPATIAL_GLYPHS + 1]).is_err());
    assert_eq!(layer.glyphs, admitted);
    for invalid in [
        SpatialGlyph {
            centre: [f32::NAN, 0.0, 0.0],
            ..valid
        },
        SpatialGlyph { size: 0.0, ..valid },
        SpatialGlyph {
            size: f32::INFINITY,
            ..valid
        },
        SpatialGlyph {
            angle: f32::NAN,
            ..valid
        },
        SpatialGlyph {
            color: [1.0, 0.0, 0.0, 0.5],
            ..valid
        },
        SpatialGlyph {
            color: [1.1, 0.0, 0.0, 1.0],
            ..valid
        },
        SpatialGlyph {
            orientation: GlyphOrientation::WorldPlane {
                right: [0.0; 3],
                up: [0.0, 1.0, 0.0],
            },
            ..valid
        },
        SpatialGlyph {
            orientation: GlyphOrientation::WorldPlane {
                right: [2.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
            },
            ..valid
        },
        SpatialGlyph {
            orientation: GlyphOrientation::WorldPlane {
                right: [1.0, 0.0, 0.0],
                up: [1.0, 0.0, 0.0],
            },
            ..valid
        },
        SpatialGlyph {
            orientation: GlyphOrientation::WorldPlane {
                right: [f32::NAN, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
            },
            ..valid
        },
    ] {
        assert!(layer.set(vec![invalid]).is_err());
        assert_eq!(
            layer.glyphs, admitted,
            "invalid replacement must retain the entire admitted list"
        );
    }
    let view = SlabCamera {
        centre: [0.0; 3],
        forward: SIDE,
        half_height: 7.0,
        aspect: 1.0,
        depth: SLAB_DEPTH,
        cutaway: None,
    };
    let other = SlabCamera {
        forward: oblique(),
        ..view
    };
    let world_vertices = |data: Vec<u8>| {
        data.chunks_exact(STRIDE as usize)
            .map(|vertex| {
                std::array::from_fn::<_, 3, _>(|axis| {
                    f32::from_le_bytes(vertex[32 + axis * 4..36 + axis * 4].try_into().unwrap())
                })
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        world_vertices(geometry(&[valid], view)),
        world_vertices(geometry(&[valid], other)),
        "world plane must not follow camera rotation"
    );
    let facing = SpatialGlyph {
        orientation: GlyphOrientation::CameraFacing,
        ..valid
    };
    assert_ne!(
        world_vertices(geometry(&[facing], view)),
        world_vertices(geometry(&[facing], other)),
        "camera-facing control must actually rotate"
    );
    layer.set(Vec::new()).unwrap();
    assert!(layer.glyphs.is_empty());
}
