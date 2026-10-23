// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Proportions are inherited native recipe choices, using the sheet's paid
//! shapes and branches. Life history biases a choice; it never gates a
//! function. Feet retain their declared chain lengths and fixed leg counts.

use super::{CAPITAL, PROVISION_FIRST, kind};
use crate::{rules::{Facing, Recipe}, schema::Key};
use std::collections::BTreeSet;

pub(super) fn drawn(
    seed: u64, index: u64, producer: bool, traits: &BTreeSet<Key>,
    authored: &[(String, bool, Recipe)],
) -> Option<Recipe> {
    let r = |domain: &str, at: usize| crate::draw(seed, domain, &[index, at as u64]);
    // Walking consumers keep the existing paid chains. A palette without
    // chains retains its authored selection rather than inventing limbs.
    let choices: Vec<_> = authored.iter().filter(|(_, role, recipe)|
        *role == producer && (producer || recipe.tagmata.iter().any(|t| !t.chain.is_empty())))
        .collect();
    if choices.is_empty() { return None; }
    let capital = traits.contains(CAPITAL);
    let provision = traits.contains(PROVISION_FIRST);
    let weight = |name: &str| 1 + u64::from(
        (producer && provision && name.ends_with("producer_shrub")) ||
        (!producer && capital && name.ends_with("consumer_armoured")));
    let mut pick = r("body-proportion-plan", 0) % choices.iter().map(|c|weight(&c.0)).sum::<u64>();
    let (name, _, source) = choices.into_iter().find(|c| {
        let w = weight(&c.0);
        if pick < w { true } else { pick -= w; false }
    })?;
    let mut recipe = source.clone();
    if producer {
        if name.ends_with("producer_mat") {
            // A low collecting patch, not thirteen repeated tiles in a row.
            recipe.tagmata[0].segments = 1;
            let leaves = &mut recipe.tagmata[1];
            leaves.segments = 2 + (r("body-patch-width", 0) % 3) as u8;
            leaves.segment = kind(if capital { "block" } else { "broad" });
            leaves.per_segment = 1;
        } else if name.ends_with("producer_stalk") {
            // The old stalk predates branching. Its declared growth axis
            // can now actually point up, with the same stem and leaf kinds.
            for t in &mut recipe.tagmata { t.facing = Facing::Above; }
            recipe.tagmata[0].segments = 1;
            recipe.tagmata[1].segments = 2 + (r("body-stalk-height", 0) % 3) as u8;
            recipe.tagmata[1].segment = kind(if capital { "broad" } else { "slim" });
            recipe.tagmata[2].segments = 1 + u8::from(provision);
        } else if name.ends_with("producer_shrub") {
            recipe.tagmata[0].segments = 2 + (r("body-trunk-height", 0) % 2) as u8;
            for (i, t) in recipe.tagmata.iter_mut().enumerate().skip(1) {
                if t.bears.is_none() {
                    t.segments = 1 + (r("body-branch-reach", i) % 2) as u8;
                }
            }
        }
        // Structural variance is only on stems, never on fixed leaf sites.
        for t in &mut recipe.tagmata {
            t.variance = Some(u8::from(t.bears.is_none() && t.segments > 1));
        }
    } else {
        for (i, t) in recipe.tagmata.iter_mut().enumerate() {
            if !t.chain.is_empty() {
                // More capital gives a deeper thorax, paid at native volume.
                t.segment = kind(if capital { "block" } else { "broad" });
            } else if t.socket == Facing::Right && t.bears.as_deref() == Some("kind:shell") {
                t.segments = 3 + (r("body-shell-length", i) % 2) as u8;
            } else if t.bears.is_none() && t.segments > 1 {
                if t.facing == Facing::Below {
                    t.segments = 1;
                } else if t.variance == Some(1) {
                    t.segments = 1 + (r("body-tail-length", i) % 3) as u8;
                } else {
                    // At least two slim segments separate foot stations:
                    // shortening the trunk must not overlap paid foot pads.
                    t.segments = 2 + (r("body-backbone-length", i) % 2) as u8;
                }
            }
            // Fixed foot stations remain fixed across offspring soma draws.
            t.variance = Some(0);
        }
    }
    Some(recipe)
}
