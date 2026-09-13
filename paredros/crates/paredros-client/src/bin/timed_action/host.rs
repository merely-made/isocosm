// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{App, Live, view::AppView};
use netrender::{Compositor, PresentedFrame, Scene, SurfaceKey};
use paredros_client::body_sheet::{Hud, LOGICAL_SIZE};
use paredros_client::gpu::{self, Composer};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("Paredros: Timed Action")
                    .with_inner_size(PhysicalSize::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1])),
            )
            .unwrap(),
        );
        let surface = self.instance.create_surface(window.clone()).unwrap();
        let handles = gpu::boot(&self.instance, Some(&surface));
        let caps = surface.get_capabilities(&handles.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let mut live = Live {
            window,
            surface,
            format,
            device,
            queue,
            composer: Composer::new(handles, LOGICAL_SIZE),
            hud: Hud::new().unwrap(),
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } if !repeat => self.dispatch(code, state == ElementState::Pressed, el),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    if self.action.action().is_none() {
                        self.prepare(self.attack_direction);
                    }
                    self.held = true;
                    self.last_charge = Instant::now();
                } else {
                    self.release();
                }
            },
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            },
            WindowEvent::RedrawRequested => self.frame(el),
            WindowEvent::Focused(false) => {
                self.movement_keys = [false; 4];
                self.held = false;
            },
            WindowEvent::CloseRequested => el.exit(),
            _ => {},
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        el.set_control_flow(if self.held || self.smoke || self.motion_running() {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16))
        } else {
            ControlFlow::Wait
        });
        if self.smoke && self.opened_at.elapsed() > Duration::from_secs(30) {
            panic!("native smoke did not present within 30 seconds");
        }
        self.tick();
    }
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
struct Grab {
    master: Option<wgpu::Texture>,
}
impl Compositor for Grab {
    fn declare_surface(&mut self, _: SurfaceKey, _: [f32; 4]) {}
    fn destroy_surface(&mut self, _: SurfaceKey) {}
    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}
fn render(composer: &Composer, scene: &Scene) -> wgpu::Texture {
    let mut grab = Grab { master: None };
    composer.net.render_with_compositor(
        scene,
        paredros_client::gpu::MASTER_FORMAT,
        &mut grab,
        netrender::peniko::Color::new([0., 0., 0., 1.]),
    );
    grab.master.expect("netrender presented no master")
}
impl App {
    fn frame(&mut self, el: &ActiveEventLoop) {
        let lines = self.display_lines();
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let scene = live.hud.timed_scene("PAREDROS / TIMED ACTION", &lines);
        let master = render(&live.composer, &scene);
        if self.smoke {
            let capture = live.composer.capture(&master);
            assert!(!capture.is_trivial(), "timed action frame is trivial");
            let path = self.save_path.with_extension("png");
            capture.write_png(&path).expect("timed action capture");
            println!("timed action frame: {}", path.display());
        }
        if let wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) = live.surface.get_current_texture()
        {
            let target = frame.texture.create_view(&Default::default());
            let size = live.window.inner_size();
            live.composer.present(
                &master,
                &target,
                live.format,
                [size.width.max(1), size.height.max(1)],
            );
            live.window.pre_present_notify();
            live.queue.present(frame);
            self.presented = true;
            if self.smoke {
                el.exit();
            }
        }
    }
}
