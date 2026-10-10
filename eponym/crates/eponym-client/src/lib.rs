// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S0, the room probe: the first game code in this repository.
//!
//! One body, one room, a close camera, a fixed input trace, and a rendered
//! receipt. Everything load-bearing is consumed rather than grown here, per
//! the execution plan's stop rule:
//!
//! - the world and the room are Eponym's world on Isocosm, its played site
//!   lifted into brick ground by isometer, and a room carved into a hillside
//!   as an edit the sim keeps;
//! - movement is the walker's step over isometer's in-site space, with no
//!   kinematics of our own;
//! - geometry is `isometer-mesh`'s greedy mesher over the same bricks the
//!   walker collides against;
//! - the picture is mere's body tenant on netrender's device, composed into
//!   netrender's master frame as an external texture.
//!
//! What is Eponym's own is small and deliberate: which room, which trace,
//! where the camera sits, and the save/replay discipline the later gates
//! inherit.

#[cfg(feature = "r1-proof")]
mod brick;
pub mod crossing;
// Immutable equipment saves, kept when the body sheet that owned them was
// retired (M5 of the isomere plan). Product state, not GUI machinery.
pub mod equipment_store;
pub mod frame_health;
pub mod gpu;
pub mod probe;
// The scene producer traces terrain through `isometer::lens`, which the family
// facade carries unconditionally, so the module is unconditional too.
pub mod producer;
#[cfg(feature = "v1-proof")]
pub mod residency;
pub mod room;
pub mod scene;
// The timed-action fixture world, shared by the producer tests and the
// document host so neither grows a third copy of it.

pub use probe::{Probe, ProbeError, Save, TICKS, TRACE};
pub use room::{Room, RoomError};
