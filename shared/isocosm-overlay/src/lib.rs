// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The overlay contract between a game and the Isocosm sim (wing design
//! record ruling 154): a game submits intents stamped for a tick, the sim
//! returns events, each participant's stream derived from their attention
//! set, and a read-only view of each tick, and
//! outcomes a game settles itself come back through the handoff in the
//! sim's own terms.
//!
//! This crate holds only the shape of that boundary. It depends on nothing
//! sim-internal and on no product crate: every type here is a value that
//! round-trips through bytes, and every identifier is an opaque handle
//! minted by the sim, never a pointer into it (accepted reading D18; wing
//! design record §5.2 point 6). A game crate depends on this crate and on
//! the sim; this crate depends on neither.
//!
//! The types at the crate root are wing-level and game-neutral. [`mesocosm`],
//! [`eponym`] and [`vtt`] hold the three games' vocabularies, each a sibling
//! module that changes nothing here (ruling 197).

mod attention;
mod event;
mod handle;
mod handoff;
mod intent;
mod tick;
mod view;

pub mod eponym;
pub mod mesocosm;
pub mod vtt;

pub use attention::{AttentionChange, AttentionSet, Pointable};
pub use event::{EventRecord, EventTopic};
pub use handle::{
    CandidateHandle, EntityHandle, EventHandle, FactionHandle, LineageHandle, ParticipantHandle,
    PlaceHandle,
};
pub use handoff::HandoffEnvelope;
pub use intent::{Intent, IntentEnvelope};
pub use tick::Tick;
pub use view::ViewHandle;
