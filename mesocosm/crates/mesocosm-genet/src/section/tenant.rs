// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's lit body target, using the host's device and frame encoder.

mod geometry;

use std::collections::{BTreeMap, BTreeSet};

use isometer::{BodyLayer, SlabCamera, SubjectKey};
use tenant::{BodyId, HostDevice, LightBlock, Palette, PaletteMesh, Pose};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

struct CachedBody {
    id: BodyId,
    mesh: PaletteMesh,
    palette: Palette,
}

pub(super) struct BodyTenant {
    inner: tenant::Tenant,
    target: wgpu::Texture,
    cached: BTreeMap<SubjectKey, CachedBody>,
    light: f32,
}

impl BodyTenant {
    pub fn new(host: &HostDevice, size: [u32; 2]) -> Self {
        let mut inner = tenant::Tenant::new(host, FORMAT, size);
        inner.set_background([0.0; 4]);
        inner.set_shadows(false);
        Self {
            inner,
            target: target(&host.device, size),
            cached: BTreeMap::new(),
            light: 1.0,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        self.target = target(device, size);
    }

    pub fn view(&self) -> wgpu::TextureView {
        self.target.create_view(&Default::default())
    }

    pub fn set_light(&mut self, intensity: f32) {
        self.light = intensity;
    }

    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        layer: &mut BodyLayer,
        camera: SlabCamera,
    ) -> Result<(), String> {
        let mut retained = BTreeSet::new();
        let mut builds = 0;
        let mut bytes = 0;
        let mut parts = 0;
        for (subject, body) in layer.render_bodies() {
            builds += 1;
            let (mesh, palette, drawn_parts) = geometry::mesh(body, camera.clip())?;
            if mesh.positions.is_empty() {
                continue;
            }
            parts += drawn_parts;
            retained.insert(subject);
            match self.cached.get_mut(&subject) {
                Some(cached) if cached.mesh == mesh && cached.palette == palette => {},
                Some(cached) => {
                    self.inner
                        .set_mesh(cached.id, &mesh, &palette)
                        .map_err(|e| e.to_string())?;
                    bytes += upload_bytes(&mesh, &palette);
                    cached.mesh = mesh;
                    cached.palette = palette;
                },
                None => {
                    let id = self
                        .inner
                        .add_mesh(&mesh, &palette, Pose::default())
                        .map_err(|e| e.to_string())?;
                    bytes += upload_bytes(&mesh, &palette);
                    self.cached
                        .insert(subject, CachedBody { id, mesh, palette });
                },
            }
        }
        let removed: Vec<_> = self
            .cached
            .keys()
            .filter(|subject| !retained.contains(subject))
            .copied()
            .collect();
        for subject in removed {
            let cached = self.cached.remove(&subject).expect("cached subject");
            self.inner.remove(cached.id).map_err(|e| e.to_string())?;
        }
        let camera = geometry::camera(camera);
        if !self.inner.set_camera(&camera) {
            return Err("invalid body tenant camera".into());
        }
        let lights = LightBlock {
            ambient: tenant::Ambient {
                color: [1.0; 3],
                intensity: self.light,
            },
            ..Default::default()
        };
        if !self.inner.set_lights(&lights) {
            return Err("invalid body tenant lighting".into());
        }
        self.inner.set_depth_prepass(Some(layer.depth_texture()));
        let report = self
            .inner
            .encode(encoder, &self.target)
            .map_err(|e| format!("{e:?}"))?;
        layer.stats.mesh_builds = builds;
        layer.stats.mesh_upload_bytes = bytes;
        layer.stats.draw_parts = parts;
        layer.stats.body_tenant_internal_submissions = report.internal_submissions;
        layer.stats.body_upload_measurement =
            Some("mesh estimates; tenant frame uploads not exposed");
        if report.internal_submissions != 0 {
            return Err("body tenant submitted outside the caller encoder".into());
        }
        Ok(())
    }
}

fn upload_bytes(mesh: &PaletteMesh, palette: &Palette) -> u64 {
    // kiss3d uploads position, index, face normal, UV and the palette texture.
    (mesh.positions.len() * (12 + 4 + 12 + 8) + palette.colors.len() * 4) as u64
}

fn target(device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mesocosm lit bodies"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[FORMAT.remove_srgb_suffix()],
    })
}
