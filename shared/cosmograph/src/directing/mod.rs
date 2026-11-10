// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Directing (rulings 177, 194, 683 to 691): a participant is a placeless
//! entity, as the world is one (98); its bond to a critter is a weighted
//! relation (688); a nudge is a logged command naming a site or a thing, to
//! attend or to act (686, 690), which a deliberative critter weighs by that
//! bond when it chooses (683).

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod choice;
mod holding;
mod infeasible;
mod learn;
pub mod found;
pub mod interim;
pub mod policy;
pub mod readings;
pub use crate::lineage::revise;
pub mod tier;

pub(crate) use choice::{Chosen, Deliberated};
pub(crate) use learn::inherited;
pub use found::Played;
pub use tier::{Tier, TierLine};

/// The place a participant stands at: none.
pub const PLACELESS: Id = Id::MAX;
pub const PARTICIPANT: &str = "kingdom:participant";
/// A participant's bond to a critter, its value the weight (688).
pub const BOND: &str = "directing:bond";
/// A lineage a participant knows, by an entity of it (case 1210).
pub const KNOWS: &str = "directing:knows";
/// The critter a participant plays now.
pub const PLAYS: &str = "directing:plays";
/// The kingdom of the world-class lineages, which have no critter body.
pub const WORLD: &str = "kingdom:world";

/// What a nudge asks: to attend, or to act (690).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Aim {
    Attend,
    Act,
}

/// What a nudge names: a site, at this grain a place (690), or a thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Toward {
    Site(Id),
    Thing(Id),
}

/// How a nudge was answered: the act that answered it, where, and whether
/// it served the critter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub tick: Tick,
    pub process: Key,
    pub site: Id,
    pub served: bool,
}

impl Answer {
    pub fn resident_site(&self) -> Option<Id> { (self.site != PLACELESS).then_some(self.site) }
}

/// A nudge as the world keeps it, in the order given.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nudge {
    pub tick: Tick,
    pub participant: Id,
    pub critter: Id,
    pub aim: Aim,
    pub toward: Toward,
    /// The act it names, a process key, where the player named one (784).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub act: Option<Key>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<Answer>,
}

pub fn is_participant(e: &Entity) -> bool {
    e.kingdom == PARTICIPANT
}

/// Holds or drops `relation` whatever its value: one per subject, kind and
/// object, a held one keeping its value.
pub(crate) fn hold(relations: &mut BTreeSet<Relation>, relation: Relation, present: bool) {
    let held = related(relations, relation.subject, &relation.kind, relation.object).cloned();
    match (held, present) {
        (None, true) => {
            relations.insert(relation);
        },
        (Some(held), false) => {
            relations.remove(&held);
        },
        _ => {},
    }
}

