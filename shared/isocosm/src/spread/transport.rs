// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, schema::Tick, simulation::State};
use std::collections::BTreeSet;

/// Explicitly offered overflow on one of the source site's authored routes.
/// Automatic multi-route allocation remains separate from this admission.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dispatch { pub route: usize, pub offered: u64 }

fn route_origin(state: &State, at: Substrate) -> Result<Option<u64>> {
    Ok(match at {
        Substrate::Site(site) => Some(site),
        Substrate::Place(place) => Some(place.site),
        Substrate::Host(host) => state.population.get(host)
            .ok_or("unknown overflow host")?.resident_site(),
    })
}

/// The independent explicit-offer kernel; native commands resolve their
/// capacity and route shares from world rules instead of trusting these values.
pub fn overflow(state: &mut State, identity: &Identity, from: u64, capacity_mg: u64,
    dispatch: &[Dispatch]) -> Result<u64> {
    overflow_at(state, identity, Substrate::Site(from), capacity_mg, dispatch)
}

pub fn overflow_at(state: &mut State, identity: &Identity, at: Substrate, capacity_mg: u64,
    dispatch: &[Dispatch]) -> Result<u64> {
    let current = held(state, at, identity)?;
    let from = route_origin(state, at)?.ok_or("substrate has no resident route origin")?;
    let excess = current.saturating_sub(capacity_mg);
    let site = state.sites.get(&from).ok_or("unknown overflow source")?;
    let mut seen = BTreeSet::new();
    let mut offered = 0u64;
    let mut sent = 0u64;
    let mut transfers = Vec::new();
    for choice in dispatch {
        if !seen.insert(choice.route) { return Err("overflow route selected twice".into()); }
        let route = site.routes.get(choice.route).ok_or("unknown overflow route")?;
        if route.travel == 0 || route.transmission > 1_000_000 || route.to == from
            || !state.sites.contains_key(&route.to) { return Err("invalid overflow route".into()); }
        offered = offered.checked_add(choice.offered).ok_or("overflow offer sum overflows")?;
        if offered > excess { return Err("overflow offers exceed the source's excess".into()); }
        let amount = (u128::from(choice.offered) * u128::from(route.transmission) / 1_000_000) as u64;
        let arrives = state.tick.checked_add(route.travel).ok_or("spread arrival time overflows")?;
        sent = sent.checked_add(amount).ok_or("transmitted overflow sum overflows")?;
        if amount != 0 {
            transfers.push(Transfer { identity: identity.clone(), from, to: route.to,
                departed: state.tick, arrives, amount });
        }
    }
    if sent == 0 { return Ok(0); }
    let mut candidate = state.clone();
    crate::meaning::debit(stock::ledger_mut(&mut candidate, at)?, &identity.key(), sent)?;
    candidate.spreads.transit.extend(transfers);
    *state = candidate;
    Ok(sent)
}

/// Native capacity feeds equal route offers; only transmitted whole
/// milligrams leave. Division and transmission remainders remain at source.
pub fn resolved_overflow(state: &mut State, rules: &crate::rules::Rules,
    identity: &Identity, from: u64) -> Result<u64> {
    resolved_overflow_at(state, rules, identity, Substrate::Site(from))
}

pub fn resolved_overflow_at(state: &mut State, rules: &crate::rules::Rules,
    identity: &Identity, at: Substrate) -> Result<u64> {
    let limit = capacity(state, rules, at, identity)?;
    let excess = held(state, at, identity)?.saturating_sub(limit);
    if excess == 0 { return Ok(0); }
    // A travelling host is not an origin resident. It keeps all excess until
    // a real native arrival makes its outgoing site routes available.
    let Some(from) = route_origin(state, at)? else { return Ok(0); };
    let site = state.sites.get(&from).ok_or("unknown overflow source")?;
    if site.routes.iter().any(|route| route.transmission == 0) {
        return Err("zero-transmission overflow eligibility awaits its design ruling".into());
    }
    let eligible: Vec<usize> = (0..site.routes.len()).collect();
    if eligible.is_empty() { return Ok(0); }
    let count = u64::try_from(eligible.len()).map_err(|_| "overflow route count exceeds native integers")?;
    let offered = excess / count;
    if offered == 0 { return Ok(0); }
    let dispatch: Vec<_> = eligible.into_iter().map(|route| Dispatch { route, offered }).collect();
    overflow_at(state, identity, at, limit, &dispatch)
}

fn stocks(state: &State) -> Vec<(Substrate, Identity)> {
    let identities: std::collections::BTreeMap<_, _> = state.spreads.identities.iter()
        .map(|identity| (identity.key(), identity.clone())).collect();
    let mut sources = BTreeSet::new();
    let mut add = |at, ledger: &crate::schema::Ledger| {
        for (key, amount) in ledger {
            if *amount != 0 && let Some(identity) = identities.get(key) {
                sources.insert((at, identity.clone()));
            }
        }
    };
    for (site, stock) in &state.sites { add(Substrate::Site(*site), &stock.accounts); }
    for place in &state.spreads.places { add(Substrate::Place(place.place), &place.accounts); }
    for (first, group) in &state.population.groups {
        if !group.entity.accounts.keys().any(|key| identities.contains_key(key)) { continue; }
        for host in *first..*first + group.count { add(Substrate::Host(host), &group.entity.accounts); }
    }
    sources.into_iter().collect()
}

/// Called on a candidate after native effects, before conservation and
/// publication. Growth/arrival or a lower resolved capacity triggers spill;
/// unrelated commands and advance partitioning do not repeatedly send stocks.
pub fn dispatch_changed(before: &State, state: &mut State, rules: &crate::rules::Rules) -> Result<u64> {
    if rules.spread_capacity.is_none() { return Ok(0); }
    let mut candidate = state.clone();
    let mut sent = 0u64;
    for (at, identity) in stocks(&candidate) {
        let current = held(&candidate, at, &identity)?;
        if current == 0 { continue; }
        let limit = capacity(&candidate, rules, at, &identity)?;
        if current <= limit { continue; }
        let previous = stock::ledger(before, at).ok().and_then(|ledger| ledger.get(&identity.key()))
            .copied().unwrap_or(0);
        let lower = stock::ledger(before, at).is_ok() && before.spreads.identities.contains(&identity)
            && capacity(before, rules, at, &identity)? > limit;
        let arrived = route_origin(before, at).ok().flatten().is_none()
            && route_origin(&candidate, at)?.is_some();
        if current > previous || lower || arrived {
            sent = sent.checked_add(resolved_overflow_at(&mut candidate, rules, &identity, at)?)
                .ok_or("dispatched spread work overflows")?;
        }
    }
    *state = candidate;
    Ok(sent)
}

/// Every due destination credit succeeds before any arrival is published.
/// Failed credits retain the entire original queue and every source ledger.
pub fn settle(state: &mut State, until: Tick) -> Result<u64> {
    let mut candidate = state.clone();
    let mut pending = Vec::new();
    let mut arrivals = 0u64;
    for transfer in &state.spreads.transit {
        if transfer.arrives > until { pending.push(transfer.clone()); continue; }
        if !state.spreads.identities.contains(&transfer.identity) { return Err("unregistered transit identity".into()); }
        crate::meaning::credit(stock::ledger_mut(&mut candidate, Substrate::Site(transfer.to))?,
            &transfer.identity.key(), transfer.amount)?;
        arrivals = arrivals.checked_add(1).ok_or("spread arrival work overflows")?;
    }
    if arrivals != 0 { candidate.spreads.transit = pending; *state = candidate; }
    Ok(arrivals)
}
