// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored defensive tissue laws, independent of a part's plate geometry.

use super::{Binding, Expr, Reading, Rules};
use crate::{Result, schema::Key, spread::Stacking};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defence {
    /// Explicit function allocations designate protected tissue, not Role::Plate.
    pub tissue: BTreeSet<Key>,
    /// This result is available to the later authored modifier layers.
    pub key: Key,
    pub threshold: Expr,
    pub layers: Vec<DefenceLayer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefenceLayer {
    pub key: Key,
    pub input: Key,
    pub stacking: Stacking,
    /// Actual selected-part expressions, combined with the supplied stacking law.
    pub modifiers: Vec<Expr>,
    /// Reads this layer's input and earlier outputs; no additive/override default.
    pub reading: Expr,
}

fn expression(expr: &Expr, rules: &Rules, names: &BTreeSet<Key>) -> Result<()> {
    expr.validate()?;
    fn finite(expr: &Expr) -> bool {
        !matches!(expr, Expr::Draw { .. } | Expr::Parts { .. } | Expr::Carried { .. })
            && expr.children().into_iter().all(finite)
    }
    if !finite(expr) { return Err("defence reads the selected protected part without draws or part folds".into()); }
    for read in expr.reads() {
        let reading = read.reading;
        if let Reading::Kept { name } = reading {
            if names.contains(name) { continue; }
            return Err("defence expression names an unavailable or later layer".into());
        }
        if reading.who() != Binding::Part { return Err("defence measurement must name the protected part".into()); }
        match reading {
            Reading::Account { key, .. } if rules.accounts.contains_key(key) => {},
            Reading::Span { function, .. } | Reading::Cells { function, .. }
                | Reading::CellMass { function, .. } | Reading::Measured { function, .. }
                if rules.functions.contains_key(function) => {},
            Reading::Voxels { .. } | Reading::LivingCells { .. }
                | Reading::LostCells { .. } | Reading::CellWeight { .. } => {},
            _ => return Err("defence measurement is unsupported or names undeclared native vocabulary".into()),
        }
    }
    Ok(())
}

impl Defence {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        crate::validation::key(&self.key)?;
        if self.tissue.is_empty() || self.tissue.iter().any(|key| !rules.functions.contains_key(key)) {
            return Err("defence must explicitly designate declared tissue functions".into());
        }
        let mut names = BTreeSet::new();
        expression(&self.threshold, rules, &names)?;
        if !self.threshold.reads().iter().any(|read| read.reading.who() == Binding::Part) {
            return Err("defence threshold must read actual protected tissue".into());
        }
        names.insert(self.key.clone());
        for layer in &self.layers {
            crate::validation::key(&layer.key)?;
            crate::validation::key(&layer.input)?;
            if names.contains(&layer.key) || names.contains(&layer.input) || layer.key == layer.input {
                return Err("defence layer aliases an earlier input".into());
            }
            if layer.modifiers.is_empty()
                || (layer.stacking == Stacking::Replace && layer.modifiers.len() != 1) {
                return Err("defence layer requires explicit values; Replace requires one".into());
            }
            for modifier in &layer.modifiers { expression(modifier, rules, &names)?; }
            let mut inputs = names.clone();
            inputs.insert(layer.input.clone());
            expression(&layer.reading, rules, &inputs)?;
            names.insert(layer.key.clone());
        }
        Ok(())
    }
}
