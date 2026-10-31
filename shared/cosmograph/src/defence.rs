// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The final granted bite is compared with the selected prey part's law.

use crate::{Result, rules::{Expr, Reading, Rules}, schema::{Entity, Key, PartId}, spread::Stacking};
use std::collections::BTreeMap;

pub fn threshold(rules: &Rules, prey: &Entity, id: PartId) -> Result<Option<u64>> {
    let Some(law) = &rules.defence else { return Ok(None); };
    let part = prey.parts.get(&id).ok_or("protected part is absent")?;
    if !law.tissue.iter().any(|function| part.functions.contains(function)
        && part.cells.get(function).is_some_and(|cells| *cells > 0)) {
        return Ok(None);
    }
    if prey.geo(id).is_none() {
        return Err("protected tissue has no actual native part geometry".into());
    }
    let half = prey.extent(id);
    let mut values = BTreeMap::<Key, i64>::new();
    let evaluate = |expr: &Expr, values: &BTreeMap<Key, i64>| -> Result<i64> {
        expr.eval(&mut |reading| match reading {
            Reading::Kept { name } => values.get(name).copied().ok_or("defence layer is unavailable".into()),
            reading => i64::try_from(reading.of_part(half, part, rules.body()))
                .map_err(|_| "defence tissue reading exceeds the native expression range".into()),
        }, &mut |_, _| Err("defence has no random draw".into()))
    };
    let mut result = evaluate(&law.threshold, &values)?.max(0);
    values.insert(law.key.clone(), result);
    for layer in &law.layers {
        let modifiers = layer.modifiers.iter().map(|expr| evaluate(expr, &values))
            .collect::<Result<Vec<_>>>()?;
        let aggregate = match layer.stacking {
            Stacking::Sum => modifiers.into_iter().fold(0, i64::saturating_add),
            Stacking::Highest => *modifiers.iter().max().ok_or("defence layer has no values")?,
            Stacking::Lowest => *modifiers.iter().min().ok_or("defence layer has no values")?,
            Stacking::Product => modifiers.into_iter().fold(1, i64::saturating_mul),
            Stacking::Replace if modifiers.len() == 1 => modifiers[0],
            Stacking::Replace => return Err("replace defence layer requires one value".into()),
        };
        values.insert(layer.input.clone(), aggregate);
        result = evaluate(&layer.reading, &values)?.max(0);
        values.remove(&layer.input);
        values.insert(layer.key.clone(), result);
    }
    Ok(Some(result as u64))
}
