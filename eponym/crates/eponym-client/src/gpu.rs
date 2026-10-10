// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The picture: mere's body tenant on netrender's device.
//!
//! One `wgpu::Device` serves both. The tenant draws the room into a texture
//! it owns; netrender renders the chrome into its master and composites that
//! texture in at scene-op boundary zero, so the chrome lands over the room
//! rather than under it. Nothing here creates a second device, which is the
//! whole point of the cohesion contract this gate is proving.

#[cfg(feature = "r1-proof")]
use isometer::core::ground::Ground;
#[cfg(feature = "r1-proof")]
use isometer::lens::{
    BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTracer, CritterPose, Grade, TraceCamera,
};
use isometer::render::geometry::Vertex as MeshVertex;
#[cfg(feature = "r1-proof")]
use modulus::BrickMap;
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Renderer, Scene, SurfaceKey, WgpuHandles, create_netrender_instance,
};
use tenant::{DeviceNeeds, FrameReport};
mod body;
pub use body::{PRODUCER_PATH, ROOM_BACKGROUND, TARGET_FORMAT, Tenant};

/// The composed frame's size. Fixed, so the receipt is the same picture on
/// every machine.
pub const SIZE: [u32; 2] = [1280, 720];
/// The master texture's format. Non-srgb so a readback is already the bytes
/// a PNG wants.
pub const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The brick layout receipt shared by the Mesocosm and Eponym profiles.
#[cfg(feature = "r1-proof")]
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct BrickAbi {
    pub origin: [i16; 3],
    pub pointer_extent: [u32; 3],
    pub atlas_extent: [u32; 3],
    pub pointer_bytes: usize,
    pub atlas_bytes: usize,
}

/// What the body tenant wants from the shared device, in netrender's terms.
/// Netrender's own requirement and limit minimum are not stated here: they
/// belong to netrender and it applies them.
fn tenant_needs() -> netrender::TenantNeeds {
    let needs = DeviceNeeds::tenant();
    netrender::TenantNeeds {
        required_features: needs.required_features,
        optional_features: needs.optional_features,
        limits: needs.limits,
        label: Some("eponym room"),
        ..Default::default()
    }
}

/// Boots one device for both tenants (netrender owns the boot, R4).
pub fn boot(instance: &wgpu::Instance, compatible: Option<&wgpu::Surface<'_>>) -> WgpuHandles {
    netrender::boot_on(instance.clone(), compatible, &tenant_needs())
        .expect("no wgpu device able to host both netrender and the body tenant")
}

/// The R1 terrain tenant: Eponym camera policy over Mesocosm's existing
/// brick ABI and DDA implementation.
///
/// This is deliberately adjacent to the retained S0 body tenant. R1
/// proves shared traversal; it does not silently replace the room receipt or
/// pretend the later hybrid depth join has landed.
#[cfg(feature = "r1-proof")]
pub struct DdaTenant {
    pub view: wgpu::TextureView,
    _target: wgpu::Texture,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    map: BrickMap,
    revision: BrickRevision,
    grade: Grade,
}

#[cfg(feature = "r1-proof")]
impl DdaTenant {
    pub fn new(handles: &WgpuHandles, ground: &Ground, size: [u32; 2]) -> Result<Self, String> {
        let map = crate::brick::from_ground(ground).map_err(|error| error.to_string())?;
        Ok(Self::from_map(
            handles,
            map,
            BrickRevision(ground.revision()),
            size,
        ))
    }

    /// Starts the tracer over a caller-selected exact brick working set.
    ///
    /// Mesocosm still owns the allocation and traversal. The caller owns the
    /// camera-driven selection policy and may replace the projection below.
    pub fn from_map(
        handles: &WgpuHandles,
        map: BrickMap,
        revision: BrickRevision,
        size: [u32; 2],
    ) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Eponym R1 DDA target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        Self {
            view,
            _target: target,
            device: handles.device.clone(),
            queue: handles.queue.clone(),
            tracer: BrickTracer::with_format(
                handles.device.clone(),
                handles.queue.clone(),
                size[0],
                size[1],
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            map,
            revision,
            grade: Grade {
                fog: [0.03, 0.03, 0.045],
                fog_start: 0.62,
                palette_len: 0,
                dither: 0.0,
                fog_bands: 0.0,
                downscale: 1,
            },
        }
    }

