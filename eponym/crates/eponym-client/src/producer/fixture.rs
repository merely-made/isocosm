// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The producer tests' view of the promoted fixture world.
//!
//! The world itself lives in [`eponym_world::fixtures::session`], which the
//! session host also builds from; only the scene handle is producer-specific.

pub use eponym_world::fixtures::session::{Fixture, advance_motion};

/// The shared fixture world, its motion solved by the game's solver.
pub fn timed_action_world() -> Fixture {
    eponym_world::fixtures::session::timed_action_world_solving(eponym_motion::SOLVER)
}

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
