// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::Rules, simulation::State,
    projects::{DebitOrigin, SourceRule, SpreadDebit, SpreadOutput, StockSource}};
use crate::projects::debit::{Context, Mode, state_balances, substrate_source};

pub(super) fn command_scope(state:&State, rules:&Rules) -> bool {
    state.projects.ownership.values().any(|held| held.current_source().is_some()
        && state.spreads.owns(&held.lot.account))
        || rules.owned_stock.iter().flat_map(|rules| &rules.selections)
            .any(|rule| matches!(rule,SourceRule::Spread { .. } | SourceRule::ProducedSpread { .. } | SourceRule::Placement { .. }))
}

pub(super) fn relevant(state:&State, rules:&Rules, operation:SpreadDebit, identity:&Identity, at:Substrate) -> bool {
    let explicit = state.projects.debit_capture.with(|context|
        Ok(matches!(context.mode,Mode::Preview | Mode::Admit(_)))).ok().flatten().unwrap_or(false);
    explicit || state.projects.ownership.values().any(|held|
        held.current_source() == Some(substrate_source(at)) && held.lot.account == identity.key())
        || rules.owned_stock.iter().flat_map(|rules| &rules.selections).any(|rule| match rule {
            SourceRule::Spread { operation:named,identity:selected,at:source, .. } =>
                *named == operation && selected == identity && *source == at,
            SourceRule::ProducedSpread { origin,operation:named,at:source, .. } => {
                let parent = match state.projects.debit_capture.with(|context|
                    crate::projects::ProducedOrigin::of(&context.origin)) {
                    Ok(Some(parent)) => Some(parent),
                    // The uncaptured native clock kernel creates this exact origin below.
                    Ok(None) => Some(crate::projects::ProducedOrigin::Clock { tick:state.tick }),
                    Err(_) => None,
                };
                *named == operation && *source == at && parent.as_ref() == Some(origin)
            }, _ => false,
        })
}

/// Native stock and title writes share this candidate, including generated outputs.
pub(super) fn operation<T>(state:&mut State, rules:&Rules, operation:SpreadDebit,
    identity:&Identity, at:Substrate, run:impl FnOnce(&mut State)->Result<T>) -> Result<T> {
    operation_inner(state,rules,operation,identity,at,false,run)
}

fn operation_inner<T>(state:&mut State, rules:&Rules, operation:SpreadDebit,
    identity:&Identity, at:Substrate, force:bool, run:impl FnOnce(&mut State)->Result<T>) -> Result<T> {
    if !force && !relevant(state,rules,operation,identity,at) { return run(state); }
    let mut candidate = state.clone();
    let existing = candidate.projects.debit_capture.take();
    let had_parent = existing.is_some();
    let mut context = existing.unwrap_or_else(|| Context::new(state,rules,
        crate::digest(&(rules,&*state)),DebitOrigin::Clock { tick:state.tick },
        format!("native-stock:{}",crate::digest(&(&*state,operation,identity,at))),Mode::Authored));
    // Owned direct commands begin a blank outer boundary; the real kernel owns it.
    if context.active {
        if context.requests.is_empty() && context.paid.is_empty() && context.pieces.is_empty() {
            context.active = false;
            context.next_operation = context.operation;
        } else { return Err("native spread cannot nest inside an unfinished stock operation".into()); }
    }
    let parent = context.origin.clone();
    context.origin = DebitOrigin::NativeSpread { parent:Box::new(parent.clone()),operation,
        identity:identity.clone(),at };
    if operation == SpreadDebit::Place { context.reference_scope = true; }
    context.matter.extend(candidate.spreads.identities.iter().map(Identity::key));
    context.identities.extend(candidate.spreads.identities.iter().map(|identity| (identity.key(),identity.clone())));
    context.shadow.extend(candidate.projects.ownership.clone());
    let mut balances = state_balances(&candidate);
    if let Substrate::Host(host) = at {
        let body = candidate.population.get(host).ok_or("native spread host is absent")?;
        for (key,amount) in crate::anatomy::books(body) { balances.insert((StockSource::Entity(host),key),amount); }
    }
    context.begin(balances,Vec::new(),None)?;
    candidate.projects.debit_capture.set(Some(context));
    let result = run(&mut candidate);
    let mut context = candidate.projects.debit_capture.take().ok_or("native spread lost its stock evidence")?;
    let outcome = match result {
        Ok(outcome) => match context.finish(state_balances(&candidate)) {
            Ok(()) => Ok(outcome), Err(why) => { context.failed = true; Err(why) },
        },
        Err(why) => { context.failed = true; Err(why) },
    };
    context.origin = parent;
    if outcome.is_err() {
        if had_parent { state.projects.debit_capture.set(Some(context)); }
        return outcome;
    }
    candidate.projects.ownership.extend(context.changes());
    candidate.projects.produced.extend(context.produced_changes());
    if had_parent { candidate.projects.debit_capture.set(Some(context)); }
    *state = candidate;
    outcome
}

