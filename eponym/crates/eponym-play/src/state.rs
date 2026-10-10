// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The game's driving state over the sim (wing rulings 239, 671 and 755):
//! a sapient player's subject acts through native commands, its body,
//! needs, wounds, items and edits the sim's, while motion, admitted
//! anatomy snapshots, timed actions and strike resolution stay the game's
//! (597, 669). Each accepted intent is one tick of the sim's clock.

use std::collections::BTreeMap;

use isocosm::history::Command;
use isocosm::schema::Id;
use isometer_core::snapshot::{self, hash_bytes};

use crate::bodies::{BodyError, BodyRecord};
use crate::identity::{SubjectId, Tick};
use crate::items::{ItemPlaces, Items};
use crate::{
    Anatomies, AnatomyError, AnatomyRecord, Bodies, GameError, GameEvent, GameIntent, Movement,
    MovementProfile, MovementProjection, World,
};

mod acts;
mod combat;
mod harm;
mod motion;
mod save;

#[derive(Clone, Debug)]
pub struct GameState {
    world: World,
    movement: Movement,
    anatomies: Anatomies,
    records: BTreeMap<SubjectId, BodyRecord>,
    places: ItemPlaces,
    intents: Vec<GameIntent>,
    events: Vec<GameEvent>,
    /// Read from the sim after every accepted intent.
    bodies: Bodies,
    items: Items,
    /// The game's motion solver (rulings 233 and 597): handed by the host,
    /// never saved.
    motion: crate::MotionSolver,
}

/// Two states are one where they took the same intents to the same hash.
impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.intents == other.intents && self.state_hash().ok() == other.state_hash().ok()
    }
}

impl Eq for GameState {}

impl GameState {
    pub fn new(world: World) -> Self {
        let mut state = Self {
            world,
            movement: Movement::new(),
            anatomies: Anatomies::default(),
            records: BTreeMap::new(),
            places: ItemPlaces::default(),
            intents: Vec::new(),
            events: Vec::new(),
            bodies: Bodies::new(),
            items: Items::default(),
            motion: crate::MotionSolver::NONE,
        };
        state.lay_items();
        state.refresh();
        state
    }

    /// This state, solving motion with `solver`.
    pub fn with_motion_solver(mut self, solver: crate::MotionSolver) -> Self {
        self.motion = solver;
        self
    }
    pub fn set_motion_solver(&mut self, solver: crate::MotionSolver) {
        self.motion = solver;
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn movement(&self) -> &Movement {
        &self.movement
    }
    /// Exact accepted movement pose.
    pub fn pose(&self, subject: SubjectId) -> Option<crate::MotionPose> {
        self.movement.pose(subject)
    }
    pub fn bodies(&self) -> &Bodies {
        &self.bodies
    }
    pub fn items(&self) -> &Items {
        &self.items
    }
    /// Admitted snapshots, including ones made stale by a severing.
    pub fn anatomies(&self) -> &Anatomies {
        &self.anatomies
    }
    /// The sim's entity for `subject`.
    pub fn entity(&self, subject: SubjectId) -> Option<Id> {
        self.world.entity(subject)
    }

    /// Detailed anatomy may support a current action only at its admitted revision.
    pub fn current_anatomy(&self, subject: SubjectId) -> Result<&AnatomyRecord, GameError> {
        self.living(subject)?;
        let record = self.anatomies.get(subject).ok_or(AnatomyError::Missing(subject))?;
        let current = self.records[&subject].revision;
        if record.revision != current {
            return Err(AnatomyError::StaleRevision { known: record.revision, current }.into());
        }
        Ok(record)
    }

    /// Latest declared locomotion roles.
    pub fn movement_profile(&self, subject: SubjectId) -> Option<&MovementProfile> {
        self.intents.iter().rev().find_map(|intent| match intent {
            GameIntent::ConfigureMovementProfile { subject: found, profile, .. } if *found == subject => {
                Some(profile)
            },
            _ => None,
        })
    }

    pub fn movement_projection(&self, subject: SubjectId) -> Result<Option<MovementProjection>, GameError> {
        self.movement_projection_with_speed(subject, crate::MotionRules::default().speed)
    }

    fn movement_projection_with_speed(
        &self,
        subject: SubjectId,
        requested_speed: i64,
    ) -> Result<Option<MovementProjection>, GameError> {
        let Some(profile) = self.movement_profile(subject) else {
            return Ok(None);
        };
        Ok(Some(profile.project(self.current_anatomy(subject)?, requested_speed)?))
    }

    pub fn intents(&self) -> &[GameIntent] {
        &self.intents
    }
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }
    pub fn next_tick(&self) -> Tick {
        Tick(self.intents.len() as u64)
    }

