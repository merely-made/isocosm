// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{cell::RefCell, rc::Rc};

use super::state::Specimen;
use crate::section::{
    self, BodyFrameStats, BodyMode, BodySelection, Framing, Section, SectionFrame,
};
use isometer::{FrameRequest, SceneProducer, SceneSignature, SceneSource};
use mesocosm_core::PartId;

/// The bench's scene behind the wing's producer wrapper: the unchanged-input
/// skip and the sRGB / straight-alpha output contract are `isometer`'s, and
/// everything a receipt reads is still this bench's own.
pub(super) type BenchProducer = SceneProducer<BenchScene>;

pub(super) fn bench_producer(model: Rc<RefCell<Specimen>>) -> BenchProducer {
    SceneProducer::new(BenchScene::new(model))
}

pub(super) fn card_producer(model: Rc<RefCell<Specimen>>, card: usize) -> BenchProducer {
    SceneProducer::new(BenchScene::for_card(model, card))
}

pub(super) struct BenchScene {
    pub model: Rc<RefCell<Specimen>>,
    pub section: Option<Section>,
    pub error: Option<String>,
    pub stats: BodyFrameStats,
    population: Option<super::population::renderer::Renderer>,
    pub population_stats: Option<super::population::renderer::RenderStats>,
    pub glyph_count: usize,
    pub anchor_count: usize,
    pub mesh_upload_bytes: u64,
    pub terrain_upload_bytes: u64,
    pub terrain_write_calls: u64,
    pub instance_upload_bytes: u64,
    epoch: u64,
    revision: u64,
    size: (u32, u32),
    tint: Option<[f32; 3]>,
    card: Option<usize>,
    ground_revision: Option<u64>,
}

impl BenchScene {
    pub fn new(model: Rc<RefCell<Specimen>>) -> Self {
        Self {
            model,
            section: None,
            error: None,
            stats: BodyFrameStats::default(),
            population: None,
            population_stats: None,
            glyph_count: 0,
            anchor_count: 0,
            mesh_upload_bytes: 0,
            terrain_upload_bytes: 0,
            terrain_write_calls: 0,
            instance_upload_bytes: 0,
            epoch: 0,
            revision: 0,
            size: (0, 0),
            tint: None,
            card: None,
            ground_revision: None,
        }
    }

    pub fn for_card(model: Rc<RefCell<Specimen>>, card: usize) -> Self {
        Self {
            card: Some(card),
            ..Self::new(model)
        }
    }

    pub fn pick(
        &mut self,
        local: (f32, f32),
        logical: (f32, f32),
    ) -> Result<Option<BodySelection>, String> {
        let model = self.model.borrow();
        if self.epoch != model.epoch || self.revision != model.revision || model.creator.pending {
            return Err("The specimen view is refreshing.".into());
        }
        if ![local.0, local.1, logical.0, logical.1]
            .iter()
            .all(|v| v.is_finite())
            || logical.0 <= 0.0
            || logical.1 <= 0.0
        {
            return Err("The specimen has no usable viewport.".into());
        }
        if local.0 < 0.0 || local.1 < 0.0 || local.0 >= logical.0 || local.1 >= logical.1 {
            return Ok(None);
        }
        let section = self
            .section
            .as_mut()
            .ok_or("The specimen is not drawn yet.")?;
        let hit = section
            .pick_pixel([
                (local.0 / logical.0 * self.size.0 as f32).floor() as u32,
                (local.1 / logical.1 * self.size.1 as f32).floor() as u32,
            ])
            .map_err(|why| format!("Part query unavailable: {why:?}"))?;
        Ok(hit
            .filter(|pick| section.validate_pick(*pick, model.world(), &model.volumes))
            .map(|pick| pick.selection))
    }

