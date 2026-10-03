// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Kinds and what lineages develop from (rulings 478, 511, 516, 518, 530
//! and 531). A kind is a box whose cells its functions share within its
//! capacity; a recipe names only kinds the world holds and its lineage has
//! learned; and a lineage's domain is one the world's affinity holds.

use super::key;
use crate::{Result, anatomy, rules::*, schema::*};
use std::collections::BTreeMap;

/// The names a box reads, which a policy may prefer a facing for.
const BOXES: [&str; 4] = [
    "part-shape:lump",
    "part-shape:rod",
    "part-shape:sheet",
    "part-shape:point",
];

pub(super) fn kinds(rules: &Rules) -> Result<()> {
    for (id, kind) in &rules.kinds {
        key(id)?;
        let part = Part {
            half_extent: kind.half_extent,
            ..Default::default()
        };
        if !part.bodied() {
            return Err(format!("kind {id} has no box"));
        }
        if let Some(f) = kind
            .cells
            .keys()
            .find(|f| !rules.functions.contains_key(*f))
        {
            return Err(format!("kind {id} gives cells to an unknown function {f}"));
        }
        let cells: u64 = kind.cells.values().map(|c| u64::from(*c)).sum();
        if cells > u64::from(anatomy::capacity(&part)) {
            return Err(format!("kind {id} gives more cells than its box holds"));
        }
        if !matches!(
            kind.shape.as_str(),
            "" | "part-shape:tube" | "part-shape:shell"
        ) {
            return Err(format!("kind {id} declares a shape its box can read"));
        }
    }
    for (id, kind) in &rules.accounts {
        if let AccountKind::Matter {
            reserve: true,
            provision: true,
            ..
        } = kind
        {
            return Err(format!("{id} is both a reserve and a provision"));
        }
    }
    Ok(())
}

pub(crate) fn development(rules: &Rules, lineages: &BTreeMap<Key, Lineage>) -> Result<()> {
    let affinity = rules.affinity.clone().unwrap_or_default();
    for (id, lineage) in lineages {
        let Some(d) = &lineage.development else {
            continue;
        };
        let refuse = |why: &str| Err(format!("{id}'s development {why}"));
        let recipe = &d.recipe;
        if recipe.tagmata.is_empty() {
            return refuse("has no tagmata");
        }
        for (i, t) in recipe.tagmata.iter().enumerate() {
            if t.segments == 0 {
                return refuse("has a tagma of no segments");
            }
            if t.bears.is_some() != (t.per_segment > 0) {
                return refuse("bears a kind on no segment, or none on some");
            }
            if t.parent.is_some_and(|p| usize::from(p) >= i) {
                return refuse("branches a tagma from one not yet placed");
            }
        }
        let [odds, of] = recipe.absence;
        if of == 0 || odds > of {
            return refuse("has absence odds that are no chance");
        }
        let kinds = recipe.kinds();
        if let Some(k) = kinds.iter().find(|k| !d.lexicon.contains(*k)) {
            return Err(format!("{id}'s recipe names {k}, which its lexicon lacks"));
        }
        if let Some(k) = d.lexicon.iter().find(|k| !rules.kinds.contains_key(*k)) {
            return Err(format!("{id}'s lexicon holds {k}, which the world lacks"));
        }
        if d.policy
            .preferences
            .keys()
            .any(|k| !BOXES.contains(&k.as_str()))
        {
            return refuse("prefers a facing for a name no box reads");
        }
        if usize::from(d.policy.tolerance) >= Facing::ALL.len() {
            return refuse("tolerates more facings than there are");
        }
        if d.domain >= affinity.domains {
            return refuse("names a domain the world's affinity lacks");
        }
        if d.clutch == 0 {
            return refuse("lays clutches of no eggs");
        }
    }
    Ok(())
}