impl Simulation {
    /// The weight of `participant`'s bond to `critter`, if they have one.
    pub fn bond(&self, participant: Id, critter: Id) -> Option<i64> {
        related(&self.state.relations, participant, BOND, critter).map(|r| r.value)
    }
    /// The critter `participant` plays now: its one body where its product
    /// keeps one, else the lowest of those it plays (1065).
    pub fn plays(&self, participant: Id) -> Option<Id> {
        self.played(participant).into_iter().next()
    }
    /// Every body `participant` plays, lowest first (1062, 1065).
    pub fn played(&self, participant: Id) -> Vec<Id> {
        let at = |object| Relation::new(participant, PLAYS, object);
        let held = self.state.relations.range(at(0)..=at(Id::MAX));
        held.filter(|r| r.kind == PLAYS).map(|r| r.object).collect()
    }
    /// The body a reading views as: `requested` when the participant plays
    /// it, else its one or lowest played body (1065).
    pub fn observer(&self, participant: Id, requested: Option<Id>) -> Result<Option<Id>> {
        match requested {
            Some(id) if !self.plays_body(participant, id) =>
                Err("the reading's body is not one this participant plays".into()),
            Some(id) => Ok(Some(id)),
            None => Ok(self.plays(participant)),
        }
    }
    /// Whether `participant` plays `critter`, among however many it plays.
    pub fn plays_body(&self, participant: Id, critter: Id) -> bool {
        self.state.relations.iter().any(|r| r.kind == PLAYS && r.subject == participant && r.object == critter)
    }
    /// The participants bonded to `critter`, with each bond's weight.
    pub fn bonded(&self, critter: Id) -> Vec<(Id, i64)> {
        let rs = &self.state.relations;
        let bonds = rs.iter().filter(|r| r.kind == BOND && r.object == critter);
        bonds.map(|r| (r.subject, r.value)).collect()
    }
    fn participant(&self, id: Id) -> Result<&Entity> {
        let e = self.state.population.get(id).filter(|e| is_participant(e));
        e.ok_or_else(|| format!("{id} is no participant"))
    }
    /// A participant joins, placeless and kept (688).
    pub(crate) fn join(&mut self) -> Result<Id> {
        let entity = Entity {
            drawn: None,
            lineage: "participant:player".into(),
            kingdom: PARTICIPANT.into(),
            scale: "scale:meso".into(),
            provenance: Provenance::Intrinsic("participant:player".into()),
            method: Method::Inert,
            place: PLACELESS,
            arrived: self.state.tick,
            visits: vec![],
            journey: None,
            born: self.state.tick,
            alive: true,
            body_revision: 1,
            body: None,
            parts: Default::default(),
            traits: Default::default(),
            accounts: Default::default(),
            skills: Default::default(),
            tenets: Default::default(),
            disposition: [0; 5],
            soma: vec![],
            absent: vec![],
            systems: Default::default(),
            varied: vec![],
            patch: None,
            authored: None,
            settled: None,
            spent: None,
            learned: None,
        };
        if self.state.population.count() >= self.genesis.rules.limits.entities {
            return Err("entity limit".into());
        }
        let id = self.state.population.insert(entity, 1)?;
        self.state.roots.insert(id);
        Ok(id)
    }
    /// `participant` takes up `critter`: a bond it already holds keeps its
    /// weight; otherwise the bond to the critter it played before seeds it
    /// by the world's setting, where both are of one lineage (178).
    pub(crate) fn take(&mut self, participant: Id, critter: Id) -> Result<()> {
        self.take_up(participant, critter, false)
    }
    /// `participant` takes up `critter` beside the bodies it plays (1065),
    /// its bond seeded as `take` seeds it.
    pub(crate) fn take_also(&mut self, participant: Id, critter: Id) -> Result<()> {
        if self.plays_body(participant, critter) {
            return Err("the participant already plays this critter".into());
        }
        self.take_up(participant, critter, true)
    }
    /// `participant` puts down `critter`, one body it plays; its bond is kept,
    /// and the critter acts as itself again (1065).
    pub(crate) fn leave(&mut self, participant: Id, critter: Id) -> Result<()> {
        self.participant(participant)?;
        if !self.plays_body(participant, critter) {
            return Err("the participant does not play this critter".into());
        }
        hold(&mut self.state.relations, Relation::new(participant, PLAYS, critter), false);
        Ok(())
    }
    fn take_up(&mut self, participant: Id, critter: Id, beside: bool) -> Result<()> {
        self.participant(participant)?;
        let e = self
            .state
            .population
            .get(critter)
            .ok_or("unknown critter")?;
        // One path takes up a life in both games (779): a deliberative
        // critter is directed, a normative one, a sophont, is driven (671).
        if !e.alive || !matches!(e.method, Method::Deliberative | Method::Normative) {
            return Err("only a living deliberative critter or sophont is taken up".into());
        }
        let lineage = e.lineage.clone();
        let rules = self.genesis.rules.directing();
        let before = self.plays(participant);
        let forebear = before
            .filter(|b| {
                self.state
                    .population
                    .get(*b)
                    .is_some_and(|f| f.lineage == lineage)
            })
            .and_then(|b| self.bond(participant, b));
        // A lineage's regard for the participant seeds a bond not yet held (case 1214).
        let regard = rules.regard.and(self.state.lineages.get(&lineage))
            .and_then(|l| l.regard.get(&participant)).copied();
        let weight = self
            .bond(participant, critter)
            .unwrap_or_else(|| regard.map_or_else(|| rules.inherit(forebear), |r| r.clamp(0, rules.most)));
        // Take switches: every body played before is put down. TakeAlso keeps them.
        if !beside {
            for before in self.played(participant) {
                hold(&mut self.state.relations, Relation::new(participant, PLAYS, before), false);
            }
        }
        self.set_bond(participant, critter, weight);
        let plays = Relation::new(participant, PLAYS, critter);
        hold(&mut self.state.relations, plays, true);
        // The played critter is its own row, so it acts as itself.
        self.state.population.lift(critter)?;
        Ok(())
    }
    /// Sets a bond's weight, keeping what it was for an advance to put back.
    pub(crate) fn set_bond(&mut self, participant: Id, critter: Id, weight: i64) {
        let new = Relation {
            value: weight,
            ..Relation::new(participant, BOND, critter)
        };
        let s = &mut self.state;
        let held = related(&s.relations, participant, BOND, critter).cloned();
        if let Some(j) = &mut self.journal {
            if let Some(held) = &held {
                j.relation(held, true);
            }
            j.relation(&new, s.relations.contains(&new));
        }
        if let Some(held) = held {
            s.relations.remove(&held);
        }
        s.relations.insert(new);
    }
    /// Records what `participant` has come to know and each lineage's new
    /// regard for it (cases 1210, 1214); refused unless the rule is set.
    pub(crate) fn set_regard(&mut self, participant: Id, knows: &[Id], regard: &[(Key, i64)]) -> Result<()> {
        self.participant(participant)?;
        let most = match self.genesis.rules.directing().regard {
            Some(_) => self.genesis.rules.directing().most,
            None => return Err("the world keeps no regard".into()),
        };
        if let Some(missing) = knows.iter().find(|id| self.state.population.get(**id).is_none()) {
            return Err(format!("{missing} is no entity to know a lineage by"));
        }
        if let Some((key, _)) = regard.iter().find(|(k, _)| !self.state.lineages.contains_key(k)) {
            return Err(format!("{key} is no lineage"));
        }
        for object in knows {
            let new = Relation::new(participant, KNOWS, *object);
            if related(&self.state.relations, participant, KNOWS, *object).is_none() {
                if let Some(j) = &mut self.journal {
                    j.relation(&new, false);
                }
                self.state.relations.insert(new);
            }
        }
        for (key, value) in regard {
            if let Some(line) = self.state.lineages.get_mut(key) {
                line.regard.insert(participant, (*value).clamp(0, most));
            }
        }
        Ok(())
    }
    /// Logs a nudge; refused unless the participant is bonded to a living
    /// critter and what it names is there (686, 690).
    pub(crate) fn nudge(
        &mut self,
        participant: Id,
        critter: Id,
        aim: Aim,
        to: Toward,
        act: Option<Key>,
    ) -> Result<()> {
        self.participant(participant)?;
        if !self.state.population.get(critter).is_some_and(|e| e.alive) {
            return Err("the critter is not living".into());
        }
        if self.bond(participant, critter).is_none() {
            return Err("no bond to nudge by".into());
        }
        let named = match to {
            Toward::Site(id) => self.state.sites.contains_key(&id),
            Toward::Thing(id) => self
                .state
                .population
                .get(id)
                .is_some_and(|e| !is_participant(e)),
        };
        if !named {
            return Err("the nudge names nothing in the world".into());
        }
        self.state.nudges.push(Nudge {
            tick: self.state.tick,
            participant,
            critter,
            aim,
            toward: to,
            act,
            answer: None,
        });
        Ok(())
    }
    /// Each bonded participant's latest nudge to `critter`, while it is
    /// unanswered and live at `tick`, by index.
    pub(crate) fn live_nudges(&self, critter: Id, tick: Tick) -> Vec<usize> {
        let span = self.genesis.rules.directing().span;
        let mut latest = std::collections::BTreeMap::new();
        for (i, n) in self.state.nudges.iter().enumerate() {
            if n.critter == critter && n.tick <= tick {
                latest.insert(n.participant, i);
            }
        }
        let live = |i: &usize| {
            let n = &self.state.nudges[*i];
            n.answer.is_none() && tick < n.tick.saturating_add(span)
        };
        latest.into_values().filter(live).collect()
    }
}

#[cfg(test)]
mod choice_tests;
#[cfg(test)]
pub(crate) mod holding_tests;
#[cfg(test)]
mod known_tests;
#[cfg(test)]
mod movement_tie_tests;
#[cfg(test)]
mod feeding_tests;
#[cfg(test)]
mod pass_start_tests;
#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod ruled_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tier_tests;
#[cfg(test)]
mod wound_tests;

#[cfg(test)]
mod learning_tests;
