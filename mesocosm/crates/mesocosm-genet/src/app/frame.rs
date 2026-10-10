// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One frame: step the run, read the played site, draw the section and the
//! chrome lanes over it.

use super::*;
use crate::section::SectionFrame;

impl Host {
    pub(super) fn frame(&mut self, event_loop: &ActiveEventLoop) {
        self.advance();
        // What the runtime answered, as the strings `assert event` matches.
        self.note_outcomes();
        let rebound = self.refresh_scene();
        self.update_follow();
        self.update_inspection();

        let at = self.follow_at();
        let half = section::half_height_or_default(self.config.slab_half_height);
        let centre = section::centre_on(at, self.pan, half, self.config.camera);
        let checkpoint = self.runtime.checkpoint().cloned();
        let review = self.runtime.review().cloned();
        let proposed = self.runtime.proposed().to_vec();
        if let Some(review) = &review {
            self.board_row = self.board_row.min(review.offers.len().saturating_sub(1));
        }
        let board_row = self.board_row;
        let trend = self.runtime.trend();
        let fresh = self.runtime.trace()[self.shown.min(self.runtime.trace().len())..].to_vec();
        self.shown = self.runtime.trace().len();
        let dev = self.dev_reading();
        let focused = self.config.dev.then(|| self.followed()).flatten();
        let (steps, critter) = (self.steps, self.runtime.critter());

        let Some(scene) = &self.scene else { return };
        let Some(gpu) = &mut self.gpu else { return };
        if rebound && let Err(error) = gpu.section.set_ground(&scene.ground) {
            eprintln!("section: {error}");
        }
        gpu.section.set_half_height(half);
        gpu.section
            .configure_bodies(self.config.body_mode, self.config.body_budget);
        gpu.section
            .configure_terrain(self.config.terrain_style, centre, at[1] as f32);
        gpu.section
            .set_body_focus(focused, self.inspection.selected);
        let (pose, dropped) = match section::pose_of(scene, 1.0) {
            Some((pose, dropped)) => (Some(pose), dropped),
            None => (None, 0),
        };
        self.body_capsules_dropped = dropped;
        let roster = section::roster_of(scene, gpu.section.slab_window(centre), 1.0);

        let surface_texture = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                return;
            },
            _ => return,
        };
        let view = surface_texture.texture.create_view(&Default::default());
        let section_frame = SectionFrame {
            scene,
            dirty: &[],
            centre,
            pose: pose.as_ref(),
            roster: &roster,
        };
        let sim = self.runtime.sim();
        if let Some(lanes) = &mut gpu.chrome {
            let frame = (gpu.config.width, gpu.config.height);
            // The section is one closed tenant producer; submit it before
            // Netrender imports its display texture.
            let mut tenant = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesocosm section tenant"),
                });
            if let Err(error) = gpu.section.render(&mut tenant, section_frame) {
                eprintln!("section: {error}");
            }
            gpu.queue.submit(Some(tenant.finish()));

            lanes.hud.refresh(&lanes.device, sim, critter);
            lanes
                .vitals
                .refresh(&lanes.device, sim, critter, &fresh, steps, &trend);
            lanes
                .board
                .refresh(&lanes.device, review.as_ref(), &proposed, &trend, board_row);
            let held = checkpoint.as_ref().filter(|_| !lanes.board.standing());
            lanes.checkpoint.refresh(&lanes.device, sim, held);

            let framed = lanes.device.frame_master(
                gpu.section.display_texture(),
                frame,
                gpu.section.body_stats().fallback_bodies as u64,
                gpu.section.body_view().as_ref(),
            );
            gpu.last_tenant_receipt = Some(framed.receipt);
            let master = framed.texture.create_view(&Default::default());
            let mut chrome = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesocosm chrome into master"),
                });
            lanes.composite(&mut chrome, &master, frame, dev.as_ref());
            gpu.queue.submit(Some(chrome.finish()));

            let mut present = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mesocosm master to surface"),
                });
            lanes
                .device
                .draw_surface(&mut present, &view, &master, frame);
            gpu.queue.submit(Some(present.finish()));
        } else {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("chromeless frame"),
                });
            if let Err(error) = gpu.section.draw(&mut encoder, &view, section_frame) {
                eprintln!("section: {error}");
            }
            gpu.queue.submit(Some(encoder.finish()));
        }
        gpu.queue.present(surface_texture);

        self.frames += 1;
        let hit_limit = self.config.frames.is_some_and(|limit| self.frames >= limit);
        // A scenario decides when its own run ends, after the frame it
        // asserts about was drawn (DT4).
        if self.scenario.is_some() {
            self.drive_scenario(event_loop, hit_limit);
        } else if hit_limit {
            self.finish(event_loop);
        }
    }
}

impl Lanes {
    /// Every standing lane over the master: the corner readings, the dev
    /// tile under `--dev`, then whichever question stands.
    pub(super) fn composite(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        master: &wgpu::TextureView,
        frame: (u32, u32),
        dev: Option<&mesocosm_views::Dev>,
    ) {
        self.hud.composite(&self.device, encoder, master, frame);
        self.vitals.composite(&self.device, encoder, master, frame);
        if let Some(dev) = dev {
            self.dev.refresh(&self.device, dev);
            self.dev.composite(&self.device, encoder, master, frame);
        }
        self.checkpoint
            .composite(&self.device, encoder, master, frame);
        self.board.composite(&self.device, encoder, master, frame);
    }
}