    /// Publishes a newer source revision, selected projection, or both.
    ///
    /// The source revision may advance while the selected keys remain stable.
    /// The product-owned projection revision must advance when selection or
    /// slot assignment changes. Regressing either identity is refused.
    pub fn replace_map(
        &mut self,
        map: BrickMap,
        revision: BrickRevision,
    ) -> Result<(), &'static str> {
        let current_projection = self.map.projection_revision().0;
        let next_projection = map.projection_revision().0;
        if revision.0 < self.revision.0 {
            return Err("replacement map regresses its Ground revision");
        }
        if next_projection < current_projection {
            return Err("replacement map regresses its projection revision");
        }
        if revision == self.revision && next_projection == current_projection {
            return Err("replacement map advances neither source nor projection");
        }
        self.map = map;
        self.revision = revision;
        Ok(())
    }

    pub fn draw(
        &mut self,
        camera: TraceCamera,
        pose: &CritterPose,
    ) -> Result<BrickDiagnostics, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Eponym R1 DDA frame"),
            });
        let diagnostics = self
            .tracer
            .encode(
                &mut encoder,
                &self.view,
                BrickFrameInput::for_camera(&self.map, self.revision, camera, &self.grade)
                    .with_pose(pose),
            )
            .map_err(|error| error.to_string())?;
        self.queue.submit([encoder.finish()]);
        Ok(diagnostics)
    }

    pub fn abi(&self) -> BrickAbi {
        BrickAbi {
            origin: self.map.origin(),
            pointer_extent: self.map.pointer_extent(),
            atlas_extent: self.map.atlas_extent(),
            pointer_bytes: std::mem::size_of_val(self.map.pointers()),
            atlas_bytes: self.map.atlas().len(),
        }
    }
}

/// The D1 join, tracer first: the shared brick traversal draws the room's
/// rock into its own colour and depth, and that depth is the body tenant's
/// pre-pass, so bodies draw only where nearer and netrender layers them
/// over the traced colour (L3 interleave, wing ruling 733).
#[cfg(feature = "d1-proof")]
pub struct JoinTenant {
    pub view: wgpu::TextureView,
    _colour: wgpu::Texture,
    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    map: BrickMap,
    revision: BrickRevision,
    grade: Grade,
}

#[cfg(feature = "d1-proof")]
impl JoinTenant {
    const COLOUR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    pub fn new(handles: &WgpuHandles, ground: &Ground, size: [u32; 2]) -> Result<Self, String> {
        let map = crate::brick::from_ground(ground).map_err(|error| error.to_string())?;
        let texture = |label, format, usage| {
            handles.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: size[0],
                    height: size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | usage,
                view_formats: &[],
            })
        };
        let colour = texture(
            "Eponym D1 traced colour",
            Self::COLOUR,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = texture(
            "Eponym D1 traced depth",
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        Ok(Self {
            view: colour.create_view(&Default::default()),
            _colour: colour,
            depth_view: depth.create_view(&Default::default()),
            depth,
            device: handles.device.clone(),
            queue: handles.queue.clone(),
            tracer: BrickTracer::with_format(
                handles.device.clone(),
                handles.queue.clone(),
                size[0],
                size[1],
                Self::COLOUR,
            ),
            map,
            revision: BrickRevision(ground.revision()),
            grade: Grade {
                fog: [0.03, 0.03, 0.045],
                fog_start: 0.62,
                palette_len: 0,
                dither: 0.0,
                fog_bands: 0.0,
                downscale: 1,
            },
        })
    }

    /// The traced depth, the body tenant's pre-pass.
    pub fn depth(&self) -> &wgpu::Texture {
        &self.depth
    }

    /// Traces one frame into cleared colour and depth. `clip_from_world`
    /// must be the matrix the body tenant projects with, or the two
    /// pictures disagree about where surfaces sit.
    pub fn draw(
        &mut self,
        camera: TraceCamera,
        clip_from_world: [[f32; 4]; 4],
    ) -> Result<BrickDiagnostics, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Eponym D1 traced pre-pass"),
            });
        let [r, g, b, a] = ROOM_BACKGROUND.map(f64::from);
        drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Eponym D1 clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
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
        }));
        let diagnostics = self
            .tracer
            .encode_with_depth(
                &mut encoder,
                &self.view,
                &self.depth_view,
                BrickFrameInput::for_camera(&self.map, self.revision, camera, &self.grade)
                    .with_clip_from_world(clip_from_world),
            )
            .map_err(|error| error.to_string())?;
        self.queue.submit([encoder.finish()]);
        Ok(diagnostics)
    }

    pub fn abi(&self) -> BrickAbi {
        BrickAbi {
            origin: self.map.origin(),
            pointer_extent: self.map.pointer_extent(),
            atlas_extent: self.map.atlas_extent(),
            pointer_bytes: std::mem::size_of_val(self.map.pointers()),
            atlas_bytes: self.map.atlas().len(),
        }
    }
}

/// The netrender half: chrome into the master, tenant texture composited in
/// under it, and the master handed back.
pub struct Composer {
    pub net: Renderer,
    size: [u32; 2],
}

