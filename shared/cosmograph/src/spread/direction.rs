// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An actual territory's authored choice of outgoing native routes (498).

use super::*;
use crate::{Result, rules::{Effect, Process, Rules}, schema::Id, simulation::{Simulation, State}};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(crate) struct Direction { pub actor: Id, pub from: Id, pub destinations: BTreeSet<Id> }

pub(crate) fn declarations(process: &Process) -> Vec<(Id, &BTreeSet<Id>)> {
    process.commitments.iter().chain(&process.effects).filter_map(|effect| match effect {
        Effect::DirectOverflow { from, destinations } => Some((*from, destinations)),
        _ => None,
    }).collect()
}

fn routes(state: &State, from: Id, destinations: &BTreeSet<Id>) -> Result<Vec<Dispatch>> {
    if destinations.is_empty() { return Err("directed overflow selects no destination".into()); }
    let source = state.sites.get(&from).ok_or("directed overflow source site is absent")?;
    destinations.iter().map(|destination| {
        let matches: Vec<_> = source.routes.iter().enumerate().filter(|(_, route)| route.to == *destination).collect();
        if matches.len() != 1 { return Err("directed overflow destination needs one unambiguous authored route".into()); }
        let (route, physical) = matches[0];
        if physical.transmission == 0 || physical.transmission > 1_000_000 || physical.travel == 0
            || *destination == from || !state.sites.contains_key(destination) {
            return Err("directed overflow selected an ineligible native route".into());
        }
        Ok(Dispatch { route, offered: 0 })
    }).collect()
}

/// Native capacity and route rates resolve the whole conserved share.
pub fn resolved_overflow_routes(state: &mut State, rules: &Rules, actor: Id, from: Id,
    destinations: &BTreeSet<Id>) -> Result<u64> {
    let identity = Identity::Territory(actor);
    let at = Substrate::Site(from);
    let mut selected = routes(state, from, destinations)?;
    let limit = capacity(state, rules, at, &identity)?;
    let excess = held(state, at, &identity)?.saturating_sub(limit);
    if excess == 0 { return Err("directed territory has no local capacity excess".into()); }
    let count = u64::try_from(selected.len()).map_err(|_| "directed overflow route count overflows")?;
    for choice in &mut selected { choice.offered = excess / count; }
    owned::operation(state, rules, crate::projects::SpreadDebit::Overflow, &identity, at,
        |candidate| overflow_at(candidate, &identity, at, limit, &selected))
}

impl Simulation {
    pub(crate) fn direction_ready(&self, actor: Id, process: &Process) -> Result<()> {
        for (from, destinations) in declarations(process) {
            routes(self.state(), from, destinations)?;
            let identity = Identity::Territory(actor);
            let at = Substrate::Site(from);
            if held(self.state(), at, &identity)? <= capacity(self.state(), &self.genesis.rules, at, &identity)? {
                return Err("directed territory has no local capacity excess".into());
            }
        }
        Ok(())
    }

    pub(crate) fn publish_direction(&mut self, direction: &Direction, process: &str, action: &str) -> Result<()> {
        use crate::projects::{DebitOrigin, debit::{Context, Mode}};
        let created = self.state.projects.debit_capture.with(|_| Ok(())).map(|v| v.is_none())?;
        if created {
            let context = Context::new(&self.state, &self.genesis.rules,
                crate::digest(&(&self.revision, &self.state)),
                DebitOrigin::Process { process: process.into(), actor: direction.actor, target: None },
                action.into(), Mode::Authored);
            self.state.projects.debit_capture.set(Some(context));
        }
        let result = resolved_overflow_routes(&mut self.state, &self.genesis.rules, direction.actor,
            direction.from, &direction.destinations);
        if created {
            let context = self.state.projects.debit_capture.take().ok_or("directed overflow lost debit capture")?;
            result?;
            context.complete()?;
        } else { result?; }
        Ok(())
    }
}
