// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Eponym's world on Isocosm (wing ruling 755): a native session founded by
//! Eponym's pack, the one site it is played in lifted by isometer (670,
//! 696), and Eponym's meanings over its sites. A carve is an edit the sim
//! keeps, made through its carver's ledger; an inherited site a fact the
//! sim keeps. Saves carry genesis facts and ordered intents; restore
//! regrows the world and replays them.

use std::collections::BTreeMap;

use isocosm::history::Command;
use isocosm::schema::Id;
use isocosm::{Execution, Session};
use isocosm_overlay::eponym::{ActTarget, Actuation, EponymIntent, TimedAct};
use isocosm_overlay::{ActKey, EntityHandle, WorldPoint};
use isometer_core::ground::Ground;
use isometer_core::snapshot::{self, hash_bytes};
use isometer_space::volume::Volume;
use isometer_space::{Edit, Op, edit::Shape};
use serde::{Deserialize, Serialize};

use crate::identity::{SubjectId, Tick};
use crate::{HistoryFactId, NativeFixtureProfile, NativeFixtureRecipe, SiteKind, SlotId, WorldMap};

pub const GENERATOR_VERSION: u32 = 4;
/// Rock held under the lifted window's lowest soil.
const DEPTH: i64 = 8;

/// `side` sites a side, each `2 * extent` cells across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub side: u16,
    pub extent: i32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            side: 8,
            extent: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldIntent {
    Carve {
        tick: Tick,
        by: SubjectId,
        centre: [i32; 3],
        radius: i32,
    },
    InheritSite {
        tick: Tick,
        slot: SlotId,
        kind: SiteKind,
        fact: HistoryFactId,
    },
}

