// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Paid restoration of recorded allocations (810), keeping a tombstone's
//! identity and geometry (719). Unknown prior allocations are never guessed.

use crate::{
    Result, anatomy, growth,
    mosaic::{CellId, Tract},
    rules::Rules,
    schema::*,
};
use std::collections::BTreeMap;

/// The next known lost cell, nearest the root first. A tombstone must
/// have a living parent and its original box must still be free.
pub(crate) fn next(e: &Entity) -> Option<(PartId, CellId)> {
    if !e.alive || !e.parts.values().any(|p| p.lost.iter().any(|c| p.losses.contains_key(c))) {
        return None;
    }
    let doc = e.body.as_ref()?;
    let placed = growth::placed(e);
    let mut ids: Vec<_> = doc.parts.iter().map(|g| g.id).collect();
    ids.sort_by_key(|id| (doc.depth(*id).unwrap_or(u32::MAX), *id));
    for id in ids {
        let p = e.parts.get(&id)?;
        let cell = p.lost.iter().copied().find(|c| p.losses.contains_key(c));
        let Some(cell) = cell else { continue };
        if !e.lives(id) {
            let g = doc.part(id)?;
            if !g.attachment.is_some_and(|a| e.lives(a.parent))
                || !growth::free(e, &placed, *placed.get(&id)?, g.half_extent)
            {
                continue;
            }
        }
        return Some((id, cell));
    }
    None
}

/// What a landing asks for healing, with descendant revival attributed
/// to the closest living parent that the carriage can reach.
pub(crate) fn asks(e: &Entity, rules: &Rules, lands: &[Key]) -> BTreeMap<PartId, u64> {
    let mut asks = BTreeMap::<PartId, u64>::new();
    let mut planned = e.clone();
    while let Some((id, cell)) = next(&planned) {
        let mut host = id;
        while !e.lives(host) {
            let Some(parent) = e.parent_of(host) else {
                return asks;
            };
            host = parent;
        }
        let bound = |body: &Entity| {
            lands
                .iter()
                .map(|k| {
                    if body.lives(id) {
                        anatomy::bound(body.extent(id), &body.parts[&id], rules, k)
                    } else {
                        0
                    }
                })
                .fold(0, u64::saturating_add)
        };
        let before = bound(&planned);
        let price = anatomy::cell_mass(e.extent(id), rules.body());
        if restore(&mut planned, id, cell).is_err() {
            break;
        }
        let reopened = bound(&planned).saturating_sub(before);
        let w = asks.entry(host).or_default();
        *w = w.saturating_add(price).saturating_add(reopened);
    }
    asks
}

/// Restores one cell exactly as recorded; a free cell stays free.
pub(crate) fn restore(e: &mut Entity, id: PartId, cell: CellId) -> Result<()> {
    let half = e.extent(id);
    let reviving = !e.lives(id);
    let revision = if reviving {
        e.body_revision
            .checked_add(1)
            .ok_or("body revision overflow")?
    } else {
        e.body_revision
    };
    let p = e.part_mut(&id).ok_or("missing healing part")?;
    let allocation = p.losses.remove(&cell).ok_or("unknown lost allocation")?;
    if !p.lost.contains(&cell) {
        return Err("healing a present cell".into());
    }
    p.lost.retain(|c| *c != cell);
    if let Some(function) = allocation {
        p.functions.insert(function.clone());
        *p.cells.entry(function.clone()).or_default() += 1;
        match p.tracts.iter_mut().find(|t| t.function == function) {
            Some(t) => t.cells.push(cell),
            None => p.tracts.push(Tract {
                function,
                cells: vec![cell],
            }),
        }
    }
    crate::mosaic::sync(p, half);
    if reviving
        && !e
            .body
            .as_mut()
            .ok_or("missing healing geometry")?
            .revive(id)
    {
        return Err("a tombstone cannot revive here".into());
    }
    e.body_revision = revision;
    Ok(())
}
