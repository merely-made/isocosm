// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The drift (case 1208; lineage-rounds D2, D9): at an epoch boundary a
//! line's template moves to the average of what its members drew, each
//! living member weighted one plus its offspring born in the epoch, as an
//! evolution strategy moves its mean. A child draws afresh around the
//! template, so a draw that worked leaves more young and pulls it along.
//! Integer throughout; ties round toward the value now held.

use crate::{
    Result, Session,
    directing::policy::LineagePolicy,
    history::Command,
    rules::Development,
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

/// Each living group of `lineage`, by its first id: its count plus the
/// offspring its first member bore at or after `since`.
pub fn weights(sim: &Simulation, lineage: &str, since: Tick) -> BTreeMap<Id, u64> {
    let state = sim.state();
    let mut out: BTreeMap<Id, u64> = state
        .population
        .groups
        .iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == lineage)
        .map(|(&first, g)| (first, g.count))
        .collect();
    for r in state.relations.iter().filter(|r| r.kind == "sim:child") {
        let young = state.population.get(r.object).is_some_and(|e| e.born >= since);
        if let (true, Some(w)) = (young, out.get_mut(&r.subject)) {
            *w += 1;
        }
    }
    out
}

/// `sum / total` rounded to the nearest integer; an exact half goes to
/// whichever neighbour is nearer `current`.
fn mean(sum: i128, total: i128, current: i128) -> i128 {
    let (q, r) = (sum.div_euclid(total), sum.rem_euclid(total));
    match (2 * r).cmp(&total) {
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Greater => q + 1,
        std::cmp::Ordering::Equal if current <= q => q,
        std::cmp::Ordering::Equal => q + 1,
    }
}

/// The members' weighted mean of one value each holds, or none if no one holds it.
fn pooled(held: impl Iterator<Item = (u64, Option<i128>)>, current: i128) -> Option<i128> {
    let (sum, total) = held
        .filter_map(|(w, x)| Some((i128::from(w), x?)))
        .fold((0, 0), |(s, t), (w, x)| (s + w * x, t + w));
    (total > 0).then(|| mean(sum, total, current))
}

/// The line's development with each tagma's segments moved to its weighted
/// members' mean; none if it has no development, nothing moved, or the
/// world would refuse the result.
pub fn development(sim: &Simulation, lineage: &str, since: Tick) -> Option<Development> {
    let had = sim.state().lineages.get(lineage)?.development.as_ref()?;
    let weighed = weights(sim, lineage, since);
    let groups = &sim.state().population.groups;
    let mut moved = had.clone();
    for (i, t) in moved.recipe.tagmata.iter_mut().enumerate() {
        let held = weighed.iter().map(|(id, &w)| {
            (w, groups[id].entity.soma.get(i).map(|&s| i128::from(s)))
        });
        if let Some(m) = pooled(held, i128::from(t.segments)) {
            t.segments = m.clamp(1, 255) as u8;
        }
    }
    (moved != *had && sim.revisable(lineage, &moved).is_ok()).then_some(moved)
}

/// The line's policy with each weight moved to its weighted members'
/// mean as they express it; none if it has no policy or nothing moved.
pub fn policy(sim: &Simulation, lineage: &str, since: Tick) -> Option<LineagePolicy> {
    let had = sim.state().lineages.get(lineage)?.policy.as_ref()?;
    let size = sim.genesis().rules.directing().policy.expression;
    let groups = &sim.state().population.groups;
    let expressed: Vec<(u64, LineagePolicy)> = weights(sim, lineage, since)
        .into_iter()
        .map(|(id, w)| match groups[&id].entity.drawn {
            Some(drawn) => (w, had.varied(drawn, size)),
            None => (w, had.clone()),
        })
        .collect();
    let at = |current: i128, bound: i128, get: &dyn Fn(&LineagePolicy) -> i128| {
        let held = expressed.iter().map(|(w, p)| (*w, Some(get(p))));
        pooled(held, current).map_or(current, |m| m.clamp(-bound, bound))
    };
    let (wide, narrow) = (i128::from(i16::MAX), i128::from(i8::MAX));
    let mut moved = had.clone();
    for (k, w) in moved.drives.iter_mut().enumerate() {
        *w = at(i128::from(*w), wide, &|p| i128::from(p.drives[k])) as i16;
    }
    moved.priority = at(i128::from(had.priority), wide, &|p| i128::from(p.priority)) as i16;
    for (u, unit) in moved.hidden.iter_mut().enumerate() {
        for (j, w) in unit.iter_mut().enumerate() {
            *w = at(i128::from(*w), narrow, &|p| i128::from(p.hidden[u][j])) as i8;
        }
    }
    for (k, w) in moved.out.iter_mut().enumerate() {
        *w = at(i128::from(*w), narrow, &|p| i128::from(p.out[k])) as i8;
    }
    (moved != *had).then_some(moved)
}

/// What the drift commits for `lineage`: nothing unless the world's rules
/// ask for it; then its development, then its policy, where they moved.
pub fn commands(sim: &Simulation, lineage: &str, since: Tick) -> Vec<Command> {
    if !sim.genesis().rules.directing().drift {
        return vec![];
    }
    let lineage_key = || lineage.to_string();
    let revise = development(sim, lineage, since)
        .map(|development| Command::Revise { lineage: lineage_key(), development });
    let adopt = policy(sim, lineage, since)
        .map(|policy| Command::Policy { lineage: lineage_key(), policy });
    revise.into_iter().chain(adopt).collect()
}

/// Sends `commands`, returning whether any was sent.
///
/// # Errors
/// Fails if the world refuses a command.
pub fn commit(session: &mut Session, lineage: &str, since: Tick) -> Result<bool> {
    let sent = commands(&session.sim, lineage, since);
    for command in &sent {
        session.command(command.clone())?;
    }
    Ok(!sent.is_empty())
}
