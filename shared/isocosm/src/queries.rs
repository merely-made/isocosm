// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::*, schema::*, simulation::*};

impl Simulation {
    pub(crate) fn bound(&self, actor: Id, target: Option<Id>, binding: Binding) -> Result<Id> {
        match binding {
            Binding::Actor => Ok(actor),
            Binding::Target => target.ok_or("target is required".into()),
            Binding::Place => Err("place is not a body".into()),
        }
    }
    pub(crate) fn ledger(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        binding: Binding,
    ) -> Result<&Ledger> {
        if binding == Binding::Place {
            return Ok(&self.state.sites.get(&place).ok_or("site missing")?.accounts);
        }
        let id = self.bound(actor, target, binding)?;
        Ok(&self
            .state
            .population
            .get(id)
            .ok_or("body missing")?
            .accounts)
    }
    pub(crate) fn ledger_mut(
        &mut self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        binding: Binding,
    ) -> Result<&mut Ledger> {
        if binding == Binding::Place {
            return Ok(&mut self
                .state
                .sites
                .get_mut(&place)
                .ok_or("site missing")?
                .accounts);
        }
        let id = self.bound(actor, target, binding)?;
        Ok(&mut self
            .state
            .population
            .groups
            .get_mut(&id)
            .ok_or("body binding changed")?
            .entity
            .accounts)
    }
    pub(crate) fn query(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        query: &Query,
    ) -> Result<String> {
        let body = |b| -> Result<&Entity> {
            self.state
                .population
                .get(self.bound(actor, target, b)?)
                .ok_or("body missing".into())
        };
        let (yes, reading) = match query {
            Query::Alive(b) => {
                let v = body(*b)?.alive;
                (v, v.to_string())
            },
            Query::Trait { who, key } => {
                let v = body(*who)?.traits.contains(key);
                (v, v.to_string())
            },
            Query::Account { who, key, at_least } => {
                let v = self
                    .ledger(actor, target, place, *who)?
                    .get(key)
                    .copied()
                    .unwrap_or(0);
                (v >= *at_least, v.to_string())
            },
            Query::Below { who, key, amount } => {
                let v = self
                    .ledger(actor, target, place, *who)?
                    .get(key)
                    .copied()
                    .unwrap_or(0);
                (v < *amount, v.to_string())
            },
            Query::Age { at_least } => {
                let v = self.state.tick.saturating_sub(body(Binding::Actor)?.born);
                (v >= *at_least, v.to_string())
            },
            Query::Condition { key, at_least } => {
                let v = self
                    .state
                    .sites
                    .get(&place)
                    .and_then(|s| s.conditions.get(key));
                (v.is_some_and(|v| v >= at_least), format!("{v:?}"))
            },
            Query::Part {
                who,
                revision,
                part,
            } => {
                let b = body(*who)?;
                let p = b.parts.get(part);
                (
                    b.body_revision == *revision && p.is_some_and(|p| !p.severed),
                    format!("revision {}: {p:?}", b.body_revision),
                )
            },
            Query::Related { kind } => {
                let v = self.state.relations.contains(&Relation {
                    subject: actor,
                    kind: kind.clone(),
                    object: target.ok_or("target required")?,
                });
                (v, v.to_string())
            },
        };
        if yes {
            Ok(format!("{query:?} = {reading}"))
        } else {
            Err(format!("missing requirement: {query:?} = {reading}"))
        }
    }
    pub(crate) fn target_matches(&self, actor: Id, target: Option<Id>, p: &Process) -> bool {
        let Some(selector) = &p.target else {
            return true;
        };
        let Some(target) = target.filter(|id| *id != actor) else {
            return false;
        };
        let Some(a) = self.state.population.get(actor) else {
            return false;
        };
        let Some(b) = self.state.population.get(target) else {
            return false;
        };
        (!selector.same_place || a.place == b.place)
            && selector.alive.is_none_or(|alive| alive == b.alive)
            && selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| lineage == &b.lineage)
    }
    pub(crate) fn choose_target(&self, actor: Id, p: &Process) -> Option<Id> {
        p.target.as_ref()?;
        let entity = self.state.population.get(actor)?;
        // Reject an ineligible actor before searching the population for food.
        // This is only a read shortcut; apply still checks every requirement.
        if p.requires.iter().any(|q| {
            matches!(q, Query::Trait {who:Binding::Actor,key}
            if !entity.traits.contains(key))
        }) {
            return None;
        }
        let place = entity.place;
        for (&first, group) in &self.state.population.groups {
            let candidate = if first == actor {
                first.checked_add(1)?
            } else {
                first
            };
            if candidate >= first + group.count {
                continue;
            }
            if self.target_matches(actor, Some(candidate), p)
                && p.requires
                    .iter()
                    .all(|q| self.query(actor, Some(candidate), place, q).is_ok())
            {
                return Some(candidate);
            }
        }
        None
    }
}
