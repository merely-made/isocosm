// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent C11 stock interpreter. Its candidates contain histogram and
//! substrate data, never a native Simulation or reconstructed native State.

mod direction;
mod named;
mod stocks;
mod transport;
mod strain;

pub(crate) use direction::directing;

use super::*;
use crate::spread::{self, CapacityModifier, Identity, Substrate, Transfer};
use crate::rules::Rules;
use std::collections::BTreeSet;

/// Equal unobserved host departures remain counted, not expanded into IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Batch { pub transfer: Transfer, pub count: u64 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Details {
    pub identities: BTreeSet<Identity>,
    pub modifiers: Vec<CapacityModifier>,
    pub transit: Vec<Batch>,
    pub named: BTreeMap<Id, Entity>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Model {
    pub tick: Tick,
    pub sites: BTreeMap<Id, Site>,
    pub bins: BTreeMap<Entity, u64>,
    pub lineages: BTreeMap<Key, Lineage>,
    pub details: Details,
}

impl<'w> Crowd<'w> {
    /// Explicit founding identities for later host-addressed operations.
    /// Unaddressed surface hosts retain their complete exact-state bins.
    pub fn for_spreads(world: &'w ProbeWorld, dynamics: u64, named: &BTreeSet<Id>) -> Result<Self> {
        world.genesis.validate()?;
        world.spread_supported()?;
        let mut crowd = Self::assemble(world, dynamics, Variant::Histogram)?;
        crowd.spreads = Some(Details::found(world, &mut crowd.bins, named)?);
        crowd.spread_snapshot()?.validate(&world.genesis.rules)?;
        crowd.matter = crowd.total_matter();
        crowd.index_journeys();
        Ok(crowd)
    }

    pub(crate) fn spread_snapshot(&self) -> Result<Model> {
        Ok(Model { tick:self.tick, sites:self.sites.clone(), bins:self.bins.clone(),
            lineages:self.lineages.clone(), details:self.spreads.clone()
                .ok_or("crowd has no independent spread timeline")? })
    }

    fn publish_spread(&mut self, model: Model) {
        self.tick = model.tick;
        self.sites = model.sites;
        self.bins = model.bins;
        self.lineages = model.lineages;
        self.spreads = Some(model.details);
        self.index_journeys();
    }

    /// Apply exactly the same authored stock input as the native reference.
    /// Refusal publishes neither partial stock nor any candidate queue.
    pub fn spread(&mut self, command: &spread::Command) -> Result<String> {
        let before = self.spread_snapshot()?;
        let mut candidate = before.clone();
        let rules = &self.world.genesis.rules;
        let result = candidate.apply(command, rules, self.dynamics)?;
        candidate.validate(rules)?;
        candidate.changed(&before, rules)?;
        candidate.validate(rules)?;
        if candidate.matter(rules) != self.matter {
            return Err("independent spread command changed total matter".into());
        }
        self.publish_spread(candidate);
        Ok(result)
    }

    pub fn advance_spreads(&mut self, ticks: Tick) -> Result<()> {
        let mut candidate = self.spread_snapshot()?;
        let rules = &self.world.genesis.rules;
        candidate.advance(ticks, rules)?;
        if candidate.matter(rules) != self.matter {
            return Err("independent spread clock changed total matter".into());
        }
        self.publish_spread(candidate);
        Ok(())
    }

    pub fn spread_held(&self, at: Substrate, identity: &Identity) -> Result<u64> {
        self.spread_snapshot()?.held(at, identity)
    }

    pub fn spread_capacity(&self, at: Substrate, identity: &Identity) -> Result<u64> {
        self.spread_snapshot()?.capacity(&self.world.genesis.rules, Some(at), identity)
    }

    pub(crate) fn spread_rules(&self) -> &Rules { &self.world.genesis.rules }

    pub fn spread_names(&self) -> Result<BTreeSet<Id>> {
        Ok(self.spread_snapshot()?.details.named.keys().copied().collect())
    }
}

impl Model {
    fn apply(&mut self, command: &spread::Command, rules: &Rules, seed: u64) -> Result<String> {
        Ok(match command {
            spread::Command::Register(identity) => {
                self.details.identities.insert(identity.clone());
                "registered".into()
            },
            spread::Command::ForkStrain { parent, at, amount, traits } =>
                self.fork(rules, seed, parent, *at, *amount, traits.clone())?,
            spread::Command::SetModifier(modifier) => {
                self.details.modifiers.retain(|m| m.key != modifier.key);
                self.details.modifiers.push(modifier.clone());
                self.details.modifiers.sort_by(|a,b| a.key.cmp(&b.key));
                "modified".into()
            },
            spread::Command::RemoveModifier { key } => {
                crate::validation::key(key)?;
                self.details.modifiers.retain(|m| &m.key != key);
                "modifier removed".into()
            },
            spread::Command::Overflow { identity, at } =>
                format!("dispatched:{}", self.overflow(rules, identity, *at)?),
            spread::Command::Place(placement) =>
                format!("placed:{}", self.place(rules, placement)?),
        })
    }

    pub(crate) fn matter(&self, rules:&Rules) -> u128 {
        let keys:BTreeSet<_> = self.details.identities.iter().map(Identity::key).collect();
        let bodies = self.bins.iter().map(|(e,n)| (e,*n))
            .chain(self.details.named.values().map(|e| (e,1)));
        let held = |ledger:&Ledger| -> u128 {
            ledger.iter().filter(|(key,_)| matches!(rules.accounts.get(*key),
                Some(crate::rules::AccountKind::Matter { .. }))
                || !rules.accounts.contains_key(*key) && keys.contains(*key))
                .map(|(_,amount)| u128::from(*amount)).sum()
        };
        bodies.map(|(e,n)| held(&crate::anatomy::books(e)) * u128::from(n)).sum::<u128>()
            + self.sites.values().map(|site| held(&site.accounts)).sum::<u128>()
            + self.details.transit.iter().map(|batch|
                u128::from(batch.transfer.amount) * u128::from(batch.count)).sum::<u128>()
    }
}
