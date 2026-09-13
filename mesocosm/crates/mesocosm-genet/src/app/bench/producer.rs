// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{cell::RefCell, rc::Rc};

use super::state::Specimen;
use crate::section::{
    self, BodyFrameStats, BodyMode, BodySelection, Framing, Section, SectionFrame,
};
use cambium_rootstock::{
    ProducedTexture, ProducerContext, SourceAlpha, SourceEncoding, TextureProducer,
};
use mesocosm_core::PartId;

pub(super) struct BenchScene {
    pub model: Rc<RefCell<Specimen>>,
    pub section: Option<Section>,
    pub error: Option<String>,
    pub stats: BodyFrameStats,
    pub renders: u64,
    pub mesh_upload_bytes: u64,
    pub instance_upload_bytes: u64,
    epoch: u64,
    revision: u64,
    size: (u32, u32),
    tint: Option<[f32; 3]>,
    card: Option<usize>,
}

impl BenchScene {
    pub fn new(model: Rc<RefCell<Specimen>>) -> Self {
        Self {
            model,
            section: None,
            error: None,
            stats: BodyFrameStats::default(),
            renders: 0,
            mesh_upload_bytes: 0,
            instance_upload_bytes: 0,
            epoch: 0,
            revision: 0,
            size: (0, 0),
            tint: None,
            card: None,
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
            .creator
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
        let epoch = if self.card.is_some() {
            model.comparison.as_ref().map_or(model.epoch, |c| c.epoch)
        } else {
            model.epoch
        };
        if !needs_frame
            && self.section.is_some()
            && self.epoch == epoch
            && self.revision == model.revision
            && self.size == size
            && self.tint == Some(tint)
        {
            return Ok(None);
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
        let mut centre = world.position().unwrap_or([0, 0, 0]).map(|v| v as f32);
        let mut half = 28.0;
        let mut depth = section::SLAB_DEPTH;
        let isolated = model.isolated || self.card.is_some();
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
        let marks = if self.card.is_none() && model.spatial.enabled {
            match world.controlled() {
                Some(organism) => match section.presentation_bounds(organism, &model.volumes)? {
                    Some(bounds) => model.spatial.marks(bounds),
                    None => Vec::new(),
                },
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        if isolated && !marks.is_empty() {
            let mut min = centre;
            let mut max = centre;
            for mark in &marks {
                for i in 0..3 {
                    min[i] = min[i].min(mark.centre[i] - mark.size);
                    max[i] = max[i].max(mark.centre[i] + mark.size);
                }
            }
            let (_, glyph_half, glyph_depth) = fit((min, max), size, model.camera);
            half = half.max(glyph_half);
            depth = depth.max(glyph_depth);
        }
        section.set_glyphs(marks)?;
        section.set_half_height(half);
        section.set_body_preview(isolated, depth);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("specimen bench shared-depth scene"),
        });
        section.render(
            &mut encoder,
            SectionFrame {
                world,
                volumes: &model.volumes,
                ground: world.ground(),
                dirty: &[],
                centre,
                pose: None,
                roster: &[],
            },
        )?;
        queue.submit(Some(encoder.finish()));
        self.stats = section.body_stats();
        self.mesh_upload_bytes += self.stats.mesh_upload_bytes;
        self.instance_upload_bytes += self.stats.instance_upload_bytes;
        self.renders = self
            .renders
            .checked_add(1)
            .expect("bench frame generation exhausted");
        self.epoch = epoch;
        self.revision = model.revision;
        self.size = size;
        self.tint = Some(tint);
        Ok(Some(section.encoded_view().clone()))
    }

    pub fn retire(&mut self) {
        self.section = None;
        self.revision = 0;
        self.size = (0, 0);
    }
}

impl TextureProducer for BenchScene {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        let color = cx.frame.appearance.color().unwrap_or([1.0; 4]);
        if color[3] != 1.0 {
            self.error = Some(
                "Body tint needs an opaque colour. Use viewport opacity to fade the scene.".into(),
            );
            return None;
        }
        let size = (cx.frame.physical_size[0], cx.frame.physical_size[1]);
        let tint = [color[0], color[1], color[2]].map(|channel| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        });
        match self.render_scene(cx.device, cx.queue, size, tint, cx.frame.needs_frame) {
            Ok(view) => {
                self.error = None;
                view.map(|view| ProducedTexture {
                    view,
                    generation: self.renders,
                    alpha: SourceAlpha::Straight,
                    encoding: SourceEncoding::Srgb,
                })
            },
            Err(why) => {
                self.error = Some(why);
                None
            },
        }
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
