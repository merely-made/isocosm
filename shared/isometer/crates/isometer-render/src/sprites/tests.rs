// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{MaterialPalette, Renderer};
use isometer_core::VolumeRef;
use isometer_mesh::{BodyMesh, Volume};

const WIDTH: u32 = 64;
const ROW_BYTES: u32 = WIDTH * 4;
const TILE: u32 = 32;

fn renderer() -> Option<Renderer> {
    match Renderer::headless(WIDTH, WIDTH) {
        Ok(renderer) => Some(renderer),
        Err(crate::RenderError::NoAdapter) => {
            eprintln!("skipping sprite pixel receipt: no GPU adapter");
            None
        },
        Err(error) => panic!("sprite renderer setup failed: {error:?}"),
    }
}

struct Target {
    texture: wgpu::Texture,
    colour: wgpu::TextureView,
    depth: wgpu::TextureView,
}

impl Target {
    fn new(device: &wgpu::Device) -> Self {
        let create = |format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("sprite test target"),
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
        let texture = create(
            COLOUR,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = create(DEPTH, wgpu::TextureUsages::RENDER_ATTACHMENT);
        Self {
            colour: texture.create_view(&Default::default()),
            depth: depth.create_view(&Default::default()),
            texture,
        }
    }

    fn clear(&self, encoder: &mut wgpu::CommandEncoder, depth: f32) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.colour,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(depth),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
    }

    fn read(&self, host: &Renderer, mut encoder: wgpu::CommandEncoder) -> Vec<u8> {
        let buffer = host.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite test readback"),
            size: u64::from(ROW_BYTES * WIDTH),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ROW_BYTES),
                    rows_per_image: Some(WIDTH),
                },
            },
            self.texture.size(),
        );
        host.queue().submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        let (send, recv) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            send.send(result).unwrap()
        });
        host.device()
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        recv.recv().unwrap().unwrap();
        let pixels = slice.get_mapped_range().unwrap().to_vec();
        buffer.unmap();
        pixels
    }
}

fn pixel(frame: &[u8], x: u32, y: u32) -> &[u8] {
    let at = (y * ROW_BYTES + x * 4) as usize;
    &frame[at..at + 4]
}

#[test]
fn layers_refresh_independently_and_held_images_follow_root_and_depth() {
    let Some(host) = renderer() else { return };
    let (device, queue) = (host.device(), host.queue());
    let mut atlas = SpriteAtlas::new(device, COLOUR, TILE, 2, 4).unwrap();
    assert_eq!(atlas.retained_bytes(), u64::from(TILE * TILE * 2 * 8));
    let target = Target::new(device);
    let projection = SpriteProjection::new(
        [Vec3::X.to_array(), Vec3::Y.to_array(), Vec3::Z.to_array()],
        ([0.0; 3], [1.0; 3]),
        TILE,
    )
    .unwrap();
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let red = [[0, 0, 0], [235, 12, 20]];
    let green = [[0, 0, 0], [10, 236, 18]];
    let clip = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.4))
        * Mat4::from_scale(Vec3::new(0.5, 0.5, 0.1));
    let mut sprites = [
        SpriteDraw {
            slot: 0,
            projection,
            origin: [-1.5, -0.5, 0.0],
            scale: 1.0,
            coverage: 16,
        },
        SpriteDraw {
            slot: 1,
            projection,
            origin: [0.5, -0.5, 0.0],
            scale: 1.0,
            coverage: 16,
        },
    ];

    let mut encoder = device.create_command_encoder(&Default::default());
    for (slot, palette) in [red, green].iter().enumerate() {
        let body = LiveBody {
            palette: Some(MaterialPalette::new(palette)),
            ..LiveBody::new(&mesh, [100.0; 3])
        };
        let stats = atlas
            .bake_voxel(device, queue, &mut encoder, slot as u32, projection, body)
            .unwrap();
        assert_eq!(stats.draw_parts, 1);
        assert_eq!(stats.mesh_uploads, usize::from(slot == 0));
    }
    let mut shoot =
        |mut encoder: wgpu::CommandEncoder, depth, sprites: &[SpriteDraw], clip: Mat4| {
            target.clear(&mut encoder, depth);
            let uploaded = atlas
                .draw(
                    device,
                    queue,
                    &mut encoder,
                    &target.colour,
                    &target.depth,
                    clip.to_cols_array_2d(),
                    sprites,
                )
                .unwrap();
            (target.read(&host, encoder), uploaded)
        };
    let (first, uploaded) = shoot(encoder, 1.0, &sprites, clip);
    assert!(uploaded > 0);
    let left = pixel(&first, 16, 32);
    let right = pixel(&first, 48, 32);
    assert!(left[0] > 100 && left[1] < 40 && left[3] == 255, "{left:?}");
    assert!(
        right[1] > 100 && right[0] < 40 && right[3] == 255,
        "{right:?}"
    );
    assert_eq!(pixel(&first, 32, 32)[3], 0);

    let (held, uploaded) = shoot(
        device.create_command_encoder(&Default::default()),
        1.0,
        &sprites,
        clip,
    );
    assert_eq!(uploaded, 0);
    assert_eq!(held, first);

    sprites[0].origin = [0.0, -0.5, -1.0];
    let (moved, uploaded) = shoot(
        device.create_command_encoder(&Default::default()),
        0.325,
        &sprites,
        clip,
    );
    assert!(uploaded > 0, "placement moved, pixels stayed resident");
    assert_eq!(pixel(&moved, 16, 32)[3], 0, "old root must be clear");
    assert_eq!(
        pixel(&moved, 40, 32),
        left,
        "near moved root survives terrain"
    );
    assert_eq!(
        pixel(&moved, 49, 32)[3],
        0,
        "unchanged far root is occluded"
    );

    sprites[0].origin = [0.0, -0.5, 4.0];
    sprites[1].origin = [-1.5, -0.5, 6.0];
    let perspective = Mat4::perspective_lh(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 20.0);
    let (projected, _) = shoot(
        device.create_command_encoder(&Default::default()),
        0.8,
        &sprites,
        perspective,
    );
    assert_eq!(
        pixel(&projected, 35, 32),
        left,
        "reproject the sampled front depth"
    );
    assert_eq!(
        pixel(&projected, 27, 32)[3],
        0,
        "far image is behind the terrain"
    );
}

