// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body carries, and how a child's systems and cells vary (rulings
//! 568, 573, 574, 576 to 580, 583, 584 and 587). A founded body carries the
//! world's defaults it realizes; cells a development allocates bring the
//! defaults they realize; a child carries its parent's systems, those its
//! body cannot realize dormant, and its parent's varied cells where its
//! parts lie. At its recipe's odds one of its cells takes another grown
//! function, and then, at the recipe's riff odds, one role of one system
//! swaps or adds a function its body expresses, kept only if the system is
//! still realized.

use super::realizes;
use crate::{
    rules::{Fill, Recipe, Role, Rules, Seeding, System},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

/// The world's defaults `e` realizes: what a founded body carries (574).
pub fn founded(e: &Entity, rules: &Rules) -> BTreeMap<Key, System> {
    let realized = rules.systems.iter().filter(|(_, s)| realizes(e, s));
    realized.map(|(k, s)| (k.clone(), s.clone())).collect()
}

/// The defaults `e` lacks that a development's allocation made it realize,
/// as it stood `before`: what the cells allocated bring (587).
pub fn take_up(e: &mut Entity, rules: &Rules, before: &Entity) {
    for (k, s) in &rules.systems {
        if !e.systems.contains_key(k) && realizes(e, s) && !realizes(before, s) {
            e.systems.insert(k.clone(), s.clone());
        }
    }
}

/// One cell of `p` moved from `from` to `to`; false where it holds none.
fn shift(p: &mut Part, from: &str, to: &str) -> bool {
    let Some(n) = p.cells.get_mut(from).filter(|n| **n > 0) else {
        return false;
    };
    *n -= 1;
    if *n == 0 {
        p.cells.remove(from);
        p.functions.remove(from);
    }
    *p.cells.entry(to.into()).or_default() += 1;
    p.functions.insert(to.into());
    true
}

/// The varied cells down a line, applied to the part where each lies
/// (577), in the order they arose.
pub fn inherit(e: &mut Entity) {
    for v in e.varied.clone() {
        let at = e
            .parts
            .keys()
            .copied()
            .find(|id| e.situs(*id) == Some(v.situs));
        if let Some(p) = at.and_then(|id| e.parts.get_mut(&id)) {
            shift(p, &v.from, &v.to);
        }
    }
}

/// The varied cells of `varied` lying in a part `part` grows as, at
/// `situs`, applied.
pub fn regrow(part: &mut Part, situs: Option<[u8; 3]>, varied: &[Varied]) {
    for v in varied.iter().filter(|v| situs == Some(v.situs)) {
        shift(part, &v.from, &v.to);
    }
}

fn met([odds, of]: [u32; 2], draw: u64) -> bool {
    odds > 0 && draw % u64::from(of.max(1)) < u64::from(odds)
}

fn pick<T: Clone>(from: &[T], draw: u64) -> Option<T> {
    match from.len() {
        0 => None,
        n => Some(from[(draw % n as u64) as usize].clone()),
    }
}

/// A child's own variation, by its soma's seed: at its recipe's odds, one
/// of its cells, drawn uniformly, takes another grown function, drawn
/// uniformly (578, 579, 584). Returns the cell where one varied.
pub fn vary(child: &mut Entity, rules: &Rules, recipe: &Recipe, seed: u64) -> Option<Varied> {
    let draw = |domain: &str| crate::draw(seed, domain, &[]);
    if !met(recipe.vary, draw("vary")) {
        return None;
    }
    let cells: Vec<(PartId, Key)> = child
        .living()
        .flat_map(|(id, p)| {
            p.cells
                .iter()
                .flat_map(move |(f, n)| std::iter::repeat_n((id, f.clone()), *n as usize))
        })
        .collect();
    let (id, from) = pick(&cells, draw("vary-cell"))?;
    let grown: Vec<Key> = rules
        .functions
        .iter()
        .filter(|(k, f)| f.seeding == Seeding::Grown && **k != from)
        .map(|(k, _)| k.clone())
        .collect();
    let to = pick(&grown, draw("vary-to"))?;
    let situs = child.situs(id)?;
    let part = child.parts.get_mut(&id)?;
    shift(part, &from, &to);
    let v = Varied { situs, from, to };
    child.varied.push(v.clone());
    Some(v)
}

/// A riff, as a child's systems vary (568, 573, 580).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Riff {
    pub system: Key,
    pub role: Role,
    pub swapped: Option<Fill>,
    pub added: Key,
}

/// A child's riff, by its soma's seed: at its recipe's odds, one system it
/// carries and one role, drawn uniformly, take a function its body
/// expresses that the role does not name, drawn uniformly, swapped for one
/// of the role's fills or added at even odds, a swap into an empty role an
/// addition. Kept only if the child still realizes the system (491).
pub fn riff(child: &mut Entity, recipe: &Recipe, seed: u64) -> Option<Riff> {
    let draw = |domain: &str| crate::draw(seed, domain, &[]);
    if !met(recipe.riff, draw("riff")) {
        return None;
    }
    let keys: Vec<Key> = child.systems.keys().cloned().collect();
    let system = pick(&keys, draw("riff-system"))?;
    let role = pick(&Role::ALL, draw("riff-role"))?;
    let swap = draw("riff-swap") % 2 == 0;
    let mut riffed = child.systems[&system].clone();
    let expressed: BTreeSet<&Key> = child.living().flat_map(|(_, p)| &p.functions).collect();
    let open: Vec<Key> = expressed
        .into_iter()
        .filter(|f| !riffed.routes(f, role))
        .cloned()
        .collect();
    let added = pick(&open, draw("riff-function"))?;
    let fills = riffed.role_mut(role);
    let listed: Vec<Fill> = fills.iter().cloned().collect();
    let swapped = match swap {
        true => pick(&listed, draw("riff-out")),
        false => None,
    };
    if let Some(out) = &swapped {
        fills.remove(out);
    }
    fills.insert(Fill::Function(added.clone()));
    if !realizes(child, &riffed) {
        return None;
    }
    child.systems.insert(system.clone(), riffed);
    Some(Riff {
        system,
        role,
        swapped,
        added,
    })
}
