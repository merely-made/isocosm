//! The DM-authority replication core, as two pure synchronous state
//! machines. They consume [`NetMessage`]s and emit [`Outbound`]s; the
//! transport (in-memory or iroh) is a dumb pump. No async, no I/O, no
//! networking here — that is what makes the whole protocol testable by
//! routing messages in a loop and asserting the peers converge.

use std::collections::HashMap;

use isometry_campaign::{
    GenerationRecord, GenerationRecordError, InventoryError, ItemId, ItemInstance,
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
mod connection;
mod native;
mod native_host;
mod native_storylet;
pub mod native_table;
mod native_time;
pub use native_time::NativeTimeResolved;
mod native_party;
pub use native_party::NativePartyResolved;
mod native_journey;
pub use native_journey::NativeJourneyResolved;
mod native_hook;
pub use native_hook::NativeHookResolved;
mod native_pack;
pub use native_pack::NativePackResolved;
mod native_physical;
pub use native_physical::NativePhysicalResolved;
mod native_items;
mod native_item_metadata;
pub use native_items::NativeItemsResolved;
mod native_institutions;
mod native_secret;
pub use native_secret::NativeSecretResolved;
mod native_composed_storylet;
pub use native_composed_storylet::NativeComposedStoryletResolved;
pub use native_institutions::{InstitutionSender,NativeInstitutionInput,NativeInstitutionQuote,NativeInstitutionsResolved};
pub use native::{NativePublication, NativeTravelResolved, TravelMember, TravelTableResult};
mod travel;

// The 2026-07-24 split moved the bodies into the four modules above while this
// file kept the shared imports they read through `use super::*`. Re-exported
// here so the session layer's surface is unchanged: `lib.rs` still publishes
// exactly these four names.
pub use apply::{GameError, apply_game};
pub use client::ClientSession;
pub use host::HostSession;
pub use travel::{resolve_transition, resolve_transition_for_party};
