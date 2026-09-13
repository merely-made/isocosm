// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{cell::RefCell, rc::Rc};

use mesocosm_core::{OrganismId, PartId, history::History};
use mesocosm_mesh::VolumeMap;
use mesocosm_views::PartInspection;

use super::{super::creator::Creator, producer::BenchScene};
use crate::section::{BodySelection, CameraMode};

pub(super) struct Specimen {
    pub creator: Creator,
    pub volumes: VolumeMap,
    pub epoch: u64,
    pub revision: u64,
    pub selected: Option<BodySelection>,
    pub yaw: f32,
    pub spatial: super::spatial::Spatial,
    pub trial: Option<super::trial::WorldTrial>,
    pub isolated: bool,
    pub camera: CameraMode,
    pub content: Option<mesocosm_mesh::content::ContentPack>,
    pub comparison: Option<super::comparison::Comparison>,
}

impl Specimen {
    pub fn changed(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("bench revision exhausted");
    }

    pub fn replaced(&mut self) {
        self.trial = None;
        self.comparison = None;
        self.epoch = self.epoch.checked_add(1).expect("bench epoch exhausted");
        self.selected = None;
        // Show the face of authored starts; the turn controls remain available.
        self.yaw = if self.creator.request.criteria.archetype.is_some() {
            std::f32::consts::PI
        } else {
            0.0
        };
        self.changed();
    }

    pub fn subject(&self) -> Option<OrganismId> {
        self.selected
            .map(|s| s.organism)
            .or_else(|| self.world().controlled_id())
    }

    pub fn reading(&self) -> PartInspection {
        self.selected
            .map_or_else(PartInspection::default, |selected| {
                mesocosm_views::part_of(
                    self.world(),
                    selected.organism,
                    selected.part,
                    self.trial
                        .as_ref()
                        .map(|t| t.driver.history())
                        .unwrap_or(&History::new()),
                )
            })
    }
}

pub(super) struct Bench {
    pub model: Rc<RefCell<Specimen>>,
    pub scene: Rc<RefCell<BenchScene>>,
    pub events: Vec<String>,
    pub notice: String,
    pub published_error: Option<String>,
    pub tint: usize,
    pub decorated: bool,
    pub visible: bool,
    pub transformed: bool,
    pub overlay_clicks: u64,
    pub cards: Vec<Rc<RefCell<BenchScene>>>,
    pub export_directory: std::path::PathBuf,
    pub restore: Option<super::comparison::SavedComparison>,
    pub generation: super::generation_controls::Controls,
    pub effects: super::effects::Effects,
}

impl Bench {
    pub fn poll(&mut self) -> bool {
        let mut model = self.model.borrow_mut();
        let was_pending = model.creator.pending;
        if !was_pending {
            return false;
        }
        model.creator.poll();
        if !model.creator.pending {
            model.replaced();
            self.events.push("specimen-ready".into());
            self.notice = model.creator.notice.clone();
            if let Some(saved) = self.restore.take() {
                model.creator.select(saved.selection.source.candidate);
                match model.compare(saved.selection.round) {
                    Ok(()) => {
                        model.comparison.as_mut().unwrap().selected = saved.selection.selected;
                        self.notice = "Saved comparison restored.".into();
                    },
                    Err(why) => self.notice = why,
                }
            }
            return true;
        }
        false
    }

    pub fn candidate(&mut self, backwards: bool) {
        self.restore = None;
        let mut model = self.model.borrow_mut();
        let count = model.creator.count();
        if count == 0 {
            return;
        }
        let current = model.creator.selected;
        model
            .creator
            .select((current + if backwards { count - 1 } else { 1 }) % count);
        model.replaced();
        self.notice.clear();
        self.events.push("candidate-changed".into());
    }

    pub fn reroll(&mut self) {
        self.restore = None;
        let mut model = self.model.borrow_mut();
        model.creator.request.variation = model.creator.request.variation.wrapping_add(1);
        model.creator.regenerate();
        model.replaced();
        self.notice.clear();
        self.events.push("reroll".into());
    }

    pub fn turn(&mut self, delta: f32) {
        let mut model = self.model.borrow_mut();
        model.yaw = (model.yaw + delta).rem_euclid(std::f32::consts::TAU);
        model.changed();
        self.events.push("pose-changed".into());
    }

    pub fn clear(&mut self) {
        let mut model = self.model.borrow_mut();
        model.selected = None;
        model.changed();
        self.notice.clear();
        self.events.push("selection-cleared".into());
    }

    pub fn select(&mut self, part: PartId) {
        let selection = self.scene.borrow_mut().select(part);
        self.accept_selection(selection);
    }

    pub fn pick(&mut self, local: (f32, f32), size: (f32, f32)) {
        let result = self.scene.borrow_mut().pick(local, size);
        match result {
            Ok(selection) => self.accept_selection(selection),
            Err(why) => self.notice = why,
        }
    }

    fn accept_selection(&mut self, selected: Option<BodySelection>) {
        let mut model = self.model.borrow_mut();
        model.selected = selected;
        model.changed();
        self.notice = if selected.is_some() {
            String::new()
        } else {
            "No visible part here.".into()
        };
        self.events.push(
            if selected.is_some() {
                "part-selected"
            } else {
                "part-cleared"
            }
            .into(),
        );
    }

    pub fn toggle_habitat(&mut self) {
        let mut model = self.model.borrow_mut();
        model.isolated = !model.isolated;
        if model.isolated
            && model
                .selected
                .is_some_and(|s| Some(s.organism) != model.world().controlled_id())
        {
            model.selected = None;
        }
        model.changed();
        self.events.push("view-changed".into());
    }

    pub fn save(&mut self) {
        let mut model = self.model.borrow_mut();
        model.creator.save_draft();
        self.notice = format!(
            "{} Criteria exclude resized content; Save specimen preserves it.",
            model.creator.notice
        );
    }
}
