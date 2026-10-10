// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shapes, the function catalogue and the part a process binds (rulings
//! 276, 338 to 341, 492 and 504). A function fits at least one of the
//! world's shapes, and any part may express it; a process binds a part
//! through at most one `Expresses`, and reads it as a part can be: its life,
//! its traits and its own ledger, never parts of its own.

use super::key;
use crate::{Result, rules::*, schema::*};

pub(super) fn catalogue(rules: &Rules) -> Result<()> {
    for shape in &rules.shapes {
        key(shape)?;
    }
    for (id, f) in &rules.functions {
        key(id)?;
        if f.shapes.is_empty() {
            return Err(format!("{id} fits no shape"));
        }
        if let Some(shape) = f.shapes.iter().find(|s| !rules.shapes.contains(*s)) {
            return Err(format!("{id} names an unknown shape {shape}"));
        }
    }
    Ok(())
}

/// Systems name only catalogue functions, and a route carries something
/// (rulings 489 and 564).
pub(super) fn systems(rules: &Rules) -> Result<()> {
    for (id, s) in &rules.systems {
        key(id)?;
        if let Some(f) = s
            .functions()
            .into_iter()
            .find(|f| !rules.functions.contains_key(*f))
        {
            return Err(format!("{id} names {f}, which the catalogue lacks"));
        }
    }
    match rules.carriage {
        Some(c) if c.per_cell == 0 => Err("a route carries nothing".into()),
        _ => Ok(()),
    }
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
        } => Ok(true),
        Query::Part {
            who: Binding::Part, ..
        } => Err("a part keeps no parts".into()),
        _ => Ok(false),
    }
}

/// Whether `e` writes the bound part, refusing what a part cannot take; a
/// guarded effect writes what its inner effect does.
fn writes_part(e: &Effect) -> Result<bool> {
    match e {
        Effect::Trait {
            who: Binding::Part, ..
        }
        | Effect::Allocate { .. } => Ok(true),
        Effect::When { .. } => {
            let writes: Result<Vec<bool>> = e.branches().map(writes_part).collect();
            Ok(writes?.into_iter().any(|w| w))
        },
        Effect::Spend {
            to: Binding::Part, ..
        } => Err("a part's ledger is written through its body".into()),
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
        } => Err("a part's ledger is written through its body".into()),
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

/// A part's declared shape is one the world names, or none in a part from
/// before shapes, and each function it expresses is in the catalogue, on
/// whatever shape (ruling 492); it holds only matter.
pub(crate) fn part(rules: &Rules, e: &Entity, id: PartId) -> Result<()> {
    let part = &e.parts[&id];
    let shape = e.declared(id);
    if !shape.is_empty() && !rules.shapes.iter().any(|s| s == shape) {
        return Err(format!("unknown part shape {shape}"));
    }
    for f in &part.functions {
        if !rules.functions.contains_key(f) {
            return Err(format!("unknown function {f}"));
        }
    }
    if let Some(k) = part
        .matter
        .keys()
        .find(|k| !crate::meaning::matter(rules, k))
    {
        return Err(format!("a part holds {k}, which is not matter"));
    }
    // Its cells go only to what it expresses, never more than it has
    // (ruling 453).
    if let Some(f) = part.cells.keys().find(|f| !part.functions.contains(*f)) {
        return Err(format!(
            "a part holds cells for {f}, which it does not express"
        ));
    }
    let held: u64 = part.cells.values().map(|c| u64::from(*c)).sum();
    let capacity = crate::anatomy::living_cells(e.extent(id), part);
    if held > u64::from(capacity) {
        return Err(format!(
            "a part holds {held} cells in a capacity of {capacity}"
        ));
    }
    Ok(())
}
