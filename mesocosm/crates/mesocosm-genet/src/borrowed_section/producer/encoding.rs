// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Source {
    pub(super) fn encode_frame(
        &mut self,
        request: &FrameRequest<'_>,
        strict: bool,
    ) -> Result<Option<wgpu::TextureView>, String> {
        if request.device != &self.host.device || request.queue != &self.host.queue {
            return Err("Mesocosm section cannot migrate onto a foreign GPU".into());
        }
        let size = [request.size[0].max(1), request.size[1].max(1)];
        let costs = self.costs.clone();
        costs.measure("resource_setup", || self.target(size));
        let Some(input) = self.handle.snapshot() else {
            self.section = None;
            self.ground = None;
            self.presented = None;
            self.presented_input = None;
            self.accepted = None;
            let mut encoder =
                self.host
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Mesocosm withdrawn close-up clears previous image"),
                    });
            Self::clear(&mut encoder, &self.target.as_ref().unwrap().1);
            costs.count("withdrawn_frames_encoded", 1);
            costs.measure("queue_submit", || {
                self.host.queue.submit([encoder.finish()])
            });
            return Ok(Some(self.target.as_ref().unwrap().2.clone()));
        };
        let camera = input.camera(request.aspect);
        let result = (|| {
            let (was_animating, terrain_before) = costs.measure("resource_setup", || {
                if self.section.is_none() {
                    costs.count("section_create_calls", 1);
                    self.section = Some(Section::new(
                        &self.host,
                        size[0],
                        size[1],
                        wgpu::TextureFormat::Rgba8UnormSrgb,
                        &input.scene.ground,
                        Framing::default(),
                    )?);
                    costs.count("sections_created", 1);
                }
                let section = self.section.as_mut().unwrap();
                let was_animating = section.animating();
                let terrain_before = section.terrain_changes();
                costs.count("section_resize_calls", 1);
                if section.extent() != size {
                    costs.count("section_extent_changes", 1);
                }
                section.resize(size[0], size[1]);
                if self
                    .ground
                    .as_ref()
                    .is_none_or(|old| !same_terrain(old, &input.scene))
                {
                    costs.count("ground_rebind_calls", 1);
                    section.set_ground(&input.scene.ground)?;
                }
                self.ground = Some(input.scene.clone());
                section.configure_bodies(BodyMode::Voxels, section::DEFAULT_BODY_BUDGET);
                if let Some(view) = &input.presentation {
                    section.set_mode(view.mode);
                    section.set_view_depth(view.depth)?;
                    section.configure_bodies(view.bodies, view.budget);
                    section.set_half_height(view.half_height);
                    section.set_body_light(view.light);
                    section.set_sky(view.sky);
                    section.configure_terrain(view.terrain, view.centre, view.terrain_y);
                    section.set_body_focus(view.focus, view.selection);
                }
                section.set_forward(Some(camera.forward))?;
                section.set_perspective(camera.frustum().map(|_| camera));
                Ok::<_, String>((was_animating, terrain_before))
            })?;
            let section = self.section.as_mut().unwrap();
            let (encoder, pose, roster) = costs.measure("draw_work", || {
                let mut encoder =
                    self.host
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Mesocosm section on the present device"),
                        });
                let draw = &self.target.as_ref().unwrap().1;
                Self::clear(&mut encoder, draw);
                costs.count("input_body_records", input.scene.bodies.len() as u64);
                let pose = section::pose_of(&input.scene, 1.0).map(|(pose, _)| pose);
                let roster =
                    section::roster_of(&input.scene, section.slab_window(camera.centre), 1.0);
                costs.count("pose_records", u64::from(pose.is_some()));
                costs.count("roster_records", roster.len() as u64);
                section.draw(
                    &mut encoder,
                    draw,
                    SectionFrame {
                        scene: &input.scene,
                        dirty: &[],
                        centre: camera.centre,
                        pose: pose.as_ref(),
                        roster: &roster,
                    },
                )?;
                Ok::<_, String>((encoder, pose, roster))
            })?;
            costs.measure("queue_submit", || {
                self.host.queue.submit([encoder.finish()])
            });
            let capsules = input
                .presentation
                .as_ref()
                .is_some_and(|view| view.bodies == BodyMode::Capsules);
            let body = if capsules {
                Default::default()
            } else {
                section.body_stats()
            };
            let populated =
                body.draw_parts > 0 || (capsules && (pose.is_some() || !roster.is_empty()));
            self.costs.drawn(
                &body,
                section.terrain_diagnostics(),
                section.terrain_changes().saturating_sub(terrain_before),
                section.terrain_stats(),
                populated,
            );
            if std::env::var_os("MESOCOSM_DEBUG_MOTION").is_some() {
                eprintln!(
                    "viewport {:?}: animation {was_animating} -> {}",
                    input.revision,
                    section.animating()
                );
            }
            if !strict {
                self.recording.frame(
                    section,
                    &self.target.as_ref().unwrap().0,
                    &input,
                    was_animating,
                )?;
            }
            costs.count("accepted_capture_calls", 1);
            let accepted = costs.measure("accepted_query_capture", || {
                Accepted::capture(section, input.clone(), self.handle.lifetime(), populated)
            });
            if let Some(queries) = &accepted.queries {
                let stats = queries.capture_stats();
                costs.count("fresh_query_snapshots", 1);
                costs.count("query_placed_records_copied", stats.placed_bodies as u64);
                costs.count("query_surface_records_copied", stats.surfaces as u64);
                costs.count(
                    "query_terrain_pointer_words_copied",
                    stats.terrain_pointer_words as u64,
                );
                costs.count(
                    "query_terrain_atlas_bytes_copied",
                    stats.terrain_atlas_bytes as u64,
                );
                costs.count(
                    "query_terrain_dense_payload_bytes_copied",
                    (stats.terrain_pointer_words as u64) * std::mem::size_of::<u32>() as u64,
                );
                costs.count(
                    "query_terrain_dense_payload_bytes_copied",
                    stats.terrain_atlas_bytes as u64,
                );
                costs.count(
                    "query_terrain_maps_copied",
                    u64::from(stats.terrain_present),
                );
            }
            self.accepted = Some(accepted);
            self.presented = Some(camera);
            self.presented_input = Some(input.fingerprint);
            self.handle.error.borrow_mut().take();
            Ok::<(), String>(())
        })();
        if let Err(why) = result {
            // A failed encode cannot keep pixels from an earlier native cut.
            self.section = None;
            self.ground = None;
            self.presented = None;
            self.presented_input = None;
            *self.handle.error.borrow_mut() = Some(why.clone());
            self.accepted = None;
            if strict {
                return Err(why);
            }
            let mut encoder =
                self.host
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Mesocosm refused frame clears previous image"),
                    });
            Self::clear(&mut encoder, &self.target.as_ref().unwrap().1);
            costs.count("refused_frames_cleared", 1);
            costs.measure("queue_submit", || {
                self.host.queue.submit([encoder.finish()])
            });
        }
        Ok(Some(self.target.as_ref().unwrap().2.clone()))
    }
}
