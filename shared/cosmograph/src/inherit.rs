// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Young grow from the line's recipe (lineage-rounds D40, 2026-11-13), an
//! optional founding rule: `Founding.inherit`, carried on `Rules.inherit`.
//!
//! A generic birth clones its parent (821), so a revision a line takes at a
//! boundary, or the drift of its recipe, never reached the young of an A1
//! line: the parent kept the new body and the young kept the old. Under the
//! rule every generic birth of a line that has a development grows the young
//! from the line's current recipe instead, so revisions and drift pass down.
//! The young keep their parent's variation (its soma) where it still fits
//! the recipe and nothing is drawn: the grouped execution draws nothing, and
//! both executions clone the parent first and so hatch the same body. Keeping
//! it is what lets drift, which averages the members' bodies, still move. The
//! matter is untouched: the clone's matter is cleared before, and the child's
//! tissue is only its parent's provision moved (TD6).
//! The repertoire rule implies it for the lines that took a lowered kind.
//! Unset, the world founds, hashes and runs as before, and a save without it
//! keeps copying.

use crate::{
    Founding, Result, development::{self, Soma},
    rules::Rules,
    schema::{Entity, Lineage},
    simulation::Genesis,
};

/// Lays the rule on the founded world's rules. Does nothing where the founding
/// sets none.
///
/// # Errors
/// Fails where the founding is no ecology.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    if !founding.inherit {
        return Ok(());
    }
    if !founding.ecology {
        return Err("inherit is built for an ecology".into());
    }
    g.rules.inherit = true;
    Ok(())
}

/// The soma a child keeps: its parent's, which the clone carries, wherever it
/// still fits the recipe, else the recipe's own segments. Tagmata match by
/// position, so a tagma the recipe gained or lost leaves the others matched.
/// A tagma keeps the parent's segment count where it lies in the range the
/// recipe could draw (its segments plus or minus its variance, 1 to 255),
/// else takes the recipe's own, so a revision past the line's variance
/// reaches the young at once; an absence is kept where its tagma is kept, the
/// segment is still there and the tagma may still lack it. Drawn by nothing.
fn kept(rules: &Rules, d: &crate::rules::Development, parent: &crate::schema::Entity) -> Soma {
    let recipe = &d.recipe;
    let mut keep = vec![];
    let segments = recipe.tagmata.iter().enumerate().map(|(i, t)| {
        let spread = i32::from(t.variance.unwrap_or(recipe.variance));
        let own = i32::from(t.segments);
        let (lo, hi) = ((own - spread).clamp(1, 255), (own + spread).clamp(1, 255));
        match parent.soma.get(i).copied().filter(|n| (lo..=hi).contains(&i32::from(*n))) {
            Some(n) => { keep.push(i as u8); n },
            None => t.segments,
        }
    }).collect::<Vec<u8>>();
    let absent = parent.absent.iter().copied()
        .filter(|(i, at)| keep.contains(i) && *at < segments[usize::from(*i)] && development::may_lack(rules, &recipe.tagmata[usize::from(*i)]))
        .collect();
    Soma { segments, absent, seed: 0 }
}

/// Grows `child` anew from its line's recipe where the world's rule is set,
/// or the repertoire's is and the line has taken a lowered kind, keeping its
/// parent's variation where it still fits ([`kept`]). The clone already holds
/// the parent's soma, so both executions agree without a draw. Otherwise the
/// child keeps its parent's form as before.
///
/// # Errors
/// Fails if the line's development cannot grow a body.
pub(crate) fn rebody(rules: &Rules, line: Option<&Lineage>, child: &mut Entity) -> Result<()> {
    let lowered = rules.repertoire.as_ref();
    let Some(d) = line
        .and_then(|l| l.development.as_ref())
        .filter(|d| rules.inherit || lowered.is_some_and(|l| d.recipe.kinds().iter().any(|k| l.kinds.contains_key(k))))
    else {
        return Ok(());
    };
    let soma = kept(rules, d, child);
    child.embody(development::develop(rules, d, &soma)?);
    child.soma = soma.segments;
    child.absent = soma.absent;
    Ok(())
}