#[test]
fn live_and_sprite_passes_choose_complementary_pixels() {
    let Some(host) = renderer() else { return };
    let (device, queue) = (host.device(), host.queue());
    let mut atlas = SpriteAtlas::new(device, COLOUR, TILE, 1, 4).unwrap();
    let mut live = LiveBodyRenderer::new(device, COLOUR, 4);
    let target = Target::new(device);
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 1, 1], 1));
    let projection = SpriteProjection::new(
        [Vec3::X.to_array(), Vec3::Y.to_array(), Vec3::Z.to_array()],
        ([0.0; 3], [1.0; 3]),
        TILE,
    )
    .unwrap();
    let origin = [-0.5, -0.5, 0.0];
    let red = [[0, 0, 0], [235, 12, 20]];
    let green = [[0, 0, 0], [10, 236, 18]];
    let sprite = LiveBody {
        palette: Some(MaterialPalette::new(&red)),
        ..LiveBody::new(&mesh, origin)
    };
    let body = LiveBody {
        palette: Some(MaterialPalette::new(&green)),
        ..LiveBody::new(&mesh, origin)
    };
    let clip = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.4))
        * Mat4::from_scale(Vec3::new(0.5, 0.5, 0.1));
    let clip = clip.to_cols_array_2d();
    let mut bake = device.create_command_encoder(&Default::default());
    atlas
        .bake_voxel(device, queue, &mut bake, 0, projection, sprite)
        .unwrap();
    queue.submit(Some(bake.finish()));

    let mut draw = |coverage| {
        let mut encoder = device.create_command_encoder(&Default::default());
        target.clear(&mut encoder, 1.0);
        live.draw_hybrid(
            device,
            queue,
            &mut encoder,
            &target.colour,
            &target.depth,
            clip,
            None,
            &[body],
            &[coverage],
        )
        .unwrap();
        atlas
            .draw(
                device,
                queue,
                &mut encoder,
                &target.colour,
                &target.depth,
                clip,
                &[SpriteDraw {
                    slot: 0,
                    projection,
                    origin,
                    scale: 1.0,
                    coverage,
                }],
            )
            .unwrap();
        target.read(&host, encoder)
    };
    let live_pixels = draw(0);
    let sprite_pixels = draw(16);
    let bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
    for coverage in 0..=16 {
        let frame = draw(coverage);
        for y in 0..WIDTH {
            for x in 0..WIDTH {
                let source = if bayer[(y % 4 * 4 + x % 4) as usize] < coverage {
                    &sprite_pixels
                } else {
                    &live_pixels
                };
                assert_eq!(
                    pixel(&frame, x, y),
                    pixel(source, x, y),
                    "{x},{y} at {coverage}"
                );
            }
        }
    }
    assert_ne!(pixel(&live_pixels, 32, 32), pixel(&sprite_pixels, 32, 32));
    let alpha_control: Vec<_> = pixel(&live_pixels, 32, 32)
        .iter()
        .zip(pixel(&sprite_pixels, 32, 32))
        .map(|(&a, &b)| u8::midpoint(a, b))
        .collect();
    assert_ne!(alpha_control, pixel(&live_pixels, 32, 32));
    assert_ne!(alpha_control, pixel(&sprite_pixels, 32, 32));
}

