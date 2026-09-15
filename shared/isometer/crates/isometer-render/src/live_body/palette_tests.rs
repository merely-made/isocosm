// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a body's colour table does to its pixels, pinned pixel by pixel.
//!
//! The fallback path is asserted against arithmetic computed here on the CPU —
//! `srgb(material_colour(material) * face_shade(..) * tint)`, which is what
//! the pass has always drawn — so any change to it, a palette leaking into a
//! body that carries none included, fails these rather than passing silently.
//!
//! **Tolerance.** One code value per channel. The comparison crosses a GPU's
//! `pow` against this machine's, and `Rgba8Unorm` quantises the result; a
//! wrong colour misses by far more than one code value, and the crate's own
//! header already refuses byte equality across drivers.

use super::*;
use isometer_core::VolumeRef;
use isometer_mesh::Volume;

const SIZE: u32 = 16;
const TOLERANCE: i32 = 1;

/// Straight down: every probe pixel lands on a `+Y` face, whose shade is
/// exactly 1.0, so a palette entry survives to the frame as its own byte.
const ABOVE: [[f32; 4]; 4] = [
    [0.9, 0.0, 0.0, 0.0],
    [0.0, 0.0, -0.2, 0.0],
    [0.0, 0.9, 0.0, 0.0],
    [-0.9, -0.9, 0.7, 1.0],
];

/// Along `+X`: the probe lands on a `+X` face, shade 0.78, which is where the
/// face shading is checked to still multiply a table entry.
const ASIDE: [[f32; 4]; 4] = [
    [0.0, 0.0, -0.2, 0.0],
    [0.0, 0.9, 0.0, 0.0],
    [0.9, 0.0, 0.0, 0.0],
    [-0.9, -0.9, 0.7, 1.0],
];

fn gpu() -> Option<crate::Renderer> {
    match crate::Renderer::headless(SIZE, SIZE) {
        Ok(renderer) => Some(renderer),
        Err(crate::RenderError::NoAdapter) => {
            eprintln!("no adapter; skipping live body palette receipt");
            None
        },
        Err(error) => panic!("headless renderer failed: {error:?}"),
    }
}

struct Probe {
    colour: wgpu::Texture,
    colour_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    renderer: LiveBodyRenderer,
}

impl Probe {
    fn new(device: &wgpu::Device) -> Self {
        let size = wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        };
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("palette probe colour"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("palette probe depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        Self {
            colour_view: colour.create_view(&Default::default()),
            depth_view: depth.create_view(&Default::default()),
            colour,
            renderer: LiveBodyRenderer::new(device, wgpu::TextureFormat::Rgba8Unorm, 8),
        }
    }

    /// One cleared frame of bodies, read back as RGBA8 rows.
    fn shoot(
        &mut self,
        host: &crate::Renderer,
        clip: [[f32; 4]; 4],
        bodies: &[LiveBody<'_>],
    ) -> (Vec<u8>, BodyDrawStats) {
        let (device, queue) = (host.device(), host.queue());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("palette probe clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.colour_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
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
        let stats = self
            .renderer
            .draw(
                device,
                queue,
                &mut encoder,
                &self.colour_view,
                &self.depth_view,
                clip,
                None,
                bodies,
            )
            .expect("the probe frame draws");
        let unpadded = SIZE * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("palette probe readback"),
            size: (padded * SIZE) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            self.colour.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(SIZE),
                },
            },
            wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the readback maps");
        let mapped = slice.get_mapped_range().expect("mapped bytes");
        let mut pixels = Vec::with_capacity((unpadded * SIZE) as usize);
        for row in 0..SIZE {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        staging.unmap();
        (pixels, stats)
    }
}

fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
    let at = ((y * SIZE + x) * 4) as usize;
    [frame[at], frame[at + 1], frame[at + 2]]
}

