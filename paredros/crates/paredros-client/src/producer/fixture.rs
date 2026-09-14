// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The producer tests' view of the promoted fixture world.
//!
//! The world itself lives in [`crate::session_fixture`], which the P2 host
//! also builds from; only the scene handle is producer-specific.

pub use crate::session_fixture::{Fixture, advance_motion, timed_action_world};

use super::{SceneHandle, SceneModel};

impl Fixture {
    /// A handle over a copy of the current session, framed on the keeper.
    pub fn scene(&self) -> SceneHandle {
        SceneModel::new(self.action.session().clone(), self.keeper).into_handle()
    }
}
