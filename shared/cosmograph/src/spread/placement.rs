// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Local placement of declared same-lineage matter, or unchanged spread stock.

use super::*;
use crate::{Result, rules::{AccountKind, Rules}, schema::{Id, Key}, simulation::State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Supply {
    Matter { from: Substrate, account: Key },
    Spread { from: Substrate, identity: Identity },
}

impl Supply {
    pub fn from(&self) -> Substrate {
        match self { Self::Matter { from, .. } | Self::Spread { from, .. } => *from }
    }
    pub fn account(&self) -> Key {
        match self { Self::Matter { account, .. } => account.clone(), Self::Spread { identity, .. } => identity.key() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement { pub source: Supply, pub identity: Identity, pub at: Substrate, pub amount: u64 }

fn resident_site(state: &State, at: Substrate) -> Result<Id> {
    stock::ledger(state, at)?;
    let site = match at {
        Substrate::Site(id) => id,
        Substrate::Place(place) => place.site,
        Substrate::Host(host) => state.population.get(host).and_then(|entity| entity.resident_site())
            .ok_or("local spread placement has no resident host")?,
    };
    if !state.sites.contains_key(&site) { return Err("local spread placement has no native site".into()); }
    Ok(site)
}

pub fn place_stock(state: &mut State, rules: &Rules, placement: &Placement) -> Result<u64> {
    let from = placement.source.from();
    if placement.amount == 0 || resident_site(state, from)? != resident_site(state, placement.at)? {
        return Err("spread placement needs positive stock within one actual resident site".into());
    }
    if rules.accounts.contains_key(&placement.identity.key()) {
        return Err("spread placement cannot alias an ordinary rule account".into());
    }
    held(state, placement.at, &placement.identity)?;
    let lineage = provenance(state, &placement.identity.key()).ok_or("spread placement has no actual native lineage")?;
    match &placement.source {
        Supply::Matter { from, account } => {
            if let Substrate::Host(host) = from
                && state.population.get(*host).is_some_and(|body| body.parts.values()
                    .any(|part| part.matter.get(account).is_some_and(|amount| *amount > 0))) {
                return Err("spread placement of anatomical source matter needs an admitted allocation path".into());
            }
            match rules.accounts.get(account) {
            Some(AccountKind::Matter { lineage: source, .. }) if source == &lineage => {},
            _ => return Err("spread placement source must declare exactly the target's native matter lineage".into()),
            }
        },
        Supply::Spread { identity, .. } => {
            if identity != &placement.identity { return Err("spread placement cannot change a transported native identity".into()); }
            held(state, from, identity)?;
        },
    }
    let account = placement.source.account();
    let balance = stock::ledger(state, from)?.get(&account).copied().unwrap_or(0);
    if balance < placement.amount { return Err("spread placement lacks its exact actual source quantity".into()); }
    owned::placement_operation(state, rules, placement, |candidate| {
        owned::placement_begin(candidate, &placement.source, placement.at, &placement.identity)?;
        crate::projects::debit::guard_debit_at(candidate, crate::projects::debit::substrate_source(from),
            &account, placement.amount, balance)?;
        let mut next = candidate.clone();
        crate::meaning::debit(stock::ledger_mut(&mut next, from)?, &account, placement.amount)?;
        crate::meaning::credit(stock::ledger_mut(&mut next, placement.at)?, &placement.identity.key(), placement.amount)?;
        owned::placement_output(&next, placement.at, &placement.identity, placement.amount)?;
        *candidate = next;
        Ok(placement.amount)
    })
}