    pub fn select(&mut self, part: PartId) -> Option<BodySelection> {
        let model = self.model.borrow();
        if model.creator.pending || self.epoch != model.epoch || self.revision != model.revision {
            return None;
        }
        let section = self.section.as_mut()?;
        let subject = model.subject()?;
        let count = model
            .world()
            .organisms
            .iter()
            .find(|o| o.id == subject)?
            .body()
            .parts
            .len();
        let mut selected = None;
        for _ in 0..count {
            selected = section.select_part(subject, selected, false);
            if let Some(selection) = selected.filter(|s| s.part == part) {
                return section
                    .validate_selection(selection, model.world(), &model.volumes)
                    .then_some(selection);
            }
        }
        None
    }

    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        tint: [f32; 3],
        needs_frame: bool,
    ) -> Result<Option<wgpu::TextureView>, String> {
        let model = self.model.borrow();
        let epoch = self.epoch(&model);
        if let Some(workload) = &model.population {
            if self.population.is_none() || self.size != size {
                self.population = Some(super::population::renderer::Renderer::new(
                    device, queue, size, workload,
                )?);
            } else if !needs_frame && self.revision == model.revision && self.tint == Some(tint) {
                return Ok(None);
            }
            let renderer = self.population.as_mut().unwrap();
            let stats = renderer.render(device, queue, workload, model.yaw, tint)?;
            self.stats = BodyFrameStats {
                candidates: stats.body_count,
                voxel_bodies: stats.body_count,
                voxel_parts: stats.draw_parts,
                draw_parts: stats.draw_parts,
                mesh_builds: stats.mesh_builds,
                mesh_upload_bytes: stats.mesh_upload_bytes as u64,
                instance_upload_bytes: stats.instance_upload_bytes as u64,
                frame_upload_bytes: stats.frame_upload_bytes as u64,
                ..Default::default()
            };
            self.mesh_upload_bytes += self.stats.mesh_upload_bytes;
            self.instance_upload_bytes += self.stats.instance_upload_bytes;
            self.population_stats = Some(stats);
            self.epoch = epoch;
            self.revision = model.revision;
            self.size = size;
            self.tint = Some(tint);
            return Ok(Some(renderer.view().clone()));
        }
        let world = model
            .card_world(self.card)
            .ok_or("Alternative is not admitted.")?;
        if self
            .section
            .as_ref()
            .is_none_or(|s| s.mode() != model.camera)
            || self.epoch != epoch
        {
            self.ground_revision = None;
            self.section = Some(Section::new(
                device.clone(),
                queue.clone(),
                size.0,
                size.1,
                wgpu::TextureFormat::Rgba8Unorm,
                world.ground(),
                Framing::new(28.0, model.camera),
            )?);
        } else if self.size != size {
            self.section.as_mut().unwrap().resize(size.0, size.1);
        }
        let section = self.section.as_mut().unwrap();
        section.configure_bodies(BodyMode::Voxels, section::DEFAULT_BODY_BUDGET);
        if let Some(organism) = world.controlled() {
            let subject = organism.id;
            section
                .set_body_yaw(subject, model.yaw)
                .map_err(|e| format!("Body pose: {e:?}"))?;
            // CSS white is neutral; retain the organism's ordinary palette.
            let base = crate::app::look_of(organism).0;
            let appearance = [0, 1, 2].map(|i| base[i] * tint[i]);
            section
                .set_body_tint(subject, Some(appearance))
                .map_err(|e| format!("Body appearance: {e:?}"))?;
        }
        let selected = if self.card.is_none() {
            model.selected
        } else {
            None
        };
        section.set_body_focus(selected.map(|s| s.organism), selected);
        let mut centre = world
            .position()
            .or_else(|| model.source_world().position())
            .unwrap_or([0, 0, 0])
            .map(|v| v as f32);
        let mut half = 28.0;
        let mut depth = section::SLAB_DEPTH;
        let isolated = (model.isolated && model.trial.is_none()) || self.card.is_some();
        if isolated {
            if let Some(organism) = world.controlled() {
                if let Some(bounds) = section.presentation_bounds(organism, &model.volumes)? {
                    (centre, half, depth) = fit(bounds, size, model.camera);
                }
            }
        }
        if self.card.is_some() {
            if let Some(comparison) = &model.comparison {
                if comparison.shared_scale {
                    for card in &comparison.cards {
                        if let Some(organism) = card.world.as_ref().and_then(|w| w.controlled()) {
                            if let Some(bounds) =
                                section.presentation_bounds(organism, &model.volumes)?
                            {
                                half = half.max(fit(bounds, size, model.camera).1);
                            }
                        }
                    }
                }
            }
        }
        self.anchor_count = 0;
        let mut attachment_anchors = Vec::new();
        let attachment_subject = model
            .selected
            .and_then(|s| world.organisms.iter().find(|o| o.id == s.organism))
            .or_else(|| world.controlled());
        let marks = if self.card.is_none() && model.trial.is_some() {
            // A trial mark borne by a living part sits on that part's own
            // meshed face, so the anchors are read here, where the scene can
            // answer for a posed body. The spatial preview's own anchor
            // count stays its own; this path never touches it.
            let anchors = match world.controlled() {
                Some(organism) => section.glyph_anchors(organism, &model.volumes, None)?,
                None => Vec::new(),
            };
            model.trial.as_ref().unwrap().anchored_marks(&anchors)
        } else if self.card.is_none() && model.spatial.enabled {
            match attachment_subject {
                Some(organism) => match section.presentation_bounds(organism, &model.volumes)? {
                    Some(bounds) => {
                        let anchors =
                            section.glyph_anchors(organism, &model.volumes, model.selected)?;
                        self.anchor_count = anchors.len();
                        attachment_anchors = anchors;
                        model.spatial.marks(bounds, &attachment_anchors)
                    },
                    None => Vec::new(),
                },
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        if isolated && !marks.is_empty() {
            if let Some(organism) = attachment_subject {
                if let Some(bounds) = section.presentation_bounds(organism, &model.volumes)? {
                    (centre, half, depth) = fit(
                        model.spatial.framing_bounds(bounds, &attachment_anchors),
                        size,
                        model.camera,
                    );
                }
            }
        }
        self.glyph_count = marks.len();
        section.set_glyphs(marks)?;
        section.set_half_height(half);
        section.set_body_preview(isolated, depth);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("specimen bench shared-depth scene"),
        });
        let dirty: Vec<_> =
            if self.card.is_none() && self.ground_revision != Some(world.ground().revision()) {
                model
                    .trial
                    .as_ref()
                    .map(|t| t.dirty.iter().copied().collect())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
        section.render(
            &mut encoder,
            SectionFrame {
                world,
                volumes: &model.volumes,
                ground: world.ground(),
                dirty: &dirty,
                centre,
                pose: None,
                roster: &[],
            },
        )?;
        queue.submit(Some(encoder.finish()));
        self.ground_revision = Some(world.ground().revision());
        if let Some(terrain) = section.terrain_diagnostics() {
            self.terrain_upload_bytes += terrain.brick_upload_bytes;
            self.terrain_write_calls +=
                u64::from(terrain.pointer_write_calls) + u64::from(terrain.atlas_write_calls);
        }
        self.stats = section.body_stats();
        self.mesh_upload_bytes += self.stats.mesh_upload_bytes;
        self.instance_upload_bytes += self.stats.instance_upload_bytes;
        self.epoch = epoch;
        self.revision = model.revision;
        self.size = size;
        self.tint = Some(tint);
        Ok(Some(section.encoded_view().clone()))
    }

    pub fn retire(&mut self) {
        self.section = None;
        self.population = None;
        self.population_stats = None;
        self.ground_revision = None;
        self.revision = 0;
        self.size = (0, 0);
    }
}