impl Composer {
    pub fn new(handles: WgpuHandles, size: [u32; 2]) -> Self {
        let net = create_netrender_instance(
            handles,
            NetrenderOptions {
                tile_cache_size: Some(64),
                enable_vello: true,
                ..Default::default()
            },
        )
        .expect("netrender init");
        Self { net, size }
    }

    /// One composed master: chrome over room.
    pub fn compose(&self, chrome: &Scene, tenant: &wgpu::TextureView) -> wgpu::Texture {
        self.compose_layers(chrome, &[tenant])
    }

    /// Chrome over `layers`, each over the one before it (D1: traced rock,
    /// then bodies whose background is transparent).
    pub fn compose_layers(&self, chrome: &Scene, layers: &[&wgpu::TextureView]) -> wgpu::Texture {
        let place =
            ExternalTexturePlacement::new([0.0, 0.0, self.size[0] as f32, self.size[1] as f32]);
        // Boundary zero: the room goes under every chrome op, so the bar
        // paints over the picture instead of being hidden by it.
        let external: Vec<_> = layers
            .iter()
            .map(|view| ExternalTextureComposite::new(view, place).with_scene_op_boundary(0))
            .collect();
        let mut grab = MasterGrab { master: None };
        self.net.render_with_compositor_and_external_textures(
            chrome,
            MASTER_FORMAT,
            &mut grab,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        grab.master.expect("netrender presented no master")
    }

    /// Compose the normal body-tenant room through Netrender's RG3a opaque
    /// tenant graph task. The room is placed at scene-op boundary zero, so
    /// the chrome paints over it; the tenant's internal render remains closed
    /// behind this borrowed target descriptor.
    pub fn compose_opaque_tenant(
        &self,
        chrome: &Scene,
        tenant: &Tenant,
        tenant_report: FrameReport,
    ) -> (wgpu::Texture, netrender::OpaqueTenantReceipt) {
        assert_eq!(tenant_report.internal_submissions, 0);
        let input = netrender::OpaqueTenantInput::new(
            tenant.target_texture(),
            netrender::OpaqueTenantMetadata::new(
                "eponym-client",
                PRODUCER_PATH,
                0,
                0,
                ExternalTexturePlacement::new([0.0, 0.0, self.size[0] as f32, self.size[1] as f32]),
            )
            .with_reported_physical_submission_count(1),
        );
        let mut grab = MasterGrab { master: None };
        let receipt = self.net.render_with_opaque_tenant(
            chrome,
            MASTER_FORMAT,
            &mut grab,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &input,
        );
        (grab.master.expect("netrender presented no master"), receipt)
    }

    /// Blits the composed master onto a surface texture, filling it. The
    /// window may be any size; the receipt is always [`SIZE`].
    pub fn present(
        &self,
        master: &wgpu::Texture,
        target: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        target_size: [u32; 2],
    ) {
        let view = master.create_view(&Default::default());
        self.net.compose_external_texture(
            &view,
            target,
            format,
            target_size[0],
            target_size[1],
            ExternalTexturePlacement::new([0.0, 0.0, target_size[0] as f32, target_size[1] as f32]),
        );
    }

    pub fn capture(&self, master: &wgpu::Texture) -> Capture {
        Capture::of(
            self.net
                .wgpu_device
                .read_rgba8_texture(master, self.size[0], self.size[1]),
            self.size,
        )
    }
}

struct MasterGrab {
    master: Option<wgpu::Texture>,
}

impl Compositor for MasterGrab {
    fn declare_surface(&mut self, _key: SurfaceKey, _bounds: [f32; 4]) {}
    fn destroy_surface(&mut self, _key: SurfaceKey) {}
    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}

/// A read-back frame, with the one property that separates a receipt from a
/// blank: how many distinct colours are actually in it.
pub struct Capture {
    pub pixels: Vec<u8>,
    pub size: [u32; 2],
    pub distinct: usize,
}

impl Capture {
    fn of(pixels: Vec<u8>, size: [u32; 2]) -> Self {
        let mut seen = std::collections::HashSet::new();
        for pixel in pixels.chunks_exact(4) {
            seen.insert([pixel[0], pixel[1], pixel[2]]);
        }
        Self {
            pixels,
            size,
            distinct: seen.len(),
        }
    }

    /// A frame of one flat colour is a failure that a written file hides.
    /// The threshold is deliberately low: this catches "nothing rendered",
    /// not "the picture is ugly".
    pub fn is_trivial(&self) -> bool {
        self.distinct < 16
    }

    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let mut encoder = png::Encoder::new(file, self.size[0], self.size[1]);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut writer| writer.write_image_data(&self.pixels))
            .map_err(std::io::Error::other)
    }
}
