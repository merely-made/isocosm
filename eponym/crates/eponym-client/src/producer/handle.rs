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

use eponym_identity::SubjectId;
use eponym_world::timed_action::{TimedActionError, TimedActionSession};
use eponym_world::{GameError, GameEvent, GameIntent, GameState, Session};

use super::policy::{Appearance, CameraPolicy};

/// What holds the one `Session`.
///
/// `TimedActionSession` owns its `Session` by value and deliberately exposes
/// no mutable handle on it (its own doc comment says so), so a host that wants
/// both charged strikes and the producer's view cannot keep two. This slot is
/// the answer: the timed-action wrapper goes *inside* the model rather than
/// beside it, and `session()` reads through. There is still exactly one
/// `Session`, and every panel reads it.
pub enum Held {
    /// A session nothing else wraps. What the producer tests use.
    Plain(Session),
    /// A session the bounded timed-action grammar owns.
    Timed(Box<TimedActionSession>),
}

impl Held {
    pub fn session(&self) -> &Session {
        match self {
            Self::Plain(session) => session,
            Self::Timed(action) => action.session(),
        }
    }
}

/// One played session plus the presentation policy over it.
///
/// World state lives in the `Session`; everything else on this struct is
/// presentation and reaches no intent.
pub struct SceneModel {
    pub(super) held: Held,
    played: SubjectId,
    pub camera: CameraPolicy,
    pub appearance: Appearance,
    /// Draw the traced terrain under the bodies. The bodies-only arm is the
    /// isolated preview: same camera, same depth, no ground.
    pub terrain: bool,
    /// The shared scene this model draws through, built on the first frame
    /// because the device only arrives with a producer request. `None` after a
    /// retirement, which is the whole of this host's GPU release.
    pub(super) scene: Option<isometer::Scene>,
    /// What [`Self::scene`] was last sized for. `Scene` keeps its own width and
    /// height privately, so the resize decision is made here.
    pub(super) scene_size: [u32; 2],
    /// The ground revision the bound brick map was built from. The map is
    /// rebuilt wholesale when this moves, so an ordinary frame costs no CPU
    /// walk of the world.
    pub(super) terrain_map_revision: Option<u64>,
}

impl SceneModel {
    pub fn new(session: Session, played: SubjectId) -> Self {
        Self::held(Held::Plain(session), played)
    }

    /// The same model over a session the timed-action grammar owns.
    pub fn timed(action: TimedActionSession, played: SubjectId) -> Self {
        Self::held(Held::Timed(Box::new(action)), played)
    }

    fn held(held: Held, played: SubjectId) -> Self {
        Self {
            held,
            played,
            camera: CameraPolicy::default(),
            appearance: Appearance::default(),
            terrain: true,
            scene: None,
            scene_size: [0; 2],
            terrain_map_revision: None,
        }
    }

    /// Wraps the model in the handle a producer and a host share.
    pub fn into_handle(self) -> SceneHandle {
        Rc::new(RefCell::new(self))
    }

    pub fn session(&self) -> &Session {
        self.held.session()
    }

    /// The timed-action wrapper, when this model holds one.
    pub fn action(&self) -> Option<&TimedActionSession> {
        match &self.held {
            Held::Timed(action) => Some(action),
            Held::Plain(_) => None,
        }
    }

    /// Mutable access for the charge/release path, which is the wrapper's own
    /// grammar rather than a plain intent.
    pub fn action_mut(&mut self) -> Option<&mut TimedActionSession> {
        match &mut self.held {
            Held::Timed(action) => Some(action),
            Held::Plain(_) => None,
        }
    }

    pub fn game(&self) -> &GameState {
        self.session().game()
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
    pub fn apply(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, TimedActionError> {
        self.apply_batch(&[intent])
    }

    /// One accepted cut. The timed arm reconciles the open action against the
    /// batch, so an injury that severs a contributing limb repairs the action
    /// rather than leaving it addressing a part that no longer exists.
    pub fn apply_batch(
        &mut self,
        intents: &[GameIntent],
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        match &mut self.held {
            Held::Plain(session) => {
                let mut events = Vec::new();
                for intent in intents {
                    events.extend(session.apply_game(intent.clone())?);
                }
                Ok(events)
            },
            Held::Timed(action) => action.apply_game_batch(intents),
        }
    }

    /// Replaces the session wholesale, for save/load and for a test that
    /// drives intents through another session wrapper.
    pub fn set_session(&mut self, session: Session) {
        self.held = Held::Plain(session);
    }

    /// Replaces the timed-action wrapper wholesale, for F9 load.
    pub fn set_action(&mut self, action: TimedActionSession) {
        self.held = Held::Timed(Box::new(action));
    }

    /// Where the frame is centred: the played subject's exact pose, or its
    /// logical cell when it has no pose yet.
    pub fn follow_centre(&self) -> Result<[f32; 3], GameError> {
        let game = self.game();
        if let Some(pose) = game.pose(self.played) {
            let scale = eponym_world::MOTION_SCALE as f32;
            return Ok([0, 1, 2].map(|i| pose.position[i] as f32 / scale));
        }
        let at = game
            .movement()
            .position(self.played)
            .ok_or(GameError::Movement(
                eponym_world::MovementError::MissingSubject(self.played),
            ))?;
        Ok(at.map(|v| v as f32))
    }
}

/// The shared handle. Cheap to clone; every clone sees one session.
pub type SceneHandle = Rc<RefCell<SceneModel>>;
