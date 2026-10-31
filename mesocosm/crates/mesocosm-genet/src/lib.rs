// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's windowed host.
//!
//! The window, device and frame loop are isomere's Assembly. Input becomes
//! core envelopes, the shared runtime steps a native
//! Cosmograph session at a fixed rate, and the world comes back out to be drawn.
//! If a rule ever appears in this crate, it is in the wrong crate.
//!
//! The player directs and never drives (wing rulings 671, 679): keys send
//! nudges, checkpoint answers, a speciation and dev intents. The main view is
//! the lens brick tracer's section over the played critter's lifted site.
//! Camera motion is presentation only and never reaches the session's log.

//! The actual Section is a same-device viewport producer. Cambium owns the
//! vitals, checkpoint, trait board and dev controls; the existing painted
//! minimap is registered directly as a leaf. These views change no world state.

//! Mesquite drives taproot scenarios through that same document and viewport;
//! keys and semantic actions share [`app::actions`].

pub mod app;
pub mod chrome;
pub mod borrowed_section;
pub mod dev;
mod directing;
pub mod hud;
pub mod input;
pub mod maps;
pub mod played;
pub mod review;
pub mod section;
pub mod succession;
pub mod vitals;

pub use app::{Host, HostConfig};
pub use played::{PlayedReceipt, PlayedTrace};
