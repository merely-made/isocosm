// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lens's retained resources kept current: maps, target, uniforms,
//! bind groups and capture, each rebuilt only when its inputs change.

use super::*;

impl Lens {
    pub(super) fn ensure_maps(
        &mut self,
        input: FrameInput<'_>,
        diagnostics: &mut FrameDiagnostics,
    ) -> Result<(), LensError> {
        let recreate = self
            .maps
            .as_ref()
            .is_none_or(|maps| maps.side != input.maps.side);
        if recreate {
            self.maps = Some(create_maps(&self.device, input.maps, diagnostics));
            self.march_bind = None;
            self.grade_bind = None;
            diagnostics.map_recreated = true;
        }
        let maps = self.maps.as_mut().unwrap();
        if diagnostics.map_recreated || maps.revision != input.map_revision {
            let change = if diagnostics.map_recreated {
                MapChange::Full
            } else {
                input.map_change
            };
            upload_maps(&self.queue, maps, input.maps, change, diagnostics)?;
            maps.revision = input.map_revision;
        }
        Ok(())
    }

    pub(super) fn ensure_target(&mut self, downscale: u32, diagnostics: &mut FrameDiagnostics) {
        let downscale = downscale.max(1);
        let expected = (self.width, self.height, downscale);
        let ready = self
            .target
            .as_ref()
            .is_some_and(|target| (target.width, target.height, target.downscale) == expected);
        if ready {
            return;
        }
        let width = (self.width / downscale).max(1);
        let height = (self.height / downscale).max(1);
        let marched = make_target(&self.device, "lens marched", width, height, FRAME_FORMAT);
        let marched_view = marched.create_view(&Default::default());
        self.target = Some(TargetResources {
            width: self.width,
            height: self.height,
            downscale,
            _marched: marched,
            marched_view,
        });
        self.grade_bind = None;
        diagnostics.resource_creations += 1;
        diagnostics.target_recreated = true;
    }

    pub(super) fn write_uniforms(
        &mut self,
        input: FrameInput<'_>,
        diagnostics: &mut FrameDiagnostics,
    ) {
        let march = MarchParams {
            eye: input.flight.eye,
            yaw: input.flight.yaw,
            pitch: input.flight.pitch,
            fov: input.flight.fov,
            far: input.flight.far,
            map_side: input.maps.side as f32,
        };
        write_changed(
            &self.queue,
            &self.march_params,
            &mut self.last_march,
            march,
            diagnostics,
        );
        let critter = critter_params(input.pose);
        write_changed(
            &self.queue,
            &self.critter_params,
            &mut self.last_critter,
            critter,
            diagnostics,
        );
        let grade = GradeParams {
            fog: input.grade.fog,
            fog_start: input.grade.fog_start,
            palette_len: input.grade.palette_len.min(input.maps.palette.len() as u32),
            dither: input.grade.dither,
            fog_bands: input.grade.fog_bands,
            _pad: 0.0,
        };
        write_changed(
            &self.queue,
            &self.grade_params,
            &mut self.last_grade,
            grade,
            diagnostics,
        );
    }

    pub(super) fn ensure_bind_groups(&mut self, diagnostics: &mut FrameDiagnostics) {
        if self.march_bind.is_none() {
            let maps = self.maps.as_ref().unwrap();
            self.march_bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lens march"),
                layout: &self.march_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&maps.height_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&maps.color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.march_params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.critter_params.as_entire_binding(),
                    },
                ],
            }));
            diagnostics.resource_creations += 1;
            diagnostics.bind_group_rebuilds += 1;
        }
        if self.grade_bind.is_none() {
            let maps = self.maps.as_ref().unwrap();
            let target = self.target.as_ref().unwrap();
            self.grade_bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lens grade"),
                layout: &self.grade_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&target.marched_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.nearest),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&maps.palette_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.grade_params.as_entire_binding(),
                    },
                ],
            }));
            diagnostics.resource_creations += 1;
            diagnostics.bind_group_rebuilds += 1;
        }
    }

    pub(super) fn ensure_capture(&mut self) {
        let ready = self
            .capture
            .as_ref()
            .is_some_and(|capture| capture.width == self.width && capture.height == self.height);
        if ready {
            return;
        }
        let target = make_target(
            &self.device,
            "lens capture target",
            self.width,
            self.height,
            FRAME_FORMAT,
        );
        let target_view = target.create_view(&Default::default());
        let padded_bytes_per_row = (self.width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lens readback"),
            size: (padded_bytes_per_row * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.capture = Some(CaptureResources {
            width: self.width,
            height: self.height,
            target,
            target_view,
            staging,
            padded_bytes_per_row,
        });
        self.pending_resource_creations += 2;
    }
}