/// The shader's own encode, as bytes.
fn encoded(linear: [f32; 3]) -> [u8; 3] {
    linear.map(|channel| {
        let display = if channel <= 0.003_130_8 {
            channel * 12.92
        } else {
            1.055 * channel.max(0.0).powf(1.0 / 2.4) - 0.055
        };
        (display.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

/// What the pass has always drawn for one material on one face.
fn hashed(material: u8, shade: f32) -> [u8; 3] {
    encoded(crate::material_colour(material).map(|channel| channel * shade))
}

fn from_palette(colour: PaletteColour, shade: f32) -> [u8; 3] {
    encoded(colour.map(|channel| palette::linear_from_display(channel) * shade))
}

#[track_caller]
fn near(got: [u8; 3], want: [u8; 3], what: &str) {
    let off = (0..3)
        .map(|i| (i32::from(got[i]) - i32::from(want[i])).abs())
        .max()
        .unwrap_or(0);
    assert!(off <= TOLERANCE, "{what}: got {got:?}, wanted {want:?}");
}

fn cube(material: u8) -> (VolumeRef, Volume) {
    (VolumeRef::from_tag(material), Volume::solid([2, 2, 2], material))
}

/// I6's third clause: a body with no table draws exactly as it did, and the
/// pinned value is the fallback arithmetic itself rather than a magic byte.
#[test]
fn a_body_without_a_table_draws_the_hashed_material_colour() {
    let Some(host) = gpu() else { return };
    let mut probe = Probe::new(host.device());
    let (reference, volume) = cube(7);
    let mesh = BodyMesh::single(reference, &volume);
    let body = [LiveBody::new(&mesh, [0.0; 3])];

    let (above, stats) = probe.shoot(&host, ABOVE, &body);
    near(pixel(&above, 8, 8), hashed(7, 1.0), "the lit top face");
    assert_eq!(
        stats.palette_upload_bytes, 0,
        "a frame with no table writes no table"
    );

    let (aside, _) = probe.shoot(&host, ASIDE, &body);
    near(pixel(&aside, 8, 8), hashed(7, 0.78), "the shaded +X face");
    assert_ne!(
        pixel(&above, 8, 8),
        pixel(&aside, 8, 8),
        "the two faces differ by their shading, which the pin must see"
    );
}

/// I6's first two clauses: the table colours the material, and the body tint
/// and the face shading still multiply it.
#[test]
fn a_table_recolours_the_material_and_keeps_shading_and_tint() {
    let Some(host) = gpu() else { return };
    let mut probe = Probe::new(host.device());
    let (reference, volume) = cube(7);
    let mesh = BodyMesh::single(reference, &volume);
    // Entry 0 is the empty material; entry 7 is the one this cube is made of.
    let mut colours = vec![[0, 0, 0]; 8];
    colours[7] = [200, 40, 90];
    let palette = MaterialPalette::new(&colours);
    let body = [LiveBody {
        palette: Some(palette),
        ..LiveBody::new(&mesh, [0.0; 3])
    }];

    let (above, stats) = probe.shoot(&host, ABOVE, &body);
    assert!(stats.palette_upload_bytes > 0, "the table reaches the GPU");
    near(
        pixel(&above, 8, 8),
        [200, 40, 90],
        "an unshaded face is the table entry itself",
    );

    let (aside, repeated) = probe.shoot(&host, ASIDE, &body);
    near(
        pixel(&aside, 8, 8),
        from_palette([200, 40, 90], 0.78),
        "the +X face is the entry under the same shade the hash gets",
    );
    assert_eq!(
        repeated.palette_upload_bytes, 0,
        "an unchanged table uploads nothing"
    );

    let dimmed = [LiveBody {
        tint: [0.5; 3],
        ..body[0]
    }];
    let (tinted, _) = probe.shoot(&host, ABOVE, &dimmed);
    near(
        pixel(&tinted, 8, 8),
        encoded([200, 40, 90].map(|c| palette::linear_from_display(c) * 0.5)),
        "the tint still multiplies after the table",
    );
}

/// I6's fallback clause at material granularity: a table shorter than the
/// highest id in the frame colours what it names and leaves the rest hashed.
#[test]
fn a_short_table_falls_back_per_material() {
    let Some(host) = gpu() else { return };
    let mut probe = Probe::new(host.device());
    // Left column material 2, right column material 30; one table names 2 only.
    let mut volume = Volume::solid([2, 2, 2], 2);
    for y in 0..2 {
        for z in 0..2 {
            volume.set(1, y, z, 30);
        }
    }
    let mesh = BodyMesh::single(VolumeRef::from_tag(3), &volume);
    let colours = vec![[0, 0, 0], [0, 0, 0], [30, 190, 120]];
    let body = [LiveBody {
        palette: Some(MaterialPalette::new(&colours)),
        ..LiveBody::new(&mesh, [0.0; 3])
    }];

    let (above, _) = probe.shoot(&host, ABOVE, &body);
    near(
        pixel(&above, 4, 8),
        [30, 190, 120],
        "the named material takes its table colour",
    );
    near(
        pixel(&above, 11, 8),
        hashed(30, 1.0),
        "the unnamed material keeps the hash rather than failing",
    );
}

/// Two bodies under one table share its slot, so a palette costs batches only
/// when the palettes actually differ.
#[test]
fn bodies_sharing_a_table_share_its_slot() {
    let Some(host) = gpu() else { return };
    let mut probe = Probe::new(host.device());
    let (reference, volume) = cube(7);
    let mesh = BodyMesh::single(reference, &volume);
    let colours = vec![[0, 0, 0]; 8];
    let other = vec![[9, 9, 9]; 8];
    let shared = MaterialPalette::new(&colours);
    let bodies = [
        LiveBody {
            palette: Some(shared),
            ..LiveBody::new(&mesh, [0.0; 3])
        },
        LiveBody {
            palette: Some(MaterialPalette::new(&colours)),
            ..LiveBody::new(&mesh, [4.0, 0.0, 0.0])
        },
    ];
    let (_, stats) = probe.shoot(&host, ABOVE, &bodies);
    assert_eq!(stats.draws, 1, "one volume, one table, one draw");
    assert_eq!(stats.instances, 2);

    let split = [
        bodies[0],
        LiveBody {
            palette: Some(MaterialPalette::new(&other)),
            ..bodies[1]
        },
    ];
    let (_, stats) = probe.shoot(&host, ABOVE, &split);
    assert_eq!(stats.draws, 2, "two tables split the batch");
    assert_eq!(
        stats.mesh_builds, 0,
        "and the immutable volume cache is untouched by either"
    );
}

#[test]
fn an_empty_table_is_the_same_as_no_table() {
    let mut tables = Vec::new();
    assert_eq!(palette_slot(None, &mut tables), 0);
    assert_eq!(palette_slot(Some(MaterialPalette::new(&[])), &mut tables), 0);
    assert!(tables.is_empty());

    let colours = vec![[1, 2, 3]; 4];
    assert_eq!(
        palette_slot(Some(MaterialPalette::new(&colours)), &mut tables),
        1
    );
    let same = colours.clone();
    assert_eq!(
        palette_slot(Some(MaterialPalette::new(&same)), &mut tables),
        1,
        "an equal table is the same slot, whoever owns the bytes"
    );
    assert_eq!(tables.len(), 1);
}

#[test]
fn a_table_is_bounded_by_the_material_id_space() {
    let long = vec![[7, 7, 7]; PALETTE_ENTRIES + 32];
    let palette = MaterialPalette::new(&long);
    assert_eq!(palette.len(), PALETTE_ENTRIES);
    assert_eq!(palette.colour(255), Some([7, 7, 7]));

    let short = [[1, 2, 3], [4, 5, 6]];
    let palette = MaterialPalette::new(&short);
    assert_eq!(palette.colour(1), Some([4, 5, 6]));
    assert_eq!(palette.colour(2), None, "an unnamed id has no entry");
    let linear = palette.linear(0).expect("entry 0 is named")[0];
    assert!(
        (linear - 1.0 / 255.0 / 12.92).abs() < 1e-9,
        "a dark byte decodes down the linear segment, got {linear}"
    );
}
