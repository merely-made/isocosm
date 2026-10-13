// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual controlled stock actions, with native preview and exact lot quantities.

use super::GameState;
use crate::identity::{SubjectId, Tick};
use crate::{GameError, GameEvent, ItemError, ItemId, ItemKind, ItemLocation, WorldError};
use isocosm::{projects::{DebitPreview, DebitSelection, OwnedActed}, schema::Id};
use isocosm_overlay::{ActKey, EntityHandle, eponym::{EponymIntent, StockAct}};

impl GameState {
    /// A read-only preview of the real native operation at the current cut.
    pub fn prepare_stock_act(&self, subject: SubjectId, process: &str, target: Option<Id>)
        -> Result<DebitPreview, GameError>
    {
        let actor = self.stock_actor(subject, target)?;
        self.world.sim().prepare_owned_act(actor, target, process, None)
            .map_err(|_| WorldError::Refused.into())
    }

    pub fn prepare_owned_meal(&self, subject: SubjectId, item: ItemId)
        -> Result<(String, DebitPreview), GameError>
    {
        let process = self.meal_process(subject, item)?;
        let preview = self.prepare_stock_act(subject, &process, Some(item.0))?;
        Ok((process, preview))
    }

    fn stock_actor(&self, subject: SubjectId, target: Option<Id>) -> Result<Id, GameError> {
        self.living(subject)?;
        let actor = self.entity_of(subject)?;
        let participant = self.participant.ok_or(WorldError::NotPlayed)?;
        if self.world.sim().plays(participant) != Some(actor) {
            return Err(WorldError::NotPlayed.into());
        }
        let at = self.at(subject)?;
        if self.world.sim().state().population.get(actor)
            .and_then(|body| body.resident_site()) != Some(self.world.site())
        { return Err(WorldError::Refused.into()); }
        if let Some(target) = target.filter(|target| *target != actor) {
            let here = if let Some(item) = self.items.get(ItemId(target)) {
                matches!(item.location, ItemLocation::At(position) if position == at)
                    || matches!(item.location, ItemLocation::Carried(owner)
                        | ItemLocation::Attached { subject:owner, .. } if owner == subject)
            } else {
                self.world.subjects().find_map(|(subject, entity)| (entity == target).then_some(subject))
                    .and_then(|subject| self.at(subject).ok()).is_some_and(|position| position == at)
            };
            if !here { return Err(WorldError::Refused.into()); }
        }
        Ok(actor)
    }

    pub(super) fn stock_act(&mut self, tick: Tick, subject: SubjectId, process: &str,
        target: Option<Id>, selections: &[DebitSelection]) -> Result<Vec<GameEvent>, GameError>
    {
        let actor = self.stock_actor(subject, target)?;
        let selections = serde_json::to_vec(selections).map_err(|_| GameError::Encode)?;
        let out = self.world.submit(self.participant, &EponymIntent::StockAct(StockAct {
            subject:EntityHandle(actor), process:ActKey(process.into()),
            target:target.map(EntityHandle), selections,
        }))?;
        let result: OwnedActed = serde_json::from_str(out.outcomes.last().ok_or(WorldError::Refused)?)
            .map_err(|_| WorldError::Decode)?;
        if !result.receipt.accepted() { return Err(WorldError::Refused.into()); }
        Ok(vec![GameEvent::StockAct { tick, subject, process:process.into(), result }])
    }

    fn meal_process(&self, subject: SubjectId, item: ItemId) -> Result<String, GameError> {
        self.living(subject)?;
        let food = self.items.get(item).ok_or(ItemError::Missing(item))?;
        if food.location != ItemLocation::Carried(subject) {
            return Err(ItemError::NotCarried(item, subject).into());
        }
        if food.kind != ItemKind::Food { return Err(ItemError::WrongKind(item, food.kind).into()); }
        Ok(match self.world.material_profile() {
            Some(_) => self.world.material_consumption(item)
                .ok_or(WorldError::Unbuilt("this recipe has no declared consumption process"))?,
            None => crate::founding::EAT,
        }.into())
    }

    pub(super) fn eat_owned(&mut self, tick: Tick, subject: SubjectId, item: ItemId,
        selections: &[DebitSelection]) -> Result<Vec<GameEvent>, GameError>
    {
        let process = self.meal_process(subject, item)?;
        let mut events = self.stock_act(tick, subject, &process, Some(item.0), selections)?;
        let sim = self.world.sim();
        let food = sim.state().population.get(item.0).ok_or(WorldError::Refused)?;
        let stocks = isocosm::anatomy::books(food);
        let remains = stocks.iter().any(|(key, amount)| *amount > 0
            && (matches!(sim.genesis().rules.accounts.get(key),
                Some(isocosm::rules::AccountKind::Matter { .. })) || sim.state().spreads.owns(key)));
        if !remains { self.act(subject, Some(item.0), crate::founding::SPENT)?; }
        events.push(GameEvent::Ate { tick, subject, item,
            hunger:self.need(subject, crate::founding::RESERVE) });
        self.exert(subject, 0, 1, tick, events)
    }
}
