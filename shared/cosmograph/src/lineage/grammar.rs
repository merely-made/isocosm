// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rule edits at the review (cases 1202, 1206): a line with a growth grammar
//! may tune one number of one rule, or add a rule that differentiates its
//! last run to a kind it has learned. Each offer is a development whose
//! tagmata unfold afresh from the edited grammar.

use crate::rules::{Development, Rule, regrown, unfold};
use crate::schema::*;
use std::collections::BTreeSet;

/// The numbers a rule holds, in the order they are tuned.
const KNOBS: [&str; 6] = ["by", "every", "depth", "taper", "at", "total"];

/// The developments `d` could take by one edit to its grammar, named, at
/// most `most`: each number of each rule moved up and down by one, then a
/// differentiation of the last run to each learned kind the body lacks. None
/// without a grammar.
pub fn rules(d: &Development, most: usize) -> Vec<(Key, Development)> {
    candidates(d, None).into_iter().take(most).collect()
}

/// Every such edit, the additions only of the kinds in `only` when given
/// (case 1200); not yet held to the world's rules.
pub(super) fn candidates(d: &Development, only: Option<&BTreeSet<Key>>) -> Vec<(Key, Development)> {
    let Some(grammar) = &d.grammar else {
        return vec![];
    };
    let mut out = vec![];
    for (i, rule) in grammar.rules.iter().enumerate() {
        for knob in KNOBS {
            for (way, up) in [("up", true), ("down", false)] {
                let Some(moved) = moved(rule, knob, up) else {
                    continue;
                };
                let mut edited = d.clone();
                if let Some(g) = edited.grammar.as_mut() {
                    g.rules[i] = moved;
                }
                out.push((format!("revision:rule-{i}-{knob}-{way}"), edited));
            }
        }
    }
    out.extend(differentiations(d, d.lexicon.iter().filter(|k| only.is_none_or(|o| o.contains(*k)))));
    out.into_iter()
        .filter_map(|(name, v)| regrown(&v).ok().filter(|r| r != d).map(|r| (name, r)))
        .collect()
}

/// The developments `d` could take by one further growth rule (case 1206):
/// for each operator its kingdom weighs, a rule of it on a run drawn by
/// `seed`, placed before the closing budget. The operators and their bounds
/// are the ones its founders drew from (`bodied::catalogue::growth`), so a
/// line goes on generating its body after founding by the same means; not
/// yet held to the world's rules.
pub(super) fn additions(d: &Development, kingdom: &str, seed: u64) -> Vec<(Key, Development)> {
    let (Some(grammar), Some(weights)) = (&d.grammar, crate::catalog::catalog().kingdom(kingdom)) else {
        return vec![];
    };
    let Ok(body) = unfold(grammar) else { return vec![] };
    let sampler = crate::bodied::GrowthSampler {
        kingdom: weights,
        draw: |domain: &str, slot: u64| crate::draw(seed, domain, &[slot]),
        valid: |_: &crate::rules::Grammar| true,
        seasons: false,
    };
    let weighed = crate::bodied::GROWTH_OPERATORS.iter().filter(|name| weights.operators.get(**name).is_some_and(|w| *w > 0));
    let mut out = vec![];
    for (slot, operator) in weighed.enumerate() {
        let Some(rule) = sampler.propose(operator, &body, slot as u64) else { continue };
        let mut edited = d.clone();
        if let Some(g) = edited.grammar.as_mut() {
            let closing = |r: &Rule| matches!(r, Rule::Allocate { .. } | Rule::Partition(_));
            let at = g.rules.iter().position(closing).unwrap_or(g.rules.len());
            g.rules.insert(at, rule);
        }
        out.push((format!("revision:rule-add-{operator}"), edited));
    }
    out.into_iter()
        .filter_map(|(name, v)| regrown(&v).ok().filter(|r| r != d).map(|r| (name, r)))
        .collect()
}

/// The edits differentiating the last run of `d`'s grammar to each of `kinds`
/// its body does not name, a kind outside the lexicon joining it (the
/// repertoire's, D37); not yet held to the world's rules.
pub(super) fn differentiations<'k>(d: &Development, kinds: impl Iterator<Item = &'k Key>) -> Vec<(Key, Development)> {
    let mut out = vec![];
    let Some(grammar) = &d.grammar else { return out };
    if let Ok(body) = unfold(grammar)
        && let Some(last) = body.last()
        && last.segments >= 2
    {
        let (tagma, at) = ((body.len() - 1) as u8, last.segments / 2);
        let named: BTreeSet<Key> = body.iter().map(|t| t.segment.clone())
            .chain(body.iter().filter_map(|t| t.bears.clone()))
            .chain(body.iter().flat_map(|t| t.chain.iter().map(|l| l.kind.clone())))
            .collect();
        for kind in kinds.filter(|k| !named.contains(*k)) {
            let mut edited = d.clone();
            edited.lexicon.insert(kind.clone());
            if let Some(g) = edited.grammar.as_mut() {
                let at_rule = g.rules.iter().position(|r| matches!(r, Rule::Partition(_))).unwrap_or(g.rules.len());
                g.rules.insert(at_rule, Rule::Differentiate { tagma, at, kind: kind.clone() });
            }
            out.push((format!("revision:rule-differentiate-{kind}"), edited));
        }
    }
    out.into_iter()
        .filter_map(|(name, v)| regrown(&v).ok().filter(|r| r != d).map(|r| (name, r)))
        .collect()
}

/// `rule` with its number `knob` moved by one, where it has the number and
/// the move leaves it valid.
fn moved(rule: &Rule, knob: &str, up: bool) -> Option<Rule> {
    let step = |n: u8| if up { n.checked_add(1) } else { n.checked_sub(1) };
    let mut r = rule.clone();
    match (&mut r, knob) {
        (Rule::Repeat { by, .. }, "by") => *by = step(*by)?,
        (Rule::Branch { every, .. }, "every") => *every = step(*every).filter(|n| *n > 0)?,
        (Rule::Branch { depth, .. }, "depth") => *depth = step(*depth)?,
        (Rule::Branch { taper, .. }, "taper") => *taper = step(*taper)?,
        (Rule::Differentiate { at, .. }, "at") => *at = step(*at)?,
        (Rule::Allocate { total }, "total") => {
            *total = if up { total.checked_add(1)? } else { total.checked_sub(1)? }
        },
        _ => return None,
    }
    Some(r)
}
