// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tokens as live bodies, reached through the facade.
//!
//! A host with a voxel recipe builds an [`isometer_mesh::TokenBody`] and hands
//! this scene the document and the volumes, exactly as it would for any other
//! body. This module is the one line of glue that saves it from naming the
//! mesh crate's `VolumeMap` itself, plus where its colours come from.
//!
//! # Colour
//!
//! [`material_colours`] turns a bake [`Palette`](isometer_mesh::bake::Palette) into the table a token's
//! materials index into — entry `i + 1` is palette index `i`, the offset
//! [`Volume::from_voxels`](isometer_mesh::Volume::from_voxels) introduces so that palette index `0` is not mistaken
//! for an empty cell.
//!
//! That table is what [`BodyLayer::set_palette`](crate::BodyLayer::set_palette)
//! takes, so a token drawn live carries its recipe's own colours — I6, found
//! by I4 and done in
//! `isometry/design_docs/2026-09-15_board_on_isometer_plan.md`:
//!
//! ```ignore
//! scene.bodies_mut().set_palette(subject, Some(material_colours(&palette)));
//! ```
//!
//! It stays distinct from [`isometer_render::PartMaterial`], the per-part
//! material input the body layer takes, which is a tissue-channel *density*: a
//! `PartId`, a channel below `TISSUE_CHANNELS` (five), and a fraction. That
//! one carries no colour and the table does not touch it. A subject with no
//! table registered still draws by the renderer's own `material_colour` hash,
//! exactly as before there were tables.

pub use isometer_mesh::token::{
    MissingLayer, Silhouette, TokenBody, material_colours, mesh_silhouette,
};

use crate::bodies::SceneVolumes;