    /// The sim's hash beside the game's own record.
    pub fn state_hash(&self) -> Result<u64, GameError> {
        let record = (&self.movement, &self.anatomies, &self.records, &self.places, &self.intents);
        let bytes = snapshot::encode(&record).map_err(|_| GameError::Encode)?;
        Ok(hash_bytes(&bytes) ^ self.world.state_hash()?)
    }

    fn living(&self, subject: SubjectId) -> Result<(), GameError> {
        let body = self.bodies.get(subject).ok_or(BodyError::MissingSubject(subject))?;
        if !body.named() {
            return Err(BodyError::Unnamed(subject).into());
        }
        if !body.alive() {
            return Err(BodyError::Dead(subject).into());
        }
        Ok(())
    }

    fn entity_of(&self, subject: SubjectId) -> Result<Id, GameError> {
        self.world.entity(subject).ok_or(BodyError::MissingSubject(subject).into())
    }

    /// `subject` runs `process`, toward `target` where it names one.
    fn act(&mut self, subject: SubjectId, target: Option<Id>, process: &str) -> Result<(), GameError> {
        let actor = self.entity_of(subject)?;
        self.act_as(actor, target, process)
    }

    fn act_as(&mut self, actor: Id, target: Option<Id>, process: &str) -> Result<(), GameError> {
        let command = Command::Act { actor, target, process: process.into(), cause: None };
        let outcome = self.world.command(command)?;
        let receipt: isocosm::simulation::Receipt =
            serde_json::from_str(&outcome).map_err(|_| GameError::Decode)?;
        match receipt.accepted() {
            true => Ok(()),
            false => Err(crate::WorldError::Refused.into()),
        }
    }

    /// Reads bodies and items from the sim again.
    fn refresh(&mut self) {
        self.bodies = Bodies::read(&self.world, &self.records);
        self.items = Items::read(&self.world, &self.places);
    }

    pub fn apply(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, GameError> {
        let expected = self.next_tick();
        if intent.tick() != expected {
            return Err(GameError::WrongTick { expected, actual: intent.tick() });
        }
        let mut next = self.clone();
        let events = next.admit(&intent)?;
        next.world.tick()?;
        next.intents.push(intent);
        next.events.extend(events.iter().cloned());
        next.refresh();
        *self = next;
        Ok(events)
    }

    fn admit(&mut self, intent: &GameIntent) -> Result<Vec<GameEvent>, GameError> {
        let tick = intent.tick();
        if let GameIntent::ReviseCanon { revision, seed, cause, .. } = intent {
            cause.validate()?;
            return Ok(vec![GameEvent::CanonRevised { tick, revision: *revision, seed: *seed, cause: cause.clone() }]);
        }
        let subject = intent.subject().expect("every intent but a world-level revision names a subject");
        match intent {
            GameIntent::AdvanceMotion { revision, step, input, rules, .. } => {
                self.advance_motion(tick, subject, *revision, *step, *input, *rules)
            },
            GameIntent::ResolveVolley { actor, target, strikes, rules, .. } => {
                self.resolve_volley(tick, *actor, *target, strikes, *rules)
            },
            _ => self.admit_act(tick, subject, intent),
        }
    }

    /// Items the played site starts with, by its kind (legacy's layout).
    fn lay_items(&mut self) {
        use crate::items::ItemKind::*;
        let slot = crate::SlotId::surface(self.world.site());
        let kind = self.world.map().site(slot).map(|s| s.kind);
        let mut kinds = vec![Food];
        match kind {
            Some(crate::SiteKind::Settlement) => kinds.extend([Food, Food, Dressing, Dressing]),
            Some(crate::SiteKind::Ruin) => kinds.push(Scrap),
            _ => {},
        }
        let Some(top) = self.world.ground().surface(0, 0) else { return };
        let at = [0, top + 1, 0];
        for kind in kinds {
            let mut accounts = isocosm::schema::Ledger::new();
            if kind == Food {
                accounts.insert(crate::founding::FOOD.into(), crate::founding::MEAL);
            }
            let arrival = isocosm::arrival::Arrival {
                lineage: kind.lineage().into(),
                site: self.world.site(),
                method: isocosm::schema::Method::Inert,
                accounts,
                traits: Default::default(),
                skills: Default::default(),
                disposition: [0; 5],
                character: None,
            };
            let Ok(outcome) = self.world.command(Command::Arrive(arrival)) else { continue };
            if let Some(id) = outcome.strip_prefix("entity:").and_then(|s| s.parse().ok()) {
                self.places.at.insert(crate::ItemId(id), at);
            }
        }
    }
}
