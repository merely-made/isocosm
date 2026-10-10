// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The legacy world's moves, written as native [`Flow`]s (ruling 750).
//!
//! The record is native; this is the legacy world's vocabulary for it, and
//! leaves with that world (755). Its three compartments are accounts by
//! key: the ground's `soil` on the enclosure, a body's `substance` and
//! `reserve` on its organism. The dev source is native's [`Holder::Dev`],
//! and its placements are made by the `PlaceMatter` command, as native's
//! are. A move's lineage and kingdom are read off the body when it moves.

pub use crate::flows::Flow;
use crate::flows::{Composition, Holder, Kind, MadeBy};
use crate::legacy::mesocosm::body::SpeciesId;
use crate::legacy::mesocosm::history::RecordedEvent;
use crate::legacy::mesocosm::organism::{Kingdom, Organism, OrganismId};
use crate::legacy::mesocosm::places::Places;
use crate::schema::{Id, Key};

mod build;
mod read;
pub use read::{Accounts, Trend, WARN_AFTER_TICKS};

/// The site the enclosure's soil is held on.
pub const ENCLOSURE: Id = 0;

/// Where matter sits when it is not moving: the ground, what a body
/// weighs, what it has banked, and the dev source outside the enclosure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Account {
    Soil,
    Substance,
    Reserve,
    Dev,
}

impl Account {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Soil | Self::Dev => "soil",
            Self::Substance => "substance",
            Self::Reserve => "reserve",
        }
    }

    /// Whether this is a body's account.
    pub fn is_body(self) -> bool {
        matches!(self, Self::Substance | Self::Reserve)
    }

    /// The account a side of a move names.
    pub fn of(side: &(Holder, Key)) -> Option<Self> {
        match (side.0, side.1.as_str()) {
            (Holder::Dev, _) => Some(Self::Dev),
            (Holder::Site(_), "soil") => Some(Self::Soil),
            (Holder::Entity(_) | Holder::Part(..), "substance") => Some(Self::Substance),
            (Holder::Entity(_) | Holder::Part(..), "reserve") => Some(Self::Reserve),
            _ => None,
        }
    }

    /// What the dev source issued over `flows`.
    pub fn issued_mg(flows: &[Flow]) -> u64 {
        crate::flows::issued(flows)
    }
}

/// Why matter moved, as the legacy world names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Process {
    Uptake,
    Upkeep,
    Feeding,
    Travel,
    Birth,
    Death,
    Decay,
    Deposit,
    Spill,
    Graft,
    Develop,
    /// A dev tool's placement, made by the `PlaceMatter` command.
    Place,
}

const PROCESSES: [Process; 11] = [
    Process::Uptake,
    Process::Upkeep,
    Process::Feeding,
    Process::Travel,
    Process::Birth,
    Process::Death,
    Process::Decay,
    Process::Deposit,
    Process::Spill,
    Process::Graft,
    Process::Develop,
];

impl Process {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Uptake => "uptake",
            Self::Upkeep => "upkeep",
            Self::Feeding => "feeding",
            Self::Travel => "travel",
            Self::Birth => "birth",
            Self::Death => "death",
            Self::Decay => "decay",
            Self::Deposit => "deposit",
            Self::Spill => "spill",
            Self::Graft => "graft",
            Self::Develop => "develop",
            Self::Place => "PlaceMatter",
        }
    }

    pub fn made_by(self) -> MadeBy {
        match self {
            Self::Place => MadeBy::Command(self.key().into()),
            _ => MadeBy::Process(self.key().into()),
        }
    }

    pub fn of(made_by: &MadeBy) -> Option<Self> {
        match made_by {
            MadeBy::Command(k) => (k == Self::Place.key()).then_some(Self::Place),
            MadeBy::Process(k) => PROCESSES.into_iter().find(|p| p.key() == k),
        }
    }
}

/// A kingdom's native key.
pub const fn kingdom_key(kingdom: Kingdom) -> &'static str {
    match kingdom {
        Kingdom::Producer => "kingdom:flora",
        Kingdom::Consumer => "kingdom:fauna",
        Kingdom::Decomposer => "kingdom:myco",
    }
}

/// The kingdom a native key names.
pub fn kingdom_of(key: &str) -> Option<Kingdom> {
    [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer]
        .into_iter()
        .find(|k| kingdom_key(*k) == key)
}

