// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Growth toward a lineage's recipe (rulings 478, 479, 485, 495 and 510):
//! the next part a body lacks, nearest the root first, where it goes, and
//! what expressing it costs. A part grows where development would have put
//! it, or failing that at the first free box its plan's facings find on the
//! same parent, as isometer resolves one.

use crate::{
    Result, anatomy,
    development::{Soma, develop, flush},
    rules::{Development, Policy, Rules},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

/// Each part's pivot in its root's frame.
pub fn placed(e: &Entity) -> BTreeMap<Id, [i32; 3]> {
    let mut at: BTreeMap<Id, [i32; 3]> = BTreeMap::new();
    for &id in e.parts.keys() {
        // Walk up to a part already placed or the root, then back down.
        let mut chain = vec![id];
        while let Some(parent) = e.parts[chain.last().unwrap()].parent {
            if at.contains_key(&parent) || chain.contains(&parent) || !e.parts.contains_key(&parent)
            {
                break;
            }
            chain.push(parent);
        }
        for id in chain.into_iter().rev() {
            let p = &e.parts[&id];
            let base = p.parent.and_then(|q| at.get(&q).copied()).unwrap_or([0; 3]);
            at.insert(id, [0, 1, 2].map(|i| base[i] + p.offset[i]));
        }
    }
    at
}

/// Whether a box of `half` at `at` overlaps none of the body's living
/// parts, touching not counted (isometer's test).
pub fn free(e: &Entity, placed: &BTreeMap<Id, [i32; 3]>, at: [i32; 3], half: [i32; 3]) -> bool {
    e.parts.iter().filter(|(_, p)| !p.severed).all(|(id, p)| {
        let there = placed[id];
        (0..3).any(|i| (at[i] - there[i]).abs() >= half[i].abs() + p.half_extent[i].abs())
    })
}

/// Where on `parent` a box of `half` goes: `preferred` if free, else the
/// first free facing `policy` tries for its name, flush on the parent.
pub fn seat(
    e: &Entity,
    policy: &Policy,
    parent: Id,
    half: [i32; 3],
    preferred: Option<[i32; 3]>,
) -> Option<[i32; 3]> {
    let placed = placed(e);
    let base = placed.get(&parent).copied()?;
    let host = e.parts.get(&parent)?.half_extent;
    let tried = preferred.into_iter();
    let facings = policy.candidates(anatomy::boxed(half));
    let flushed = facings.into_iter().map(|f| flush(host, half, f));
    tried.chain(flushed).find(|offset| {
        let at = [0, 1, 2].map(|i| base[i] + offset[i]);
        free(e, &placed, at, half)
    })
}

/// Where a part of `half` taken in lands (rulings 510 and 516), as isometer
/// resolves one: for each facing its plan tries for the part's name, each
/// living part in order, the first flush box overlapping nothing. A part
/// taken in lands alone, its mirror not sought.
pub fn resolve(e: &Entity, policy: &Policy, half: [i32; 3]) -> Option<(Id, [i32; 3])> {
    let placed = placed(e);
    for facing in policy.candidates(anatomy::boxed(half)) {
        for (id, host) in e.parts.iter().filter(|(_, p)| !p.severed) {
            let offset = flush(host.half_extent, half, facing);
            let at = [0, 1, 2].map(|i| placed[id][i] + offset[i]);
            if free(e, &placed, at, half) {
                return Some((*id, offset));
            }
        }
    }
    None
}

/// The segments a body grows toward (495): an anamorphic lineage's recipe
/// counts, a body keeping any it drew beyond them; otherwise those it drew.
fn target(d: &Development, e: &Entity) -> Vec<u8> {
    let recipe = d.recipe.tagmata.iter().map(|t| t.segments);
    let drawn = |i: usize| e.soma.get(i).copied();
    recipe
        .enumerate()
        .map(|(i, r)| match (d.anamorphic, drawn(i)) {
            (true, Some(s)) => s.max(r),
            (false, Some(s)) => s,
            (_, None) => r,
        })
        .collect()
}

/// The next part the body lacks (rulings 478 and 479): of the parts its
/// recipe develops at the counts it grows toward, every borne kind present
/// (an absent limb grows in later), the first in development's order whose
/// situs no part of the body holds, severed or not (regrowing what was cut
/// waits on healing, 485), whose parent it holds alive and which has a free
/// seat. Returns it ready to attach.
pub fn lacking(rules: &Rules, d: &Development, e: &Entity) -> Result<Option<Part>> {
    let soma = Soma {
        segments: target(d, e),
        absent: vec![],
    };
    let ideal = develop(rules, d, &soma)?;
    let held: BTreeSet<[u8; 3]> = e.parts.values().filter_map(|p| p.situs).collect();
    let alive: BTreeMap<[u8; 3], Id> = e
        .parts
        .iter()
        .filter(|(_, p)| !p.severed)
        .filter_map(|(id, p)| p.situs.map(|s| (s, *id)))
        .collect();
    for part in ideal.values() {
        let Some(situs) = part.situs else { continue };
        if held.contains(&situs) {
            continue;
        }
        let parent_situs = part.parent.and_then(|q| ideal[&q].situs);
        let Some(parent) = parent_situs.and_then(|s| alive.get(&s).copied()) else {
            continue;
        };
        let Some(offset) = seat(e, &d.policy, parent, part.half_extent, Some(part.offset)) else {
            continue;
        };
        return Ok(Some(Part {
            parent: Some(parent),
            offset,
            ..part.clone()
        }));
    }
    Ok(None)
}

/// The adult mass of the parts `d`'s recipe develops, at the counts the
/// body grows toward, whose situs the body does not hold.
pub fn lacking_mass(rules: &Rules, d: &Development, e: &Entity) -> u64 {
    let soma = Soma {
        segments: target(d, e),
        absent: vec![],
    };
    let Ok(ideal) = develop(rules, d, &soma) else {
        return 0;
    };
    let held: BTreeSet<[u8; 3]> = e.parts.values().filter_map(|p| p.situs).collect();
    let b = rules.body();
    ideal
        .values()
        .filter(|p| p.situs.is_some_and(|s| !held.contains(&s)))
        .map(|p| anatomy::ceiling(p, b))
        .fold(0, u64::saturating_add)
}

/// What expressing a new part's functions costs (PD2): one cell's mass for
/// each cell it expresses.
pub fn price(rules: &Rules, p: &Part) -> u64 {
    let cells: u64 = p.cells.values().map(|c| u64::from(*c)).sum();
    cells.saturating_mul(anatomy::cell_mass(p, rules.body()))
}
