// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The perspective views of the played body (cases 1131, 1140, 1143): over
//! the shoulder, and first person from the eye the sim names, its most
//! sensing part, else the top of the root. Presentation only: the eye and the
//! body drawn are read off a copy of the scene and never reach the world.

use isometer::core::PartId;
use isometer::{PerspectivePolicy, SceneVolumes, SlabCamera, View};

use super::SiteScene;
use super::bodies::HostBodies;

/// Which perspective is asked for and what it faces. A host reading, hashed
/// with the picture's other inputs, so a switch re-presents the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Looking {
    pub view: View,
    /// The played body's most sensing part, if any part senses.
    pub sensing: Option<PartId>,
    /// Flat: what first person and the shoulder face.
    pub heading: [f32; 3],
    /// The played body's height in cells: the policy's distances scale to it.
    pub height: f32,
}

impl Looking {
    pub fn overview() -> Self {
        Self { view: View::Overview, sensing: None, heading: [0.0, 0.0, 1.0], height: isometer::REFERENCE_BODY_HEIGHT }
    }
}

/// Where the played body sees from, posed as the frame will draw it.
pub fn eye(scene: &SiteScene, sensing: Option<PartId>) -> Option<[f32; 3]> {
    let body = scene.body(scene.played?)?;
    let drawn = HostBodies::new().scene_body(body, scene.played);
    isometer::eye_of(&drawn, SceneVolumes::DeclaredSolid(&scene.volumes), sensing).ok().flatten()
}

/// The scene a perspective view draws, and its eye. First person draws the
/// played body's parts wholly below the eye; the copy leaves the world's
/// body whole. `None` for the eye where the played body has none here.
pub fn seen(scene: &SiteScene, looking: &Looking) -> (SiteScene, Option<[f32; 3]>) {
    let mut seen = scene.clone();
    let eye = (looking.view != View::Overview).then(|| eye(scene, looking.sensing)).flatten();
    if looking.view == View::FirstPerson && eye.is_some()
        && let Some(id) = scene.played
        && let Some(body) = seen.bodies.iter_mut().find(|b| b.id == id)
    {
        body.document = isometer::below_the_eye(&body.document, looking.sensing);
    }

    (seen, eye)
}

/// Which way a body faces at a quarter-turn yaw: its chain runs along +z at
/// yaw zero, turned as the voxel body is.
pub fn facing(yaw: isometer::core::Yaw) -> [f32; 3] {
    let (sin, cos) = super::capsules::yaw_radians(yaw).sin_cos();
    [sin, 0.0, cos]
}

/// The shared presets from `eye`, or `None` for the overview.
pub fn camera(looking: &Looking, eye: [f32; 3], aspect: f32) -> Option<SlabCamera> {
    PerspectivePolicy::default().for_body(looking.height).camera(looking.view, eye, looking.heading, aspect)
}
