// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A per-body colour table for material ids.
//!
//! This is a *colour table*, not a material expression: it says what colour a
//! material id takes and nothing else. [`super::PartMaterial`]'s tissue
//! channels are untouched by it, and the two never meet — one is a density
//! summary the shader marks over a face, the other is the face's base colour.
//!
//! A body without a table draws exactly as before: the vertex already carries
//! `material_colour(material) * face_shade(..)`, and the shader keeps that
//! value whenever the table does not name the material. A short table is
//! therefore not an error — it falls back per material, so a body whose
//! palette stops at id 7 still draws id 30 by the hash.
//!
//! Entries are display-encoded sRGB bytes, the bake's own vocabulary, and are
//! decoded to linear here so the shader's own `srgb()` encode returns the byte
//! the palette named on an unshaded face.

use bytemuck::{Pod, Zeroable};

/// Palette entries one body may address. A material id is a `u8`, so this
/// spans the whole id space; entry `0` is the empty material and never drawn.
pub const PALETTE_ENTRIES: usize = 256;

/// One display-encoded sRGB colour, as `isometer_mesh`'s bake writes it.
pub type PaletteColour = [u8; 3];

/// Bytes one palette occupies in the shared uniform buffer. A multiple of
/// every platform's uniform offset alignment.
pub(super) const BLOCK_BYTES: u64 = (PALETTE_ENTRIES * 16) as u64;

/// A bounded colour table indexed by material id.
///
/// Built from a borrowed slice so a host can hand the same table to every
/// token drawn from one recipe without copying it per frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialPalette<'a>(&'a [PaletteColour]);

impl<'a> MaterialPalette<'a> {
    /// Entries past [`PALETTE_ENTRIES`] are dropped rather than refused: a
    /// material id cannot address them, so they could never be drawn.
    pub fn new(colours: &'a [PaletteColour]) -> Self {
        Self(&colours[..colours.len().min(PALETTE_ENTRIES)])
    }

    pub fn entries(&self) -> &'a [PaletteColour] {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The display bytes material `id` takes, or `None` when the table is too
    /// short to name it and the hashed colour stands.
    pub fn colour(&self, id: u8) -> Option<PaletteColour> {
        self.0.get(usize::from(id)).copied()
    }

    /// The same entry as linear light, which is what the shader multiplies by
    /// the face shade and the body tint.
    pub fn linear(&self, id: u8) -> Option<[f32; 3]> {
        self.colour(id).map(|colour| colour.map(linear_from_display))
    }
}

/// The exact inverse of the shader's `srgb()`, so a palette byte survives the
/// round trip on an unshaded, untinted face.
pub fn linear_from_display(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Which uniform slot a body's table lands in, sharing one slot between
/// bodies that carry the same table. Slot `0` is the zero block: no table, or
/// an empty one, both of which draw by the hashed colour.
pub(super) fn palette_slot<'a>(
    palette: Option<MaterialPalette<'a>>,
    tables: &mut Vec<MaterialPalette<'a>>,
) -> u32 {
    let Some(palette) = palette.filter(|table| !table.is_empty()) else {
        return 0;
    };
    let index = tables
        .iter()
        .position(|table| table.entries() == palette.entries())
        .unwrap_or_else(|| {
            tables.push(palette);
            tables.len() - 1
        });
    index as u32 + 1
}

/// One palette as the shader reads it: linear rgb, with `w` marking an entry
/// the table actually names. An all-zero block is "no table": every material
/// falls back, which is what an absent and an empty palette both mean.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PaletteBlock {
    entries: [[f32; 4]; PALETTE_ENTRIES],
}

impl PaletteBlock {
    pub(super) fn of(palette: MaterialPalette<'_>) -> Self {
        let mut block = Self::zeroed();
        for (slot, colour) in block.entries.iter_mut().zip(palette.entries()) {
            let linear = colour.map(linear_from_display);
            *slot = [linear[0], linear[1], linear[2], 1.0];
        }
        block
    }
}

/// The frame's palettes as one bound uniform buffer, addressed per draw by a
/// dynamic offset. Slot `0` is the zero block, so a body without a table
/// still binds something and still takes the fallback path.
pub(super) struct PaletteTable {
    pub(super) layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    slots: usize,
    uploaded: Vec<u8>,
}

impl PaletteTable {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesocosm live body palette layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(BLOCK_BYTES),
                },
                count: None,
            }],
        });
        let (buffer, bind_group) = allocate(device, &layout, 1);
        Self {
            layout,
            buffer,
            bind_group,
            slots: 1,
            uploaded: vec![0; BLOCK_BYTES as usize],
        }
    }

    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(super) fn offset(slot: u32) -> u32 {
        slot * BLOCK_BYTES as u32
    }

    /// Makes the frame's blocks current, returning the bytes uploaded. An
    /// unchanged set of palettes uploads nothing, exactly as an unchanged
    /// instance batch does.
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        blocks: &[PaletteBlock],
    ) -> usize {
        if blocks.len() > self.slots {
            let slots = blocks.len().next_power_of_two();
            (self.buffer, self.bind_group) = allocate(device, &self.layout, slots);
            self.slots = slots;
            self.uploaded = vec![0; slots * BLOCK_BYTES as usize];
            // A fresh buffer is zeroed, so only non-zero blocks need writing;
            // the comparison below finds them.
        }
        let mut written = 0;
        for (slot, block) in blocks.iter().enumerate() {
            let bytes = bytemuck::bytes_of(block);
            let at = slot * BLOCK_BYTES as usize;
            if self.uploaded[at..at + bytes.len()] == *bytes {
                continue;
            }
            queue.write_buffer(&self.buffer, at as wgpu::BufferAddress, bytes);
            self.uploaded[at..at + bytes.len()].copy_from_slice(bytes);
            written += bytes.len();
        }
        written
    }
}

fn allocate(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    slots: usize,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mesocosm live body palettes"),
        size: slots as wgpu::BufferAddress * BLOCK_BYTES,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("mesocosm live body palette bind group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: wgpu::BufferSize::new(BLOCK_BYTES),
            }),
        }],
    });
    (buffer, bind_group)
}
