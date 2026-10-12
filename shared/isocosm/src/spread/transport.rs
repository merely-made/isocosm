// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, schema::Tick, simulation::State};
use std::collections::BTreeSet;

/// Explicitly offered overflow on one of the source site's authored routes.
/// Automatic multi-route allocation remains separate from this admission.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dispatch { pub route: usize, pub offered: u64 }

/// Native site-grain transport (502). Capacity and offers are explicit,
/// not inferred from a soil stock or an unruled mutation policy.
pub fn overflow(state: &mut State, identity: &Identity, from: u64, capacity_mg: u64,
    dispatch: &[Dispatch]) -> Result<u64> {
    let current = held(state, Substrate::Site(from), identity)?;
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
    crate::meaning::debit(stock::ledger_mut(&mut candidate, Substrate::Site(from))?, &identity.key(), sent)?;
    candidate.spreads.transit.extend(transfers);
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
