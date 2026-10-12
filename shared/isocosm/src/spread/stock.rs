// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::Rules, schema::{Key, Ledger}, simulation::State};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn ledger(state: &State, at: Substrate) -> Result<&Ledger> {
    match at {
        Substrate::Site(id) => state.sites.get(&id).map(|s| &s.accounts).ok_or("unknown substrate site".into()),
        Substrate::Host(id) => state.population.get(id).map(|e| &e.accounts).ok_or("unknown substrate host".into()),
        Substrate::Place(id) => state.spreads.places.iter().find(|p| p.place == id)
            .map(|p| &p.accounts).ok_or("unknown finer substrate PlaceId".into()),
    }
}

pub(crate) fn ledger_mut(state: &mut State, at: Substrate) -> Result<&mut Ledger> {
    match at {
        Substrate::Site(id) => state.sites.get_mut(&id).map(|s| &mut s.accounts).ok_or("unknown substrate site".into()),
        Substrate::Host(id) => Ok(&mut state.population.lift(id)?.accounts),
        Substrate::Place(id) => state.spreads.places.iter_mut().find(|p| p.place == id)
            .map(|p| &mut p.accounts).ok_or("unknown finer substrate PlaceId".into()),
    }
}

pub fn held(state: &State, at: Substrate, identity: &Identity) -> Result<u64> {
    if !state.spreads.identities.contains(identity) { return Err("unregistered spread identity".into()); }
    if matches!((at, identity), (Substrate::Host(_), Identity::Territory(_))) {
        return Err("territory stocks belong to places, not host accounts".into());
    }
    Ok(ledger(state, at)?.get(&identity.key()).copied().unwrap_or(0))
}

/// Provenance is native identity, never an account-prefix guess (TG5 seam).
pub fn provenance(state: &State, key: &str) -> Option<Key> {
    let identity = state.spreads.identities.iter().find(|identity| identity.key() == key)?;
    match identity {
        Identity::Territory(id) => state.population.get(*id).map(|e| e.lineage.clone()),
        Identity::Surface(lineage) => state.lineages.contains_key(lineage).then(|| lineage.clone()),
    }
}

/// Ruling 500: catalogue functions named by the lineage's traits express
/// their local mass, without invented part cells or geometry measurements.
pub fn functions(state: &State, rules: &Rules, at: Substrate, identity: &Identity)
    -> Result<BTreeMap<Key, u64>> {
    let mass = held(state, at, identity)?;
    let lineage = provenance(state, &identity.key()).ok_or("spread has no native provenance")?;
    let traits = &state.lineages.get(&lineage).ok_or("unknown spread lineage")?.traits;
    Ok(traits.iter().filter(|key| rules.functions.contains_key(*key))
        .map(|key| (key.clone(), mass)).collect())
}

/// Registered spread stocks use reserved keys, so none are counted twice
/// alongside the ordinary rule accounts. Unregistered keys count as before.
pub fn ledger_mass(ledger: &Ledger, rules: &Rules, spreads: &Spreads) -> u128 {
    crate::meaning::mass(ledger, rules) + ledger.iter()
        .filter(|(key, _)| !rules.accounts.contains_key(*key) && spreads.owns(key))
        .map(|(_, mass)| u128::from(*mass)).sum::<u128>()
}

pub(crate) fn validate(spreads: &Spreads, rules: &Rules,
    population: &crate::population::Population,
    sites: &BTreeMap<crate::schema::Id, crate::schema::Site>,
    lineages: &BTreeMap<Key, crate::schema::Lineage>) -> Result<()> {
    let mut keys = BTreeSet::new();
    for identity in &spreads.identities {
        let key = identity.key();
        crate::validation::key(&key)?;
        if !keys.insert(key.clone()) || rules.accounts.contains_key(&key) {
            return Err("spread key aliases another stock or ordinary account".into());
        }
        match identity {
            Identity::Territory(id) => {
                let owner = population.get(*id).ok_or("territory has no native entity")?;
                if !lineages.contains_key(&owner.lineage) { return Err("territory has no native lineage".into()); }
                if population.groups.values().any(|g| g.entity.accounts.get(&key).is_some_and(|n| *n != 0)
                    || g.entity.parts.values().any(|p| p.matter.get(&key).is_some_and(|n| *n != 0))) {
                    return Err("territory matter must not be duplicated in entity or part accounts".into());
                }
            },
            Identity::Surface(lineage) => {
                if !lineages.contains_key(lineage) { return Err("surface has no native lineage".into()); }
            },
        }
    }
    if population.groups.values().any(|g| g.entity.parts.values()
        .any(|p| p.matter.keys().any(|key| keys.contains(key)))) {
        return Err("spread stocks belong to places or host ledgers, not part tissue".into());
    }
    let mut places = BTreeSet::new();
    for place in &spreads.places {
        if !sites.contains_key(&place.place.site) || !places.insert(place.place) {
            return Err("unknown or duplicated finer substrate PlaceId".into());
        }
        if place.accounts.keys().any(|key| !rules.accounts.contains_key(key) && !keys.contains(key)) {
            return Err("unknown finer-place stock".into());
        }
    }
    if spreads.places.windows(2).any(|pair| pair[0].place >= pair[1].place) {
        return Err("finer-place ledgers must be in canonical PlaceId order".into());
    }
    for transfer in &spreads.transit {
        if !spreads.identities.contains(&transfer.identity) || !sites.contains_key(&transfer.from)
            || !sites.contains_key(&transfer.to) || transfer.from == transfer.to
            || transfer.amount == 0 || transfer.arrives <= transfer.departed {
            return Err("invalid conserved spread transit".into());
        }
        if !sites[&transfer.from].routes.iter().any(|route| route.to == transfer.to
            && route.travel == transfer.arrives - transfer.departed) {
            return Err("spread transit does not name an authored route duration".into());
        }
    }
    Ok(())
}