/// The token's volumes as a frame's volume source.
pub fn scene_volumes(token: &TokenBody) -> SceneVolumes<'_> {
    SceneVolumes::Voxels(&token.volumes)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use isometer_mesh::bake::{BakeParams, Sheet, bake_facing, demo};
    use isometer_mesh::{LiveBodyProjector, VolumeSource};

    use super::*;
    use crate::bodies::Pose;
    use crate::camera::SlabCamera;
    use crate::core::SpeciesId;
    use crate::scene::{Scene, SceneFrame, SceneHost};
    use crate::{PaletteColour, SceneBody, SubjectKey};

    /// The board's own stamp, as `isometer_mesh`'s token receipts use it.
    const BOARD: BakeParams = BakeParams {
        half_w: 2,
        cube_h: 2,
        facings: 4,
        margin: 2,
    };
    const SIZE: u32 = 64;
    /// One code value per channel, as the renderer's own palette receipts
    /// use: the comparison crosses a GPU's `pow` against this machine's and
    /// an `Rgba8Unorm` quantisation.
    const TOLERANCE: i32 = 1;
    /// Every face shade the live pass applies, from `face_shade`.
    const SHADES: [f32; 6] = [1.0, 0.45, 0.78, 0.66, 0.86, 0.58];

    #[test]
    fn a_token_body_resolves_through_the_scenes_volume_source() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        let SceneVolumes::Voxels(volumes) = scene_volumes(&token) else {
            panic!("a token carries authored voxels, not declared boxes");
        };
        assert!(volumes.volume(token.volume).is_some());
        assert_eq!(volumes.len(), 1);
    }

    #[test]
    fn a_token_body_projects_through_the_same_projector_the_body_layer_uses() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        let mut projector = LiveBodyProjector::new();
        let (mesh, _) = projector
            .project_body(&token.document, &token.volumes)
            .expect("a token body projects");

        assert_eq!(mesh.placement_count(), 1);
        assert!(mesh.drawn_quads() > 0);
    }

    /// The colour table is correctly offset, and it is the table the body
    /// layer is fed.
    #[test]
    fn the_material_colour_table_is_the_palette_shifted_by_one() {
        let (_, palette) = demo::hero();
        let colours = material_colours(&palette);

        assert_eq!(colours.len(), palette.0.len() + 1);
        assert_eq!(colours[1], palette.color(0));
        assert_eq!(
            isometer_render::TISSUE_CHANNELS,
            5,
            "PartMaterial is five tissue channels, not a colour table"
        );
    }

    fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
        pollster::block_on(adapter.request_device(&Default::default())).ok()
    }

    struct Host;
    impl SceneHost for Host {}

    /// The shader's own sRGB encode, as bytes.
    fn encoded(linear: [f32; 3]) -> PaletteColour {
        linear.map(|channel| {
            let display = if channel <= 0.003_130_8 {
                channel * 12.92
            } else {
                1.055 * channel.max(0.0).powf(1.0 / 2.4) - 0.055
            };
            (display.clamp(0.0, 1.0) * 255.0).round() as u8
        })
    }

    /// Undoes the bake's own display-space shading (1.06 top, 0.74 left, 0.55
    /// right) to recover the palette entry a baked pixel was painted from.
    fn baked_bases(sheet: &Sheet, palette: &[PaletteColour]) -> BTreeSet<PaletteColour> {
        let mut bases = BTreeSet::new();
        for pixel in sheet.rgba.chunks_exact(4).filter(|pixel| pixel[3] > 0) {
            for entry in palette.iter().skip(1) {
                let painted = [1.06_f32, 0.74, 0.55].map(|factor| {
                    entry.map(|channel| ((f32::from(channel) * factor).round()).clamp(0.0, 255.0) as u8)
                });
                if painted.iter().any(|shaded| *shaded == pixel[..3]) {
                    bases.insert(*entry);
                }
            }
        }
        bases
    }

    /// One frame of one token, read back as RGBA8.
    fn shoot(colours: Option<Vec<PaletteColour>>) -> (Vec<u8>, TokenBody, Vec<PaletteColour>) {
        let (device, queue) = device().expect("checked by the caller");
        let (hero, palette) = demo::hero();
        let table = material_colours(&palette);
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);
        let mut scene = Scene::new(device, queue, SIZE, SIZE).expect("a scene");
        scene.bodies_mut().set_palette(SubjectKey(1), colours);
        let size = token.size().map(|axis| axis as f32);
        let bodies = [SceneBody {
            subject: SubjectKey(1),
            document: &token.document,
            pose: Pose {
                position: [0.0; 3],
                yaw_radians: 0.0,
            },
            scale: 1.0,
            grounded: false,
            tint: [1.0; 3],
            materials: &[],
            always_visible: false,
        }];
        let mut encoder = scene.device.create_command_encoder(&Default::default());
        scene
            .render(
                &mut encoder,
                SceneFrame {
                    camera: SlabCamera {
                        centre: [size[0] / 2.0, size[1] / 2.0, size[2] / 2.0],
                        forward: [0.0, 0.0, -1.0],
                        half_height: size[1] * 0.6,
                        aspect: 1.0,
                        depth: 64.0,
                        cutaway: None,
                    },
                    bodies: &bodies,
                    volumes: scene_volumes(&token),
                    terrain: None,
                    dirty: &[],
                    grade: isometer_lens::Grade::clay(),
                    terrain_appearance: None,
                    body_budget: 4,
                    capsules: None,
                },
                &mut Host,
            )
            .expect("a token frame encodes");
        scene.queue.submit([encoder.finish()]);
        // Straight off the display texture rather than through
        // `Scene::capture`, whose composite re-encodes into an sRGB target:
        // nothing but the body pass belongs between the shader and this
        // comparison.
        let pixels = read_back(&scene);
        (pixels, token, table)
    }

    /// The display texture's RGBA8 rows, with the copy padding stripped.
    fn read_back(scene: &Scene) -> Vec<u8> {
        let texture = scene.display_texture();
        let unpadded = SIZE * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let staging = scene.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("token palette readback"),
            size: (padded * SIZE) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = scene.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
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
        scene.queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        scene
            .device
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
        pixels
    }

    /// Which palette entry, under which face shade, explains a drawn pixel.
    fn explain(pixel: [u8; 3], table: &[PaletteColour]) -> Option<PaletteColour> {
        table.iter().skip(1).copied().find(|entry| {
            SHADES.iter().any(|shade| {
                let want = encoded(
                    entry.map(|channel| {
                        crate::render::live_body::linear_from_display(channel) * shade
                    }),
                );
                (0..3).all(|i| (i32::from(pixel[i]) - i32::from(want[i])).abs() <= TOLERANCE)
            })
        })
    }

    /// I6's last clause: a token drawn through the scene under its recipe's
    /// own table wears the baked sprite's colours.
    ///
    /// **What is compared, and why not pixel for pixel.** The two projections
    /// do not frame alike — the bake's is the 2:1 iso stamp, this is the
    /// scene's own orthographic slab — and they do not shade alike either:
    /// the bake multiplies *display* bytes by 1.06, 0.74 and 0.55 per face,
    /// while the live pass multiplies *linear* light by `face_shade`. So the
    /// renderer's shading is factored out of both sides and what is compared
    /// is the palette entry each pixel was painted from: every drawn pixel
    /// must be one entry under one of the six face shades, and every entry
    /// the live render shows must be one the baked sprite shows too. Aligning
    /// the two rasters is the board's camera work (I5 and B2), not this.
    #[test]
    fn a_token_drawn_through_the_scene_wears_its_baked_colours() {
        if device().is_none() {
            eprintln!("no adapter; skipping isometer token palette receipt");
            return;
        }
        let (_, palette) = demo::hero();
        let (pixels, _token, table) = shoot(Some(material_colours(&palette)));
        let baked = baked_bases(&bake_facing(&demo::hero().0, &palette, 0, &BOARD), &table);
        assert!(baked.len() > 1, "the demo hero is more than one colour");

        let mut drawn = 0;
        let mut shown = BTreeSet::new();
        for pixel in pixels.chunks_exact(4) {
            let pixel = [pixel[0], pixel[1], pixel[2]];
            if pixel == [0, 0, 0] {
                continue; // the frame's cleared background
            }
            drawn += 1;
            let entry = explain(pixel, &table)
                .unwrap_or_else(|| panic!("a drawn pixel {pixel:?} is no palette entry"));
            assert!(
                baked.contains(&entry),
                "the live render shows {entry:?}, which the baked sprite never paints"
            );
            shown.insert(entry);
        }
        assert!(drawn > 100, "the token must cover the frame, got {drawn} px");
        assert!(
            shown.len() > 1,
            "a one-colour agreement proves nothing, got {shown:?}"
        );

        // The control: without the table, the same frame is the renderer's
        // hashed colours, which are not the recipe's.
        let (bare, _, _) = shoot(None);
        let explained = bare
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .filter(|pixel| *pixel != [0, 0, 0])
            .filter(|pixel| explain(*pixel, &table).is_some())
            .count();
        assert!(
            explained * 10 < drawn,
            "a body with no table must not draw the palette's colours, {explained} of {drawn} did"
        );
    }
}
