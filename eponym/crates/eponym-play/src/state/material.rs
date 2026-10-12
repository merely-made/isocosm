// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native item custody with game-owned in-site placement (ruling 776).

use crate::founding::{CARRIES, DROP};
use crate::identity::{SubjectId, Tick};
use crate::{GameError, GameEvent, ItemError, ItemId, ItemLocation, WorldError};

use super::GameState;

impl GameState {
    /// Recipe-profile worlds price custody by accepted native matter. The
    /// old profile keeps its original nominal carry rule and saved balance.
    pub(super) fn check_item_capacity(
        &self, subject: SubjectId, item: ItemId,
    ) -> Result<(), GameError> {
        let body = self.bodies.get(subject).ok_or(crate::BodyError::MissingSubject(subject))?;
        let found = self.items.get(item).ok_or(ItemError::Missing(item))?;
        if self.world.material_profile().is_none() {
            let carried = self.items.carried_mass_mg(subject);
            let project_mass = self.project_burden(subject)?;
            let attempted = u128::from(carried).checked_add(u128::from(found.kind.mass_mg()))
                .and_then(|mass| mass.checked_add(project_mass)).ok_or(WorldError::Refused)?;
            if project_mass > 0 && attempted > u128::from(body.profile.carry_capacity_mg) {
                return Err(ItemError::NativeOverCapacity {
                    capacity_mg:body.profile.carry_capacity_mg, attempted_mg:attempted,
                }.into());
            }
            if project_mass == 0 && !body.can_carry(carried, found.kind.mass_mg()) {
                return Err(ItemError::OverCapacity { capacity_mg: body.profile.carry_capacity_mg,
                    attempted_mg: carried.saturating_add(found.kind.mass_mg()) }.into());
            }
            return Ok(());
        }
        let mass = |id| self.item_reading(id).map(|reading| reading.matter_mass_mg)
            .map_err(|_| GameError::World(WorldError::Refused));
        let attempted_mg = self.actual_burden(subject)?.checked_add(mass(item)?)
            .ok_or(WorldError::Refused)?;
        if attempted_mg > u128::from(body.profile.carry_capacity_mg) {
            return Err(ItemError::NativeOverCapacity {
                capacity_mg: body.profile.carry_capacity_mg, attempted_mg,
            }.into());
        }
        Ok(())
    }

    pub(super) fn drop_item(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        item: ItemId,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let found = self.items.get(item).ok_or(ItemError::Missing(item))?;
        if !matches!(found.location,
            ItemLocation::Carried(owner) | ItemLocation::Attached { subject: owner, .. }
            if owner == subject
        ) {
            return Err(ItemError::NotCarried(item, subject).into());
        }
        let at = self.at(subject)?;
        let carrier = self.entity_of(subject)?;
        self.act(subject, Some(item.0), DROP)?;
        if self.world.sim().state().relations.iter().any(|relation|
            relation.kind == CARRIES && relation.subject == carrier && relation.object == item.0
        ) {
            return Err(WorldError::Refused.into());
        }
        self.places.worn.remove(&item);
        self.places.at.insert(item, at);
        Ok(vec![GameEvent::Dropped { tick, subject, item, at }])
    }
}