impl BenchScene {
    /// The bench's own epoch: a card reads the comparison's, the main leaf the
    /// model's.
    fn epoch(&self, model: &Specimen) -> u64 {
        if self.card.is_some() {
            model.comparison.as_ref().map_or(model.epoch, |c| c.epoch)
        } else {
            model.epoch
        }
    }

    /// The document's CSS colour as a linear tint. The conversion stays here:
    /// the wing's producer carries no colour policy, and the bench's neutral
    /// white must land as an unmodified kingdom palette.
    fn tint(request: &FrameRequest<'_>) -> Result<[f32; 3], String> {
        let color = request.color.unwrap_or([1.0; 4]);
        if color[3] != 1.0 {
            return Err(
                "Body tint needs an opaque colour. Use viewport opacity to fade the scene.".into(),
            );
        }
        Ok([color[0], color[1], color[2]].map(|channel| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        }))
    }
}

impl SceneSource for BenchScene {
    /// The bench's skip key, unchanged: the specimen's epoch and revision, the
    /// viewport and the tint. It stays host-opaque because the bench's bodies
    /// are the section's to walk, not this leaf's.
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        let tint = match Self::tint(request) {
            Ok(tint) => tint,
            Err(why) => {
                self.error = Some(why.clone());
                return Err(why);
            },
        };
        self.error = None;
        let model = self.model.borrow();
        let epoch = self.epoch(&model);
        Ok(SceneSignature {
            size: request.size,
            host: vec![
                epoch,
                model.revision,
                u64::from(tint[0].to_bits()),
                u64::from(tint[1].to_bits()),
                u64::from(tint[2].to_bits()),
            ],
            ..Default::default()
        })
    }

    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        let tint = Self::tint(request)?;
        let size = (request.size[0], request.size[1]);
        match self.render_scene(
            request.device,
            request.queue,
            size,
            tint,
            request.needs_frame,
        ) {
            Ok(view) => {
                self.error = None;
                Ok(view)
            },
            Err(why) => {
                self.error = Some(why.clone());
                Err(why)
            },
        }
    }

    fn presented_camera(&self) -> Option<isometer::SlabCamera> {
        self.section.as_ref().and_then(Section::presented_camera)
    }

    fn cached_bodies(&self) -> usize {
        self.section.as_ref().map_or(0, Section::cached_bodies)
    }

    fn suspend(&mut self) {
        self.retire();
    }

    fn retire(&mut self) {
        BenchScene::retire(self);
    }
}

fn fit(
    (min, max): ([f32; 3], [f32; 3]),
    size: (u32, u32),
    mode: section::CameraMode,
) -> ([f32; 3], f32, f32) {
    let [right, up, forward] = section::camera_basis(mode, None);
    let span = |axis: [f32; 3]| {
        (0..3)
            .map(|i| (max[i] - min[i]) * axis[i].abs())
            .sum::<f32>()
    };
    let centre = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5);
    let half = (span(up).max(span(right) * size.1 as f32 / size.0.max(1) as f32) * 0.65).max(2.0);
    let horizontal = (forward[0].powi(2) + forward[2].powi(2)).sqrt();
    let depth = span([forward[0] / horizontal, 0.0, forward[2] / horizontal]) + 2.0;
    (centre, half, depth)
}
