// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Proportions are inherited native recipe choices, using the sheet's paid
//! shapes and branches. Life history biases a choice; it never gates a
//! function. Saved limb bounds use paid templates; leg counts remain fixed.

use super::{CAPITAL, PROVISION_FIRST, kind};
use crate::{
    rules::{Facing, Recipe},
    schema::Key,
};
use std::collections::BTreeSet;

/// Clone the existing paid limb templates, using the same full contract-cell
/// allocation as the native limb palette. Geometry and its bill stay native.
pub(super) fn limbs(
    seed: u64,
    index: u64,
    bounds: [u8; 2],
    kinds: &mut std::collections::BTreeMap<Key, crate::rules::Template>,
    recipe: &mut Recipe,
) -> crate::Result<()> {
    if bounds == [0, 0] {
        return Ok(());
    }
    let scale = bounds[0]
        + (crate::draw(seed, "body-limb-scale", &[index]) % u64::from(bounds[1] - bounds[0] + 1))
            as u8;
    if scale == 1 {
        return Ok(());
    }
    let mut replaced: std::collections::BTreeMap<Key, Key> = std::collections::BTreeMap::new();
    for link in recipe.tagmata.iter_mut().flat_map(|t| &mut t.chain) {
        if let Some(key) = replaced.get(&link.kind) {
            link.kind = key.clone();
            continue;
        }
        let original = kinds.get(&link.kind).ok_or("limb template absent")?;
        // Multi-function authored parts retain their explicit allocation.
        if original.cells.len() != 1 || !original.cells.contains_key("function:contract") {
            continue;
        }
        let mut sized = original.clone();
        sized.half_extent = sized.half_extent.map(|h| h * i32::from(scale));
        sized.cells.insert(
            "function:contract".into(),
            crate::anatomy::capacity(sized.half_extent),
        );
        let key = format!("{}-scale-{scale}", link.kind);
        if kinds.get(&key).is_some_and(|old| old != &sized) {
            return Err("limb size kind collision".into());
        }
        kinds.insert(key.clone(), sized);
        replaced.insert(link.kind.clone(), key.clone());
        link.kind = key;
    }
    Ok(())
}

pub(super) fn drawn(
    seed: u64,
    index: u64,
    producer: bool,
    traits: &BTreeSet<Key>,
    authored: &[(String, bool, Recipe)],
) -> Option<Recipe> {
    let r = |domain: &str, at: usize| crate::draw(seed, domain, &[index, at as u64]);
    // Walking consumers keep the existing paid chains. A palette without
    // chains retains its authored selection rather than inventing limbs.
    let choices: Vec<_> = authored
        .iter()
        .filter(|(_, role, recipe)| {
            *role == producer && (producer || recipe.tagmata.iter().any(|t| !t.chain.is_empty()))
        })
        .collect();
    if choices.is_empty() {
        return None;
    }
    let capital = traits.contains(CAPITAL);
    let provision = traits.contains(PROVISION_FIRST);
    let weight = |name: &str| {
        1 + u64::from(
            (producer && provision && name.ends_with("producer_shrub"))
                || (!producer && capital && name.ends_with("consumer_armoured")),
        )
    };
    let mut pick = r("body-proportion-plan", 0) % choices.iter().map(|c| weight(&c.0)).sum::<u64>();
    let (name, _, source) = choices.into_iter().find(|c| {
        let w = weight(&c.0);
        if pick < w {
            true
        } else {
            pick -= w;
            false
        }
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
            for t in &mut recipe.tagmata {
                t.facing = Facing::Above;
            }
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
