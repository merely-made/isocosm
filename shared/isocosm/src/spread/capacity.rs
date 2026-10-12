// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored lineage mechanics and capacity modifiers (427, 429, 827).
//! Named trait inputs are native presence readings, not invented gene weights.

use super::*;
use crate::{Result, rules::{Expr, Reading, Rules}, schema::{Key, WorldTraits}, simulation::State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stacking { Sum, Highest, Lowest, Product, Replace }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityLayer {
    /// This layer's output name, available only to later layers.
    pub key: Key,
    /// The world's effect vocabulary, independent of any glyph correspondence.
    pub effect: Key,
    pub stacking: Stacking,
    /// Name of this layer's aggregate explicit values inside `reading`.
    pub input: Key,
    /// Declares how the aggregate and earlier readings yield capacity.
    pub reading: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityRules {
    /// Name of the trait mechanic's result, available to all later layers.
    pub key: Key,
    /// Expression input name -> native lineage trait key. Presence is 1 or 0.
    pub trait_inputs: BTreeMap<Key, Key>,
    pub mechanic: Expr,
    /// World order, never modifier insertion order or a fixed stacking default.
    pub layers: Vec<CapacityLayer>,
}

fn expression(expr: &Expr, names: &BTreeSet<Key>) -> Result<()> {
    expr.validate()?;
    fn shape(expr: &Expr) -> bool {
        !matches!(expr, Expr::Draw { .. } | Expr::Parts { .. } | Expr::Carried { .. })
            && expr.children().into_iter().all(shape)
    }
    if !shape(expr) { return Err("capacity reads named native inputs without draws or body parts".into()); }
    for read in expr.reads() {
        match read.reading {
            Reading::Kept { name } if names.contains(name) => {},
            _ => return Err("capacity expression names an undeclared or later input".into()),
        }
    }
    Ok(())
}

fn evaluate(expr: &Expr, values: &BTreeMap<Key, i64>) -> Result<i64> {
    expr.eval(&mut |reading| match reading {
        Reading::Kept { name } => values.get(name).copied().ok_or("capacity input is unavailable".into()),
        _ => Err("capacity expression has no such native reading".into()),
    }, &mut |_, _| Err("capacity has no random draw".into()))
}

impl CapacityRules {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        crate::validation::key(&self.key)?;
        let mut names = BTreeSet::new();
        for (name, trait_key) in &self.trait_inputs {
            crate::validation::key(name)?;
            if !rules.traits.contains(trait_key) { return Err("capacity mechanic names an unknown lineage trait".into()); }
            names.insert(name.clone());
        }
        expression(&self.mechanic, &names)?;
        if self.mechanic.reads().is_empty() {
            return Err("capacity mechanic must read declared lineage traits".into());
        }
        if !names.insert(self.key.clone()) { return Err("capacity mechanic aliases a trait input".into()); }
        for layer in &self.layers {
            crate::validation::key(&layer.key)?;
            crate::validation::key(&layer.effect)?;
            crate::validation::key(&layer.input)?;
            if names.contains(&layer.key) || names.contains(&layer.input) || layer.key == layer.input {
                return Err("capacity layer aliases an earlier input".into());
            }
            let mut inputs = names.clone();
            inputs.insert(layer.input.clone());
            expression(&layer.reading, &inputs)?;
            names.insert(layer.key.clone());
        }
        Ok(())
    }

    /// Fundamental effects retain the world's existing glyph vocabulary.
    pub fn validate_world(&self, world: &WorldTraits) -> Result<()> {
        if self.layers.iter().any(|layer| !world.canon.glyphs.iter().any(|glyph| glyph.effect == layer.effect)) {
            return Err("capacity modifier names an effect outside the world's vocabulary".into());
        }
        Ok(())
    }
}

fn stacked(stacking: Stacking, values: &[u64]) -> Result<i64> {
    let values: Vec<i64> = values.iter().map(|value| i64::try_from(*value)
        .map_err(|_| "capacity modifier exceeds the native signed expression range".into()))
        .collect::<Result<_>>()?;
    Ok(match stacking {
        Stacking::Sum => values.into_iter().fold(0, i64::saturating_add),
        Stacking::Highest => *values.iter().max().ok_or("capacity layer has no values")?,
        Stacking::Lowest => *values.iter().min().ok_or("capacity layer has no values")?,
        Stacking::Product => values.into_iter().fold(1, i64::saturating_mul),
        Stacking::Replace => {
            if values.len() != 1 { return Err("replace capacity layer needs one applicable value or separately ordered layers".into()); }
            values[0]
        },
    })
}

pub fn capacity(state: &State, rules: &Rules, at: Substrate, identity: &Identity) -> Result<u64> {
    held(state, at, identity)?;
    let policy = rules.spread_capacity.as_ref().ok_or("world has no declared spread capacity mechanic")?;
    let lineage = provenance(state, &identity.key()).ok_or("capacity has no native lineage")?;
    let traits = &state.lineages.get(&lineage).ok_or("capacity lineage is absent")?.traits;
    let mut values: BTreeMap<Key, i64> = policy.trait_inputs.iter()
        .map(|(name, key)| (name.clone(), i64::from(traits.contains(key)))).collect();
    let mut result = evaluate(&policy.mechanic, &values)?.max(0);
    values.insert(policy.key.clone(), result);
    for layer in &policy.layers {
        let modifiers: Vec<_> = state.spreads.modifiers.iter().filter(|modifier| {
            modifier.layer == layer.key
                && modifier.identity.as_ref().is_none_or(|selected| selected == identity)
                && match modifier.source {
                    CapacitySource::World => true,
                    CapacitySource::Substrate(substrate) => substrate == at,
                }
        }).map(|modifier| modifier.milligrams).collect();
        if !modifiers.is_empty() {
            values.insert(layer.input.clone(), stacked(layer.stacking, &modifiers)?);
            result = evaluate(&layer.reading, &values)?.max(0);
            values.remove(&layer.input);
        }
        values.insert(layer.key.clone(), result);
    }
    Ok(result as u64)
}

pub(crate) fn validate_modifiers(spreads: &Spreads, rules: &Rules,
    population: &crate::population::Population,
    sites: &BTreeMap<crate::schema::Id, crate::schema::Site>) -> Result<()> {
    let Some(policy) = &rules.spread_capacity else {
        return if spreads.modifiers.is_empty() { Ok(()) }
            else { Err("capacity modifiers have no world-declared mechanic".into()) };
    };
    policy.validate(rules)?;
    let mut keys = BTreeSet::new();
    for modifier in &spreads.modifiers {
        crate::validation::key(&modifier.key)?;
        if !keys.insert(&modifier.key) || !policy.layers.iter().any(|layer| layer.key == modifier.layer)
            || modifier.identity.as_ref().is_some_and(|identity| !spreads.identities.contains(identity)) {
            return Err("capacity modifier key, layer or spread selector is invalid".into());
        }
        i64::try_from(modifier.milligrams).map_err(|_| "capacity modifier exceeds the native signed expression range")?;
        match modifier.source {
            CapacitySource::World => {},
            CapacitySource::Substrate(Substrate::Site(site)) if sites.contains_key(&site) => {},
            CapacitySource::Substrate(Substrate::Host(host)) if population.get(host).is_some() => {},
            CapacitySource::Substrate(Substrate::Place(place)) if spreads.places.iter().any(|p| p.place == place) => {},
            _ => return Err("capacity modifier source is not a native substrate".into()),
        }
        if matches!((&modifier.identity, modifier.source),
            (Some(Identity::Territory(_)), CapacitySource::Substrate(Substrate::Host(_)))) {
            return Err("territory capacity belongs to a place, not a host".into());
        }
    }
    if spreads.modifiers.windows(2).any(|pair| pair[0].key >= pair[1].key) {
        return Err("capacity modifiers must be in canonical key order".into());
    }
    for layer in policy.layers.iter().filter(|layer| layer.stacking == Stacking::Replace) {
        let modifiers: Vec<_> = spreads.modifiers.iter().filter(|modifier| modifier.layer == layer.key).collect();
        for (index, a) in modifiers.iter().enumerate() {
            for b in &modifiers[index + 1..] {
                let identity = a.identity.is_none() || b.identity.is_none() || a.identity == b.identity;
                let source = a.source == CapacitySource::World || b.source == CapacitySource::World || a.source == b.source;
                let host = matches!(a.source, CapacitySource::Substrate(Substrate::Host(_)))
                    || matches!(b.source, CapacitySource::Substrate(Substrate::Host(_)));
                let territory = matches!(a.identity.as_ref().or(b.identity.as_ref()), Some(Identity::Territory(_)));
                if identity && source && !(host && territory) {
                    return Err("replace capacity layer has overlapping values; declare separate ordered layers".into());
                }
            }
        }
    }
    Ok(())
}
