// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reference incidence, not author-assigned token ordinals, establishes flow shape.

use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn sorted(mut rows: Vec<Value>) -> Vec<Value> {
    rows.sort_by_key(|value| value.to_string());
    rows
}

pub(super) fn shape(projection: &MechanicalProjection, paid_only: bool) -> Value {
    let mut accounts: BTreeMap<u32, (&StockType, Vec<Value>)> = BTreeMap::new();
    let ordered = projection.controls.get("native_limit").and_then(Value::as_str) == Some("ordered_total");
    for (output, ports) in [(false, &projection.inputs), (true, &projection.outputs)] {
        if paid_only && output { continue; }
        for (index, port) in ports.iter().enumerate() {
            let quantity = if paid_only {
                let returned = projection.outputs.iter().filter(|row| row.binding == port.binding
                    && row.account == port.account && row.kind == port.kind)
                    .map(|row| u128::from(row.quantity)).sum::<u128>();
                port.quantity.saturating_sub(returned.min(u128::from(u64::MAX)) as u64)
            } else { port.quantity };
            if paid_only && quantity == 0 { continue; }
            accounts.entry(port.account).or_insert((&port.kind, vec![])).1.push(json!({
                "output":output,"binding":port.binding,"quantity":quantity,
                "ordered_position":if ordered {Some(index)} else {None},
            }));
        }
    }
    let mut lineages: BTreeMap<u32, Vec<Value>> = BTreeMap::new();
    let mut other = vec![];
    for (_, (kind, occurrences)) in accounts {
        let occurrences = sorted(occurrences);
        match kind {
            StockType::Matter { lineage, world, reserve, provision } =>
                lineages.entry(*lineage).or_default().push(json!({
                    "world":world,"reserve":reserve,"provision":provision,"occurrences":occurrences,
                })),
            kind => other.push(json!({"kind":kind,"occurrences":occurrences})),
        }
    }
    let matter = sorted(lineages.into_values().map(|rows| json!(sorted(rows))).collect());
    let mut controls = if paid_only {
        json!({"native_limit":projection.controls.get("native_limit").cloned().unwrap_or(json!("exact")),
            "quantity":projection.controls.get("quantity")})
    } else { projection.controls.clone() };
    if let Some(fields) = controls.as_object_mut() {
        if fields.contains_key("condition") { fields.insert("condition".into(), json!(0)); }
        if projection.family == OperationFamily::Allocate {
            let from = fields.get("from").and_then(Value::as_u64);
            let to = fields.get("to").and_then(Value::as_u64);
            fields.insert("from".into(), json!(from.map(|_| 0)));
            fields.insert("to".into(), json!(if from.is_some() && from == to {0} else {1}));
        }
    }
    json!({"matter_reference_groups":matter,"other_stock_nodes":sorted(other),"controls":controls})
}
