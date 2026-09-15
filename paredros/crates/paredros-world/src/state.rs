// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The coordinator over separately owned world, movement, body, and item state.

use crate::bodies::{Bodies, BodyError};
use crate::items::{ItemError, ItemKind, ItemLocation, Items};
use crate::{
    Anatomies, AnatomyError, AnatomyRecord, DeathCause, GameError, GameEvent, GameIntent, Movement,
    MovementError, MovementEvent, MovementProfile, MovementProjection, World,
};
use isometer_core::snapshot::{self, hash_bytes};
use mesocosm_core::places::spot;
use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

mod combat;
mod motion;
mod save;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    world: World,
    movement: Movement,
    bodies: Bodies,
    anatomies: Anatomies,
    items: Items,
    intents: Vec<GameIntent>,
    events: Vec<GameEvent>,
}

impl GameState {
    pub fn new(world: World) -> Self {
        let items = Items::generate(&world);
        Self {
            world,
            movement: Movement::new(),
            bodies: Bodies::new(),
            anatomies: Anatomies::default(),
            items,
            intents: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn movement(&self) -> &Movement {
        &self.movement
    }

    /// Exact accepted movement pose. Integer movement positions remain a
    /// derived compatibility projection for legacy world consumers.
    pub fn pose(&self, subject: SubjectId) -> Option<crate::MotionPose> {
        self.movement.pose(subject)
    }

    pub fn bodies(&self) -> &Bodies {
        &self.bodies
    }

    pub fn items(&self) -> &Items {
        &self.items
    }

    /// Historical admitted snapshots, including ones made stale by injury.
    pub fn anatomies(&self) -> &Anatomies {
        &self.anatomies
    }

    /// Detailed anatomy may support a current action only at its admitted revision.
    pub fn current_anatomy(&self, subject: SubjectId) -> Result<&AnatomyRecord, GameError> {
        self.living(subject)?;
        let record = self
            .anatomies
            .get(subject)
            .ok_or(AnatomyError::Missing(subject))?;
        let current = self
            .bodies
            .get(subject)
            .expect("validated subject")
            .revision;
        if record.revision != current {
            return Err(AnatomyError::StaleRevision {
                known: record.revision,
                current,
            }
            .into());
        }
        Ok(record)
    }

    /// Latest declared locomotion roles. The returned projection is rebuilt
    /// from the current anatomy, so severance changes supports without mutating
    /// an independent shape cache.
    pub fn movement_profile(&self, subject: SubjectId) -> Option<&MovementProfile> {
        self.intents.iter().rev().find_map(|intent| match intent {
            GameIntent::ConfigureMovementProfile {
                subject: found,
                profile,
                ..
            } if *found == subject => Some(profile),
            _ => None,
        })
    }

    pub fn movement_projection(
        &self,
        subject: SubjectId,
    ) -> Result<Option<MovementProjection>, GameError> {
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
        Ok(Some(profile.project(
            self.current_anatomy(subject)?,
            requested_speed,
        )?))
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

    pub fn state_hash(&self) -> Result<u64, GameError> {
        snapshot::encode(self)
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| GameError::Encode)
    }

    fn living(&self, subject: SubjectId) -> Result<(), GameError> {
        let body = self
            .bodies
            .get(subject)
            .ok_or(BodyError::MissingSubject(subject))?;
        if !body.named() {
            return Err(BodyError::Unnamed(subject).into());
        }
        if !body.alive() {
            return Err(BodyError::Dead(subject).into());
        }
        Ok(())
    }

    fn exert(
        &mut self,
        subject: SubjectId,
        hunger: u16,
        fatigue: u16,
        tick: Tick,
        events: &mut Vec<GameEvent>,
    ) {
        let died = self
            .bodies
            .get_mut(subject)
            .expect("a validated body exists")
            .exert(hunger, fatigue, tick);
        if died {
            events.push(GameEvent::Died {
                tick,
                subject,
                cause: DeathCause::Starvation,
            });
        }
    }

    pub fn apply(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, GameError> {
        let expected = self.next_tick();
        if intent.tick() != expected {
            return Err(GameError::WrongTick {
                expected,
                actual: intent.tick(),
            });
        }
        let tick = intent.tick();
        if let GameIntent::ReviseCanon {
            revision,
            seed,
            cause,
            ..
        } = &intent
        {
            cause.validate()?;
            let event = GameEvent::CanonRevised {
                tick,
                revision: *revision,
                seed: *seed,
                cause: cause.clone(),
            };
            return Ok(self.accept(intent, vec![event]));
        }
        let subject = intent
            .subject()
            .expect("every intent but a world-level revision names a subject");
        let events = match &intent {
            GameIntent::AdvanceMotion {
                revision,
                step,
                input,
                rules,
                ..
            } => self.advance_motion(tick, subject, *revision, *step, *input, *rules)?,
            GameIntent::ConfigureMovementProfile {
                revision, profile, ..
            } => {
                self.living(subject)?;
                if profile.source_revision != *revision {
                    return Err(crate::MotionError::InvalidProfile.into());
                }
                profile.validate_at(self.current_anatomy(subject)?)?;
                vec![GameEvent::MovementProfileConfigured {
                    tick,
                    subject,
                    profile: profile.clone(),
                }]
            },
            GameIntent::AttachItem {
                item,
                part,
                revision,
                ..
            } => {
                let record = self.current_anatomy(subject)?;
                if record.revision != *revision {
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current: record.revision,
                    }
                    .into());
                }
                let target = record
                    .document
                    .part(*part)
                    .ok_or(AnatomyError::MissingPart(*part))?;
                if target.severed {
                    return Err(AnatomyError::SeveredPart(*part).into());
                }
                self.items.attach(*item, subject, *part)?;
                vec![GameEvent::ItemAttached {
                    tick,
                    subject,
                    item: *item,
                    part: *part,
                }]
            },
            GameIntent::DetachItem { item, .. } => {
                self.living(subject)?;
                self.items.detach(*item, subject)?;
                vec![GameEvent::ItemDetached {
                    tick,
                    subject,
                    item: *item,
                }]
            },
            GameIntent::ResolveVolley {
                actor,
                target,
                strikes,
                rules,
                ..
            } => self.resolve_volley(tick, *actor, *target, strikes, *rules)?,
            GameIntent::ReconcileAnatomy {
                from_revision,
                revision,
                severed_parts,
                ..
            } => {
                self.living(subject)?;
                let current = self
                    .bodies
                    .get(subject)
                    .expect("validated subject")
                    .revision;
                if current != *revision {
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current,
                    }
                    .into());
                }
                // Validate all fallible inputs before either owner changes.
                let at = self
                    .movement
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                self.anatomies
                    .reconcile(subject, *from_revision, *revision, severed_parts)?;
                let lost: Vec<_> = self
                    .anatomies
                    .get(subject)
                    .expect("reconciled snapshot")
                    .document
                    .parts
                    .iter()
                    .filter(|part| part.severed)
                    .map(|part| part.id)
                    .collect();
                let released = self.items.release_attached(subject, &lost, at);
                let mut events = vec![GameEvent::AnatomyReconciled {
                    tick,
                    subject,
                    from_revision: *from_revision,
                    revision: *revision,
                }];
                events.extend(released.into_iter().map(|item| GameEvent::ItemReleased {
                    tick,
                    subject,
                    item,
                    at,
                }));
                events
            },
            GameIntent::AdmitAnatomy {
                revision, document, ..
            } => {
                self.living(subject)?;
                let current = self
                    .bodies
                    .get(subject)
                    .expect("validated subject")
                    .revision;
                if *revision != current {
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current,
                    }
                    .into());
                }
                self.anatomies
                    .admit(subject, *revision, document.as_ref().clone())?;
                vec![GameEvent::AnatomyAdmitted {
                    tick,
                    subject,
                    revision: *revision,
                }]
            },
            GameIntent::Generate { body_seed, at, .. } => {
                if self.bodies.get(subject).is_some() {
                    return Err(BodyError::SubjectExists(subject).into());
                }
                if self.movement.position(subject).is_some() {
                    return Err(MovementError::SubjectExists(subject).into());
                }
                if !self
                    .world
                    .ground()
                    .stands(*at, mesocosm_core::places::WALKER_HEIGHT)
                {
                    return Err(MovementError::InvalidStart(*at).into());
                }
                let revision = self
                    .bodies
                    .generate(self.world.seed(), *body_seed, subject)?
                    .revision;
                self.movement.spawn(&self.world, subject, *at)?;
                vec![GameEvent::Generated {
                    tick,
                    subject,
                    revision,
                }]
            },
            GameIntent::Name { name, .. } => {
                name.validate()?;
                let body = self
                    .bodies
                    .get_mut(subject)
                    .ok_or(BodyError::MissingSubject(subject))?;
                body.name(name.clone(), tick)?;
                vec![GameEvent::Named {
                    tick,
                    subject,
                    name: name.clone(),
                }]
            },
            GameIntent::Move { toward, .. } => {
                self.living(subject)?;
                if !self.bodies.get(subject).unwrap().mobile() {
                    return Err(BodyError::Immobile(subject).into());
                }
                let moved = self.movement.step(&self.world, subject, *toward)?;
                let event = match moved {
                    MovementEvent::Moved { from, to, .. } => GameEvent::Moved {
                        tick,
                        subject,
                        from,
                        to,
                    },
                    MovementEvent::Held { at, .. } => GameEvent::Held { tick, subject, at },
                    MovementEvent::Spawned { .. } | MovementEvent::ContactMoved { .. } => {
                        unreachable!("legacy step cannot emit another movement kind")
                    },
                };
                let mut events = vec![event];
                self.exert(subject, 1, 2, tick, &mut events);
                events
            },
            GameIntent::Observe { target, .. } => {
                self.living(subject)?;
                let body = self.bodies.get(subject).unwrap();
                let at = self
                    .movement
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let visible = spot(self.world.ground(), at, *target, body.profile.sight_range);
                let mut events = vec![GameEvent::Observed {
                    tick,
                    subject,
                    target: *target,
                    visible,
                }];
                self.exert(subject, 1, 1, tick, &mut events);
                events
            },
            GameIntent::Take { item, .. } => {
                self.living(subject)?;
                let at = self
                    .movement
                    .position(subject)
                    .ok_or(MovementError::MissingSubject(subject))?;
                let found = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if found.location != ItemLocation::At(at) {
                    return Err(ItemError::NotHere(*item).into());
                }
                let body = self.bodies.get(subject).unwrap();
                let carried = self.items.carried_mass_mg(subject);
                let attempted = carried.saturating_add(found.kind.mass_mg());
                if !body.can_carry(carried, found.kind.mass_mg()) {
                    return Err(ItemError::OverCapacity {
                        capacity_mg: body.profile.carry_capacity_mg,
                        attempted_mg: attempted,
                    }
                    .into());
                }
                self.items.take(*item, subject)?;
                let mut events = vec![GameEvent::Took {
                    tick,
                    subject,
                    item: *item,
                }];
                self.exert(subject, 1, 1, tick, &mut events);
                events
            },
            GameIntent::Eat { item, .. } => {
                self.living(subject)?;
                let food = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if food.location != ItemLocation::Carried(subject) {
                    return Err(ItemError::NotCarried(*item, subject).into());
                }
                if food.kind != ItemKind::Food {
                    return Err(ItemError::WrongKind(*item, food.kind).into());
                }
                self.items.consume(*item)?;
                let body = self.bodies.get_mut(subject).unwrap();
                body.eat();
                let mut events = vec![GameEvent::Ate {
                    tick,
                    subject,
                    item: *item,
                    hunger: body.needs.hunger,
                }];
                self.exert(subject, 0, 1, tick, &mut events);
                events
            },
            GameIntent::Rest { .. } => {
                self.living(subject)?;
                let dressing = (self.bodies.get(subject).unwrap().wound > 0)
                    .then(|| {
                        self.items
                            .carried_by(subject)
                            .find(|item| item.kind == ItemKind::Dressing)
                            .map(|item| item.id)
                    })
                    .flatten();
                if let Some(item) = dressing {
                    self.items.consume(item)?;
                }
                let body = self.bodies.get_mut(subject).unwrap();
                let (recovered, revision) = body.rest(dressing.is_some());
                let mut events = vec![GameEvent::Rested {
                    tick,
                    subject,
                    recovered,
                    fatigue: body.needs.fatigue,
                    revision,
                }];
                self.exert(subject, 1, 0, tick, &mut events);
                events
            },
            GameIntent::Fall { distance, .. } => {
                self.living(subject)?;
                let body = self.bodies.get_mut(subject).unwrap();
                let (harm, revision, died) = body.fall(*distance, tick);
                let mut events = vec![GameEvent::Injured {
                    tick,
                    subject,
                    distance: *distance,
                    harm,
                    revision,
                }];
                if died {
                    events.push(GameEvent::Died {
                        tick,
                        subject,
                        cause: DeathCause::Fall,
                    });
                } else {
                    self.exert(subject, 1, 1, tick, &mut events);
                }
                events
            },
            // Returned above: a world-level revision has no subject to route.
            GameIntent::ReviseCanon { .. } => unreachable!("world-level intent"),
            GameIntent::Wait { .. } => {
                self.living(subject)?;
                let mut events = vec![GameEvent::Waited { tick, subject }];
                self.exert(subject, 2, 1, tick, &mut events);
                events
            },
        };
        Ok(self.accept(intent, events))
    }

    /// Records one accepted intent and the events it produced.
    fn accept(&mut self, intent: GameIntent, mut events: Vec<GameEvent>) -> Vec<GameEvent> {
        let returned = events.clone();
        self.intents.push(intent);
        self.events.append(&mut events);
        returned
    }
}
