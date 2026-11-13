// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Trait x kingdom x effect (lineage-rounds D37): every row of the product
//! that is not nonsense becomes a part.
//!
//! Deterministic by construction: no randomness, and the loops run in the
//! datasheets' order (traits, then effects, then [`KINGDOMS`]), so the same
//! sheets always give the same parts in the same order.
//!
//! The filter says what nonsense is, each clause a rule a row states:
//! - the effect's site must be one the trait can carry a payload at;
//! - the trait's category must be one the effect accepts, and, where the
//!   effect names processes, the trait's process must be one of them (a
//!   grinding effect rides on an intake, not on a leaf);
//! - where the effect lists `only` traits, the trait must be one;
//! - the kingdom must be one the trait lists and the effect allows;
//! - the two must serve a model in common (the part serves the overlap).
//!
//! A row whose fingerprint (`Part::fingerprint`; the memo's section 2, test
//! 4) matches an authored part or an earlier generated one is dropped, not
//! renamed: two names for one thing.

use std::collections::{BTreeMap, BTreeSet};

use super::refs::{KINGDOMS, process_name};
use super::{Effect, Model, Origin, Part, Status, Trait};

/// What the generator did not emit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dropped {
    /// Pairs of trait, effect and kingdom looked at.
    pub considered: usize,
    /// Refused by the nonsense filter.
    pub filtered: usize,
    /// Refused because the fingerprint was already held.
    pub duplicate: usize,
}

fn sorted_union(a: &[String], b: &[String]) -> Vec<String> {
    a.iter().chain(b).cloned().collect::<BTreeSet<_>>().into_iter().collect()
}

fn short(key: &str) -> &str {
    key.split_once(':').map_or(key, |(_, name)| name)
}

/// Whether the filter allows `effect` to ride on `t` in `kingdom`.
pub(super) fn sensible(t: &Trait, effect: &Effect, kingdom: &str) -> bool {
    t.kingdoms.iter().any(|k| k == kingdom)
        && (effect.kingdoms.is_empty() || effect.kingdoms.iter().any(|k| k == kingdom))
        && (effect.only.is_empty() || effect.only.contains(&t.key))
        && t.sites.contains(&effect.site)
        && effect.accepts.contains(&t.category)
        && (effect.processes.is_empty()
            || effect.processes.iter().any(|p| p == process_name(&t.process)))
        && t.models.iter().any(|m| effect.models.contains(m))
}

/// Generates the product's parts, skipping any whose fingerprint an
/// `authored` part already holds.
pub fn generate(traits: &[Trait], effects: &[Effect], authored: &[Part]) -> (Vec<Part>, Dropped) {
    let mut seen: BTreeSet<String> = authored.iter().map(Part::fingerprint).collect();
    let mut parts = Vec::new();
    let mut dropped = Dropped::default();
    for t in traits {
        for effect in effects {
            for kingdom in KINGDOMS {
                if !t.kingdoms.iter().any(|k| k == kingdom) {
                    continue;
                }
                dropped.considered += 1;
                if !sensible(t, effect, kingdom) {
                    dropped.filtered += 1;
                    continue;
                }
                let models: Vec<Model> =
                    t.models.iter().copied().filter(|m| effect.models.contains(m)).collect();
                let part = Part {
                    key: format!(
                        "part:{}-{}-{}",
                        short(kingdom),
                        short(&t.key),
                        short(&effect.key)
                    ),
                    kingdom: kingdom.to_string(),
                    process: t.process.clone(),
                    shape: t.shape.clone(),
                    traits: vec![t.key.clone()],
                    effects: BTreeMap::from([(effect.site, vec![effect.key.clone()])]),
                    admits: sorted_union(&t.admits, &effect.admits),
                    emits: sorted_union(&t.emits, &effect.emits),
                    cost_mg: t.cost_mg + effect.cost_mg,
                    models,
                    analogue: format!("{}; {}", t.analogue, effect.analogue),
                    status: Status::Proposed { process: String::new() },
                    waits_on: Vec::new(),
                    origin: Origin::Generated,
                };
                if seen.insert(part.fingerprint()) {
                    parts.push(part);
                } else {
                    dropped.duplicate += 1;
                }
            }
        }
    }
    (parts, dropped)
}
