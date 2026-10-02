// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shapes, the function catalogue and the part a process binds (rulings
//! 276 and 338 to 341). A function is admitted by at least one of the
//! world's shapes; a part expresses only functions its shape admits; a
//! process binds a part through at most one `Expresses`, and reads or writes
//! it only as a part can be: its life and its traits, never a ledger.

use super::key;
use crate::{Result, rules::*, schema::*};

pub(super) fn catalogue(rules: &Rules) -> Result<()> {
    for shape in &rules.shapes {
        key(shape)?;
    }
    for (id, f) in &rules.functions {
        key(id)?;
        if f.shapes.is_empty() {
            return Err(format!("{id} is admitted by no shape"));
        }
        if let Some(shape) = f.shapes.iter().find(|s| !rules.shapes.contains(*s)) {
            return Err(format!("{id} names an unknown shape {shape}"));
        }
    }
    Ok(())
}

/// Whether `q` reads the bound part, refusing what a part cannot answer.
pub(super) fn reads_part(q: &Query) -> Result<bool> {
    match q {
        Query::Alive(Binding::Part)
        | Query::Trait {
            who: Binding::Part, ..
        } => Ok(true),
        Query::Account {
            who: Binding::Part, ..
        }
        | Query::Below {
            who: Binding::Part, ..
        }
        | Query::Holds {
            who: Binding::Part, ..
        }
        | Query::Part {
            who: Binding::Part, ..
        } => Err("a part keeps no ledger and no parts".into()),
        _ => Ok(false),
    }
}

/// Whether `e` writes the bound part, refusing what a part cannot take.
fn writes_part(e: &Effect) -> Result<bool> {
    match e {
        Effect::Trait {
            who: Binding::Part, ..
        } => Ok(true),
        Effect::Transfer {
            from: Binding::Part,
            ..
        }
        | Effect::Transfer {
            to: Binding::Part, ..
        }
        | Effect::Transform {
            who: Binding::Part, ..
        }
        | Effect::Ease {
            who: Binding::Part, ..
        }
        | Effect::Eat {
            from: Binding::Part,
            ..
        } => Err("a part keeps no ledger".into()),
        _ => Ok(false),
    }
}

/// A process binds at most one part, and uses the binding only if it has
/// one.
pub(super) fn process(p: &Process) -> Result<()> {
    let binds = p
        .requires
        .iter()
        .filter(|q| matches!(q, Query::Expresses { .. }))
        .count();
    if binds > 1 {
        return Err(format!("{} binds more than one part", p.id));
    }
    let named = |why: String| format!("{}: {why}", p.id);
    let mut uses = false;
    for q in &p.requires {
        uses |= reads_part(q).map_err(named)?;
    }
    let effects = p
        .commitments
        .iter()
        .chain(&p.effects)
        .chain(p.risk.iter().flat_map(|r| &r.effects));
    for e in effects {
        uses |= writes_part(e).map_err(named)?;
    }
    if uses && binds == 0 {
        return Err(format!("{} uses a part it does not bind", p.id));
    }
    Ok(())
}

/// A part's shape is one the world names, or none in a part from before
/// shapes, and each function it expresses is in the catalogue and admitted
/// by that shape.
pub(crate) fn part(rules: &Rules, part: &Part) -> Result<()> {
    if !part.shape.is_empty() && !rules.shapes.contains(&part.shape) {
        return Err(format!("unknown part shape {}", part.shape));
    }
    for f in &part.functions {
        if !rules.functions.contains_key(f) {
            return Err(format!("unknown function {f}"));
        }
        if !rules.admits(&part.shape, f) {
            return Err(format!(
                "a part of shape {:?} cannot express {f}",
                part.shape
            ));
        }
    }
    // Its cells go only to what it expresses, never more than it has
    // (ruling 453).
    if let Some(f) = part.cells.keys().find(|f| !part.functions.contains(*f)) {
        return Err(format!(
            "a part holds cells for {f}, which it does not express"
        ));
    }
    let held: u64 = part.cells.values().map(|c| u64::from(*c)).sum();
    if held > u64::from(part.capacity) {
        return Err(format!(
            "a part holds {held} cells in a capacity of {}",
            part.capacity
        ));
    }
    Ok(())
}
