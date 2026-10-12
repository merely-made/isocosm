// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native item custody with game-owned in-site placement (ruling 776).

use crate::founding::{CARRIES, DROP};
use crate::identity::{SubjectId, Tick};
use crate::{GameError, GameEvent, ItemError, ItemId, ItemLocation, WorldError};

use super::GameState;

impl GameState {
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
