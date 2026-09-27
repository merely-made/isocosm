// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1b: the headed stable-residency receipt.
//!
//! The V1 zoom-and-travel trace runs over one capacity-fixed brick cache.
//! Every band transition and travel retargets the cache in place: retained
//! bricks keep their atlas slots, the kilobyte-scale pointer volume moves,
//! and only loaded bricks' 512-byte slots upload. The receipt requires zero
//! texture or bind-group creation at every transition after the first
//! frame, exact per-transition upload accounting, retained one-frame
//! recovery, and records wgpu's allocator report beside the logical bytes.

use std::{sync::Arc, time::Instant};

#[path = "v1b_residency/receipt.rs"]
mod receipt;
use receipt::{FrameSample, report};

use eponym_client::{
    gpu::{self, Composer, SIZE},
    residency::{
        RESIDENT_BUDGET_BYTES, ResidencyMetrics, ResidencyScene, StableResidency, V1_FRAMES,
        visible_range, zoom_distance,
    },
    scene,
};
use isometer::core::ground::BRICK;
use isometer::lens::{
    BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTracer, Grade,
};
use modulus::{BrickMap, BrickProjectionRevision};
use netrender::WgpuHandles;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\v1b_residency.png";
const RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\v1b_residency.json";
const TRAVEL_FRAME: u64 = 72;

fn main() {
    let scene = ResidencyScene::grow();
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = StableApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        scene,
        live: None,
        samples: Vec::with_capacity(V1_FRAMES as usize),
        allocator_after_first: None,
    };
    event_loop.run_app(&mut app).expect("V1b headed run");
}

/// The tracer half, borrowing the policy-owned map per frame rather than
/// holding one: the stable cache has exactly one owner.
struct StableTenant {
    pub view: wgpu::TextureView,
    _target: wgpu::Texture,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    grade: Grade,
}

impl StableTenant {
    fn new(handles: &WgpuHandles, size: [u32; 2]) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Eponym V1b DDA target"),
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
        Self {
            view: target.create_view(&Default::default()),
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

    fn draw(
        &mut self,
        map: &BrickMap,
        revision: BrickRevision,
        camera: isometer::lens::TraceCamera,
        pose: &isometer::lens::CritterPose,
        loaded_slots: &[u32],
    ) -> Result<BrickDiagnostics, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Eponym V1b frame"),
            });
        let diagnostics = self
            .tracer
            .encode(
                &mut encoder,
                &self.view,
                BrickFrameInput::for_camera(map, revision, camera, &self.grade)
                    .with_pose(pose)
                    .changed(BrickChange::Slots(loaded_slots)),
            )
            .map_err(|error| error.to_string())?;
        self.queue.submit([encoder.finish()]);
        Ok(diagnostics)
    }
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tenant: StableTenant,
    composer: Composer,
    adapter: String,
    current: ResidencyMetrics,
    stable: StableResidency,
}

struct StableApp {
    instance: wgpu::Instance,
    scene: ResidencyScene,
    live: Option<Live>,
    samples: Vec<FrameSample>,
    allocator_after_first: Option<u64>,
}

impl ApplicationHandler for StableApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Eponym V1b: stable resident brick cache")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("V1b window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("V1b surface");
        let handles = gpu::boot(&self.instance, Some(&surface));
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let capabilities = surface.get_capabilities(&handles.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .unwrap_or(capabilities.formats[0]);
        let adapter = handles.adapter.get_info().name;
        let tenant = StableTenant::new(&handles, SIZE);
        let limits = tenant.tracer.atlas_limits();
        let stable = StableResidency::with_limits(&self.scene, limits)
            .expect("the stable cache fits the device and V1 budget");
        println!(
            "V1b world: {} exact bricks, cache capacity {}, fixed {} bytes, device 3D limit {}, focus {:?}",
            self.scene.ground.brick_count(),
            stable.capacity(),
            stable.resident_bytes(),
            limits.max_texture_dimension_3d,
            self.scene.focus
        );
        let composer = Composer::new(handles, SIZE);
        let mut live = Live {
            window,
            surface,
            format,
            device,
            queue,
            tenant,
            composer,
            adapter,
            stable,
            current: ResidencyMetrics {
                projection_revision: BrickProjectionRevision(0),
                visible_range: 0,
                resident_range: 0,
                loaded_bricks: 0,
                evicted_bricks: 0,
                resident_bricks: 0,
                pointer_bytes: 0,
                atlas_bytes: 0,
                resident_bytes: 0,
            },
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            },
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {},
        }
    }
}

impl StableApp {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let frame = self.samples.len() as u64;
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let began = Instant::now();
        let distance = zoom_distance(frame);
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let visible = visible_range(distance, aspect);
        let travel = frame == TRAVEL_FRAME;
        if travel {
            assert!(
                self.scene.move_focus_x(BRICK),
                "the V1b travel receipt needs a valid destination stance"
            );
        }

