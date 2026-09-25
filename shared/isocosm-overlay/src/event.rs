// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::tick::Tick;

/// An opaque key naming one kind of event. The sim defines what keys exist,
/// the same way it names processes and accounts by string key
/// (`isocosm::schema::Key`, not depended on here), and a game decodes a
/// payload by its topic.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventTopic(pub String);

/// One receipt or record entry in a participant's stream.
///
/// Nobody subscribes: the stream is derived from the participant's
/// [`crate::AttentionSet`] (ruling 204), carrying events that touch who they
/// play, what they pin, what they examine and the game's own care, with
/// record entries that reach the played entity; in survival mode, only what
/// the played critter can know (rulings 180, 213). Each peer derives its own
/// participant's stream from the shared sim, so the stream never crosses the
/// network.
///
/// The payload is opaque bytes rather than a typed enum: the events a sim
/// process can produce are the sim's own vocabulary, and a typed payload
/// here would make this crate depend on it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecord {
    pub tick: Tick,
    pub topic: EventTopic,
    pub payload: Vec<u8>,
}
