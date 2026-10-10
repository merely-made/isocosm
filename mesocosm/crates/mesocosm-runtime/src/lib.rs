// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's host-neutral runtime.
//!
//! Sits between a native Isocosm [`Session`](isocosm::Session) and any host
//! (the switch, wing ruling 681). A host owns the window, the device and the
//! frame loop; it hands elapsed time in, queues contract envelopes, which
//! [`Runtime`] translates into native commands (686), and reads the world
//! back to draw it. The critter is directed, never driven (671, 679).
//!
//! [`Clock`] converts elapsed microseconds into whole fixed steps, so frame
//! delivery never reaches the simulation.

pub mod clock;
pub mod effect_experiment;
pub mod glyphs;
pub mod readings;
pub mod review;
pub mod runtime;
pub mod succession;
pub mod tactile;
pub mod voxel_profile;

pub use clock::{Advance, Clock};
pub use readings::{FlowWindows, JUDGEMENT_TICKS, RETENTION_TICKS, Trend};
pub use review::{Offer, Reading, Review};
pub use runtime::{
    DEFAULT_MAX_STEPS_PER_ADVANCE, Envelope, Founded, Receipt, Refusal, Replayed, Runtime,
};
pub use succession::{Birth, Boundary, Checkpoint, Loss, Occasion};
pub use tactile::{TactileCapsule, TactileError, TactileHit, TactilePick, TactileWorld};