impl WorldIntent {
    pub const fn tick(self) -> Tick {
        match self {
            Self::Carve { tick, .. } | Self::InheritSite { tick, .. } => tick,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldEvent {
    Carved {
        tick: Tick,
        by: SubjectId,
        centre: [i32; 3],
        radius: i32,
        removed: u32,
        ground_revision: u64,
    },
    SiteInherited {
        tick: Tick,
        slot: SlotId,
        kind: SiteKind,
        fact: HistoryFactId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSave {
    pub generator_version: u32,
    pub seed: u64,
    pub config: WorldConfig,
    pub base_hash: u64,
    pub intents: Vec<WorldIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldError {
    InvalidSide,
    InvalidExtent,
    InvalidRadius,
    MissingSlot(SlotId),
    OutOfOrder {
        previous: Tick,
        next: Tick,
    },
    GeneratorDiverged {
        saved: u32,
        current: u32,
    },
    BaseDiverged {
        saved: u64,
        regrown: u64,
    },
    /// The sim refused what was asked of it.
    Refused,
    /// An intent named a sophont its participant does not play (152).
    NotPlayed,
    /// The contract carried what native cannot take yet (794).
    Unbuilt(&'static str),
    /// Play offered at the founding door, which takes none (794).
    NotFounding,
    /// A fixture recipe was offered outside its declared founding profile.
    NotFixture,
    Encode,
    Decode,
}

#[derive(Clone, Debug)]
pub struct World {
    seed: u64,
    config: WorldConfig,
    generator_version: u32,
    base_hash: u64,
    session: Session,
    site: Id,
    volume: Volume,
    map: WorldMap,
    /// Each subject's entity in the sim.
    subjects: BTreeMap<SubjectId, Id>,
    intents: Vec<WorldIntent>,
    events: Vec<WorldEvent>,
    last_tick: Option<Tick>,
}

impl World {
    pub fn generate(seed: u64, config: WorldConfig) -> Result<Self, WorldError> {
        Self::generate_version(seed, config, GENERATOR_VERSION)
    }
    pub fn generate_native_fixture(
        seed: u64, config: WorldConfig, profile: NativeFixtureProfile,
    ) -> Result<Self, WorldError> {
        let version = match profile {
            NativeFixtureProfile::PreciseCombatV1 => crate::founding::fixture::GENERATOR_VERSION,
        };
        Self::generate_version(seed, config, version)
    }

    // Version 3 is an explicit source proposal, not a claimed released archive format.
    fn generate_version(seed: u64, config: WorldConfig, version: u32) -> Result<Self, WorldError> {
        let movement = match version {
            2 => isocosm::schema::Movement::ImmediateV3,
            3 | GENERATOR_VERSION | crate::founding::fixture::GENERATOR_VERSION => isocosm::schema::Movement::Journey,
            _ => return Err(WorldError::GeneratorDiverged { saved:version, current:GENERATOR_VERSION }),
        };
        if !(2..=16).contains(&config.side) {
            return Err(WorldError::InvalidSide);
        }
        if !(8..=1 << 19).contains(&config.extent) {
            return Err(WorldError::InvalidExtent);
        }
        let side = config.extent as u64 * 2;
        let founding = match version {
            GENERATOR_VERSION => crate::founding::genesis,
            crate::founding::fixture::GENERATOR_VERSION => crate::founding::fixture_genesis,
            _ => crate::founding::legacy_genesis_v2,
        };
        let genesis = founding(seed, u32::from(config.side), side)
            .map_err(|_| WorldError::InvalidExtent)?;
        let base_hash = hash_bytes(isocosm::digest(&genesis).as_bytes());
        let session =
            Session::new_with_movement(genesis, Execution::Individuals, movement).map_err(|_| WorldError::Refused)?;
        let map = WorldMap::generate(&session.sim);
        let site = map
            .slots_of_kind(SiteKind::Settlement)
            .next()
            .map_or(0, |s| s.site);
        let end = [side as i64; 2];
        let volume = session
            .sim
            .volume(site, [0, 0], end, DEPTH)
            .map_err(|_| WorldError::Refused)?;
        Ok(Self {
            seed,
            config,
            base_hash,
            generator_version: version,
            session,
            site,
            volume,
            map,
            subjects: BTreeMap::new(),
            intents: Vec::new(),
            events: Vec::new(),
            last_tick: None,
        })
    }

    pub const fn generator_version(&self) -> u32 { self.generator_version }
    pub(crate) const fn developed_bodies(&self) -> bool {
        matches!(self.generator_version, GENERATOR_VERSION | crate::founding::fixture::GENERATOR_VERSION)
    }
    pub(crate) fn select_native_recipe(&mut self, recipe: Option<NativeFixtureRecipe>) -> Result<(), WorldError> {
        if self.generator_version != crate::founding::fixture::GENERATOR_VERSION {
            return if recipe.is_none() { Ok(()) } else { Err(WorldError::NotFixture) };
        }
        let development = crate::founding::fixture::development(self.sim().genesis(), recipe)
            .ok_or(WorldError::Refused)?;
        self.run(Command::Revise { lineage: crate::founding::SOPHONT.into(), development })?;
        Ok(())
    }
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    pub const fn config(&self) -> WorldConfig {
        self.config
    }
    pub const fn base_hash(&self) -> u64 {
        self.base_hash
    }

    pub(crate) fn borrowed_contract(&mut self) -> crate::contract::Borrowed<'_> {
        crate::contract::Borrowed::over_volume(&mut self.session, self.site, &self.volume)
    }
    /// The native session the world runs in.
    pub fn session(&self) -> &Session {
        &self.session
    }
    pub fn sim(&self) -> &isocosm::Simulation {
        &self.session.sim
    }
    /// The site Eponym is played in.
    pub fn site(&self) -> Id {
        self.site
    }
    /// The played site, lifted with every edit.
    pub fn volume(&self) -> &Volume {
        &self.volume
    }
    pub fn ground(&self) -> &Ground {
        self.volume.ground()
    }
    pub fn map(&self) -> &WorldMap {
        &self.map
    }
    pub fn intents(&self) -> &[WorldIntent] {
        &self.intents
    }
    pub fn events(&self) -> &[WorldEvent] {
        &self.events
    }
    /// The sim's entity for `subject`.
    pub fn entity(&self, subject: SubjectId) -> Option<Id> {
        self.subjects.get(&subject).copied()
    }
    pub fn subjects(&self) -> impl Iterator<Item = (SubjectId, Id)> + '_ {
        self.subjects.iter().map(|(s, e)| (*s, *e))
    }
    pub fn bind(&mut self, subject: SubjectId, entity: Id) {
        self.subjects.insert(subject, entity);
    }

    /// Runs `command` in the sim, logged in its session: the contract's
    /// lowering and the founding door only.
    pub(crate) fn run(&mut self, command: Command) -> Result<String, WorldError> {
        self.session
            .command(command)
            .map_err(|_| WorldError::Refused)
    }

    /// The host's founding door: arrivals, bodies admitted and authored
    /// content, never play, which crosses the contract (794).
    pub fn found(&mut self, command: Command) -> Result<String, WorldError> {
        match command {
            Command::Arrive(_) | Command::Embody { .. } | Command::Develop { .. }
            | Command::Assert(_) => self.run(command),
            Command::Edit { by: None, .. } => self.run(command),
            _ => Err(WorldError::NotFounding),
        }
    }

    /// The sim's clock moves one tick.
    pub fn tick(&mut self) -> Result<(), WorldError> {
        self.session
            .advance(1)
            .map(|_| ())
            .map_err(|_| WorldError::Refused)
    }

    pub fn apply(&mut self, intent: WorldIntent) -> Result<WorldEvent, WorldError> {
        let tick = intent.tick();
        if let Some(previous) = self.last_tick
            && tick < previous
        {
            return Err(WorldError::OutOfOrder {
                previous,
                next: tick,
            });
        }
        let event = match intent {
            WorldIntent::Carve {
                tick,
                by,
                centre,
                radius,
            } => {
                if radius < 0 {
                    return Err(WorldError::InvalidRadius);
                }
                let at = crate::walking::to_site(&self.volume, centre);
                let shape = Shape::Sphere {
                    centre: at,
                    radius: radius as u64,
                };
                let edit = Edit {
                    op: Op::Carve,
                    shape,
                };
                match self.entity(by) {
                    Some(body) => {
                        let act = TimedAct {
                            act: ActKey(crate::contract::CARVE.into()),
                            target: Some(ActTarget::Region {
                                centre: WorldPoint(centre),
                                radius: radius as u32,
                            }),
                        };
                        let drive = Actuation {
                            body: EntityHandle(body),
                            motion: None,
                            acts: vec![act],
                        };
                        self.submit(None, &EponymIntent::Drive(drive))?;
                    },
                    None => {
                        self.found(Command::Edit {
                            site: self.site,
                            edit: edit.clone(),
                            by: None,
                        })?;
                    },
                }
                let atlas = self.session.sim.atlas().map_err(|_| WorldError::Refused)?;
                let removed = self
                    .volume
                    .apply(&atlas, self.site, &edit)
                    .map_err(|_| WorldError::Refused)?;
                let ground_revision = self.volume.revision();
                WorldEvent::Carved {
                    tick,
                    by,
                    centre,
                    radius,
                    removed,
                    ground_revision,
                }
            },
            WorldIntent::InheritSite {
                tick,
                slot,
                kind,
                fact,
            } => {
                if !self.map.inherit(slot, kind, fact) {
                    return Err(WorldError::MissingSlot(slot));
                }
                let assertion = isocosm::asserted::Assertion::Fact(isocosm::asserted::Fact {
                    about: format!("site:{}", slot.site),
                    key: format!("inherit:{}", fact.0),
                    kind: crate::founding::SITE_FACT.into(),
                    text: kind.name().into(),
                    tags: Default::default(),
                });
                self.found(Command::Assert(assertion))?;
                WorldEvent::SiteInherited {
                    tick,
                    slot,
                    kind,
                    fact,
                }
            },
        };
        self.intents.push(intent);
        self.events.push(event);
        self.last_tick = Some(tick);
        Ok(event)
    }

    /// The sim's state hash beside the game's own record.
    pub fn state_hash(&self) -> Result<u64, WorldError> {
        let record = (self.base_hash, &self.subjects, &self.intents, &self.events);
        let game = snapshot::encode(&record).map_err(|_| WorldError::Encode)?;
        Ok(hash_bytes(&game) ^ self.session.sim.state_hash())
    }

    pub fn save_record(&self) -> WorldSave {
        WorldSave {
            generator_version: self.generator_version,
            seed: self.seed,
            config: self.config,
            base_hash: self.base_hash,
            intents: self.intents.clone(),
        }
    }

    pub fn save(&self) -> Result<Vec<u8>, WorldError> {
        snapshot::encode(&self.save_record()).map_err(|_| WorldError::Encode)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, WorldError> {
        let save: WorldSave = snapshot::decode(bytes).map_err(|_| WorldError::Decode)?;
        Self::restore_record(save)
    }

    pub fn restore_record(save: WorldSave) -> Result<Self, WorldError> {
        if ![2, 3, GENERATOR_VERSION, crate::founding::fixture::GENERATOR_VERSION].contains(&save.generator_version) {
            return Err(WorldError::GeneratorDiverged {
                saved: save.generator_version,
                current: GENERATOR_VERSION,
            });
        }
        let mut world = Self::generate_version(save.seed, save.config, save.generator_version)?;
        if save.base_hash != world.base_hash {
            return Err(WorldError::BaseDiverged {
                saved: save.base_hash,
                regrown: world.base_hash,
            });
        }
        for intent in save.intents {
            world.apply(intent)?;
        }
        Ok(world)
    }
}

#[cfg(test)]
mod journey_tests;
