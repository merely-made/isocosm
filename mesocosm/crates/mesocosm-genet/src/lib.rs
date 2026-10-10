// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's windowed host.
//!
//! Owns the window, the device, and the frame loop. Holds **no game state**:
//! input becomes contract envelopes, the shared runtime steps a native
//! Isocosm session at a fixed rate, and the world comes back out to be drawn.
//! If a rule ever appears in this crate, it is in the wrong crate.
//!
//! The player directs and never drives (wing rulings 671, 679): keys send
//! nudges, checkpoint answers, a speciation and dev intents. The main view is
//! the lens brick tracer's section over the played critter's lifted site.
//! Camera motion is presentation only and never reaches the session's log.

//! Five chrome lanes ride the frame, all through [`chrome`]: the painted
//! minimap ([`hud`]), the cambium vitals panel ([`vitals`]), the individual
//! checkpoint ([`succession`]), the trait board ([`review`]), and the dev lane
//! ([`dev`]). None of them touches the world.

//! **The harness is taproot's** (DT4): this host implements `Automatable`
//! and `Driveable`, so a text `taproot::Scenario` drives a run; see
//! [`app::drive`] and [`app::actions`].

pub mod app;
pub mod chrome;
pub mod dev;
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
