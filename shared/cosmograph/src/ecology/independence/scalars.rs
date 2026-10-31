// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exact scalar declarations over the supported native consumer fields.
//! These are authored limits and coefficients, not observed per-account shares.

use super::{project, ControlField, Conservation, ConsumedScalar, Dimension, EffectAddress,
    EffectPhase, EffectStep, IndependenceSpec, ScalarEffect, ScalarSource, ScalarValue};
use crate::{Result, rules::{AccountKind, Effect, Process, Query}, schema::{Key, Tick},
    simulation::Genesis};
use std::collections::{BTreeMap, BTreeSet};

struct Resolved {
    dimension: Dimension,
    consumer: Key,
    cadence: Tick,
    conservation: Conservation,
    value: ScalarValue,
}

type Fields = BTreeMap<ScalarSource, Resolved>;

pub(super) fn consumed(genesis: &Genesis, spec: &IndependenceSpec) -> Result<Vec<ConsumedScalar>> {
    if spec.version != 1 { return Err("unsupported scalar declaration version".into()); }
    let consumers: BTreeSet<_> = spec.extras.iter().map(|extra| &extra.consumer.process).collect();
    let mut fields = Fields::new();
    for key in consumers {
        let process = genesis.rules.processes.get(key).ok_or("scalar consumer process is absent")?;
        if &process.id != key { return Err("scalar consumer key and native process identity differ".into()); }
        let cadence = process.period.filter(|tick| *tick > 0)
            .ok_or("scalar consumer has no positive native scheduled cadence")?;
        if process.risk.is_some() { return Err("risk-dependent scalar coverage is unsupported".into()); }
        insert(&mut fields, process, cadence, ScalarSource::Period(key.clone()), Dimension::Ticks,
            Conservation::NotStock, ScalarValue::Unsigned(cadence))?;
        insert(&mut fields, process, cadence, ScalarSource::Priority(key.clone()), Dimension::Priority,
            Conservation::NotStock, ScalarValue::Signed(i64::from(process.priority)))?;
        prerequisites(genesis, process, cadence, &mut fields)?;
        for (commitment, effects) in [(true, &process.commitments), (false, &process.effects)] {
            for (index, _) in effects.iter().enumerate() {
                let index = u32::try_from(index).map_err(|_| "too many native scalar effect ordinals")?;
                let source = ScalarEffect { process: key.clone(), commitment, index };
                effect_fields(genesis, process, cadence, source, &mut fields)?;
            }
        }
    }
    let (mut names, mut sources) = (BTreeSet::new(), BTreeSet::new());
    let mut result = Vec::with_capacity(spec.scalars.len());
    for declaration in &spec.scalars {
        crate::validation::key(&declaration.name)?;
        if !names.insert(&declaration.name) || !sources.insert(&declaration.source) {
            return Err("scalar declaration names and native fields must be unique".into());
        }
        let field = fields.get(&declaration.source)
            .ok_or("scalar declaration is outside the supported consumed native fields")?;
        if declaration.dimension != field.dimension || declaration.consumer != field.consumer
            || declaration.cadence != field.cadence || declaration.conservation != field.conservation {
            return Err(format!("scalar {} does not state its actual native consumer, units, cadence and conservation",
                declaration.name));
        }
        result.push(ConsumedScalar { declaration: declaration.clone(), value: field.value.clone() });
    }
    if fields.len() != sources.len() {
        return Err("scalar declarations do not cover every supported consumed field of the extra consumers".into());
    }
    Ok(result)
}

fn insert(fields: &mut Fields, process: &Process, cadence: Tick, source: ScalarSource,
    dimension: Dimension, conservation: Conservation, value: ScalarValue) -> Result<()> {
    let field = Resolved { dimension, consumer: process.id.clone(), cadence, conservation, value };
    if fields.insert(source, field).is_some() {
        return Err("native scalar derivation repeats a consumed field".into());
    }
    Ok(())
}

fn stock(genesis: &Genesis, key: &str) -> Result<(Dimension, Conservation)> {
    let kind = genesis.rules.accounts.get(key).ok_or("scalar stock account is absent")?;
    let conservation = if matches!(kind, AccountKind::Matter { .. }) {
        Conservation::NativeMatter
    } else { Conservation::OtherStock };
    Ok((Dimension::Stock(kind.clone()), conservation))
}

fn matter(genesis: &Genesis, key: &str) -> Result<()> {
    if !matches!(genesis.rules.accounts.get(key), Some(AccountKind::Matter { .. })) {
        return Err("native total-matter scalar references a non-matter account".into());
    }
    Ok(())
}

fn spend_units(genesis: &Genesis, from: &[Key], into: Option<&Key>) -> Result<(Dimension, Conservation)> {
    let first = from.first().ok_or("native spend scalar has no stock source")?;
    let kind = genesis.rules.accounts.get(first).ok_or("native spend account is absent")?;
    if matches!(kind, AccountKind::Matter { .. }) {
        for key in from { matter(genesis, key)?; }
        if let Some(key) = into { matter(genesis, key)?; }
        Ok((Dimension::MatterQuantity, Conservation::NativeMatter))
    } else {
        if into.is_some() || from.iter().any(|key| genesis.rules.accounts.get(key) != Some(kind)) {
            return Err("mixed-dimension native spend bound is unsupported".into());
        }
        stock(genesis, first)
    }
}

