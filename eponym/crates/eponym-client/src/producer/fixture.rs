// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The producer tests' view of the promoted fixture world.
//!
//! The world itself lives in [`eponym_world::fixtures::session`], which the
//! session host also builds from; only the scene handle is producer-specific.

pub use eponym_world::fixtures::session::{Fixture, advance_motion, timed_action_world};

use super::{SceneHandle, SceneModel};

/// The one producer-specific reading of the shared fixture. An extension
/// trait because [`Fixture`] is the world crate's type now.
pub trait FixtureScene {
    /// A handle over a copy of the current session, framed on the keeper.
    fn scene(&self) -> SceneHandle;
}

impl FixtureScene for Fixture {
    fn scene(&self) -> SceneHandle {
        SceneModel::new(self.action.session().clone(), self.keeper).into_handle()
    }
}
