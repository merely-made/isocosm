// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The body half: mere's lit tenant drawing the room and the body into a
//! texture netrender composites (wing rulings 471, 734 to 736, L7).

use super::MeshVertex;
use netrender::WgpuHandles;
use tenant::{BodyId, FrameReport, HostDevice, LightBlock, PaletteMesh, Pose};

/// The producer path the frame-health and opaque-tenant receipts name.
pub const PRODUCER_PATH: &str = "tenant::Tenant::encode (opaque)";
/// The tenant's colour target. sRGB, listing its linear view, because the
/// tenant's tonemap encodes gamma itself and draws through the linear view.
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// Underground dark, not black: the room reads as a place with air in it.
pub const ROOM_BACKGROUND: [f32; 4] = [0.03, 0.03, 0.045, 1.0];

/// The room and the body, drawn into a texture netrender will composite.
pub struct Tenant {
    pub view: wgpu::TextureView,
    target: wgpu::Texture,
    size: [u32; 2],
    queue: wgpu::Queue,
    device: wgpu::Device,
    inner: tenant::Tenant,
    room: Option<BodyId>,
    body: Option<BodyId>,
}

impl Tenant {
    pub fn new(handles: &WgpuHandles, size: [u32; 2]) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("room tenant target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[TARGET_FORMAT.remove_srgb_suffix()],
        });
        let view = target.create_view(&Default::default());
        let host = HostDevice {
            instance: handles.instance.clone(),
            adapter: handles.adapter.clone(),
            device: handles.device.clone(),
            queue: handles.queue.clone(),
        };
        let mut inner = tenant::Tenant::new(&host, TARGET_FORMAT, size);
        inner.set_background(ROOM_BACKGROUND);
        Self {
            view,
            target,
            size,
            queue: handles.queue.clone(),
            device: handles.device.clone(),
            inner,
            room: None,
            body: None,
        }
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    /// The physical colour target, borrowed by netrender's opaque tenant.
    pub fn target_texture(&self) -> &wgpu::Texture {
        &self.target
    }

    pub fn set_room(&mut self, vertices: &[MeshVertex]) {
        self.room = Some(self.put(self.room, vertices));
    }

    pub fn set_body(&mut self, vertices: &[MeshVertex]) {
        self.body = Some(self.put(self.body, vertices));
    }

    /// Lights the next frames; the torch is `scene::torch`.
    pub fn light(&mut self, block: &LightBlock) {
        assert!(self.inner.set_lights(block), "invalid light block");
    }

    pub fn look(&mut self, camera: &crate::scene::Camera) {
        assert!(self.inner.set_camera(&camera.tenant()), "invalid camera");
    }

    pub fn set_background(&mut self, rgba: [f32; 4]) {
        self.inner.set_background(rgba);
    }

    /// Starts each frame from a tracer's depth (D1), or clears with `None`.
    pub fn set_depth_prepass(&mut self, depth: Option<&wgpu::Texture>) {
        self.inner.set_depth_prepass(depth);
    }

    /// Draws one frame into the tenant texture, recorded into one encoder
    /// this submits.
    pub fn draw(&mut self) -> FrameReport {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("eponym body tenant frame"),
            });
        let report = self
            .inner
            .encode(&mut encoder, &self.target)
            .expect("tenant frame");
        self.queue.submit(std::iter::once(encoder.finish()));
        report
    }

    fn put(&mut self, slot: Option<BodyId>, vertices: &[MeshVertex]) -> BodyId {
        let positions = vertices.iter().map(|v| v.position).collect();
        let colors: Vec<[f32; 4]> = vertices
            .iter()
            .map(|v| [v.color[0], v.color[1], v.color[2], 1.0])
            .collect();
        let (mesh, palette) = PaletteMesh::from_colored(positions, &colors);
        match slot {
            Some(id) => {
                self.inner
                    .set_mesh(id, &mesh, &palette)
                    .expect("mesher triangles");
                id
            },
            None => self
                .inner
                .add_mesh(&mesh, &palette, Pose::default())
                .expect("mesher triangles"),
        }
    }
}