fn effect_fields(genesis: &Genesis, process: &Process, cadence: Tick,
    source: ScalarEffect, fields: &mut Fields) -> Result<()> {
    let address = EffectAddress { process: source.process.clone(),
        phase: if source.commitment { EffectPhase::Commitment } else { EffectPhase::Outcome },
        path: vec![EffectStep::Item(source.index)] };
    let (_, effect) = project::leaf(genesis, &address)?;
    let amount = |fields: &mut Fields, value: &crate::rules::Amount, units: (Dimension, Conservation)| {
        insert(fields, process, cadence, ScalarSource::Amount(source.clone()), units.0, units.1,
            ScalarValue::Unsigned(project::constant(value)?))
    };
    match effect {
        Effect::Transfer { account, amount: value, .. } => amount(fields, value, stock(genesis, account)?),
        Effect::Transform { take, give, .. } => {
            for (input, rows) in [(true, take), (false, give)] {
                for (account, value) in rows {
                    let (dimension, conservation) = stock(genesis, account)?;
                    let field = if input { ScalarSource::Input { effect: source.clone(), account: account.clone() } }
                        else { ScalarSource::Output { effect: source.clone(), account: account.clone() } };
                    insert(fields, process, cadence, field, dimension, conservation, ScalarValue::Unsigned(*value))?;
                }
            }
            Ok(())
        },
        Effect::Birth { provision } => {
            for (account, value) in provision {
                let (dimension, conservation) = stock(genesis, account)?;
                insert(fields, process, cadence, ScalarSource::Input { effect: source.clone(), account: account.clone() },
                    dimension, conservation, ScalarValue::Unsigned(*value))?;
            }
            Ok(())
        },
        Effect::Spend { from, into, amount: value, .. } =>
            amount(fields, value, spend_units(genesis, from, into.as_ref())?),
        Effect::Eat { of, into, whole, amount: value, .. } => {
            if of.is_empty() || *whole {
                return Err("all-stock or whole-part meals need additional scalar evidence".into());
            }
            for key in of { matter(genesis, key)?; }
            matter(genesis, into)?;
            amount(fields, value, (Dimension::MatterQuantity, Conservation::NativeMatter))
        },
        Effect::Convert { from, to, amount: value, .. } => {
            if from.is_empty() { return Err("native conversion scalar has no stock source".into()); }
            for key in from { matter(genesis, key)?; }
            matter(genesis, to)?;
            amount(fields, value, (Dimension::MatterQuantity, Conservation::NativeMatter))
        },
        Effect::Condition { key, delta } => {
            if !genesis.rules.conditions.contains(key) { return Err("scalar condition is absent".into()); }
            insert(fields, process, cadence,
                ScalarSource::Control { effect: source.clone(), field: ControlField::ConditionDelta },
                Dimension::ConditionLevel, Conservation::NotStock, ScalarValue::Signed(*delta))
        },
        Effect::Wound { cells, .. } | Effect::Allocate { cells, .. } => insert(fields, process, cadence,
            ScalarSource::Control { effect: source.clone(), field: ControlField::Cells },
            Dimension::Cells, Conservation::NotStock, ScalarValue::Unsigned(project::constant(cells)?)),
        // These fields are identities or flags; no scalar amount is authored here.
        Effect::Death | Effect::Trait { .. } | Effect::Relate { .. }
        | Effect::Tell { .. } | Effect::Record { .. } | Effect::FoundPolity { .. }
        | Effect::Note { lifetime: None, .. } => Ok(()),
        // Move consumes authored route time; profiled/body operations consume further law fields.
        _ => Err("guarded, indirect or unsupported native effect scalars cannot be omitted from coverage".into()),
    }
}

fn prerequisites(genesis: &Genesis, process: &Process, cadence: Tick, fields: &mut Fields) -> Result<()> {
    if let Some(key) = &process.need_account {
        let (dimension, conservation) = stock(genesis, key)?;
        insert(fields, process, cadence, ScalarSource::NeedBelow(process.id.clone()),
            dimension, conservation, ScalarValue::Unsigned(process.need_below))?;
    }
    for (index, query) in process.requires.iter().enumerate() {
        let (dimension, conservation, value) = match query {
            Query::Account { key, at_least, .. } => {
                let (dimension, conservation) = stock(genesis, key)?;
                (dimension, conservation, ScalarValue::Unsigned(*at_least))
            },
            Query::Below { key, amount, .. } => {
                let (dimension, conservation) = stock(genesis, key)?;
                (dimension, conservation, ScalarValue::Unsigned(*amount))
            },
            Query::Age { at_least } =>
                (Dimension::Ticks, Conservation::NotStock, ScalarValue::Unsigned(*at_least)),
            Query::Condition { key, at_least } => {
                if !genesis.rules.conditions.contains(key) { return Err("scalar prerequisite condition is absent".into()); }
                (Dimension::ConditionLevel, Conservation::NotStock, ScalarValue::Signed(*at_least))
            },
            Query::Holds { at_least, .. } =>
                (Dimension::MatterQuantity, Conservation::NativeMatter, ScalarValue::Unsigned(*at_least)),
            // Exact body revisions and part IDs are identity witnesses, not quantities.
            Query::Alive(_) | Query::Trait { .. } | Query::Related { .. }
            | Query::Expresses { .. } | Query::Routes { .. } | Query::Part { .. } => continue,
            _ => return Err("dynamic, mood or indirect prerequisite scalar units are unsupported".into()),
        };
        let index = u32::try_from(index).map_err(|_| "too many native prerequisite ordinals")?;
        insert(fields, process, cadence, ScalarSource::Requirement { process: process.id.clone(), index },
            dimension, conservation, value)?;
    }
    Ok(())
}
