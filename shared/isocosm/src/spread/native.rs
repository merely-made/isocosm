// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, schema::{Key, Tick}, simulation::{Simulation, State, Work}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Authored spread changes and native capacity-driven spill, recorded by Session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Register(Identity),
    ForkStrain { parent: Key, at: Substrate, amount: u64, traits: BTreeSet<Key> },
    SetModifier(CapacityModifier),
    RemoveModifier { key: Key },
    Overflow { identity: Identity, at: Substrate },
}

fn validate_state(state: &State, rules: &crate::rules::Rules) -> Result<()> {
    validate(&state.spreads, rules, &state.population, &state.sites, &state.lineages)?;
    if state.polities.values().any(|p| p.accounts.keys().any(|k| state.spreads.owns(k))) {
        return Err("spread stocks belong to places or hosts, not polities".into());
    }
    if state.spreads.transit.iter().any(|t| t.departed > state.tick || t.arrives <= state.tick) {
        return Err("spread transit is outside the current clock".into());
    }
    Ok(())
}

impl Simulation {
    /// A candidate is validated and conserved before any native facts change.
    pub fn spread(&mut self, command: &Command) -> Result<String> {
        let mut candidate = self.state.clone();
        let before = self.matter();
        let rules = &self.genesis.rules;
        let outcome = match command {
            Command::Register(identity) => {
                candidate.spreads.identities.insert(identity.clone());
                "registered".into()
            },
            Command::ForkStrain { parent, at, amount, traits } => {
                fork_strain(&mut candidate, rules, self.genesis.dynamics_seed(), parent,
                    *at, *amount, traits.clone())?
            },
            Command::SetModifier(modifier) => {
                candidate.spreads.modifiers.retain(|present| present.key != modifier.key);
                candidate.spreads.modifiers.push(modifier.clone());
                candidate.spreads.modifiers.sort_by(|a, b| a.key.cmp(&b.key));
                "modified".into()
            },
            Command::RemoveModifier { key } => {
                crate::validation::key(key)?;
                candidate.spreads.modifiers.retain(|present| &present.key != key);
                "modifier removed".into()
            },
            Command::Overflow { identity, at } => {
                format!("dispatched:{}", resolved_overflow_at(&mut candidate, rules, identity, *at)?)
            },
        };
        validate_state(&candidate, rules)?;
        dispatch_changed(&self.state, &mut candidate, rules)?;
        validate_state(&candidate, rules)?;
        if crate::simulation::matter(&candidate, rules) != before {
            return Err("spread change does not conserve matter".into());
        }
        self.state = candidate;
        Ok(outcome)
    }

    /// Both independent clock queues land in tick order, before any due pass.
    /// A spread transfer is not counted as a member in Work::arrivals.
    pub(crate) fn arrive_all_until(&mut self, until: Tick, work: &mut Work) -> Result<()> {
        loop {
            let journey = self.journeys.as_ref().and_then(crate::journey::Journeys::next);
            let spread = self.state.spreads.transit.iter().map(|t| t.arrives).min();
            let Some(tick) = journey.into_iter().chain(spread).min().filter(|t| *t <= until) else { break; };
            if tick < self.state.tick { return Err("spread or journey arrival precedes the current clock".into()); }
            self.arrive_until(tick, work)?;
            self.land_spreads(tick)?;
        }
        Ok(())
    }

    fn land_spreads(&mut self, tick: Tick) -> Result<()> {
        if !self.state.spreads.transit.iter().any(|t| t.arrives <= tick) { return Ok(()); }
        let mut candidate = self.state.clone();
        let before = self.matter();
        settle(&mut candidate, tick)?;
        candidate.tick = tick;
        dispatch_changed(&self.state, &mut candidate, &self.genesis.rules)?;
        validate_state(&candidate, &self.genesis.rules)?;
        if crate::simulation::matter(&candidate, &self.genesis.rules) != before {
            return Err("spread arrival does not conserve matter".into());
        }
        if let Some(journal) = &mut self.journal {
            for transfer in &self.state.spreads.transit {
                if transfer.arrives <= tick {
                    let site = self.state.sites.get(&transfer.to).ok_or("spread destination absent")?;
                    journal.site(transfer.to, site);
                }
            }
            for (id, site) in &self.state.sites {
                if candidate.sites.get(id) != Some(site) { journal.site(*id, site); }
            }
        }
        self.state = candidate;
        Ok(())
    }
}
