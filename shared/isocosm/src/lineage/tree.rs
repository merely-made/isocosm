// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Descent between lineages, read off each line's parent (legacy
//! `species::Lineages`). Nothing removes a lineage, so an extinct line
//! keeps its place in the tree.

use crate::schema::{Key, Lineage};
use std::collections::{BTreeMap, BTreeSet};

/// `id` and everything it descends from, nearest first.
pub fn ancestry(lines: &BTreeMap<Key, Lineage>, id: &str) -> Vec<Key> {
    let mut line = vec![];
    let mut seen = BTreeSet::new();
    let mut at = lines.contains_key(id).then(|| id.to_string());
    // A decoded state could hold a cycle; stop at the first repeat.
    while let Some(k) = at.filter(|k| seen.insert(k.clone())) {
        at = lines.get(&k).and_then(|l| l.parent.clone());
        line.push(k);
    }
    line
}

/// The nearest lineage both descend from.
pub fn common_ancestor(lines: &BTreeMap<Key, Lineage>, a: &str, b: &str) -> Option<Key> {
    let theirs: BTreeSet<Key> = ancestry(lines, b).into_iter().collect();
    ancestry(lines, a).into_iter().find(|k| theirs.contains(k))
}

/// Forks since two lines diverged: the longer walk to their common
/// ancestor. `None` when they share none, as two founding lines do.
pub fn distance(lines: &BTreeMap<Key, Lineage>, a: &str, b: &str) -> Option<u32> {
    let shared = common_ancestor(lines, a, b)?;
    let legs = |from: &str| {
        let steps = ancestry(lines, from).iter().position(|k| *k == shared);
        steps.map(|n| n as u32)
    };
    Some(legs(a)?.max(legs(b)?))
}

/// Whether `id` descends from `ancestor`, never from itself.
pub fn descends_from(lines: &BTreeMap<Key, Lineage>, id: &str, ancestor: &str) -> bool {
    id != ancestor && ancestry(lines, id).iter().any(|k| k == ancestor)
}

/// The lines split off `id`, in key order.
pub fn children(lines: &BTreeMap<Key, Lineage>, id: &str) -> Vec<Key> {
    let of = |l: &Lineage| l.parent.as_deref() == Some(id);
    lines
        .iter()
        .filter(|(_, l)| of(l))
        .map(|(k, _)| k.clone())
        .collect()
}
