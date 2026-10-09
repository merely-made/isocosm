// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body developed from its lineage's recipe (rulings 478, 510, 511 and
//! 531), as Mesocosm develops one. A child draws its soma by its seed: each
//! tagma's segment count drifts within its variance, and a segment's borne
//! kind may develop absent at the recipe's odds, never a kind that feeds,
//! senses or fixes. The soma develops into parts: segments flush one behind
//! another along their tagma's run, borne kinds flush on the segment at the
//! tagma's socket, a lateral socket mirrored where the plan is bilateral.
//! What the parts hold is the birth's to give.

use crate::{
    Result,
    rules::{Anchor, Development, Facing, Recipe, Rules, Template},
    schema::*,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a child drew from its recipe: each tagma's segment count, and the
/// segments, by tagma and index, whose borne kind is absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soma {
    pub segments: Vec<u8>,
    pub absent: Vec<(u8, u8)>,
    /// The seed it was drawn by, which the child's riff and its varied
    /// cell draw by too (573, 578).
    #[serde(default)]
    pub seed: u64,
}

/// Kinds a child never lacks (531's reading): those that feed, sense or fix.
const NEVER_ABSENT: [&str; 3] = ["function:intake", "function:sense", "function:fix"];

fn present(t: &Template) -> bool {
    NEVER_ABSENT
        .iter()
        .any(|f| t.cells.get(*f).copied().unwrap_or(0) > 0)
}

/// The soma `seed` draws from `recipe`.
pub fn soma(rules: &Rules, recipe: &Recipe, seed: u64) -> Soma {
    let draw = |domain: &str, i: usize| crate::draw(seed, domain, &[i as u64]);
    let mut segments = vec![];
    let mut absent = vec![];
    for (i, t) in recipe.tagmata.iter().enumerate() {
        let spread = i32::from(t.variance.unwrap_or(recipe.variance));
        let drift = match spread {
            0 => 0,
            s => (draw("soma-drift", i) % (2 * s as u64 + 1)) as i32 - s,
        };
        let count = (i32::from(t.segments) + drift).clamp(1, 255) as u8;
        segments.push(count);
        let [odds, of] = recipe.absence;
        let borne = t.bears.as_ref().and_then(|k| rules.kinds.get(k));
        if t.per_segment == 0 || borne.is_none_or(present) || odds == 0 {
            continue;
        }
        if draw("soma-absent", i) % u64::from(of) < u64::from(odds) {
            let at = draw("soma-absent-at", i) % u64::from(count);
            absent.push((i as u8, at as u8));
        }
    }
    Soma {
        segments,
        absent,
        seed,
    }
}

/// A part of `t` at `situs`, attached to `parent` at `offset`, holding
/// nothing.
fn part(t: &Template, situs: [u8; 3], parent: Option<Id>, offset: [i32; 3]) -> Part {
    Part {
        parent,
        half_extent: t.half_extent,
        offset,
        situs: Some(situs),
        cells: t
            .cells
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(f, n)| (f.clone(), *n))
            .collect(),
        functions: t
            .cells
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(f, _)| f.clone())
            .collect(),
        shape: t.shape.clone(),
        ..Default::default()
    }
}

/// The pivot-to-pivot offset that sets a child of half-extent `child`
/// flush against a parent of `parent` along `facing`.
pub fn flush(parent: [i32; 3], child: [i32; 3], facing: Facing) -> [i32; 3] {
    let (axis, sign) = facing.axis();
    let mut offset = [0; 3];
    offset[axis] = sign * (parent[axis].abs() + child[axis].abs());
    offset
}

/// The parts `soma` develops into, root first, holding nothing.
pub fn develop(rules: &Rules, d: &Development, soma: &Soma) -> Result<BTreeMap<Id, Part>> {
    let recipe = &d.recipe;
    if soma.segments.len() != recipe.tagmata.len() {
        return Err("a soma drawn from another recipe".into());
    }
    let kind = |k: &Key| rules.kinds.get(k).ok_or(format!("unknown kind {k}"));
    let mut parts: BTreeMap<Id, Part> = BTreeMap::new();
    // Each tagma's segments, by part id, for later tagmata to branch from.
    let mut spines: Vec<Vec<Id>> = vec![];
    let mut last: Option<Id> = None;
    for (i, t) in recipe.tagmata.iter().enumerate() {
        let segment = kind(&t.segment)?;
        let mut spine = vec![];
        for s in 0..soma.segments[i] {
            let parent = match (spine.last(), t.parent) {
                (Some(prev), _) => Some(*prev),
                (None, Some(p)) => {
                    let from: &Vec<Id> = &spines[usize::from(p)];
                    Some(match t.anchor {
                        Anchor::Base => from[0],
                        Anchor::Middle => from[from.len() / 2],
                        Anchor::Tip => from[from.len() - 1],
                    })
                },
                (None, None) => last,
            };
            let offset = match parent {
                Some(p) => flush(parts[&p].half_extent, segment.half_extent, t.facing),
                None => [0; 3],
            };
            let id = parts.len() as Id;
            parts.insert(id, part(segment, [i as u8, s, 0], parent, offset));
            spine.push(id);
            last = Some(id);
            let Some(bears) = &t.bears else { continue };
            if soma.absent.contains(&(i as u8, s)) {
                continue;
            }
            let borne = kind(bears)?;
            let sockets = match d.policy.mirrors(t.socket) {
                true => vec![t.socket, t.socket.mirrored()],
                false => vec![t.socket],
            };
            let (run, _) = t.facing.axis();
            let mut slot = 0u8;
            for ordinal in 0..t.per_segment {
                let along = (2 * i32::from(ordinal) + 1 - i32::from(t.per_segment))
                    * borne.half_extent[run].abs().max(1);
                for socket in &sockets {
                    let mut offset = flush(segment.half_extent, borne.half_extent, *socket);
                    offset[run] += along;
                    let at = parts.len() as Id;
                    slot = slot.checked_add(1).ok_or("too many borne parts")?;
                    parts.insert(at, part(borne, [i as u8, s, slot], Some(id), offset));
                }
            }
        }
        spines.push(spine);
    }
    Ok(parts)
}
