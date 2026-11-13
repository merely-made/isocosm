// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The catalog's parts reaching a line (lineage-rounds D37; `crate::
//! repertoire`): the offers the played line weighs at its review and the
//! candidates an unplayed line takes at a boundary are the same list, so the
//! forked copy scores them as it scores any revision.
//!
//! A line of the kingdom a lowered kind was lowered for (or any, where the
//! rule lets lines cross kingdoms) may be offered it in two ways:
//!
//! - a mouth, whose port rests on the function the line's own feed binds
//!   (`ecology:feed-N`), takes the place of the mouth the line was founded
//!   with: on the head's bearing where the mouth is borne below it, else, for
//!   a line whose root is its mouth, the head segment (a root variant keeps
//!   its cell of reproduce). A mouth on any other function would never be
//!   bound by the line's feed, so none is offered;
//! - any other lowered organ is added to the tail, as a tagma of one segment
//!   of its kind, or, in a line with a grammar, as the grammar's own
//!   differentiation of its last run (`grammar::differentiations`).
//!
//! A kind the edit names joins the line's lexicon, which `Simulation::
//! revisable` allows only for the kinds the world lowered. At most
//! `directing.variants` are offered at a time, from a window of the sorted
//! list that moves with the line's revision (a draw keyed by the world's
//! seed), so successive boundaries walk the catalog and none starves.

use super::grammar;
use crate::{
    Session,
    rules::{Anchor, Development, Facing, Tagma},
    schema::*,
};

/// The head's mouth edited to `kind`, where the line has one.
fn swapped(d: &Development, kind: &Key, root: Option<&Key>) -> Option<Development> {
    let mut v = d.clone();
    let head = match v.grammar.as_mut() {
        Some(g) => g.seed.first_mut()?,
        None => v.recipe.tagmata.first_mut()?,
    };
    match (&head.bears, head.socket) {
        (Some(_), Facing::Below) => head.bears = Some(kind.clone()),
        _ => head.segment = root?.clone(),
    }
    let named = match (&head.bears, &head.segment) {
        (Some(b), s) if b == kind => (b.clone(), s.clone()),
        (_, s) => (s.clone(), s.clone()),
    };
    v.lexicon.extend([named.0, named.1]);
    match v.grammar {
        Some(_) => crate::rules::regrown(&v).ok(),
        None => Some(v),
    }
}

/// `d` with a tagma of one segment of `kind` behind its body.
fn tailed(d: &Development, kind: &Key) -> Development {
    let mut v = d.clone();
    v.recipe.tagmata.push(Tagma {
        chain: vec![],
        segments: 1,
        segment: kind.clone(),
        bears: None,
        per_segment: 0,
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        socket: Facing::Right,
        variance: None,
    });
    v.lexicon.insert(kind.clone());
    v
}

/// The repertoire's edits to `d`, named, in a window of at most `most`; none
/// unless the world set the rule. Not yet held to the world's rules.
pub(super) fn offers(session: &Session, lineage: &str, d: &Development, most: usize) -> Vec<(Key, Development)> {
    let sim = &session.sim;
    let rules = &sim.genesis().rules;
    let Some(lowered) = &rules.repertoire else { return vec![] };
    let Some(line) = sim.state().lineages.get(lineage) else { return vec![] };
    let named = d.recipe.kinds();
    let feed = lineage.strip_prefix("lineage:")
        .and_then(|n| rules.processes.get(&format!("ecology:feed-{n}")))
        .and_then(|p| p.port_feeding.as_ref())
        .map(|f| f.function.clone());
    let mut out: Vec<(Key, Development)> = vec![];
    let reach = |kind: &&Key| lowered.reaches(&line.kingdom, kind) && !named.contains(*kind);
    let organs: Vec<&Key> = lowered.kinds.iter()
        .filter(|(k, e)| !e.mouth && e.root_of.is_none() && reach(k)).map(|(k, _)| k).collect();
    for (kind, e) in lowered.kinds.iter().filter(|(k, e)| e.mouth && e.root_of.is_none() && reach(k)) {
        if feed.as_ref() == Some(&e.function)
            && let Some(v) = swapped(d, kind, e.root.as_ref())
        {
            out.push((format!("revision:repertoire-mouth-{kind}"), v));
        }
    }
    match d.grammar {
        Some(_) => out.extend(grammar::differentiations(d, organs.iter().copied())),
        None => out.extend(organs.iter().map(|k| (format!("revision:repertoire-tail-{k}"), tailed(d, k)))),
    }
    out.retain(|(_, v)| sim.revisable(lineage, v).is_ok());
    out.sort_by(|a, b| a.0.cmp(&b.0));
    if out.len() > most && most > 0 {
        let start = crate::draw(sim.genesis().seed, "repertoire-window", &[u64::from(line.revision)]) as usize % out.len();
        out.rotate_left(start);
        out.truncate(most);
    }
    out
}
