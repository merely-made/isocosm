// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::{Query, Rules}};

pub(super) fn catalogue(rules: &Rules) -> Result<()> {
    crate::spatial::validate(rules)?;
    if let Some(stock) = &rules.owned_stock { stock.validate(rules)?; }
    for (name, material) in &rules.material_recipes {
        super::key(name)?;
        material.validate(rules)?;
    }
    if let Some(projects) = &rules.projects { projects.validate(rules)?; }
    Ok(())
}

pub(super) fn query(rules: &Rules, query: &Query) -> Result<()> {
    if let Query::Project { definition, process, .. } = query {
        if !rules.projects.as_ref().is_some_and(|catalogue|
            catalogue.definitions.contains_key(definition))
            || !rules.processes.contains_key(process)
        {
            return Err(format!("unknown project definition or process: {definition}, {process}"));
        }
    }
    Ok(())
}
