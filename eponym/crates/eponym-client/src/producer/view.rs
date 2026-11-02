// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The camera a frame is seen through (cases 1131, 1140, 1143): the
//! overview's slab, or a perspective on the played body from the eye the
//! sim names, its most sensing part, posed as the frame will draw it.

use eponym_play::MotionPose;
use isometer::core::{BodyDocument, PartId};
use isometer::{DeclaredExtentVolumes, SceneBody, SceneVolumes, SlabCamera, SubjectKey};

use super::handle::SceneModel;
use super::policy::View;
use super::source::{anatomy, scene_pose};

impl SceneModel {
    /// This frame's camera under the model's view.
    pub fn view_camera(&self, aspect: f32) -> Result<SlabCamera, String> {
        let centre = self.follow_centre().map_err(|error| format!("{error:?}"))?;
        let overview = self.camera.camera(centre, aspect).ok_or("invalid scene camera")?;
        if self.view == View::Overview {
            return Ok(overview);
        }
        let eye = self.eye()?.ok_or("the played body has no eye to look from")?;
        let heading = self.aim;
        self.perspective
            .camera(self.view, eye, heading, aspect)
            .ok_or_else(|| format!("invalid {} camera", self.view.name()))
    }

    /// The played body as first person draws it: only its parts wholly
    /// below the eye, so the body does not fill its own view and its limbs
    /// show at the frame's edge. `None` in any other view.
    pub fn first_person_document(&self, document: &BodyDocument) -> Option<BodyDocument> {
        (self.view == View::FirstPerson).then_some(())?;
        let game = self.game();
        let part = game.world().entity(self.played())
            .and_then(|id| game.world().sim().state().population.get(id))
            .and_then(cosmograph::anatomy::sensing_part);
        Some(isometer::below_the_eye(document, part))
    }

    /// Where the played body sees from: the top of its most sensing part,
    /// else of its root, as the frame poses the body.
    pub fn eye(&self) -> Result<Option<[f32; 3]>, String> {
        let game = self.game();
        let played = self.played();
        let Some(record) = anatomy(game, played) else { return Ok(None) };
        let Some(pose) = self.drawn_pose(played)
            .or_else(|| game.movement().position(played).map(MotionPose::at_cell)) else { return Ok(None) };
        let scale = self.appearance.scale;
        let body = SceneBody {
            subject: SubjectKey(played.0),
            document: &record.document,
            pose: scene_pose(&record.document, pose, scale),
            scale,
            grounded: true,
            tint: self.appearance.played,
            materials: &[],
            always_visible: true,
        };
        let volumes = DeclaredExtentVolumes::from_documents([record.document.doc()], self.appearance.material);
        let part: Option<PartId> = game.world().entity(played)
            .and_then(|id| game.world().sim().state().population.get(id))
            .and_then(cosmograph::anatomy::sensing_part);
        isometer::eye_of(&body, SceneVolumes::DeclaredSolid(&volumes), part)
    }
}
