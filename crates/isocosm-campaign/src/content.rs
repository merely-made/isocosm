//! The VTT's content and table records; native state stays in Cosmograph.
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
//! `CampaignWorld` folds native assertions and carries table metadata.
//! Faction acts are native polity processes, never an entropy-tape turn here.

mod chronicle;
mod construction;
mod fact;
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