        let page_began = Instant::now();
        let outcome = live
            .stable
            .prepare(&self.scene, visible)
            .expect("camera range has a V1b page");
        let page_prepare_us = page_began.elapsed().as_micros() as u64;
        let page_transition = outcome.is_some();
        assert!(
            !travel || page_transition,
            "same-band travel must retarget the stable cache"
        );
        let loaded_slots = outcome
            .as_ref()
            .map(|outcome| outcome.delta.loaded_slots.clone())
            .unwrap_or_default();
        if let Some(outcome) = outcome {
            live.current = outcome.metrics;
        }

        let camera = self.scene.camera(distance, aspect);
        let pose = scene::body_pose(self.scene.focus);
        let diagnostics = live
            .tenant
            .draw(
                live.stable.map(),
                BrickRevision(self.scene.ground.revision()),
                camera,
                &pose,
                &loaded_slots,
            )
            .expect("V1b DDA frame");

        // The stable claim, held every frame after the first: nothing is
        // ever created again, and a transition uploads exactly the pointer
        // volume plus its loaded slots.
        if frame > 0 {
            assert_eq!(diagnostics.resource_creations, 0, "frame {frame}");
            assert_eq!(diagnostics.bind_group_rebuilds, 0, "frame {frame}");
            assert!(!diagnostics.map_recreated, "frame {frame}");
            if page_transition {
                assert!(diagnostics.projection_replaced, "frame {frame}");
                let pointer_bytes = std::mem::size_of_val(live.stable.map().pointers()) as u64;
                assert_eq!(
                    diagnostics.brick_upload_bytes,
                    pointer_bytes + loaded_slots.len() as u64 * 512,
                    "frame {frame} must upload pointers plus its loaded slots"
                );
            } else {
                assert_eq!(diagnostics.brick_upload_bytes, 0, "frame {frame}");
            }
        }

        let chrome = scene::chrome(SIZE, frame as usize + 1, V1_FRAMES as usize);
        let master = live.composer.compose(&chrome, &live.tenant.view);
        let size = live.window.inner_size();
        use wgpu::CurrentSurfaceTexture as Acquired;
        match live.surface.get_current_texture() {
            Acquired::Success(surface_frame) | Acquired::Suboptimal(surface_frame) => {
                let target = surface_frame.texture.create_view(&Default::default());
                live.composer.present(
                    &master,
                    &target,
                    live.format,
                    [size.width.max(1), size.height.max(1)],
                );
                live.window.pre_present_notify();
                live.queue.present(surface_frame);
            },
            Acquired::Outdated | Acquired::Lost => {
                configure(live);
                live.window.request_redraw();
                return;
            },
            Acquired::Timeout | Acquired::Occluded => {
                live.window.request_redraw();
                return;
            },
            Acquired::Validation => panic!("V1b surface acquisition failed validation"),
        }

        if frame == 0 {
            self.allocator_after_first = allocated_bytes(&live.device);
        }
        let sample = FrameSample {
            frame,
            projection_revision: live.current.projection_revision.0,
            camera_distance: distance,
            visible_range: visible,
            resident_range: live.current.resident_range,
            page_transition,
            page_prepare_us,
            loaded_bricks: loaded_slots.len(),
            evicted_bricks: if page_transition {
                live.current.evicted_bricks
            } else {
                0
            },
            resident_bricks: live.current.resident_bricks,
            brick_upload_bytes: diagnostics.brick_upload_bytes,
            tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
            resource_creations: diagnostics.resource_creations,
            bind_group_rebuilds: diagnostics.bind_group_rebuilds,
            projection_replaced: diagnostics.projection_replaced,
            frame_us: began.elapsed().as_micros() as u64,
        };
        if sample.page_transition {
            println!(
                "retarget @{:02}: projection {}, visible {} -> range {}, {} resident, +{} -{}, upload {} bytes, prepare {} us, frame {} us",
                sample.frame,
                sample.projection_revision,
                sample.visible_range,
                sample.resident_range,
                sample.resident_bricks,
                sample.loaded_bricks,
                sample.evicted_bricks,
                sample.brick_upload_bytes,
                sample.page_prepare_us,
                sample.frame_us,
            );
        }
        self.samples.push(sample);

        if self.samples.len() == V1_FRAMES as usize {
            let allocator_after_last = allocated_bytes(&live.device);
            report(
                &live.composer,
                &master,
                &live.adapter,
                &self.scene,
                &live.stable,
                &self.samples,
                self.allocator_after_first,
                allocator_after_last,
                live.device.limits().max_texture_dimension_3d,
            );
            event_loop.exit();
        } else {
            live.window.request_redraw();
        }
    }
}

fn allocated_bytes(device: &wgpu::Device) -> Option<u64> {
    device
        .generate_allocator_report()
        .map(|report| report.total_allocated_bytes)
}

fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}
