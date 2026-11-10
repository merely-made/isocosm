// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a step teaches the played line's participant (cases 1210, 1214):
//! the lineages met at the played critter's site, the lineages outpaced at
//! a boundary, and the regard each line holds for the matter dealt between
//! them. Recorded as one `Command::Regard`, so a replay reaches it alike.

use super::borrowed::BorrowedInterim;
use crate::{
    directing::{self, KNOWS},
    flows::Flow,
    history::Command,
    lineage::boundary,
    rules::{Directing, Regard},
    schema::*,
    Result,
};
use std::collections::BTreeMap;

/// Matter `flow` moved, each member's `amount` for each of its `count`.
fn moved(flow: &Flow) -> u128 {
    u128::from(flow.amount) * u128::from(flow.count)
}

/// Matter the `played` line gave, and took, by each other line's bodies:
/// `(given, taken)`. Only moves between two bodies count (case 1214).
pub(crate) fn dealings(flows: &[Flow], played: &str) -> BTreeMap<Key, (u128, u128)> {
    let mut out: BTreeMap<Key, (u128, u128)> = BTreeMap::new();
    for f in flows {
        let (Some(from), Some(to)) = (&f.from_kind, &f.to_kind) else { continue };
        if from.lineage == to.lineage || from.kingdom == directing::PARTICIPANT
            || to.kingdom == directing::PARTICIPANT {
            continue;
        }
        if from.lineage == played {
            out.entry(to.lineage.clone()).or_default().0 += moved(f);
        } else if to.lineage == played {
            out.entry(from.lineage.clone()).or_default().1 += moved(f);
        }
    }
    out
}

/// A line's regard after `given` and `taken` matter, from `held` or the
/// first bond's `fresh`, kept between nought and the most (case 1214).
pub(crate) fn regarded(held: Option<i64>, given: u128, taken: u128, rule: &Regard, d: &Directing) -> i64 {
    let per = u128::from(rule.per.max(1));
    let units = |n: u128| i128::try_from(n / per).unwrap_or(i128::MAX);
    let delta = i128::from(rule.gain).saturating_mul(units(given))
        .saturating_sub(i128::from(rule.loss).saturating_mul(units(taken)));
    let now = i128::from(held.unwrap_or(d.fresh)).saturating_add(delta);
    now.clamp(0, i128::from(d.most)) as i64
}

impl BorrowedInterim<'_> {
    /// Records the lineages the participant newly knows (met at the played
    /// critter's site; and, at a boundary, outpaced) and each line's new
    /// regard from this step's `flows`, issuing nothing where there is none.
    pub(super) fn record(&mut self, played: Id, lineage: &str, flows: &[Flow], outpacing: bool) -> Result<()> {
        let sim = &self.session.sim;
        let d = sim.genesis().rules.directing();
        let Some(rule) = d.regard else { return Ok(()) };
        let state = sim.state();
        let pop = &state.population;
        let known: Vec<&str> = state.relations.iter()
            .filter(|r| r.subject == self.participant && r.kind == KNOWS)
            .filter_map(|r| pop.get(r.object).map(|e| e.lineage.as_str())).collect();
        // The lowest-id living member of each other line, by line.
        let mut lowest: BTreeMap<&str, Id> = BTreeMap::new();
        for (first, g) in &pop.groups {
            let e = &g.entity;
            if e.alive && !directing::is_participant(e) && e.lineage != lineage
                && state.lineages.get(&e.lineage).is_some_and(|l| l.kingdom != "kingdom:world") {
                lowest.entry(e.lineage.as_str()).or_insert(*first);
            }
        }
        let mut knows: BTreeMap<&str, Id> = BTreeMap::new();
        let site = pop.get(played).filter(|e| e.alive).and_then(Entity::resident_site);
        if let Some(site) = site {
            for (first, g) in &pop.groups {
                let e = &g.entity;
                if e.alive && e.resident_site() == Some(site) && lowest.contains_key(e.lineage.as_str()) {
                    knows.entry(e.lineage.as_str()).or_insert(*first);
                }
            }
        }
        if outpacing {
            let mine = boundary::standing(self.session, lineage).0;
            for (other, first) in &lowest {
                if boundary::standing(self.session, other).0 < mine {
                    knows.entry(other).or_insert(*first);
                }
            }
        }
        let knows: Vec<Id> = knows.into_iter().filter(|(l, _)| !known.contains(l)).map(|(_, id)| id).collect();
        let mut regard = Vec::new();
        for (other, (given, taken)) in dealings(flows, lineage) {
            let Some(line) = state.lineages.get(&other) else { continue };
            let held = line.regard.get(&self.participant).copied();
            let now = regarded(held, given, taken, &rule, &d);
            if now != held.unwrap_or(d.fresh) {
                regard.push((other, now));
            }
        }
        if knows.is_empty() && regard.is_empty() {
            return Ok(());
        }
        let participant = self.participant;
        self.session.command(Command::Regard { participant, knows, regard })?;
        Ok(())
    }
}
