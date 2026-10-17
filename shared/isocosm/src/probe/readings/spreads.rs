// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! C11 comparison columns. The existing C4–C10 derived columns are unchanged.
//! These read both actual ledgers and the conserved stock still travelling.

use super::*;
use crate::{rules::Rules, simulation::State, spread::{self, CapacityModifier, Identity, Substrate, Transfer}};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum StockAt { Site(Id), NamedHost(Id), HostBin(usize) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Stock {
    pub at:StockAt,
    pub identity:Identity,
    pub per_member:u64,
    pub count:u64,
    pub capacity:Option<u64>,
    pub functions:BTreeMap<Key,u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Travelling {
    pub identity:Identity,
    pub from:Id,
    pub to:Id,
    pub departed:Tick,
    pub arrives:Tick,
    pub amount:u128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Reading {
    pub tick:Tick,
    pub sites:BTreeMap<Id,Ledger>,
    pub hosts:Vec<(Entity,u64)>,
    pub named:BTreeMap<Id,Entity>,
    pub lineages:BTreeMap<Key,Lineage>,
    pub identities:BTreeSet<Identity>,
    pub modifiers:Vec<CapacityModifier>,
    pub stocks:Vec<Stock>,
    pub travelling:Vec<Travelling>,
    pub total_matter:u128,
}

fn travelling<'a>(queue:impl Iterator<Item=(&'a Transfer,u64)>) -> Vec<Travelling> {
    let mut totals:BTreeMap<(Identity,Id,Id,Tick,Tick),u128> = BTreeMap::new();
    for (transfer,count) in queue {
        let key = (transfer.identity.clone(),transfer.from,transfer.to,transfer.departed,transfer.arrives);
        *totals.entry(key).or_default() += u128::from(transfer.amount) * u128::from(count);
    }
    totals.into_iter().map(|((identity,from,to,departed,arrives),amount)|
        Travelling {identity,from,to,departed,arrives,amount}).collect()
}

/// Project the actual native state for comparison only; never reconstruct
/// an individual runtime or turn a statistical bin into a guessed ID.
pub fn native(state:&State, rules:&Rules, selected:&BTreeSet<Id>) -> Result<Reading> {
    if !state.spreads.places.is_empty() || !state.spreads.transit_marks.is_empty()
        || state.spreads.next_transit != 0 {
        return Err("C11 crowd reading is site grain without titled transit".into());
    }
    let mut names = selected.clone();
    for identity in &state.spreads.identities {
        if let Identity::Territory(id) = identity { names.insert(*id); }
    }
    let mut named = BTreeMap::new();
    for id in &names {
        named.insert(*id,aggregate::normalize(state.population.get(*id)
            .ok_or("comparison names no native host")?.clone()));
    }
    let mut hosts:BTreeMap<Entity,u64> = BTreeMap::new();
    for (&first,group) in &state.population.groups {
        let excluded = names.range(first..first+group.count).count() as u64;
        let n = group.count.checked_sub(excluded).ok_or("comparison counted a host twice")?;
        if n != 0 { *hosts.entry(aggregate::normalize(group.entity.as_ref().clone())).or_default() += n; }
    }
    let hosts:Vec<_> = hosts.into_iter().collect();
    let mut stocks = Vec::new();
    let capacity = |at,identity| rules.spread_capacity.as_ref().map(|_| spread::capacity(state,rules,at,identity)).transpose();
    for (&site,ledger) in &state.sites {
        for identity in &state.spreads.identities {
            let mass = ledger.accounts.get(&identity.key()).copied().unwrap_or(0);
            if mass == 0 { continue; }
            let at = Substrate::Site(site);
            stocks.push(Stock {at:StockAt::Site(site),identity:identity.clone(),per_member:mass,count:1,
                capacity:capacity(at,identity)?, functions:spread::functions(state,rules,at,identity)?});
        }
    }
    for (&host,body) in &named {
        for identity in &state.spreads.identities {
            let mass = body.accounts.get(&identity.key()).copied().unwrap_or(0);
            if mass == 0 { continue; }
            let at = Substrate::Host(host);
            stocks.push(Stock {at:StockAt::NamedHost(host),identity:identity.clone(),per_member:mass,count:1,
                capacity:capacity(at,identity)?,functions:spread::functions(state,rules,at,identity)?});
        }
    }
    for (index,(body,count)) in hosts.iter().enumerate() {
        for identity in &state.spreads.identities {
            let mass = body.accounts.get(&identity.key()).copied().unwrap_or(0);
            if mass == 0 { continue; }
            let Identity::Surface(lineage) = identity else { return Err("territory stock lies on a host".into()); };
            let traits = &state.lineages.get(lineage).ok_or("surface lineage absent")?.traits;
            stocks.push(Stock {at:StockAt::HostBin(index),identity:identity.clone(),per_member:mass,count:*count,
                capacity:rules.spread_capacity.as_ref().map(|p|
                    p.resolve(traits,None,identity,&state.spreads.modifiers)).transpose()?,
                functions:traits.iter().filter(|key| rules.functions.contains_key(*key))
                    .map(|key| (key.clone(),mass)).collect()});
        }
    }
    Ok(Reading {tick:state.tick,sites:state.sites.iter().map(|(id,s)| (*id,s.accounts.clone())).collect(),
        hosts,named,lineages:state.lineages.clone(),identities:state.spreads.identities.clone(),
        modifiers:state.spreads.modifiers.clone(),stocks,
        travelling:travelling(state.spreads.transit.iter().map(|t| (t,1))),
        total_matter:crate::simulation::matter(state,rules)})
}

pub fn crowd(crowd:&Crowd) -> Result<Reading> {
    let model = crowd.spread_snapshot()?;
    let rules = crowd.spread_rules();
    let mut stocks = Vec::new();
    let functions = |identity:&Identity,mass:u64| -> Result<BTreeMap<Key,u64>> {
        let traits = &model.lineages.get(&model.lineage(identity)?).ok_or("spread lineage absent")?.traits;
        Ok(traits.iter().filter(|key| rules.functions.contains_key(*key)).map(|key| (key.clone(),mass)).collect())
    };
    let mut add = |at:StockAt, substrate:Option<Substrate>, identity:&Identity,mass:u64,count:u64| -> Result<()> {
        if mass != 0 {
            stocks.push(Stock {at,identity:identity.clone(),per_member:mass,count,
                capacity:rules.spread_capacity.as_ref().map(|_| model.capacity(rules,substrate,identity)).transpose()?,
                functions:functions(identity,mass)?});
        }
        Ok(())
    };
    for (&id,site) in &model.sites {
        for identity in &model.details.identities {
            add(StockAt::Site(id),Some(Substrate::Site(id)),identity,
                site.accounts.get(&identity.key()).copied().unwrap_or(0),1)?;
        }
    }
    for (&id,body) in &model.details.named {
        for identity in &model.details.identities {
            add(StockAt::NamedHost(id),Some(Substrate::Host(id)),identity,
                body.accounts.get(&identity.key()).copied().unwrap_or(0),1)?;
        }
    }
    for (index,(body,n)) in model.bins.iter().enumerate() {
        for identity in &model.details.identities {
            add(StockAt::HostBin(index),None,identity,body.accounts.get(&identity.key()).copied().unwrap_or(0),*n)?;
        }
    }
    Ok(Reading {tick:model.tick,sites:model.sites.iter().map(|(id,s)| (*id,s.accounts.clone())).collect(),
        hosts:model.bins.iter().map(|(e,n)| (e.clone(),*n)).collect(),
        named:model.details.named.iter().map(|(id,e)| (*id,aggregate::normalize(e.clone()))).collect(),
        lineages:model.lineages.clone(),identities:model.details.identities.clone(),modifiers:model.details.modifiers.clone(),stocks,
        travelling:travelling(model.details.transit.iter().map(|b| (&b.transfer,b.count))),total_matter:model.matter(rules)})
}
