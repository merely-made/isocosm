// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A subject's acts as native commands: arriving, naming, walking, seeing,
//! carrying, eating, rest and the exertion each costs (767), admitting its
//! anatomy and wearing an item.

use isocosm::history::Command;
use isocosm_overlay::eponym::{EponymIntent, PlayerAct, PlayerActKind};
use isocosm_overlay::{EntityHandle, Pointable};

use crate::bodies::{BodyError, BodyRecord, SAFE_FALL};
use crate::founding::{
    self, DROP, EAT, ENERGY, FATIGUE, FULL, HUNGER, RESERVE, RESTS, SPENT, TAKE,
};
use crate::identity::{BodyRevisionId, SubjectId, Tick};
use crate::items::{ItemError, ItemKind, ItemLocation};
use crate::{
    AnatomyError, DeathCause, GameError, GameEvent, GameIntent, MotionError, MovementError,
    MovementEvent,
};

use super::GameState;

impl GameState {
    pub(super) fn admit_act(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        intent: &GameIntent,
    ) -> Result<Vec<GameEvent>, GameError> {
        Ok(match intent {
            GameIntent::Generate { body_seed, at, .. } => {
                self.generate(tick, subject, *body_seed, *at, None)?
            },
            GameIntent::GenerateNativeRecipe { body_seed, at, recipe, .. } => {
                self.generate(tick, subject, *body_seed, *at, Some(*recipe))?
            },
            GameIntent::Name { name, .. } => {
                name.validate()?;
                let record = self
                    .records
                    .get(&subject)
                    .copied()
                    .ok_or(BodyError::MissingSubject(subject))?;
                if record.named_at.is_some() {
                    return Err(BodyError::AlreadyNamed(subject).into());
                }
                let of = record.entity;
                let by = match self.participant {
                    Some(p) => self
                        .world
                        .sim()
                        .plays(p)
                        .ok_or(crate::WorldError::NotPlayed)?,
                    None => of,
                };
                let act = PlayerAct {
                    subject: EntityHandle(by),
                    kind: PlayerActKind::Name {
                        of: Pointable::Entity(EntityHandle(of)),
                        name: name.as_str().into(),
                    },
                };
                self.world
                    .submit(self.participant, &EponymIntent::Act(act))?;
                self.records.get_mut(&subject).expect("recorded").named_at = Some(tick);
                vec![GameEvent::Named {
                    tick,
                    subject,
                    name: name.clone(),
                }]
            },
            GameIntent::Move { toward, .. } => {
                self.living(subject)?;
                if !self.bodies.get(subject).is_some_and(|b| b.mobile()) {
                    return Err(BodyError::Immobile(subject).into());
                }
                let event = match self.movement.step(&self.world, subject, *toward)? {
                    MovementEvent::Moved { from, to, .. } => GameEvent::Moved {
                        tick,
                        subject,
                        from,
                        to,
                    },
                    MovementEvent::Held { at, .. } => GameEvent::Held { tick, subject, at },
                    _ => unreachable!("a step moves or holds"),
                };
                self.actuate_motion(subject, 0)?;
                self.exert(subject, 1, 2, tick, vec![event])?
            },
            GameIntent::Observe { target, .. } => {
                self.living(subject)?;
                let at = self.at(subject)?;
                let range = self
                    .bodies
                    .get(subject)
                    .expect("living")
                    .profile
                    .sight_range;
                let visible = crate::walking::spot(self.world.volume(), at, *target, range);
                let event = GameEvent::Observed {
                    tick,
                    subject,
                    target: *target,
                    visible,
                };
                self.exert(subject, 1, 1, tick, vec![event])?
            },
            GameIntent::Take { item, .. } => {
                self.living(subject)?;
                let at = self.at(subject)?;
                let found = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if found.location != ItemLocation::At(at) {
                    return Err(ItemError::NotHere(*item).into());
                }
                self.check_item_capacity(subject, *item)?;
                self.act(subject, Some(item.0), TAKE)?;
                self.exert(
                    subject,
                    1,
                    1,
                    tick,
                    vec![GameEvent::Took {
                        tick,
                        subject,
                        item: *item,
                    }],
                )?
            },
            GameIntent::Drop { item, .. } => self.drop_item(tick, subject, *item)?,
            GameIntent::Eat { item, .. } => {
                self.living(subject)?;
                let food = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                if food.location != ItemLocation::Carried(subject) {
                    return Err(ItemError::NotCarried(*item, subject).into());
                }
                if food.kind != ItemKind::Food {
                    return Err(ItemError::WrongKind(*item, food.kind).into());
                }
                let eat = match self.world.material_profile() {
                    Some(_) => self.world.material_consumption(*item)
                        .ok_or(crate::WorldError::Unbuilt("this recipe has no declared consumption process"))?,
                    None => EAT,
                }.to_owned();
                self.act(subject, Some(item.0), &eat)?;
                self.act(subject, Some(item.0), SPENT)?;
                let hunger = self.need(subject, RESERVE);
                self.exert(
                    subject,
                    0,
                    1,
                    tick,
                    vec![GameEvent::Ate {
                        tick,
                        subject,
                        item: *item,
                        hunger,
                    }],
                )?
            },
            GameIntent::Rest { .. } => {
                self.living(subject)?;
                let wounded = self.bodies.get(subject).is_some_and(|b| b.wound > 0);
                let dressing = self
                    .items
                    .carried_by(subject)
                    .find(|i| i.kind == ItemKind::Dressing)
                    .map(|i| i.id);
                if let Some(item) = dressing.filter(|_| wounded) {
                    self.act(subject, Some(item.0), SPENT)?;
                }
                self.act(subject, None, RESTS)?;
                // Healing waits for checkpoint 10 (712): nothing recovers yet.
                let revision = self.records[&subject].revision;
                let fatigue = self.need(subject, ENERGY);
                let event = GameEvent::Rested {
                    tick,
                    subject,
                    recovered: 0,
                    fatigue,
                    revision,
                };
                self.exert(subject, 1, 0, tick, vec![event])?
            },
            GameIntent::Fall { distance, .. } => {
                self.living(subject)?;
                let harm = distance
                    .saturating_sub(SAFE_FALL)
                    .saturating_mul(10)
                    .clamp(0, 100) as u16;
                let (revision, died, mut events) = self.harm(tick, subject, harm, &[])?;
                events.insert(
                    0,
                    GameEvent::Injured {
                        tick,
                        subject,
                        distance: *distance,
                        harm,
                        revision,
                    },
                );
                if died {
                    events.push(GameEvent::Died {
                        tick,
                        subject,
                        cause: DeathCause::Fall,
                    });
                    events
                } else {
                    self.exert(subject, 1, 1, tick, events)?
                }
            },
            GameIntent::Wait { .. } => {
                self.living(subject)?;
                self.exert(
                    subject,
                    2,
                    1,
                    tick,
                    vec![GameEvent::Waited { tick, subject }],
                )?
            },
            GameIntent::AdmitAnatomy {
                revision, document, ..
            } => {
                self.living(subject)?;
                let current = self.records[&subject].revision;
                if *revision != current {
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current,
                    }
                    .into());
                }
                if !self.world.developed_bodies() {
                    return self.admit_legacy_body(tick, subject, *revision, document);
                }
                let expected = self.native_anatomy(subject, document.species())?;
                if expected != **document { return Err(AnatomyError::NativeMismatch.into()); }
                let source = self.native_anatomy_source(subject)?;
                self.anatomies.admit(subject, *revision, expected)?;
                self.anatomies.bind_native(subject, source)?;
                vec![GameEvent::AnatomyAdmitted {
                    tick,
                    subject,
                    revision: *revision,
                }]
            },
            GameIntent::ReconcileAnatomy {
                from_revision,
                revision,
                severed_parts,
                ..
            } => {
                self.living(subject)?;
                let current = self.records[&subject].revision;
                if current != *revision {
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current,
                    }
                    .into());
                }
                self.reconcile(tick, subject, *from_revision, *revision, severed_parts)?
            },
            GameIntent::AttachItem {
                item,
                part,
                revision,
                ..
            } => {
                let record = self.current_anatomy(subject)?;
                if record.revision != *revision {
                    let current = record.revision;
                    return Err(AnatomyError::StaleRevision {
                        known: *revision,
                        current,
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
                let found = *self.items.get(*item).ok_or(ItemError::Missing(*item))?;
                match found.location {
                    ItemLocation::Carried(owner) if owner == subject => {},
                    ItemLocation::Attached { .. } => {
                        return Err(ItemError::AlreadyAttached(*item).into());
                    },
                    _ => return Err(ItemError::NotCarried(*item, subject).into()),
                }
                if found.kind != ItemKind::Dressing {
                    return Err(ItemError::WrongKind(*item, found.kind).into());
                }
                self.places.worn.insert(*item, *part);
                vec![GameEvent::ItemAttached {
                    tick,
                    subject,
                    item: *item,
                    part: *part,
                }]
            },
            GameIntent::DetachItem { item, .. } => {
                self.living(subject)?;
                let worn = self.items.get(*item).map(|i| i.location);
                if !matches!(worn, Some(ItemLocation::Attached { subject: owner, .. }) if owner == subject)
                {
                    return Err(ItemError::NotCarried(*item, subject).into());
                }
                self.places.worn.remove(item);
                vec![GameEvent::ItemDetached {
                    tick,
                    subject,
                    item: *item,
                }]
            },
            GameIntent::ConfigureMovementProfile {
                revision, profile, ..
            } => {
                self.living(subject)?;
                if profile.source_revision != *revision {
                    return Err(MotionError::InvalidProfile.into());
                }
                profile.validate_at(self.current_anatomy(subject)?)?;
                vec![GameEvent::MovementProfileConfigured {
                    tick,
                    subject,
                    profile: profile.clone(),
                }]
            },
            GameIntent::AdvanceMotion { .. }
            | GameIntent::ResolveVolley { .. }
            | GameIntent::ProjectAct { .. }
            | GameIntent::ProjectCarry { .. }
            | GameIntent::ProjectReserve { .. }
            | GameIntent::ProjectConsent { .. }
            | GameIntent::StockOwnership { .. }
            | GameIntent::StockAct { .. }
            | GameIntent::EatOwned { .. }
            | GameIntent::ReviseCanon { .. } => {
                unreachable!("routed before acts")
            },
        })
    }

    /// An outsider arrives standing at `at`, its profile drawn from its
    /// seed, holding full reserve and energy (235).
    fn generate(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        body_seed: u64,
        at: [i32; 3],
        recipe: Option<crate::NativeFixtureRecipe>,
    ) -> Result<Vec<GameEvent>, GameError> {
        if !self.world.developed_bodies() {
            if recipe.is_some() { return Err(crate::WorldError::NotFixture.into()); }
            return self.generate_legacy_body(tick, subject, body_seed, at);
        }
        if self.records.contains_key(&subject) {
            return Err(BodyError::SubjectExists(subject).into());
        }
        if self.movement.position(subject).is_some() {
            return Err(MovementError::SubjectExists(subject).into());
        }
        if !crate::walking::stands(self.world.volume(), at) {
            return Err(MovementError::InvalidStart(at).into());
        }
        self.world.select_native_recipe(recipe)?;
        let seed = isocosm::draw(self.world.seed(), "eponym:development", &[body_seed, subject.0]);
        let sim = self.world.sim();
        let body = sim.development_body(founding::SOPHONT, seed).map_err(|_| GameError::Decode)?;
        let tissue = body.parts.iter().try_fold(0u64, |total, (id, part)| {
            let geo = body.doc.part(*id).ok_or(GameError::Decode)?;
            total.checked_add(isocosm::anatomy::bound(geo.half_extent, part, &sim.genesis().rules, founding::TISSUE))
                .ok_or(GameError::Decode)
        })?;
        let accounts = [
            (founding::TISSUE, tissue),
            (RESERVE, FULL),
            (ENERGY, FULL),
        ];
        let arrival = isocosm::arrival::Arrival {
            lineage: founding::SOPHONT.into(),
            site: self.world.site(),
            method: isocosm::schema::Method::Normative,
            accounts: accounts.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            traits: Default::default(),
            skills: Default::default(),
            disposition: [0; 5],
            character: None,
        };
        let outcome = self.world.found(Command::Arrive(arrival))?;
        let entity = outcome
            .strip_prefix("entity:")
            .and_then(|s| s.parse().ok())
            .ok_or(GameError::Decode)?;
        self.world.found(Command::Develop { entity, seed })?;
        self.world.bind(subject, entity);
        let revision = BodyRevisionId(0);
        self.records.insert(
            subject,
            BodyRecord {
                entity,
                revision,
                body_seed,
                named_at: None,
                died_at: None,
            },
        );
        self.movement.spawn(&self.world, subject, at)?;
        Ok(vec![GameEvent::Generated {
            tick,
            subject,
            revision,
        }])
    }

    pub(super) fn at(&self, subject: SubjectId) -> Result<[i32; 3], GameError> {
        Ok(self
            .movement
            .position(subject)
            .ok_or(MovementError::MissingSubject(subject))?)
    }

    /// How far `subject` is below full in `account`.
    fn need(&self, subject: SubjectId, account: &str) -> u16 {
        let Some(e) = self
            .entity(subject)
            .and_then(|id| self.world.sim().state().population.get(id))
        else {
            return 0;
        };
        let want = isocosm::needs::want(&self.world.sim().genesis().rules, e, account);
        want.map_or(0, |w| w.deficit().min(100) as u16)
    }

    /// What an act costs: `hunger` spent from the reserve, `fatigue` from
    /// energy, a unit a process (767).
    pub(super) fn exert(
        &mut self,
        subject: SubjectId,
        hunger: u16,
        fatigue: u16,
        _tick: Tick,
        events: Vec<GameEvent>,
    ) -> Result<Vec<GameEvent>, GameError> {
        for _ in 0..hunger {
            self.act(subject, None, HUNGER)?;
        }
        for _ in 0..fatigue {
            self.act(subject, None, FATIGUE)?;
        }
        Ok(events)
    }

    /// Items worn on `lost` parts fall where their wearer stands.
    pub(super) fn release_worn(
        &mut self,
        subject: SubjectId,
        lost: &[isometer_core::PartId],
        at: [i32; 3],
    ) -> Result<Vec<crate::ItemId>, GameError> {
        let fallen: Vec<_> = self
            .items
            .carried_by(subject)
            .filter(|i| matches!(i.location, ItemLocation::Attached { part, .. } if lost.contains(&part)))
            .map(|i| i.id)
            .collect();
        for item in &fallen {
            self.act(subject, Some(item.0), DROP)?;
            self.places.worn.remove(item);
            self.places.at.insert(*item, at);
        }
        Ok(fallen)
    }
}
