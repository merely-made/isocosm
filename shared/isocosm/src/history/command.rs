// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Act {
        actor: Id,
        target: Option<Id>,
        process: Key,
        cause: Option<Key>,
    },
    Inspect(Id),
    Release(Id),
    Collect,
    Learn {
        entity: Id,
        event: Key,
    },
    /// The dev source places matter at a site (rulings 271, 344 and 358).
    PlaceMatter {
        site: Id,
        account: Key,
        amount: u64,
    },
    /// An edit to a site's volume (rulings 412, 696 and 739): by a member,
    /// through its ledger, or with none by the dev source.
    Edit {
        site: Id,
        edit: isometer_space::Edit,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        by: Option<Id>,
    },
    /// A game places a member in a patch or room of its site, or takes it
    /// out (rulings 422 and 740).
    Patch {
        entity: Id,
        patch: Option<isometer_space::places::PlaceId>,
    },
    /// A participant joins, a placeless entity (ruling 688).
    Join,
    /// A participant takes up a critter, its bond seeded by the world's
    /// setting (178): a deliberative one to direct, a sophont to drive
    /// (671, 779).
    Take {
        participant: Id,
        critter: Id,
    },
    /// A participant nudges its critter to attend or act (686, 690).
    Nudge {
        participant: Id,
        critter: Id,
        aim: crate::directing::Aim,
        toward: crate::directing::Toward,
        /// The act it names, a process key (784).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        act: Option<Key>,
    },
    /// The dev source forces a birth to `parent` (ruling 782).
    ForceBirth { parent: Id },
    /// A dev kills `entity` (ruling 782).
    Kill { entity: Id },
    /// Places a part's cells exactly, as an authored script proposed for a
    /// committed review offer (ruling 787).
    Express {
        entity: Id,
        part: PartId,
        tracts: Vec<(Key, Vec<crate::mosaic::CellId>)>,
    },
    /// Authored content asserted into the world (ruling 757).
    Assert(crate::asserted::Assertion),
    /// A lineage commits a variant of its development (ruling 752).
    Revise {
        lineage: Key,
        development: crate::rules::Development,
    },
    /// A founder leaves its line for a new one, named (legacy speciation).
    Speciate {
        founder: Id,
        name: Key,
    },
    /// A wound to a body, a part named or drawn by its cells (704 to 719):
    /// how a game hands back a blow it resolved (669).
    Wound {
        entity: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        part: Option<PartId>,
        cells: u32,
    },
    /// An outsider arrives (235, 238).
    Arrive(crate::arrival::Arrival),
    /// A body's geometry admitted (674).
    Embody {
        entity: Id,
        body: isometer_core::BodyDocument,
    },
    /// `by` names `of` (36, 168, 200).
    Name { by: Id, of: Id, name: String },
    /// A record of knowing (771).
    Know(crate::knowing::Knowing),
    /// A deed, an ask or a step in an agreement's life (60, 63, 67).
    Social(crate::social::Social),
    /// One window contribution, admitted and replayed natively (815).
    Window(crate::attention::Change),
    /// Develop an unbodied arrival from its native recipe, conserving stocks.
    Develop { entity: Id, seed: u64 },
    /// Explicit authored geometry or an existing native site (89, 769).
    AttachSite { place: crate::asserted::Place, attachment: crate::site_authoring::Attachment },
    /// Native territory/surface identity and host/place strain selection.
    Spread(crate::spread::Command),
    /// Exact game geometry inputs, consumed only by the native chosen actions.
    AdvanceGeometry { ticks: Tick, resolutions: Vec<crate::operations::Resolution> },
    Project(crate::projects::Change),
    /// Exact step and destination inputs consumed at native act/arrival boundaries.
    AdvanceSpatial {
        ticks: Tick,
        terrain: Vec<crate::spatial::TerrainResolution>,
        steps: Vec<crate::spatial::Resolution>,
        arrivals: Vec<crate::spatial::ArrivalResolution>,
    },
    /// Explicit biological newborn geometry; legacy spatial grammar stays exact.
    AdvanceBirthSpatial {
        ticks: Tick,
        terrain: Vec<crate::spatial::TerrainResolution>,
        steps: Vec<crate::spatial::Resolution>,
        arrivals: Vec<crate::spatial::ArrivalResolution>,
        births: Vec<crate::spatial::BirthResolution>,
    },
    /// A controlled process with exact quantities selected from owned lots.
    ActOwned {
        participant: Id,
        actor: Id,
        target: Option<Id>,
        process: Key,
        cause: Option<Key>,
        selections: Vec<crate::projects::DebitSelection>,
    },
    /// Exact owned debits for a supported direct native operation.
    Owned {
        command: Box<Command>,
        selections: Vec<crate::projects::DebitSelection>,
    },
    /// Authored geometry for an exact dev biological birth, preserving its command.
    BirthSpatial {
        command: Box<Command>,
        births: Vec<crate::spatial::BirthResolution>,
    },
    Party(crate::party::Change),
    /// Explicit genuine-act consequential goals; arcs remain derived.
    Goal(crate::goals::Change),
}