/// A lineage's native key.
pub fn lineage_key(species: SpeciesId) -> Key {
    format!("lineage:{}", species.0)
}

/// The lineage a native key names.
pub fn lineage_of(key: &str) -> Option<SpeciesId> {
    key.strip_prefix("lineage:")?.parse().ok().map(SpeciesId)
}

/// One side of a move, when that side is a body: the organism, its lineage
/// and its true kingdom, read off anatomy rather than any guise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub organism: OrganismId,
    pub lineage: SpeciesId,
    pub kingdom: Kingdom,
}

impl Subject {
    /// One anatomy read; a hot loop takes it once per organism per tick.
    pub fn of(organism: &Organism) -> Self {
        Self {
            organism: organism.id,
            lineage: organism.species,
            kingdom: organism.kingdom(),
        }
    }

    pub fn id(self) -> Id {
        u64::from(self.organism.0)
    }

    pub fn kind(self) -> Kind {
        Kind {
            lineage: lineage_key(self.lineage),
            kingdom: kingdom_key(self.kingdom).into(),
        }
    }
}

/// The world's one-tick flow buffer: beside the world, never in it, so
/// draining it cannot move a snapshot or hash. Equal to every other.
#[derive(Clone, Debug, Default)]
pub struct Ledger {
    tick: u64,
    records: Vec<Flow>,
}

impl PartialEq for Ledger {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Ledger {}

impl Ledger {
    /// Starts a tick, discarding anything the last one left undrained.
    pub fn open(&mut self, tick: u64) {
        self.tick = tick;
        self.records.clear();
    }

    /// Records a move at this tick; a move of nothing is no record.
    pub fn record(&mut self, mut flow: Flow) {
        if flow.amount == 0 {
            return;
        }
        flow.tick = self.tick;
        self.records.push(flow);
    }

    pub fn records(&self) -> &[Flow] {
        &self.records
    }

    pub fn take(&mut self) -> Vec<Flow> {
        std::mem::take(&mut self.records)
    }
}

/// The tick's two record streams at one commit point: the causal events
/// and the moves.
pub struct Records<'a> {
    tick: u64,
    places: Option<&'a Places>,
    events: &'a mut Vec<RecordedEvent>,
    flows: &'a mut Ledger,
}

impl<'a> Records<'a> {
    pub fn new(
        tick: u64,
        places: Option<&'a Places>,
        events: &'a mut Vec<RecordedEvent>,
        flows: &'a mut Ledger,
    ) -> Self {
        Self {
            tick,
            places,
            events,
            flows,
        }
    }

    /// Records that something happened to somebody, and where.
    pub fn event(&mut self, position: [i32; 3], event: crate::legacy::mesocosm::history::Event) {
        let place = self.places.and_then(|places| places.at(position));
        let envelope = crate::legacy::mesocosm::history::Envelope::new(self.tick, place, event);
        self.events.push(envelope);
    }

    /// Records that matter moved. The native record keeps holders, not
    /// positions, so `_at` goes unread until the world moves (755).
    pub fn flow(&mut self, _at: [i32; 3], flow: Flow) {
        self.flows.record(flow);
    }
}

/// An untyped composition, where the legacy world reported one.
fn untyped(amount: u64) -> Option<Composition> {
    Some(Composition::untyped(amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> Subject {
        Subject {
            organism: OrganismId(1),
            lineage: SpeciesId(3),
            kingdom: Kingdom::Producer,
        }
    }

    #[test]
    fn a_ledger_holds_one_tick_and_is_equal_to_any_other() {
        let mut ledger = Ledger::default();
        ledger.open(7);
        ledger.record(Flow::uptake(subject(), Account::Reserve, 5));
        ledger.record(Flow::uptake(subject(), Account::Reserve, 0));
        assert_eq!(ledger.records().len(), 1, "a move of nothing is no record");
        assert_eq!(ledger.records()[0].tick, 7);
        assert_eq!(ledger, Ledger::default());
        ledger.open(8);
        assert!(ledger.records().is_empty());
    }

    #[test]
    fn every_process_reads_back_from_its_maker() {
        for p in PROCESSES.into_iter().chain([Process::Place]) {
            assert_eq!(Process::of(&p.made_by()), Some(p));
        }
        for k in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
            assert_eq!(kingdom_of(kingdom_key(k)), Some(k));
        }
    }
}
