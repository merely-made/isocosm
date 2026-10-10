// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the host writes on the way out.
//!
//! Every exit (the window closing, Escape, a frame limit, a scenario's end)
//! comes through [`Host::finish`], which writes the capture, the session's
//! save as the trace, and the receipt.

use winit::event_loop::ActiveEventLoop;

use super::Host;
use crate::played::{self, FrameGraphReceipt, PartSelectionReceipt, PlayedReceipt};

impl Host {
    /// Writes what the run leaves behind, once, and stops the loop.
    pub(super) fn finish(&mut self, event_loop: &ActiveEventLoop) {
        if !self.finished {
            self.finished = true;
            self.write_capture();
            self.write_trace();
            self.write_receipt();
        }
        event_loop.exit();
    }

    /// The frame the player was last looking at, chrome included, at the path
    /// the run was told to write.
    fn write_capture(&self) {
        let Some(path) = self.config.capture.clone() else {
            return;
        };
        if let Err(error) = self.capture_to(&path) {
            eprintln!("capture: {error}");
        }
    }

    /// The same frame, at any path. **The one capture path** — a scenario's
    /// `capture <name>` verb comes through here too (DT4), so a screenshot a
    /// script asks for is byte-for-byte the one the receipt writes rather than
    /// a second read-back that could composite a different set of lanes.
    pub(crate) fn capture_to(&self, path: &std::path::Path) -> Result<(), String> {
        let Some(gpu) = &self.gpu else {
            return Err("there is no device to read a frame back from".into());
        };
        let frame = (gpu.config.width, gpu.config.height);
        let shot = if let Some(lanes) = &gpu.chrome {
            let master = lanes.device.frame_master(
                gpu.section.display_texture(),
                frame,
                gpu.section.body_stats().fallback_bodies as u64,
            );
            let master_view = master.texture.create_view(&Default::default());
            let mut encoder =
                lanes
                    .device
                    .device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("mesocosm captured chrome into master"),
                    });
            lanes
                .hud
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes
                .vitals
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            if self.config.dev {
                lanes
                    .dev
                    .composite(&lanes.device, &mut encoder, &master_view, frame);
            }
            lanes
                .checkpoint
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes
                .board
                .composite(&lanes.device, &mut encoder, &master_view, frame);
            lanes.device.queue().submit(Some(encoder.finish()));
            gpu.section.capture_from(&master.texture, |_, _, _| {})
        } else {
            gpu.section.capture(|_, _, _| {})
        };
        let Some((width, height, pixels)) = shot else {
            return Err("the section could not be read back".into());
        };
        played::write_png(path, width, height, &pixels)
    }

    /// The session's save, which `--replay` reads back.
    fn write_trace(&self) {
        let Some(path) = &self.config.trace else {
            return;
        };
        if let Err(error) = played::write_json(path, &self.runtime.save()) {
            eprintln!("trace: {error}");
        }
    }

    fn write_receipt(&mut self) {
        let receipt = self.receipt();
        println!(
            "{}{} {} steps over {} frames, hash {:016x}",
            // **The label goes first, where it cannot be read past** (DT3).
            // A run that ended an epoch, forced a birth, killed something or
            // placed matter is not an unaided playtest, and the line that
            // reports it should not be able to be skimmed as one.
            match played::assisted_label(receipt.dev_intents) {
                label if label.is_empty() => label,
                label => format!("{label} "),
            },
            receipt.mode,
            receipt.steps,
            receipt.frames,
            receipt.run.state_hash,
        );
        // Which arm this capture is. One line, beside the hash, so a sheet of
        // three captures cannot be assembled out of order. (DC4)
        println!(
            "camera: {} section, slab half-height {}",
            receipt.camera, receipt.slab_half_height
        );
        // Loud, because the alternative this replaces was a body that was
        // simply not there.
        if receipt.body_capsules_dropped > 0 {
            println!(
                "body: {} of {} parts past the lens capsule budget, drawn truncated to its widest",
                receipt.body_capsules_dropped, receipt.body_parts
            );
        }
        if let Some(fault) = &receipt.fault {
            println!("fault: the sim stopped the run: {fault}");
        }
        let Some(path) = &self.config.receipt else {
            return;
        };
        if let Err(error) = played::write_json(path, &receipt) {
            eprintln!("receipt: {error}");
        }
    }

    /// `played`, the one mode a windowed run has; replay is headless.
    pub(crate) fn mode(&self) -> &'static str {
        "played"
    }

    fn receipt(&self) -> PlayedReceipt {
        let gpu = self.gpu.as_ref();
        let section = gpu.map(|gpu| &gpu.section);
        let scene = self.scene.as_ref();
        PlayedReceipt {
            mode: self.mode(),
            seed: self.config.seed,
            population: self.config.population,
            run: self.runtime.receipt(),
            steps: self.steps,
            frames: self.frames,
            refused: self
                .runtime
                .trace()
                .iter()
                .filter(|(_, outcome)| outcome.is_err())
                .count(),
            fault: self.runtime.fault().map(str::to_owned),
            adapter: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| info.name.clone()),
            backend: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| format!("{:?}", info.backend)),
            site: scene.map(|scene| scene.site),
            ground_revision: scene.map_or(0, |scene| scene.ground.revision()),
            body_parts: self
                .runtime
                .played()
                .and_then(|e| e.body.as_ref())
                .map_or(0, |body| body.parts.len()),
            body_capsules_dropped: self.body_capsules_dropped,
            section_roster: section.map_or(0, |s| s.last_roster_members()),
            roster_capsules_dropped: section.map_or(0, |s| s.last_roster_capsules_dropped()),
            slab_half_height: section.map_or(self.config.slab_half_height, |s| s.half_height()),
            camera: section.map_or(self.config.camera, |s| s.mode()).name(),
            terrain_style: self.config.terrain_style.resolved().name(),
            bodies: self.config.body_mode.name(),
            body_budget: self.config.body_budget,
            body_projection: section.map(|s| s.body_stats()).unwrap_or_default(),
            inspecting: self.inspection.open,
            selected_part: self.inspection.selected.map(|s| PartSelectionReceipt {
                organism: s.organism,
                part: s.part.0,
                revision: s.revision.0,
            }),
            frame_graph: gpu
                .and_then(|gpu| gpu.last_tenant_receipt.as_ref())
                .map(|receipt| FrameGraphReceipt {
                    tenant_name: receipt.tenant_name.clone(),
                    producer_path: receipt.producer_path.clone(),
                    fallback_count: receipt.fallback_count,
                    scene_op_boundary: receipt.scene_op_boundary,
                    caller_reported_physical_submission_count: receipt
                        .caller_reported_physical_submission_count,
                    logical_opaque_producer_boundaries: receipt.logical_opaque_producer_boundaries,
                    graph_encoder_batches: receipt.graph_encoder_batches,
                    graph_submission_boundaries: receipt.graph_submission_boundaries,
                    logical_plan_dump: receipt.logical_plan_dump.clone(),
                }),
            trace: self.config.trace.as_ref().map(|p| p.display().to_string()),
            capture: self
                .config
                .capture
                .as_ref()
                .map(|p| p.display().to_string()),
            dev: self.config.dev,
            dev_intents: self.runtime.dev_intents(),
        }
    }
}