#[test]
fn legacy_cpu_stamp_has_similar_coverage_and_different_shading() {
    use isometer_mesh::bake::{BakeParams, Palette, bake_facing};
    use isometer_mesh::voxel::Voxels;

    let Some(host) = renderer() else { return };
    let (device, queue) = (host.device(), host.queue());
    let mut atlas = SpriteAtlas::new(device, COLOUR, WIDTH, 1, 4).unwrap();
    let target = Target::new(device);
    let params = BakeParams {
        half_w: 8,
        cube_h: 8,
        ..Default::default()
    };
    let colours = [[0, 0, 0], [180, 72, 42], [40, 100, 190]];
    let palette = Palette::new(colours.to_vec());
    let mut voxels = Voxels::new(2, 2, 2);
    let mut volume = Volume::solid([2; 3], 1);
    voxels.fill(0, 2, 0, 2, 0, 2, 1);
    for (x, y, z) in [(0, 1, 0), (1, 0, 0)] {
        voxels.set(x, y, z, 2);
        volume.set(x as u32, y as u32, z as u32, 2);
    }
    let mesh = BodyMesh::single(VolumeRef::from_tag(1), &volume);
    let body = LiveBody {
        palette: Some(MaterialPalette::new(&colours)),
        ..LiveBody::new(&mesh, [0.0; 3])
    };
    // The legacy 2:1 stamp uses non-square projection. Recreate its pixel
    // coordinates here, aligning its integer samples to the raster centres.
    let unit = params.half_w as f32 * 2.0 / WIDTH as f32;
    let anchor = (2 * params.half_w + params.margin) as f32 + 0.5;
    let offset = 2.0 * anchor / WIDTH as f32 - 1.0;
    let base = Mat4::from_cols_array_2d(&[
        [unit, -unit * 0.5, -0.05, 0.0],
        [0.0, unit, -0.05, 0.0],
        [-unit, -unit * 0.5, -0.05, 0.0],
        [offset, -offset, 0.7, 1.0],
    ]);
    let centre = Vec3::new(1.0, 0.0, 1.0);
    for facing in 0..4 {
        let rotation = Mat4::from_rotation_y(-(facing as f32) * std::f32::consts::FRAC_PI_2);
        let clip =
            base * Mat4::from_translation(centre) * rotation * Mat4::from_translation(-centre);
        let projection = SpriteProjection {
            local_from_clip: clip.inverse(),
            resolution: WIDTH,
        };
        let cpu = bake_facing(&voxels, &palette, facing, &params);
        let mut encoder = device.create_command_encoder(&Default::default());
        atlas
            .bake_voxel(device, queue, &mut encoder, 0, projection, body)
            .unwrap();
        target.clear(&mut encoder, 1.0);
        atlas
            .draw(
                device,
                queue,
                &mut encoder,
                &target.colour,
                &target.depth,
                clip.to_cols_array_2d(),
                &[SpriteDraw {
                    slot: 0,
                    projection,
                    origin: [0.0; 3],
                    scale: 1.0,
                    coverage: 16,
                }],
            )
            .unwrap();
        let gpu = target.read(&host, encoder);
        let (mut intersection, mut union, mut different_colour) = (0, 0, 0);
        for y in 0..WIDTH {
            for x in 0..WIDTH {
                let legacy = if x < cpu.w as u32 && y < cpu.h as u32 {
                    let at = ((y * cpu.w as u32 + x) * 4) as usize;
                    &cpu.rgba[at..at + 4]
                } else {
                    &[0; 4]
                };
                let rendered = pixel(&gpu, x, y);
                let both = legacy[3] == 255 && rendered[3] == 255;
                intersection += usize::from(both);
                union += usize::from(legacy[3] == 255 || rendered[3] == 255);
                different_colour += usize::from(both && legacy != rendered);
            }
        }
        eprintln!(
            "legacy CPU facing {facing}: intersection={intersection} union={union} colour_differences={different_colour}"
        );
        assert!(union > 600 && intersection * 100 > union * 90);
        assert!(
            different_colour > 0,
            "this renderer shades live voxel faces"
        );
    }
}
