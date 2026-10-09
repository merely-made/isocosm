// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The nervous system's joining (rulings 657 and 659 to 664). It carries no
//! matter; a process reading two systems gets their joint effect only as far
//! as a sense's routes reach both. A route runs from a living part
//! expressing a sense along attachments, through living parts that can
//! carry (477, a cut route's rule), and the degree is the second system's
//! parts so reached, by their cells, of all of them. Without a carried
//! system sourcing the senses, nothing joins and each system works alone.

use super::{capacity, filling, union};
use crate::{
    rules::{Role, Rules},
    schema::*,
};
use std::collections::BTreeSet;

const SENSE: &str = "function:sense";

/// The living parts filling `role` in the systems naming `function` there.
fn filled(e: &Entity, (function, role): (&str, Role), bitten: Option<Id>) -> BTreeSet<Id> {
    union(e, function, role).map_or_else(BTreeSet::new, |u| filling(e, u.role(role), bitten))
}

/// The parts a route from `from` reaches.
fn reach(e: &Entity, rules: &Rules, from: Id) -> BTreeSet<Id> {
    let open = |id: &Id| {
        e.parts
            .get(id)
            .is_some_and(|p| !p.severed && capacity(p, rules) > 0)
    };
    let mut seen = BTreeSet::new();
    if !open(&from) {
        return seen;
    }
    seen.insert(from);
    let mut queue = vec![from];
    while let Some(at) = queue.pop() {
        let parent = e.parts[&at].parent;
        let children = e.parts.iter().filter(|(_, p)| p.parent == Some(at));
        for next in parent.into_iter().chain(children.map(|(id, _)| *id)) {
            if open(&next) && seen.insert(next) {
                queue.push(next);
            }
        }
    }
    seen
}

/// How far `e`'s senses join `first` to `second`: the cells of `second`'s
/// parts reached from a sense that also reaches one of `first`'s, and the
/// cells of all of `second`'s parts.
pub fn joined(
    e: &Entity,
    rules: &Rules,
    first: (&str, Role),
    second: (&str, Role),
    bitten: Option<Id>,
) -> (u64, u64) {
    let cells = |id: &Id| {
        e.parts[id]
            .cells
            .values()
            .map(|n| u64::from(*n))
            .sum::<u64>()
    };
    let seconds = filled(e, second, bitten);
    let total = seconds.iter().map(cells).sum();
    if union(e, SENSE, Role::Source).is_none() {
        return (0, total);
    }
    let firsts = filled(e, first, bitten);
    let senses = e
        .parts
        .iter()
        .filter(|(_, p)| !p.severed && p.functions.contains(SENSE));
    let mut reached = BTreeSet::new();
    for (id, _) in senses {
        let from = reach(e, rules, *id);
        if firsts.iter().any(|f| from.contains(f)) {
            reached.extend(from);
        }
    }
    let joined = seconds.iter().filter(|id| reached.contains(*id)).map(cells);
    (joined.sum(), total)
}
