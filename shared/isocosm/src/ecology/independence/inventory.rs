// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Founding, Result, schema::Key, simulation::Genesis};
use serde_json::Value;
use std::collections::BTreeMap;

/// Only this fresh producer can create a basis; authored inputs cannot deserialize one.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedBasis { pub(super) source: Genesis }

impl GeneratedBasis {
    pub fn new(founding: &Founding) -> Result<Self> {
        ordinary(founding)?;
        Ok(Self { source: founding.generate()? })
    }
    pub fn source_digest(&self) -> Key { crate::digest(&self.source) }
}

pub(super) fn ordinary(founding: &Founding) -> Result<()> {
    if founding.preset.is_some() || founding.affinity.is_some() {
        return Err("ecology basis cannot exempt authored preset/affinity extensions or unused parameters".into());
    }
    Ok(())
}

fn scrub(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            if let Some(Value::Object(note)) = fields.get_mut("Note") { note.remove("text"); }
            if let Some(Value::Object(draw)) = fields.get_mut("Draw") { draw.remove("slot"); }
            for value in fields.values_mut() { scrub(value); }
        },
        Value::Array(rows) => { for row in rows { scrub(row); } },
        _ => {},
    }
}

pub(super) fn inventory(genesis: &Genesis) -> Result<BTreeMap<NativeReference, Value>> {
    let mut rows = BTreeMap::new();
    let mut rules = serde_json::to_value(&genesis.rules).map_err(|e| e.to_string())?;
    let fields = rules.as_object_mut().ok_or("native Rules inventory is not a record")?;
    fields.remove("ecology_witness"); // The admission declaration is not a mechanic.
    let catalogues: [(&str, fn(Key) -> NativeReference); 10] = [
        ("accounts", NativeReference::Account), ("traits", NativeReference::Trait),
        ("conditions", NativeReference::Condition), ("relations", NativeReference::Relation),
        ("shapes", NativeReference::Shape), ("functions", NativeReference::Function),
        ("processes", NativeReference::Process), ("kinds", NativeReference::Kind),
        ("systems", NativeReference::System), ("note_kinds", NativeReference::RuleField),
    ];
    for (field, reference) in catalogues {
        let Some(value) = fields.remove(field) else { continue; };
        match value {
            Value::Object(entries) => for (key, mut value) in entries {
                if field == "processes" {
                    if let Some(record) = value.as_object_mut() { record.remove("id"); record.remove("glyphs"); }
                }
                scrub(&mut value); rows.insert(reference(key), value);
            },
            Value::Array(entries) => for value in entries {
                let key = value.as_str().ok_or("native set inventory has a non-key entry")?;
                rows.insert(reference(key.into()), Value::Null);
            },
            _ => return Err("native catalogue inventory has an unsupported shape".into()),
        }
    }
    // Functional extension fields are covered whole and refuse until their actual consumer is supported.
    for (field, mut value) in fields.clone() { scrub(&mut value); rows.insert(NativeReference::RuleField(field), value); }
    for (key, line) in &genesis.lineages {
        let mut value = serde_json::to_value(line).map_err(|e| e.to_string())?;
        scrub(&mut value); rows.insert(NativeReference::Lineage(key.clone()), value);
    }
    let mut world = serde_json::to_value(&genesis.world).map_err(|e| e.to_string())?;
    let fields = world.as_object_mut().ok_or("native world inventory is not a record")?;
    fields.remove("materials");
    for (field, mut value) in fields.clone() { scrub(&mut value); rows.insert(NativeReference::WorldField(field), value); }
    for material in &genesis.world.materials {
        let mut value = serde_json::to_value(material).map_err(|e| e.to_string())?;
        value.as_object_mut().ok_or("native material is not a record")?.remove("key");
        if rows.insert(NativeReference::Material(material.key.clone()), value).is_some() {
            return Err("native material inventory repeats an identity".into());
        }
    }
    Ok(rows)
}

pub(super) fn changed(basis: &Genesis, candidate: &Genesis) -> Result<Vec<NativeReference>> {
    if basis.founding != candidate.founding { return Err("ecology candidate and generated basis have different founding inputs".into()); }
    let before = inventory(basis)?;
    let after = inventory(candidate)?;
    if before.keys().any(|key| !after.contains_key(key)) {
        return Err("ecology admission does not support removal/replacement of basis definitions".into());
    }
    Ok(after.iter().filter(|(key, value)| before.get(*key) != Some(*value)).map(|(key, _)| key.clone()).collect())
}
