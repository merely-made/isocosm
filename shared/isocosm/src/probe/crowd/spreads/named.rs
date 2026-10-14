// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Founding identities that the authored timeline actually addresses. These
//! are not the crowd's locally minted kin identities and are never guessed
//! from an equal body after a stochastic pass.

use super::*;
use crate::spread::CapacitySource;

impl Details {
    pub(super) fn found(world:&ProbeWorld, bins:&mut BTreeMap<Entity,u64>,
        selected:&BTreeSet<Id>) -> Result<Self> {
        let genesis = &world.genesis;
        let mut names = selected.clone();
        for identity in &genesis.spreads.identities {
            if let Identity::Territory(id) = identity { names.insert(*id); }
        }
        for modifier in &genesis.spreads.modifiers {
            if let CapacitySource::Substrate(Substrate::Host(id)) = modifier.source { names.insert(id); }
        }
        let mut named = BTreeMap::new();
        for id in names {
            let body = normalize(genesis.population.get(id).ok_or("named spread host is not a founding entity")?.clone());
            let n = bins.get_mut(&body).ok_or("named spread host has no actual founding bin")?;
            *n = n.checked_sub(1).ok_or("named spread host was counted twice")?;
            if *n == 0 { bins.remove(&body); }
            named.insert(id, body);
        }
        Ok(Self { identities:genesis.spreads.identities.clone(),
            modifiers:genesis.spreads.modifiers.clone(),
            transit:genesis.spreads.transit.iter().map(|transfer|
                Batch { transfer:transfer.clone(), count:1 }).collect(), named })
    }
}

impl Model {
    pub(super) fn land_bodies(&mut self, tick:Tick) -> Result<()> {
        fn land(mut body:Entity, tick:Tick, sites:&BTreeMap<Id,Site>) -> Result<Entity> {
            if let Some(journey) = body.journey && journey.arrives <= tick {
                if !sites.contains_key(&journey.to) { return Err("independent host arrival has no site".into()); }
                body.place = journey.to;
                body.arrived = journey.arrives;
                body.journey = None;
                body.patch = None;
            }
            Ok(normalize(body))
        }
        let mut bins = BTreeMap::new();
        for (body,n) in &self.bins {
            let body = land(body.clone(),tick,&self.sites)?;
            let count = bins.entry(body).or_insert(0u64);
            *count = count.checked_add(*n).ok_or("independent host bin count overflows")?;
        }
        self.bins = bins;
        for body in self.details.named.values_mut() {
            *body = land(body.clone(),tick,&self.sites)?;
        }
        Ok(())
    }
}
