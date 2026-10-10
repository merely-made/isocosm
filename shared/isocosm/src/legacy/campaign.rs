//! Isometry's campaign-state layer (worldbuilding plan W0).
//!
//! Two stores, by design: the **host-private campaign store** holds the
//! GM layer (secret facts, hidden modifiers, unrevealed history), and the
//! **shared log** carries only public projections. Session convergence is
//! a rolling hash over byte-identical event logs, so a secret must never
//! be serialized into a `GameSnapshot` or `GameEvent`; a reveal is an
//! ordinary event that *publishes* a fact. This crate is pure data:
//! no I/O, no net, no substrate geometry. The substrate stores and
//! displays these objects; system plugins interpret them.
//!
//! The host-private store and the proposal lifecycle went back to the VTT's
//! `isometry-campaign` (ruling 669); what remains waits on its families (the
//! families re-expression plan's Progress, 2026-10-09).

mod chronicle;
mod construction;
mod fact;
mod faction;
mod generator;
mod item;
mod map;
mod overmap;
mod pack;
mod world;

pub use chronicle::{
    Arrival, CHRONICLE_SCHEMA, Chronicle, ChronicleError, Deed, LOST_PART, PartOrigin, VESSEL,
};
pub use construction::{CONSTRUCTION_SCHEMA, ConstructionError, ConstructionProposal};
pub use fact::{RevealCondition, SecretFact, Visibility, WorldFact};
pub use faction::{FactionMove, FactionVerb};
pub use generator::{
    CastRoleRequest, EntropyTape, GenValue, GenValueError, GenerationRecord, GenerationRecordError,
    GeneratorFixture, GeneratorRequest, ItemProposal, MapPatchProposal, NpcProposal,
};
pub use item::{
    EquipmentSlot, HiddenItemModifier, Inventory, InventoryError, ItemId, ItemInstance,
    ItemModifier, ItemModifierKind, ItemModifierReveal,
};
pub use map::{
    EncounterAnchor, LocalMapProposal, MAX_GENERATED_MAP_EDGE, MapCellProposal, MapPoint,
    MapProposalError, MapScale, MapTransition, SpawnZone,
};
pub use overmap::{Overmap, OvermapEdge, OvermapNode};
pub use pack::{
    BeatEntry, CONTENT_PACK_FORMAT, ContentPackError, ContentPackManifest, GeneratorChoice,
    GeneratorEntry, GeneratorLockPreset,
};
pub use world::{
    CampaignDraft, CampaignWorld, DraftMap, MapInhabitant, RoleSlot, StoryletEffect, StoryletError,
    StoryletProposal, StoryletRequirements, StoryletResolution, WorldError, WorldEvent,
};
