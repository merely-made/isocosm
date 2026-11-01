// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Saved journeys, with an advance-local derived arrival index (817, 820–823).
//! Arrival is clock work, outside every process pass and its choice cache.

use crate::{Result, population::Population, schema::*, simulation::{Simulation, Work}};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(crate) struct Journeys {
    due: BTreeSet<(Tick, Id)>,
    at: BTreeMap<Id, Tick>,
}

impl Journeys {
    pub(crate) fn new(population: &Population) -> Self {
        let mut index = Self::default();
        index.touch(population, population.groups.keys().copied());
        index
    }

    pub(crate) fn touch(&mut self, population: &Population, firsts: impl IntoIterator<Item = Id>) {
        for first in firsts {
            if let Some(was) = self.at.remove(&first) {
                self.due.remove(&(was, first));
            }
            if let Some(group) = population.groups.get(&first)
                && let Some(journey) = &group.entity.journey
            {
                self.at.insert(first, journey.arrives);
                self.due.insert((journey.arrives, first));
            }
        }
    }

    fn pop(&mut self, until: Tick) -> Option<(Tick, Id)> {
        let &(tick, first) = self.due.first()?;
        if tick > until { return None; }
        self.due.pop_first();
        self.at.remove(&first);
        Some((tick, first))
    }

    pub(crate) fn next(&self) -> Option<Tick> { self.due.first().map(|(tick, _)| *tick) }
}

/// Existing authored periods still determine when route acts are due (256).
pub(crate) fn depart(entity: &mut Entity, site: &Site, to: Id, via: Option<u8>, tick: Tick, routed: bool) -> Result<()> {
    if entity.journey.is_some() { return Err("member is already travelling".into()); }
    let travel = route(site, to, via).ok_or("no route to destination")?.travel;
    let journey = departure(entity, site, to, via, tick, travel, routed)?;
    leave(entity, journey);
    Ok(())
}

/// The route an act takes: the one leaving by `via`, else the first to `to`.
/// Parallel routes between one pair differ by their border (1057).
pub fn route(site: &Site, to: Id, via: Option<u8>) -> Option<&Route> {
    site.routes.iter().find(|r| r.to == to && via.is_none_or(|side| r.border.is_some_and(|b| b.side == side)))
}

/// Routed movement records the border crossed; earlier semantics never do.
pub(crate) fn departure(entity: &Entity, site: &Site, to: Id, via: Option<u8>, tick: Tick, travel: Tick, routed: bool)
    -> Result<Journey> {
    if entity.journey.is_some() { return Err("member is already travelling".into()); }
    let route = route(site, to, via).ok_or("no route to destination")?;
    if travel == 0 { return Err("route travel must be positive".into()); }
    let arrives = tick.checked_add(travel).ok_or("journey arrival tick overflows")?;
    Ok(Journey { from: entity.place, to, departed: tick, arrives, border: if routed { route.border } else { None } })
}

pub(crate) fn leave(entity: &mut Entity, journey: Journey) {
    if entity.arrived < journey.departed {
        entity.visits.push(Visit { place: entity.place, from: entity.arrived, until: journey.departed });
    }
    entity.journey = Some(journey);
    entity.patch = None;
}

impl Simulation {
    /// Quote a direct authored pace against actual current residence and clock.
    /// This does not admit the process, its stock costs, or participant control.
    pub fn paced_journey(&self, actor: Id, process: &str, to: Id) -> Result<Journey> {
        if self.state.movement.immediate() { return Err("paced travel requires native journeys".into()); }
        let entity = self.state.population.get(actor).ok_or("movement actor is absent")?;
        let from = entity.resident_site().ok_or("movement actor has no resident site")?;
        let site = self.state.sites.get(&from).ok_or("departure site absent")?;
        let (_, profile, via) = self.genesis.rules.paced_route(process, to)?;
        let route = route(site, to, via).ok_or("no route to destination")?;
        departure(entity, site, to, via, self.state.tick, profile.duration(route.travel)?, self.state.movement.routed())
    }

    /// Land exact cohorts, including corpses, before installing the next pass.
    /// The index is rebuilt per advance; accepted splits/births replace entries.
    pub(crate) fn arrive_until(&mut self, until: Tick, work: &mut Work) -> Result<()> {
        while let Some((tick, first)) = self.journeys.as_mut().and_then(|j| j.pop(until)) {
            // A held item follows its carrier's accepted clock, never a second arrival.
            if self.item_carrier(first)?.is_some() { continue; }
            let before = (self.genesis.rules.spread_capacity.is_some()
                || self.genesis.rules.item_custody.is_some()).then(|| self.state.clone());
            let group = self.state.population.groups.get(&first).ok_or("journey cohort vanished")?;
            let count = group.count;
            let journey = group.entity.journey.clone().ok_or("journey index lost its authority")?;
            if journey.arrives != tick || tick < self.state.tick {
                return Err("journey clock differs from saved authority".into());
            }
            let arrivals = work.arrivals.checked_add(count).ok_or("arrival work overflow")?;
            // 259 budgets authored evaluations, not clock-only arrivals.
            if !self.state.sites.contains_key(&journey.to) { return Err("journey destination absent".into()); }
            self.state.tick = tick;
            if let Some(j) = &mut self.journal { j.group(first, Some(group)); }
            let entity = self.state.population.groups.get_mut(&first).unwrap().entity_mut();
            entity.place = journey.to;
            entity.arrived = tick;
            entity.journey = None;
            entity.patch = None;
            self.place_arrivals(first, count, &journey)?;
            if let Some(before) = &before { self.overflow_resolved(before)?; }
            // Carried soil stays in its conserved body ledger until an authored deposit.
            // No arrival event, selected process, receipt, draw or invented site flow.
            work.arrivals = arrivals;
            if let Some(t) = &mut self.targets { t.touch(&self.state.population, [first]); }
            if let Some(f) = &mut self.filed {
                let world = crate::schedule::World::at(&self.state, &self.genesis.rules, tick);
                f.touch(&self.state.population, [first], &world);
            }
        }
        Ok(())
    }

    /// A direct route act cannot be chosen while its actor is off-site (817).
    /// Conditional/computed branches retain their ordinary interpreter checks.
    pub(crate) fn route_ready(&self, actor: Id, p: &crate::rules::Process) -> bool {
        if self.state.movement.immediate() { return true; }
        let moves = p.commitments.iter().chain(&p.effects).filter_map(|e| match e {
            crate::rules::Effect::Move { destination, via }
                | crate::rules::Effect::MovePaced { destination, via, .. } => Some((*destination, *via)), _ => None,
        }).collect::<Vec<_>>();
        if moves.is_empty() { return true; }
        let Some(from) = self.body_at_start(actor).and_then(Entity::resident_site) else { return false; };
        self.site_at_start(from).is_some_and(|s| moves.iter().all(|(to, via)| route(s, *to, *via).is_some()))
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod location_tests;
