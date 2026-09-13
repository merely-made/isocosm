// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Subject positions and the one movement transition.
//!
//! This system knows nothing about destinations, activities, or player
//! control. It stores any number of subject positions. A caller presents an
//! input and the separately owned world resolves it through Mesocosm's shared
//! integer `step` law. Navigation is a derived query in the adjacent module
//! and never enters this save format.

use std::collections::BTreeMap;

use mesocosm_core::places::{WALKER_HEIGHT, step};
use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::{MotionError, MotionPose, World};

mod motion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementIntent {
    Spawn {
        tick: Tick,
        subject: SubjectId,
        at: [i32; 3],
    },
    Step {
        tick: Tick,
        subject: SubjectId,
        toward: [i32; 3],
    },
    /// Exact pose resolved by the product-owned continuous-motion transition.
    ContactPose {
        tick: Tick,
        subject: SubjectId,
        pose: MotionPose,
    },
}

impl MovementIntent {
    pub const fn tick(self) -> Tick {
        match self {
            Self::Spawn { tick, .. } | Self::Step { tick, .. } | Self::ContactPose { tick, .. } => {
                tick
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementEvent {
    Spawned {
        tick: Tick,
        subject: SubjectId,
        at: [i32; 3],
    },
    Moved {
        tick: Tick,
        subject: SubjectId,
        from: [i32; 3],
        to: [i32; 3],
    },
    Held {
        tick: Tick,
        subject: SubjectId,
        at: [i32; 3],
    },
    ContactMoved {
        tick: Tick,
        subject: SubjectId,
        from: [i32; 3],
        to: [i32; 3],
        pose: MotionPose,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementSave {
    pub intents: Vec<MovementIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementError {
    SubjectExists(SubjectId),
    MissingSubject(SubjectId),
    InvalidStart([i32; 3]),
    MixedMovement(SubjectId),
    WrongMotionStep { previous: u64, next: u64 },
    Motion(MotionError),
    WrongTick { expected: Tick, actual: Tick },
    Encode,
    Decode,
}

/// Positions for every currently embodied subject in one world.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Movement {
    positions: BTreeMap<SubjectId, [i32; 3]>,
    intents: Vec<MovementIntent>,
    events: Vec<MovementEvent>,
    trails: BTreeMap<SubjectId, Vec<[i32; 3]>>,
}

impl Movement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn position(&self, subject: SubjectId) -> Option<[i32; 3]> {
        self.positions.get(&subject).copied()
    }

    /// The accepted exact contact pose, or the legacy cell pose when this
    /// subject has not yet entered continuous motion.
    pub fn pose(&self, subject: SubjectId) -> Option<MotionPose> {
        self.intents
            .iter()
            .rev()
            .find_map(|intent| match intent {
                MovementIntent::ContactPose {
                    subject: found,
                    pose,
                    ..
                } if *found == subject => Some(*pose),
                _ => None,
            })
            .or_else(|| self.position(subject).map(MotionPose::at_cell))
    }

    pub fn positions(&self) -> impl Iterator<Item = (SubjectId, [i32; 3])> + '_ {
        self.positions.iter().map(|(subject, at)| (*subject, *at))
    }

    pub fn intents(&self) -> &[MovementIntent] {
        &self.intents
    }

    pub fn events(&self) -> &[MovementEvent] {
        &self.events
    }

    pub fn trail(&self, subject: SubjectId) -> Option<&[[i32; 3]]> {
        self.trails.get(&subject).map(Vec::as_slice)
    }

    pub fn next_tick(&self) -> Tick {
        Tick(self.intents.len() as u64)
    }

    pub fn spawn(
        &mut self,
        world: &World,
        subject: SubjectId,
        at: [i32; 3],
    ) -> Result<MovementEvent, MovementError> {
        self.apply(
            world,
            MovementIntent::Spawn {
                tick: self.next_tick(),
                subject,
                at,
            },
        )
    }

    pub fn step(
        &mut self,
        world: &World,
        subject: SubjectId,
        toward: [i32; 3],
    ) -> Result<MovementEvent, MovementError> {
        if self.has_contact_pose(subject) {
            return Err(MovementError::MixedMovement(subject));
        }
        self.apply(
            world,
            MovementIntent::Step {
                tick: self.next_tick(),
                subject,
                toward,
            },
        )
    }

    pub fn apply(
        &mut self,
        world: &World,
        intent: MovementIntent,
    ) -> Result<MovementEvent, MovementError> {
        let expected = self.next_tick();
        if intent.tick() != expected {
            return Err(MovementError::WrongTick {
                expected,
                actual: intent.tick(),
            });
        }
        let event = match intent {
            MovementIntent::Spawn { tick, subject, at } => {
                if self.positions.contains_key(&subject) {
                    return Err(MovementError::SubjectExists(subject));
                }
                if !world.ground().stands(at, WALKER_HEIGHT) {
                    return Err(MovementError::InvalidStart(at));
                }
                self.positions.insert(subject, at);
                self.trails.insert(subject, vec![at]);
                MovementEvent::Spawned { tick, subject, at }
            },
            MovementIntent::Step {
                tick,
                subject,
                toward,
            } => {
                if self.has_contact_pose(subject) {
                    return Err(MovementError::MixedMovement(subject));
                }
                let from = self
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let to = step(world.ground(), from, toward);
                self.positions.insert(subject, to);
                self.trails
                    .get_mut(&subject)
                    .expect("a positioned subject has a trail")
                    .push(to);
                if to == from {
                    MovementEvent::Held {
                        tick,
                        subject,
                        at: from,
                    }
                } else {
                    MovementEvent::Moved {
                        tick,
                        subject,
                        from,
                        to,
                    }
                }
            },
            MovementIntent::ContactPose {
                tick,
                subject,
                pose,
            } => {
                pose.validate().map_err(MovementError::Motion)?;
                let prior = self
                    .pose(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let expected_step = prior.step.checked_add(1).ok_or(MotionError::Overflow)?;
                if pose.step != expected_step {
                    return Err(MovementError::WrongMotionStep {
                        previous: prior.step,
                        next: pose.step,
                    });
                }
                let from = self
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let to = pose.cell().map_err(MovementError::Motion)?;
                self.positions.insert(subject, to);
                self.trails
                    .get_mut(&subject)
                    .expect("a positioned subject has a trail")
                    .push(to);
                MovementEvent::ContactMoved {
                    tick,
                    subject,
                    from,
                    to,
                    pose,
                }
            },
        };
        self.intents.push(intent);
        self.events.push(event);
        Ok(event)
    }

    pub fn state_hash(&self) -> Result<u64, MovementError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| MovementError::Encode)
    }

    pub fn save_record(&self) -> MovementSave {
        MovementSave {
            intents: self.intents.clone(),
        }
    }

    pub fn save(&self) -> Result<Vec<u8>, MovementError> {
        snapshot::encode(&self.save_record()).map_err(|_| MovementError::Encode)
    }

    pub fn restore(world: &World, bytes: &[u8]) -> Result<Self, MovementError> {
        let save: MovementSave = snapshot::decode(bytes).map_err(|_| MovementError::Decode)?;
        Self::restore_record(world, save)
    }

    pub fn restore_record(world: &World, save: MovementSave) -> Result<Self, MovementError> {
        let mut movement = Self::new();
        for intent in save.intents {
            movement.apply(world, intent)?;
        }
        Ok(movement)
    }
}

impl From<MotionError> for MovementError {
    fn from(error: MotionError) -> Self {
        Self::Motion(error)
    }
}
