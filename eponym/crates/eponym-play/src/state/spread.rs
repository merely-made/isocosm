// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Author actions are recorded game intents over actual native admission.

use super::*;
use crate::{ActorRef, HolderRef, IdentityRef, AuthorSupply, AuthorSpreadStep};
use isocosm::{asserted::Assertion, projects::{SpreadPreview, StockLot, StockSource},
    spread::{self, Identity, Supply}};

impl GameState {
    pub fn spread_actor(&self, key: &str) -> Option<Id> {
        self.events.iter().find_map(|event| match event {
            GameEvent::SpreadActorFounded { key: named, entity, .. } if named == key =>
                self.world.sim().state().population.get(*entity).map(|_| *entity),
            _ => None,
        })
    }

    fn author_spread_access(&self) -> Result<(), GameError> {
        if self.participant.is_some() || self.world.spread_profile().is_none() {
            return Err(WorldError::NotFounding.into());
        }
        Ok(())
    }

    fn spread_actor_ref(&self, actor: &ActorRef) -> Result<Id, GameError> {
        let entity = match actor {
            ActorRef::Native(id) => *id,
            ActorRef::Founder(key) => self.spread_actor(key).ok_or(WorldError::Refused)?,
            ActorRef::Subject(subject) => self.world.entity(*subject).ok_or(WorldError::Refused)?,
        };
        if self.world.sim().state().population.get(entity).is_none() { return Err(WorldError::Refused.into()); }
        Ok(entity)
    }

    fn spread_identity(&self, identity: &IdentityRef) -> Result<Identity, GameError> {
        match identity {
            IdentityRef::Territory(actor) => Ok(Identity::Territory(self.spread_actor_ref(actor)?)),
            IdentityRef::Surface(key) if self.world.sim().state().lineages.contains_key(key) =>
                Ok(Identity::Surface(key.clone())),
            _ => Err(WorldError::Refused.into()),
        }
    }

    fn spread_holder(&self, holder: &HolderRef) -> Result<StockSource, GameError> {
        let state = self.world.sim().state();
        match holder {
            HolderRef::Site(id) if state.sites.contains_key(id) => Ok(StockSource::Site(*id)),
            HolderRef::Place(id) if state.spreads.places.iter().any(|place| place.place == *id) =>
                Ok(StockSource::Place(*id)),
            HolderRef::Host(actor) => Ok(StockSource::Entity(self.spread_actor_ref(actor)?)),
            HolderRef::Transit(id) if state.spreads.transit_marks.iter().any(|mark| mark.id == *id) =>
                Ok(StockSource::Transit(*id)),
            _ => Err(WorldError::Refused.into()),
        }
    }

    fn spread_command<'a>(&self, step: &'a AuthorSpreadStep)
        -> Result<(Command, Option<&'a [isocosm::projects::DebitSelection]>), GameError> {
        use crate::author_spread::{substrate, placement};
        let pair = match step {
            AuthorSpreadStep::Register { identity } =>
                (spread::Command::Register(self.spread_identity(identity)?), None),
            AuthorSpreadStep::Place { placement: authored, selections } => {
                let source = match &authored.source {
                    AuthorSupply::Matter { from, account } => Supply::Matter {
                        from: substrate(self.spread_holder(from)?)?, account: account.clone() },
                    AuthorSupply::Spread { from, identity } => Supply::Spread {
                        from: substrate(self.spread_holder(from)?)?, identity: self.spread_identity(identity)? },
                };
                let place = placement(source, self.spread_identity(&authored.identity)?,
                    substrate(self.spread_holder(&authored.at)?)?, authored.amount);
                (spread::Command::Place(place), Some(selections.as_slice()))
            },
            AuthorSpreadStep::Overflow { identity, at, selections } =>
                (spread::Command::Overflow { identity: self.spread_identity(identity)?,
                    at: substrate(self.spread_holder(at)?)? }, Some(selections.as_slice())),
            AuthorSpreadStep::ForkStrain { parent, at, amount, traits, selections } =>
                (spread::Command::ForkStrain { parent: parent.clone(), at: substrate(self.spread_holder(at)?)?,
                    amount: *amount, traits: traits.clone() }, Some(selections.as_slice())),
        };
        Ok((Command::Spread(pair.0), pair.1))
    }

    pub fn prepare_authored_spread(&self, step: &AuthorSpreadStep) -> Result<SpreadPreview, GameError> {
        self.author_spread_access()?;
        let (command, selected) = self.spread_command(step)?;
        if selected.is_none() { return Err(WorldError::Unbuilt("registration has no owned debit preview").into()); }
        self.world.prepare_spread_command(&command).map_err(Into::into)
    }

    pub(super) fn admit_spread_author(&mut self, intent: &GameIntent) -> Result<Option<Vec<GameEvent>>, GameError> {
        if !matches!(intent, GameIntent::FoundSpreadActor { .. } | GameIntent::SpreadOwnership { .. }
            | GameIntent::AuthoredSpread { .. }) { return Ok(None); }
        self.author_spread_access()?;
        let events = match intent {
            GameIntent::FoundSpreadActor { tick, key, recipe, site } => {
                if key.trim().is_empty() || self.events.iter().any(|event|
                    matches!(event, GameEvent::SpreadActorFounded { key: named, .. } if named == key)) {
                    return Err(WorldError::Refused.into());
                }
                let founder = self.world.spread_profile().and_then(|profile| profile.native.founders.get(recipe))
                    .ok_or(WorldError::Refused)?.clone();
                let mut arrival = founder.recipe.arrival(*site).map_err(|_| WorldError::Refused)?;
                arrival.method = founder.method;
                let outcome = self.world.found(Command::Arrive(arrival))?;
                let entity = outcome.strip_prefix("entity:").and_then(|id| id.parse().ok()).ok_or(GameError::Decode)?;
                if let Some(seed) = founder.development_seed { self.world.found(Command::Develop { entity, seed })?; }
                vec![GameEvent::SpreadActorFounded { tick: *tick, key: key.clone(), recipe: recipe.clone(), site: *site, entity }]
            },
            GameIntent::SpreadOwnership { identity, lot, .. } => {
                let identity = self.spread_identity(identity)?;
                let lot = StockLot { key: lot.key.clone(), source: self.spread_holder(&lot.source)?,
                    account: identity.key(), amount: lot.amount, owners: lot.owners.clone(), collectives: lot.collectives.clone() };
                self.world.found(Command::Assert(Assertion::SpreadOwnership { identity, lot }))?;
                Vec::new()
            },
            GameIntent::AuthoredSpread { tick, step } => {
                let (command, selections) = self.spread_command(step)?;
                let command = if let Some(selections) = selections {
                    let preview = self.world.prepare_spread_command(&command)?;
                    if preview.debits.refusal.is_some() || preview.debits.requests.len() != selections.len()
                        || preview.debits.requests.iter().any(|request| selections.iter()
                            .filter(|selection| &selection.request == request).count() != 1) {
                        return Err(WorldError::Refused.into());
                    }
                    Command::Owned { command: Box::new(command), selections: selections.to_vec() }
                } else { command };
                let outcome = self.world.run(command)?;
                vec![GameEvent::AuthoredSpread { tick: *tick, outcome }]
            },
            _ => return Err(GameError::Decode),
        };
        Ok(Some(events))
    }
}
