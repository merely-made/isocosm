// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire (working name), an optional founding rule of the bodied A1
//! and stock ecologies: the bridge from the catalog to lineages (lineage-
//! rounds D37, "the part is available to lines through editing and drift;
//! no line is founded to exhibit it"; `mesocosm/design_docs/
//! 2026-11-09_lineage_rounds_plan.md`).
//!
//! The catalog (`crate::catalog`) is read-only data. Under this rule the
//! world lowers its runnable parts that act at the meal into kinds it
//! declares ([`lowering`]): a box from the part's shape, every cell given to
//! the part's process, and an intake port from what the part admits and the
//! tolerance its traits unlock, as `IntakePort` and `process::Tolerance`
//! hold them. **Nothing founds a line with one.** A line reaches them in two
//! ways, both through `lineage::repertoire`: as offers at the played line's
//! review, and as candidates an unplayed line takes at a boundary, scored by
//! the same forked copy. A kind a line takes joins its lexicon, so the
//! regrown body carries the part and its port.
//!
//! The founded feeding processes admit what each line's mouth admitted at
//! founding; under the rule they admit what the bound mouth's port admits
//! (`FeedingSources::All`, living or dead), so a mouth gained later bites
//! what it admits. A generic birth keeps its parent's form, so a revision
//! never reached the young of an A1 line; under the rule the young of a line
//! that has taken a lowered kind are grown from its recipe ([`crate::inherit::rebody`]).
//! With the guard rule (`crate::ecology::Guard`) the armour, spine and gland
//! parts lower too, as organs behind the body, and a mouth carries the
//! catalog's attack by kind. With the mutualism rule (`crate::ecology::Mutualism`)
//! the exchange parts lower as organs whose port gives and takes in a trade.
//! Unset, the world founds, hashes and runs as before.

use crate::{
    Founding, Result,
    ecology::Rates,
    rules::{FeedingSources, Rules},
    schema::Key,
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod lower;
pub use lower::{Lowering, lowering, lowering_for, lowering_in, lowering_with, lowering_all, lowering_full, needs_categories, needs_detritus, needs_guard, needs_mutualism, needs_nutrients};

/// The founding rule. Working names.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repertoire {
    /// The catalog parts to lower; empty lowers every part that can be.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub only: BTreeSet<Key>,
    /// Whether a line may reach parts of another kingdom than its own.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub across_kingdoms: bool,
}

/// One lowered kind, as the world holds it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The catalog part it was lowered from.
    pub part: Key,
    pub kingdom: Key,
    /// The native function its port rests on.
    pub function: Key,
    /// Whether it is a mouth (admits living or dead matter), else an organ
    /// that only tolerates.
    pub mouth: bool,
    /// The kind of the same part holding one cell of reproduce, for a line
    /// whose root is its mouth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<Key>,
    /// The kind this one is the root variant of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_of: Option<Key>,
}

/// What a world lowered, by kind key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lowered {
    pub across_kingdoms: bool,
    pub kinds: BTreeMap<Key, Entry>,
}

impl Lowered {
    /// Whether a line of `kingdom` may reach `kind`.
    #[must_use]
    pub fn reaches(&self, kingdom: &str, kind: &str) -> bool {
        self.kinds.get(kind).is_some_and(|e| self.across_kingdoms || e.kingdom == kingdom)
    }

    pub(crate) fn validate(&self, rules: &Rules) -> Result<()> {
        for (kind, e) in &self.kinds {
            let t = rules.kinds.get(kind).ok_or_else(|| format!("repertoire kind {kind} is not declared"))?;
            if t.port.is_none() {
                return Err(format!("repertoire kind {kind} ({}) has no port", e.part));
            }
        }
        Ok(())
    }
}

/// The kind key a part lowers to.
#[must_use]
pub fn kind_key(part: &str) -> Key {
    format!("kind:catalog-{}", part.trim_start_matches("part:"))
}

/// Lowers the catalog's parts into `g`'s kinds and lets the founded feeds
/// admit what their mouth's port admits. Does nothing where the founding
/// sets no rule.
///
/// # Errors
/// Fails where the founding is no bodied ecology, runs in world time, or
/// `only` names a part that is not in the catalog or cannot be lowered.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = &founding.repertoire else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("repertoire is built for the bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("repertoire layers on the stock and A1 rates, not world time".into());
    }
    let catalog = crate::catalog::catalog();
    let (cats, guard, mutual) = (founding.categories.is_some(), founding.guard.is_some(), founding.mutualism.is_some());
    let rules = (founding.detritus.is_some(), founding.nutrients.is_some());
    for key in &rule.only {
        let part = catalog.part_by_key(key).ok_or_else(|| format!("repertoire names {key}, which the catalog lacks"))?;
        if let Err(why) = lowering_full(part, cats, guard, mutual, rules) {
            return Err(format!("repertoire cannot lower {key}: {why}"));
        }
    }
    let mut kinds: BTreeMap<Key, Entry> = BTreeMap::new();
    for part in catalog.parts().iter().filter(|p| rule.only.is_empty() || rule.only.contains(&p.key)) {
        let Ok(Lowering { template, root }) = lowering_full(part, cats, guard, mutual, rules) else { continue };
        let key = kind_key(&part.key);
        let function = lower::function_of(&template).ok_or("a lowered kind has no function")?;
        let root_key = root.as_ref().map(|_| format!("{key}-root"));
        let entry = Entry {
            part: part.key.clone(),
            kingdom: part.kingdom.clone(),
            function,
            mouth: lower::is_mouth(&template),
            root: root_key.clone(),
            root_of: None,
        };
        let mut declare = |k: &Key, t| match g.rules.kinds.insert(k.clone(), t) {
            Some(_) => Err(format!("repertoire kind {k} is already declared")),
            None => Ok(()),
        };
        declare(&key, template)?;
        if let (Some(t), Some(k)) = (root, root_key) {
            declare(&k, t)?;
            kinds.insert(k, Entry { root: None, root_of: Some(key.clone()), ..entry.clone() });
        }
        kinds.insert(key, entry);
    }
    g.rules.repertoire = Some(Lowered { across_kingdoms: rule.across_kingdoms, kinds });
    widen(&mut g.rules);
    Ok(())
}

/// Each founded ordinary feed admits what its bound mouth's port admits, of
/// every line, living or dead, rather than the lines the founders' mouth
/// admitted: the same meal for a founded body, and a meal for a mouth a line
/// gains later.
fn widen(rules: &mut Rules) {
    for p in rules.processes.iter_mut().filter(|(id, _)| id.starts_with("ecology:feed-")).map(|(_, p)| p) {
        let Some(f) = p.port_feeding.as_mut().filter(|f| f.meal.is_none() && matches!(f.sources, FeedingSources::Only(_))) else {
            continue;
        };
        f.sources = FeedingSources::All;
        if let Some(target) = &mut p.target {
            target.alive = None;
        }
    }
}

#[cfg(test)]
mod tests;
