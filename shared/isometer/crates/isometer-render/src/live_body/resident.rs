// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_mesh::PartMesh;
use vertex::part_vertices;

/// Keep uploads in encoder order so independent passes can share GPU draw
/// buffers without a queue write changing a pass recorded earlier.
pub(crate) fn upload_in_order(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::Buffer,
    offset: u64,
    bytes: &[u8],
) {
    let source = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ordered body upload"),
        contents: bytes,
        usage: wgpu::BufferUsages::COPY_SRC,
    });
    encoder.copy_buffer_to_buffer(&source, 0, target, offset, bytes.len() as u64);
}

impl LiveBodyRenderer {
    pub(crate) fn mesh_buffer_bytes(&self) -> u64 {
        self.meshes.values().map(|mesh| mesh.vertices.size()).sum()
    }

    pub(super) fn cache_mesh(
        &mut self,
        device: &wgpu::Device,
        key: [u8; 32],
        mesh: &PartMesh,
        stats: &mut BodyDrawStats,
    ) {
        if let Some(cached) = self.meshes.get_mut(&key) {
            cached.last_used = self.clock;
            return;
        }
        while self.meshes.len() >= self.max_cached_meshes {
            let oldest = self
                .meshes
                .iter()
                .min_by_key(|(_, mesh)| mesh.last_used)
                .map(|(key, _)| *key)
                .expect("nonempty over capacity");
            self.meshes.remove(&oldest);
            self.instances.retain(|(volume, _), _| *volume != oldest);
            stats.evictions += 1;
        }
        let vertices = part_vertices(mesh);
        let vertex_count = vertices.len() as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesocosm live body volume"),
            contents: if vertices.is_empty() {
                bytemuck::bytes_of(&BodyVertex {
                    position: [0.0; 3],
                    colour: [0.0; 3],
                    material_shade: [0.0; 2],
                })
            } else {
                bytemuck::cast_slice(&vertices)
            },
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.meshes.insert(
            key,
            CachedMesh {
                vertices: buffer,
                vertex_count,
                last_used: self.clock,
            },
        );
        stats.mesh_builds += 1;
        stats.mesh_uploads += 1;
        stats.mesh_upload_bytes += size_of::<BodyVertex>() * vertices.len();
        stats.static_vertices_uploaded += vertices.len();
    }

    pub(super) fn update_instances(
        &mut self,
        device: &wgpu::Device,
        key: ([u8; 32], u32),
        instances: &[Instance],
        stats: &mut BodyDrawStats,
        mut write: impl FnMut(&wgpu::Buffer, u64, &[u8]),
    ) {
        let bytes = bytemuck::cast_slice(instances);
        let required = bytes.len().max(size_of::<Instance>());
        let replace = self
            .instances
            .get(&key)
            .is_none_or(|cached| cached.capacity < required);
        if replace {
            let capacity = required.next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("mesocosm live body instances"),
                size: capacity as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instances.insert(
                key,
                CachedInstances {
                    buffer,
                    capacity,
                    bytes: Vec::new(),
                },
            );
        }
        let cached = self.instances.get_mut(&key).expect("inserted or present");
        if cached.bytes != bytes {
            write(&cached.buffer, 0, bytes);
            stats.instance_upload_bytes += bytes.len();
            cached.bytes.clear();
            cached.bytes.extend_from_slice(bytes);
        }
    }
}
