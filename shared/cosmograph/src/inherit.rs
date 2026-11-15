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
//! Young keep their parent's variation where it fits the recipe. A world
//! with inherited variation can also change a body region at a birth, keyed
//! by child identity. The child's tissue comes from its parent's provision.
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
    g.rules.inherited_variation = founding.inherited_variation;
    Ok(())
}

/// Keep parental counts within the current recipe's range. When a structural
/// rule can renumber regions, start from that recipe's centre.
fn kept(rules: &Rules, d: &crate::rules::Development, parent: &Entity) -> Soma {
    let recipe = &d.recipe;
    let same_layout = rules.inherited_variation == 0
        || (parent.soma.len() == recipe.tagmata.len()
            && d.grammar.as_ref().is_none_or(|g| g.rules.iter().all(|rule| {
                matches!(rule, crate::rules::Rule::Repeat { .. } | crate::rules::Rule::Allocate { .. })
            })));
    let mut segments = Vec::with_capacity(recipe.tagmata.len());
    let mut retained = [false; 256];
    for (i, t) in recipe.tagmata.iter().enumerate() {
        let spread = i32::from(t.variance.unwrap_or(recipe.variance));
        let own = i32::from(t.segments);
        let (lo, hi) = ((own - spread).clamp(1, 255), (own + spread).clamp(1, 255));
        let inherited = parent.soma.get(i).copied()
            .filter(|n| same_layout && (lo..=hi).contains(&i32::from(*n)));
        retained[i] = inherited.is_some();
        segments.push(inherited.unwrap_or(t.segments));
    }
    let absent = parent.absent.iter().copied().filter(|&(region, at)| {
        let i = usize::from(region);
        retained[i] && at < segments[i] && development::may_lack(rules, &recipe.tagmata[i])
    }).collect();
    Soma { segments, absent, seed: 0 }
}

/// Grows `child` anew from its line's recipe where the world's rule is set,
/// or the repertoire's is and the line has taken a lowered kind, keeping its
/// parent's variation where it still fits ([`kept`]). The exact crowd probe
/// only calls this for births that need no per-child variation.
///
/// # Errors
/// Fails if the line's development cannot grow a body.
pub(crate) fn rebody(rules: &Rules, line: Option<&Lineage>, child: &mut Entity) -> Result<()> {
    rebody_with(rules, line, child, None, None).map(|_| ())
}

pub(crate) fn rebody_child(sim: &crate::Simulation, id: crate::schema::Id, child: &mut Entity) -> Result<bool> {
    let rules = &sim.genesis.rules;
    let seed = birth_seed(sim, id);
    let cache = seed.is_some().then_some(&sim.birth_bodies);
    rebody_with(rules, sim.state.lineages.get(&child.lineage), child, seed, cache)
}

fn birth_seed(sim: &crate::Simulation, id: crate::schema::Id) -> Option<u64> {
    (sim.genesis.rules.inherited_variation != 0)
        .then(|| crate::draw(sim.genesis.dynamics_seed(), "inherited-body", &[id]))
}

#[cfg(test)]
pub(crate) fn rebody_uncached(sim: &crate::Simulation, id: crate::schema::Id, child: &mut Entity) -> Result<bool> {
    rebody_with(&sim.genesis.rules, sim.state.lineages.get(&child.lineage), child, birth_seed(sim, id), None)
}

fn rebody_with(
    rules: &Rules, line: Option<&Lineage>, child: &mut Entity, seed: Option<u64>, cache: Option<&Cache>,
) -> Result<bool> {
    let lowered = rules.repertoire.as_ref();
    let Some((line, d)) = line
        .and_then(|line| line.development.as_ref().map(|d| (line, d)))
        .filter(|(_, d)| rules.inherit || lowered.is_some_and(|l| d.recipe.kinds().iter().any(|k| l.kinds.contains_key(k))))
    else {
        return Ok(false);
    };
    let mut soma = kept(rules, d, child);
    if let Some(seed) = seed {
        vary(rules, &d.recipe, &mut soma, seed);
        child.traits.extend(line.traits.iter().cloned());
    }
    match cache {
        Some(cache) => cache.embody(rules, (line.revision, d), &soma, child)?,
        None => child.embody(development::develop(rules, d, &soma)?),
    }
    child.soma = soma.segments;
    child.absent = soma.absent;
    Ok(true)
}

/// Retain the parent's draw, changing one bounded step in each selected region.
fn vary(rules: &Rules, recipe: &crate::rules::Recipe, soma: &mut Soma, seed: u64) {
    let chance = u64::from(rules.inherited_variation);
    for (index, tagma) in recipe.tagmata.iter().enumerate() {
        let spread = i16::from(tagma.variance.unwrap_or(recipe.variance));
        if spread == 0 {
            continue;
        }
        let draw = crate::draw(seed, "region", &[index as u64]);
        if draw % 100 >= chance {
            continue;
        }
        let centre = i16::from(tagma.segments);
        let count = i16::from(soma.segments[index]) + if (draw / 100) % 2 == 0 { -1 } else { 1 };
        soma.segments[index] = count.clamp((centre - spread).max(1), (centre + spread).min(255)) as u8;
    }
    soma.absent.retain(|&(region, at)| at < soma.segments[usize::from(region)]);
}

/// Treat the selected regions as the grammar seed and grow longer regions by repetition.
pub(crate) fn grammar(recipe: &crate::rules::Recipe) -> crate::rules::Grammar {
    let mut seed = recipe.tagmata.clone();
    let mut rules = Vec::new();
    for (index, region) in seed.iter_mut().enumerate() {
        if region.segments > 1 {
            rules.push(crate::rules::Rule::Repeat {
                tagma: index as u8,
                by: region.segments - 1,
            });
            region.segments = 1;
        }
    }
    crate::rules::Grammar { seed, rules }
}

mod cache;
pub(crate) use cache::Cache;
