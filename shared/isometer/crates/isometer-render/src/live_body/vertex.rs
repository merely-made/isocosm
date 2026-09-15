// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The live body pass's own vertex, split out of `live_body.rs` at the seam
//! the cache already has: immutable per-volume geometry on one side, per-frame
//! instance data on the other.
//!
//! `colour` is the finished fallback colour — `material_colour(material) *
//! face_shade(..)`, the exact arithmetic the pass has always used — and
//! `material` and `shade` are the same two numbers kept apart, so a body that
//! carries a colour table can rebuild the product from a table entry instead.
//! A body without a table never touches them and draws byte-identically.

use bytemuck::{Pod, Zeroable};
use isometer_mesh::PartMesh;

use crate::geometry::{face_shade, material_colour};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct BodyVertex {
    pub(super) position: [f32; 3],
    pub(super) colour: [f32; 3],
    /// The quad's material id, and how lit its face is. Kept as floats
    /// because they ride the same vertex buffer as the position.
    pub(super) material_shade: [f32; 2],
}

impl BodyVertex {
    pub(super) const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 12,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: 24,
                shader_location: 9,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    };
}

pub(super) fn part_vertices(mesh: &PartMesh) -> Vec<BodyVertex> {
    let mut vertices = Vec::with_capacity(mesh.quads.len() * 6);
    for quad in &mesh.quads {
        let base = material_colour(quad.material);
        let shade = face_shade(quad.axis, quad.positive);
        let colour = [base[0] * shade, base[1] * shade, base[2] * shade];
        let corners = quad.corners();
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(BodyVertex {
                position: corners[index].map(|value| value as f32),
                colour,
                material_shade: [f32::from(quad.material), shade],
            });
        }
    }
    vertices
}
