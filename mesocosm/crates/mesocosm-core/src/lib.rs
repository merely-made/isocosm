// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's simulation core.
//!
//! **The core owns game state; hosts only project it.** A host sends
//! [`Intent`]s and reads the world back. It never mutates world state, and it
//! never re-implements a rule.
//!
//! # Determinism
//!
//! A [`World`] is a pure function of its seed and the ordered intents applied
//! to it. Three properties hold this up, and each is a constraint on anything
//! added here later:
//!
//! - **No ambient inputs.** No clock reads, no environment, no I/O.
//! - **No unordered iteration reaching the simulation.** Collections are
//!   ordered vectors, never hash maps.
//! - **All randomness from the seeded stream** in [`rng`].
//!
//! There is a fourth, and it is the one that buys the most: **the core is
//! integer-only.** Voxel coordinates, masses in milligrams, and quarter-turn
//! rotations are exact on every platform, so a replay cannot diverge on a
//! different machine's floating-point behaviour. Float physics belongs to a
//! host, on the far side of this boundary.
//!
//! One discipline buys five things, which is why it is worth the constraint:
//! co-op, replay, save/load, time-travel debugging, and comparing two hosts
//! are the same mechanism seen from different angles.
//!
//! # Shape
//!
//! ```text
//! World  ── seed + ordered intents ──▶ World'
//!   │
//!   ├── BodyDocument   parts, attachment frames, per-part provenance
//!   ├── Morsels        loose matter available to metabolize
//!   └── snapshot()     the whole world, captured in one call
//! ```

pub mod axis;
pub mod cohort;
pub mod deep_time;
pub mod development;
pub mod discovery;
pub mod effect_experiment;
pub mod effect_pack;
pub mod embodiment;
pub mod flow;
pub mod functions;
pub mod graft;
pub mod growth;
pub mod history;
pub mod matter;
pub mod organism;
pub mod phenotype;
pub mod places;
pub mod pressure;
pub mod process;
pub mod program;
pub mod record;
pub mod rng;
pub mod rules;
pub mod score;
pub mod snapshot;
pub mod species;
pub mod voxel_profile;
pub mod world;

pub mod chronicle;

// ---------------------------------------------------------------------------
// The isometer-core re-export block.
//
// `body.rs`, `anatomy.rs`, `plan.rs`, `wire.rs`, the brick ground and the
// codec seam moved to `isometer-core` (family plan step 6, 2026-09-14). Every
// moved item is re-exported here at the path it had before the move, so
// `mesocosm-runtime`, `mesocosm-views`, `mesocosm-genet`, `paredros-world`,
// `paredros-social`, `paredros-sortie` and `wing-integration` name it exactly
// as they did. `places::{AIR, BRICK, Brick, Ground, ROCK, SOIL, SURFACE_BAND}`
// and `snapshot::{encode, decode, hash_bytes}` are restated in their own
// modules for the same reason.
// ---------------------------------------------------------------------------
pub use isometer_core::{
    Aabb, AttachError, Attachment, BodyDocument, Origin, Part, PartId, Provenance, SpeciesId,
    VolumeRef, Yaw,
};
pub use isometer_core::{BodyPlan, Facing, Role, Symmetry, classify};
pub use isometer_core::{WireError, frame, unframe};
pub use isometer_core::{anatomy, body, plan, wire};

pub use axis::{Appendage, AppendageStep, ChainFacing, Recipe, Soma, Tagma, Unspeakable};
pub use chronicle::{Chronicle, Consequence, Deed, PartOrigin, generate};
pub use cohort::{Cohort, CohortKey, CohortMember};
pub use deep_time::DeepTimeError;
pub use development::{
    DevelopmentError, PALETTE_SHAPES, PartPalette, PartTemplate, RoleShapes, develop_body,
    minimum_body_mass_mg,
};
pub use discovery::{
    Candidate, Condition, ConditionId, Discovery, Evidence, Input, Miss, Observation, Source,
    Stress,
};
pub use embodiment::{bearing_parts, embodied, expressed_traits};
pub use flow::{
    Account, Accounts, Carrier, Envelope, FlowEvent, Ledger, RecordedEvent, RecordedFlow, Subject,
    Trend, WARN_AFTER_TICKS,
};
pub use graft::{Affinity, Crossing, Domain, Verdict};
pub use growth::{Growth, resolve};
pub use history::{Ending, Event, History, MealKind, Passing};
pub use organism::{
    BodyOrgans, FaunaDecisionTrace, FaunaDrive, FaunaDriveScores, FaunaPolicy, FaunaSenses,
    FaunaTraits, Kingdom, Organism, OrganismId, Signal, Stage, Tally,
};
pub use phenotype::{
    Aim, AllocationProposal, Arrangement, BodyPhenotype, Branch, CellId, Cutting, Development,
    Explanation, Expressed, Graftage, Instruction, Lowering, Mosaic, ProposedSite, Refusal, Site,
    SiteId, SiteReading, arrange,
};
pub use places::{Place, PlaceId, Places};
pub use pressure::{AUTHORED, Force, HEAVY_DEEP, LONG_YEAR, Pressure, TIDAL_SHELF, WorldProfile};
pub use process::{
    BULK_REACH, BodyProcesses, Capability, DefinitionDigest, FeedingMode, IntakePort, NATIVE_ABI,
    NisKind, Process, ProcessDef, ProcessId, ProcessRef, Registry, Seeding, ShapeProcesses, Unmet,
};
pub use program::{
    Citation, Conditions, DeclaredSite, Filial, Founder, Preview, Program, Revision, RevisionId,
    Unexpressed,
};
pub use record::{Feat, Mark, Scale, WorldRecord};
pub use rng::Rng;
pub use rules::{DeepTimeSpan, RulesetDigest, TROPHIC_GRAMMAR_REVISION, WorldRules};
pub use score::{Reading, readings};
pub use snapshot::{SnapshotError, restore, restore_under, snapshot, state_hash};
pub use species::{InitialTissueRecipe, Lineages, Species, TissueRecipeError};
pub use world::{
    ExpressionPreview, Founding, Gland, Graft, GraftPreview, INSTINCT_IDLE_TICKS, Ineligible,
    Intent, Offer, Outcome, PLACE_MATTER_MAX_MG, Placement, Prospect, Rejection, Route,
    STARVED_UPKEEP_TICKS, Score, Unrevised, Untakeable, World,
};
