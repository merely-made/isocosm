// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::tick::Tick;

/// An opaque key naming one kind of event a game can subscribe to. The sim
/// defines what keys exist, the same way it names processes and accounts by
/// string key (`isocosm::schema::Key`, not depended on here); this crate
/// does not enumerate them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventTopic(pub String);

/// What a game is telling the sim it wants to hear about.
///
/// **Provisional, not ruled** (see the crate README's "Open forks"): this is
/// the simplest shape, an explicit topic set. The wing design record's "one
/// attention set per player" (§5.2 point 4) suggests subscription may
/// instead be *derived* from that set rather than named separately; the
/// attention set itself has no type yet.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSubscription {
    pub topics: BTreeSet<EventTopic>,
}

/// One receipt or record entry, delivered because it matched a subscription.
///
/// The payload is opaque bytes rather than a typed enum: the events a sim
/// process can produce are the sim's own vocabulary, and a typed payload
/// here would make this crate depend on it. A game (or a thin per-game
/// layer above this crate) decodes the payload it asked for by topic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecord {
    pub tick: Tick,
    pub topic: EventTopic,
    pub payload: Vec<u8>,
}
