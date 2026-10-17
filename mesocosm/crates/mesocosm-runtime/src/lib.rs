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

pub mod continuation;
pub mod borrowed;
pub mod effect_experiment;
#[cfg(feature = "endpoint")]
pub mod endpoint;
pub mod glyphs;
pub mod perception;
pub mod readings;
pub mod review;
pub mod runtime;
mod profile;
pub mod succession;
pub mod spatial;
pub mod tactile;
pub mod terrain;
pub mod voxel_profile;

pub use isocosm_host::drive::{Advance, Button, Clock, Context, InputRefusal, Named, Pick, Pointed, Press};
pub use runtime::author_spread;
pub use readings::{FlowWindows, JUDGEMENT_TICKS, RETENTION_TICKS, Trend};
pub use review::{Authored, Offer, Proposed, Reading, Review};
pub use mesocosm_phenotype::TermTable;
pub use runtime::{
    DEFAULT_MAX_STEPS_PER_ADVANCE, Envelope, Founded, GameSave, Receipt, Refusal, Replayed, Runtime,
};
pub use succession::{Birth, Boundary, Checkpoint, Loss, Occasion};
pub use tactile::{TactileCapsule, TactileError, TactileHit, TactilePick, TactileWorld};