pub(super) fn placement_relevant(state:&State, rules:&Rules, placement:&super::Placement) -> bool {
    let source = substrate_source(placement.source.from());
    let account = placement.source.account();
    state.projects.ownership.values().any(|held| held.current_source() == Some(source) && held.lot.account == account)
        || rules.owned_stock.iter().flat_map(|rules| &rules.selections).any(|rule|
            matches!(rule,SourceRule::Placement { identity,at,source:from,account:key, .. }
                if identity == &placement.identity && *at == placement.at && *from == source && key == &account))
}

pub(super) fn placement_operation<T>(state:&mut State, rules:&Rules, placement:&super::Placement,
    run:impl FnOnce(&mut State)->Result<T>) -> Result<T> {
    let force = placement_relevant(state,rules,placement);
    operation_inner(state,rules,SpreadDebit::Place,&placement.identity,placement.source.from(),force,run)
}

pub(super) fn placement_begin(state:&State, source:&super::Supply, at:Substrate, identity:&Identity) -> Result<()> {
    state.projects.debit_capture.with(|context| {
        if !matches!(context.origin,DebitOrigin::NativeSpread { operation:SpreadDebit::Place, .. }) {
            return Err("placement input lacks its actual native operation".into());
        }
        context.preserve = match source {
            super::Supply::Spread { identity:from, .. } if from == identity => Some((substrate_source(at),identity.key())),
            super::Supply::Spread { .. } => return Err("placement cannot substitute another native identity".into()),
            super::Supply::Matter { .. } => None,
        };
        Ok(())
    })?;
    Ok(())
}

pub(super) fn placement_output(state:&State, at:Substrate, identity:&Identity, amount:u64) -> Result<()> {
    state.projects.debit_capture.with(|context| {
        if !matches!(context.origin,DebitOrigin::NativeSpread { operation:SpreadDebit::Place, .. }) {
            return Err("placement output lacks its actual native operation".into());
        }
        context.emit_spread(SpreadOutput::Placement,substrate_source(at),identity.clone(),amount)
    })?;
    Ok(())
}

pub(super) fn mark_route(state:&mut State, at:Substrate, route:usize, index:usize,
    identity:&Identity, amount:u64) -> Result<()> {
    let owned = state.projects.debit_capture.with(|context|
        Ok(matches!(context.origin,DebitOrigin::NativeSpread { operation:SpreadDebit::Overflow, .. })))?.unwrap_or(false);
    if !owned { return Ok(()); }
    let id = TransitId(state.spreads.next_transit);
    state.spreads.next_transit = state.spreads.next_transit.checked_add(1).ok_or("native transit IDs exhausted")?;
    state.spreads.transit_marks.push(TransitMark { id,index,source:at,route });
    state.projects.debit_capture.with(|context|
        context.emit_spread(SpreadOutput::Route(route),StockSource::Transit(id),identity.clone(),amount))?;
    Ok(())
}

pub(super) fn strain_output(state:&State, at:Substrate, identity:Identity, amount:u64) -> Result<()> {
    let active = state.projects.debit_capture.with(|context|
        Ok(matches!(context.origin,DebitOrigin::NativeSpread { operation:SpreadDebit::ForkStrain, .. })))?.unwrap_or(false);
    if active {
        state.projects.debit_capture.with(|context|
            context.emit_spread(SpreadOutput::GeneratedStrain,substrate_source(at),identity,amount))?;
    }
    Ok(())
}

/// Delivery changes custody of each exact title, never its owners or quantity.
pub(super) fn deliver_titles(state:&mut State, mark:&TransitMark, transfer:&Transfer) -> Result<()> {
    use crate::projects::LotAt;
    let source = StockSource::Transit(mark.id);
    let mut total = 0u64;
    let mut delivered = Vec::new();
    for (key,held) in &mut state.projects.ownership {
        if held.current_source() != Some(source) { continue; }
        if held.lot.account != transfer.identity.key() || held.lot.owners.is_empty() {
            return Err("transit title names a different native identity or no owner".into());
        }
        total = total.checked_add(held.lot.amount).ok_or("transit title quantity overflow")?;
        if total > transfer.amount { return Err("transit claims exceed the actual delivered quantity".into()); }
        held.at = LotAt::Held(StockSource::Site(transfer.to));
        delivered.push((key.clone(),held.clone()));
    }
    state.projects.debit_capture.with(|context| {
        if context.active { return Err("native delivery cannot nest inside an unfinished stock operation".into()); }
        context.shadow.extend(delivered);
        Ok(())
    })?;
    Ok(())
}
