// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an act offers a body's pressing needs. Nothing here names a need,
//! an act or a way of living: an act serves a need by what its effects do
//! (`choice::serves_need`), and a move serves one where the body senses that
//! an act of its own, with nothing to work on where it stands, would find
//! it at the other end. So a grazer goes toward what its mouth admits, a
//! fixer toward light, a trader toward a body its organs fit, each by the
//! same reading and only as far as it can sense.

use super::choice::{moves, serves_need};
use crate::{rules::*, schema::*, simulation::Simulation};
use std::collections::BTreeMap;

/// What a move offers: for each site a body senses beyond its own, the
/// pressing needs, by place in the list, that going there would serve.
pub(super) type Elsewhere = BTreeMap<Id, Vec<usize>>;

/// The pressing needs, by place in `pressing`, an act of `p` serves: by
/// what it does, and, where it moves the body, by what the body senses it
/// could then do there (`Simulation::elsewhere`).
pub(super) fn served(p: &Process, pressing: &[&Need], elsewhere: &Elsewhere) -> Vec<usize> {
    let mut by: Vec<usize> = (0..pressing.len()).filter(|&k| serves_need(p, pressing[k])).collect();
    for k in moves(p).iter().flat_map(|to| elsewhere.get(to).into_iter().flatten()) {
        if !by.contains(k) {
            by.push(*k);
        }
    }
    by
}

impl Simulation {
    /// What a move offers `actor`, standing at `place` (`Elsewhere`): for
    /// each neighbouring site, where the body senses that far, the pressing
    /// needs that an act of its own serves, finds nothing to work on here,
    /// and would find there. The acts are the body's own, chosen or not: a
    /// fixer's growth and a trader's exchange count as a mouth's meal does,
    /// so what a body goes looking for is whatever its body lives on.
    pub(super) fn elsewhere(&self, actor: Id, place: Id, pressing: &[&Need]) -> Elsewhere {
        let mut out = Elsewhere::new();
        if pressing.is_empty() || self.sense_reach(actor) == 0 {
            return out;
        }
        let (Some(body), Some(here)) = (self.body_at_start(actor), self.site_at_start(place)) else {
            return out;
        };
        if body.resident_site() != Some(place) {
            return out;
        }
        // What an act asks of the place or of another body is read at each
        // site in turn; what it asks of the actor must hold already.
        let of_place = |q: &Query| {
            matches!(
                q,
                Query::Account { who: Binding::Place, .. }
                    | Query::Condition { .. }
                    | Query::Sensed { .. }
                    | Query::Members { .. }
                    | Query::MembersBelow { .. }
                    | Query::MembersPer { .. }
                    | Query::MembersPerHead { .. }
                    | Query::FoodPerHead { .. }
                    | Query::EdiblePerHead { .. }
                    | Query::SoilContact(_)
                    | Query::MigrationMotive { .. }
            )
        };
        let own = |p: &&Process| {
            p.causation != Causation::Agentless
                && moves(p).is_empty()
                && pressing.iter().any(|n| serves_need(p, n))
                && p.requires.iter().all(|q| match q {
                    Query::Trait { who: Binding::Actor, key } => body.traits.contains(key),
                    _ => true,
                })
                && p.requires.iter().filter(|q| !of_place(q) && !crate::targets::reads_prey(q)).all(|q| {
                    self.process_query(actor, None, place, self.bind_part(actor, p), p, q).is_ok()
                })
        };
        let acts: Vec<&Process> = self.genesis.rules.processes.values().filter(own).collect();
        for (k, need) in pressing.iter().enumerate() {
            let serving: Vec<&Process> = acts.iter().copied().filter(|p| serves_need(p, need)).collect();
            if serving.is_empty() || serving.iter().any(|p| self.works_at(actor, p, place)) {
                continue;
            }
            for route in &here.routes {
                if serving.iter().any(|p| self.works_at(actor, p, route.to)) {
                    out.entry(route.to).or_default().push(k);
                }
            }
        }
        out
    }
}
