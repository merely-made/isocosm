//! The DM-authority replication core, as two pure synchronous state
//! machines. They consume [`NetMessage`]s and emit [`Outbound`]s; the
//! transport (in-memory or iroh) is a dumb pump. No async, no I/O, no
//! networking here — that is what makes the whole protocol testable by
//! routing messages in a loop and asserting the peers converge.

use std::collections::HashMap;

use isocosm::legacy::campaign::{
    FactionMove, GenerationRecord, GenerationRecordError, InventoryError, ItemId, ItemInstance,
    ItemModifierReveal, MapScale, StoryletEffect, StoryletProposal, WorldError, WorldEvent,
    WorldFact,
};
use isometry_campaign::CampaignStore;
use isometry_campaign::{LowerDraftMap, LowerMap};
use isometry_core::{EventError, TileCoord, TokenId, apply};
use muniment::Journal;

use crate::protocol::{
    ActionIntent, FNV_OFFSET, GameEvent, GameSnapshot, NetMessage, Outbound, PROTOCOL_VERSION,
    PeerId, Recipient, RequestId, fold_event,
};

mod apply;
mod client;
mod host;
mod messages;
mod travel;

// The 2026-07-24 split moved the bodies into the four modules above while this
// file kept the shared imports they read through `use super::*`. Re-exported
// here so the session layer's surface is unchanged: `lib.rs` still publishes
// exactly these four names.
pub use apply::{GameError, apply_game};
pub use client::ClientSession;
pub use host::HostSession;
pub use travel::{resolve_transition, resolve_transition_for_party};
