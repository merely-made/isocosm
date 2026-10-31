// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::{Amount, Binding, Effect, Process, Query}, simulation::Genesis};
use references::References;
use serde_json::json;

pub(super) fn leaf<'a>(genesis: &'a Genesis, address: &EffectAddress) -> Result<(&'a Process, &'a Effect)> {
    let process = genesis.rules.processes.get(&address.process).ok_or("mechanical consumer process is absent")?;
    if process.risk.is_some() || address.phase == EffectPhase::Risk {
        return Err("risk-dependent mechanical evidence is unsupported; it cannot be inferred from acceptance".into());
    }
    if address.path.len() != 1 {
        return Err("guarded/nested mechanical consumers require an executed-branch witness not yet supported".into());
    }
    let EffectStep::Item(index) = address.path[0] else { return Err("mechanical address must begin with a native effect ordinal".into()); };
    let effects = if address.phase == EffectPhase::Commitment { &process.commitments } else { &process.effects };
    Ok((process, effects.get(index as usize).ok_or("mechanical effect ordinal is absent")?))
}

pub(super) fn constant(amount: &Amount) -> Result<u64> {
    match amount {
        Amount::Fixed(value) => Ok(*value),
        Amount::Computed(expr) => {
            // Use the actual saturating evaluator, rather than equating different AST spellings.
            let value = expr.eval_in(&mut |_| Err("dynamic mechanical amount requires more evidence".into()),
                &mut |_, _| Err("random draws cannot establish independence".into()),
                &mut |_| Err("body-dependent mechanical amount requires more evidence".into()))?;
            Ok(value.max(0) as u64)
        },
    }
}

pub(super) fn projection(genesis: &Genesis, address: &EffectAddress, references: &References<'_>, candidate: bool)
    -> Result<MechanicalProjection> {
    let (process, effect) = leaf(genesis, address)?;
    references.token(NativeReference::Process(process.id.clone()), candidate)?;
    for query in &process.requires {
        match query {
            Query::Trait {key,..} => { references.token(NativeReference::Trait(key.clone()),candidate)?; },
            Query::Account {key,..} | Query::Below {key,..} => { references.stock(key,candidate)?; },
            Query::Condition {key,..} => { references.token(NativeReference::Condition(key.clone()),candidate)?; },
            Query::Related {kind} => { references.token(NativeReference::Relation(kind.clone()),candidate)?; },
            Query::Expresses {function} | Query::Routes {function,..} => {
                references.token(NativeReference::Function(function.clone()),candidate)?;
            },
            _ => {},
        }
    }
    if let Some(key) = &process.need_account { references.stock(key,candidate)?; }
    if let Some(target) = &process.target {
        if let Some(lineage) = &target.lineage { references.token(NativeReference::Lineage(lineage.clone()),candidate)?; }
        for lineage in &target.among { references.token(NativeReference::Lineage(lineage.clone()),candidate)?; }
    }
    let cadence = process.period.filter(|tick| *tick > 0).ok_or("mechanical consumer has no actual scheduled cadence")?;
    let mut projection = MechanicalProjection { cadence, priority: process.priority, family: OperationFamily::Transfer,
        inputs: vec![], outputs: vec![], controls: json!(null) };
    let port = |who, key: &str, quantity| -> Result<StockPort> {
        let (account, kind) = references.stock(key, candidate)?;
        Ok(StockPort { binding: who, account, kind, quantity })
    };
    match effect {
        Effect::Transfer { from, to, account, amount } => {
            let n = constant(amount)?;
            projection.inputs.push(port(*from, account, n)?); projection.outputs.push(port(*to, account, n)?);
        },
        Effect::Transform { who, take, give, conversion } => {
            projection.family = OperationFamily::Transform;
            for (key, n) in take { projection.inputs.push(port(*who, key, *n)?); }
            for (key, n) in give { projection.outputs.push(port(*who, key, *n)?); }
            projection.inputs.sort_by_key(|row| row.account); projection.outputs.sort_by_key(|row| row.account);
            if let Some(conversion) = conversion { projection.controls = json!({"conversion":conversion}); }
        },
        Effect::Spend { from, to, amount, into } => {
            projection.family = OperationFamily::Spend;
            let n = constant(amount)?;
            for key in from {
                projection.inputs.push(port(Binding::Actor, key, n)?);
                projection.outputs.push(port(*to, into.as_ref().unwrap_or(key), n)?);
            }
            projection.controls = json!({"native_limit":"ordered_total","quantity":n});
        },
        Effect::Eat { from, amount, into, of, whole } => {
            if of.is_empty() || *whole { return Err("all-stock/whole-part meal comparison needs additional native evidence".into()); }
            projection.family = OperationFamily::Eat;
            let n = constant(amount)?;
            for key in of { projection.inputs.push(port(*from, key, n)?); }
            projection.inputs.sort_by_key(|row| row.account);
            projection.outputs.push(port(Binding::Actor, into, n)?);
            projection.controls = json!({"native_limit":"proportional_total","quantity":n});
        },
        Effect::Convert { who, from, to, amount, conversion } => {
            projection.family = OperationFamily::Convert;
            let n = constant(amount)?;
            for key in from { projection.inputs.push(port(*who, key, n)?); }
            projection.inputs.sort_by_key(|row| row.account);
            projection.outputs.push(port(*who, to, n)?);
            projection.controls = json!({"native_limit":"proportional_total","quantity":n,"conversion":conversion});
        },
        Effect::Birth { provision } => {
            projection.family = OperationFamily::Birth;
            for (key, n) in provision { projection.inputs.push(port(Binding::Actor, key, *n)?); }
            projection.inputs.sort_by_key(|row| row.account);
            projection.controls = json!({"native_destination":"biological_child"});
        },
        Effect::Condition { key, delta } => {
            projection.family = OperationFamily::Condition;
            projection.controls = json!({"condition":references.token(NativeReference::Condition(key.clone()),candidate)?,"delta":delta});
        },
        Effect::Wound { who, cells, .. } => {
            projection.family = OperationFamily::Wound;
            projection.controls = json!({"who":who,"cells":constant(cells)?});
        },
        Effect::Allocate { from, to, cells } => {
            projection.family = OperationFamily::Allocate;
            let from = from.as_ref().map(|key| references.token(NativeReference::Function(key.clone()),candidate)).transpose()?;
            projection.controls = json!({"from":from,"to":references.token(NativeReference::Function(to.clone()),candidate)?,"cells":constant(cells)?});
        },
        _ => return Err("consumed mechanical operation is unsupported by this finite admission vocabulary".into()),
    }
    Ok(projection)
}
