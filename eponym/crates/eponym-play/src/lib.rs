// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Eponym's play over Isocosm (wing rulings 239, 597, 669, 671 and 755):
//! what the game keeps once its world runs on the sim. The world, bodies,
//! needs, wounds, items, edits, deeds and knowing are the sim's; this crate
//! drives a sapient player's subject through native commands and keeps
//! what the sim's laws exclude, handed back to the game (669): motion,
//! timed actions, strike resolution, techniques, sheets, admitted anatomy
//! snapshots, control and the glyph reading's game half. Population,
//! projects and the autonomous round retired with the legacy world: the
//! sim advances every living member, and its record is native (770, 771).

pub mod anatomy;
pub mod bodies;
pub mod combat;
mod equipment;
pub mod fixtures;
pub mod founding;
pub mod glyphs;
pub mod identity;
pub mod items;
mod motion;
mod movement;
mod movement_profile;
mod navigation;
mod session;
mod sites;
mod state;
mod subject_sheet;
mod technique;
pub mod timed_action;
mod transitions;
pub mod walking;
mod world;

pub use anatomy::{
    Anatomies, AnatomyError, AnatomyRecord, MAX_ANATOMY_COORDINATE, MAX_ANATOMY_PARTS,
    MAX_ANATOMY_WORLD_COORDINATE, part_bounds,
};
pub use bodies::{
    Bodies, Body, BodyError, BodyProfile, BodyRecord, MAX_NEED, MOBILITY_WOUND, Name, Needs, SAFE_FALL,
};
pub use combat::{
    COMBAT_RULES_REVISION, CombatError, CombatRules, MAX_COMBAT_REACH, MAX_VOLLEY_STRIKES,
    ResolvedStrike, StrikeOutcome,
};
pub use equipment::AttachmentView;
pub use items::{Item, ItemError, ItemId, ItemKind, ItemLocation, ItemPlaces, Items};
pub use motion::{
    MOTION_SCALE, MotionError, MotionInput, MotionOutcome, MotionPose, MotionRules, MotionSolver,
    Solve,
};
pub use movement::{Movement, MovementError, MovementEvent, MovementIntent, MovementSave};
pub use movement_profile::{
    MOVEMENT_PROFILE_REVISION, MotionEnvelope, MovementProfile, MovementProjection, SupportBand,
};
pub use navigation::{Navigation, NavigationError};
pub use session::{
    MAX_CONTROL_INTENTS, MAX_GAME_INTENTS, MAX_SESSION_BYTES, SESSION_VERSION, Session,
    SessionError, SessionLimits, SessionSave,
};
pub use sites::{HistoryFactId, Layer, Site, SiteKind, SiteSource, SlotId, WorldMap};
pub use state::GameState;
pub use subject_sheet::{
    ActionRow, CapabilityRow, EquipmentRow, PartRow, ResourceRow, SubjectSheet, SubjectSheetInput,
};
pub use technique::{
    ActionBlocker, ActionQuery, AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment,
    BindingBlocker, BindingKind, BindingQuery, EquipmentFunction, EquipmentProjection,
    PartCapability, PartFunction, ResourceCost, ResourceKind, ResourceReserve, SourceQuery,
    SubjectBody, TechniqueId, TechniqueInputs, TechniqueKnowledge, arrest_fall,
};
pub use transitions::{
    COMBAT_GAME_STATE_VERSION, CanonRevisionCause, DeathCause, GAME_STATE_VERSION, GameError,
    GameEvent, GameIntent, GameSave, LEGACY_GAME_STATE_VERSION, MOTION_GAME_STATE_VERSION,
    PROFILE_GAME_STATE_VERSION,
};
pub use world::{
    GENERATOR_VERSION, World, WorldConfig, WorldError, WorldEvent, WorldIntent, WorldSave,
};
