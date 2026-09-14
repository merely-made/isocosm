// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The shared session handle the producer and the document host both hold.
//!
//! Cambium's producers are same-device and same-thread: rootstock calls
//! [`TextureProducer::render`](cambium_rootstock::TextureProducer::render)
//! from the host's own loop, between layout and document paint. So the handle
//! is an `Rc<RefCell<..>>` rather than an `Arc<Mutex<..>>`: the DOM panels, the
//! keyboard path and the producer are all the same thread, and a lock would
//! only buy the ability to deadlock against a borrow. Hold the borrow for one
//! statement, never across a render.

use std::{cell::RefCell, rc::Rc};

use paredros_identity::SubjectId;
use paredros_world::{GameError, GameIntent, GameState, Session, SessionError};

use super::bodies::Appearance;
use super::camera::CameraPolicy;

/// One played session plus the presentation policy over it.
///
/// World state lives in the `Session`; everything else on this struct is
/// presentation and reaches no intent.
pub struct SceneModel {
    session: Session,
    played: SubjectId,
    pub camera: CameraPolicy,
    pub appearance: Appearance,
    /// Draw the traced terrain under the bodies. The bodies-only arm is the
    /// isolated preview: same camera, same depth, no ground.
    pub terrain: bool,
}

impl SceneModel {
    pub fn new(session: Session, played: SubjectId) -> Self {
        Self {
            session,
            played,
            camera: CameraPolicy::default(),
            appearance: Appearance::default(),
            terrain: true,
        }
    }

    /// Wraps the model in the handle a producer and a host share.
    pub fn into_handle(self) -> SceneHandle {
        Rc::new(RefCell::new(self))
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn game(&self) -> &GameState {
        self.session.game()
    }

    pub fn played(&self) -> SubjectId {
        self.played
    }

    /// The subject the frame is centred on. Control changes through the
    /// session's own succession rule; this only follows it.
    pub fn set_played(&mut self, subject: SubjectId) {
        self.played = subject;
    }

    /// The one way world state changes here: the session's recorded intent
    /// path, exactly as the timed-action host drives it.
    pub fn apply(
        &mut self,
        intent: GameIntent,
    ) -> Result<Vec<paredros_world::GameEvent>, SessionError> {
        self.session.apply_game(intent)
    }

    /// Replaces the session wholesale, for save/load and for a test that
    /// drives intents through another session wrapper.
    pub fn set_session(&mut self, session: Session) {
        self.session = session;
    }

    /// Where the frame is centred: the played subject's exact pose, or its
    /// logical cell when it has no pose yet.
    pub fn follow_centre(&self) -> Result<[f32; 3], GameError> {
        let game = self.game();
        if let Some(pose) = game.pose(self.played) {
            let scale = paredros_world::MOTION_SCALE as f32;
            return Ok([0, 1, 2].map(|i| pose.position[i] as f32 / scale));
        }
        let at = game
            .movement()
            .position(self.played)
            .ok_or(GameError::Movement(
                paredros_world::MovementError::MissingSubject(self.played),
            ))?;
        Ok(at.map(|v| v as f32))
    }
}

/// The shared handle. Cheap to clone; every clone sees one session.
pub type SceneHandle = Rc<RefCell<SceneModel>>;
