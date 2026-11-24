// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Spatial requests at site grain. Distance fields live for one consideration.
use super::{Aim, Toward, readings::scopes::hops_to};
use crate::{
    rules::{Causation, Effect, Process, Query},
    schema::Id,
    simulation::Simulation,
};
use std::collections::{BTreeMap, VecDeque};

#[cfg(test)]
mod tests;

struct Request {
    index: usize,
    aim: Aim,
    field: (Id, bool),
    strength: i64,
    near_hops: u32,
}

#[derive(Default)]
pub(crate) struct Guidance {
    personal: [Option<super::experience::Preference>; 2],
    requests: Vec<Request>,
    distances: BTreeMap<(Id, bool), BTreeMap<Id, u32>>,
}

/// Every possible movement outcome, including staying put on a conditional branch.
fn movement(process: &Process) -> (Vec<Id>, bool) {
    fn sequence<'a>(effects: impl Iterator<Item = &'a Effect>, sites: &mut Vec<Id>) -> bool {
        let mut guaranteed = false;
        for effect in effects {
            match effect {
                Effect::Move { destination, .. } | Effect::MovePaced { destination, .. } => {
                    sites.push(*destination);
                    guaranteed = true;
                },
                Effect::When {
                    then, otherwise, ..
                } => {
                    let first = sequence(then.iter(), sites);
                    let second = sequence(otherwise.iter(), sites);
                    guaranteed |= first && second;
                },
                _ => {},
            }
        }
        guaranteed
    }
    let mut sites = Vec::new();
    let guaranteed = sequence(
        process.commitments.iter().chain(&process.effects),
        &mut sites,
    );
    (sites, !guaranteed)
}

fn hops_from(sim: &Simulation, source: Id) -> BTreeMap<Id, u32> {
    let mut distances = BTreeMap::from([(source, 0)]);
    let mut queue = VecDeque::from([source]);
    while let Some(site) = queue.pop_front() {
        let next = distances[&site] + 1;
        for route in &sim.state().sites[&site].routes {
            if let std::collections::btree_map::Entry::Vacant(entry) = distances.entry(route.to) {
                entry.insert(next);
                queue.push_back(route.to);
            }
        }
    }
    distances
}

impl Guidance {
    pub(super) fn new(sim: &Simulation, live: &[usize]) -> Self {
        let mut result = Self::default();
        for &index in live {
            let n = &sim.state().nudges[index];
            if !n.aim.is_direction() {
                continue;
            }
            let rules = sim.genesis().rules.directing_ref();
            let strength =
                rules.sway * sim.bond(n.participant, n.critter).unwrap_or(0) / rules.most;
            if strength <= 0 {
                continue;
            }
            let site = match n.toward {
                Toward::Site(id) => id,
                Toward::Thing(id) => {
                    let Some(site) = sim.body_at_start(id).and_then(|e| e.resident_site()) else {
                        continue;
                    };
                    site
                },
            };
            let away = n.aim == Aim::MoveAway;
            let field = (site, away);
            result.distances.entry(field).or_insert_with(|| {
                if away {
                    hops_from(sim, site)
                } else {
                    hops_to(sim, site)
                }
            });
            result.requests.push(Request {
                index,
                aim: n.aim,
                field,
                strength,
                near_hops: rules.near_hops,
            });
        }
        result
    }

    pub(super) fn for_actor(
        sim: &Simulation,
        actor: Id,
        live: &[usize],
        needs: Option<&[&crate::rules::Need]>,
    ) -> Self {
        let mut guidance = Self::new(sim, live);
        guidance.personal = sim.place_preferences(actor, needs);
        guidance
    }

    fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.personal.iter().all(Option::is_none)
    }

    /// Agreement (+1), indifference, or leaving the requested vicinity (-1).
    fn vote(&self, request: &Request, here: Id, to: Option<Id>) -> i64 {
        let distances = &self.distances[&request.field];
        let Some(&before) = distances.get(&here) else {
            return 0;
        };
        let after = to.and_then(|id| distances.get(&id).copied());
        match request.aim {
            Aim::StayNear if before <= request.near_hops => {
                if to.is_none() || after.is_some_and(|d| d <= request.near_hops) {
                    1
                } else {
                    -1
                }
            },
            Aim::MoveToward | Aim::StayNear => i64::from(after.is_some_and(|d| d < before)),
            Aim::MoveAway => i64::from(after.is_some_and(|d| d > before)),
            _ => 0,
        }
    }

    pub(super) fn score(
        &self,
        sim: &Simulation,
        process: &Process,
        place: Id,
        answers: &mut Vec<usize>,
    ) -> i64 {
        if self.is_empty() {
            return 0;
        }
        let (destinations, can_stay) = movement(process);
        let outcomes = || {
            destinations
                .iter()
                .copied()
                .map(Some)
                .chain(can_stay.then_some(None))
        };
        let mut score = self
            .personal
            .iter()
            .flatten()
            .map(|pref| {
                let vote = outcomes().map(|to| pref.vote(place, to)).min().unwrap_or(0);
                vote * i64::from(pref.weight)
            })
            .sum::<i64>();
        for req in &self.requests {
            let vote = outcomes()
                .map(|to| self.vote(req, place, to))
                .min()
                .unwrap_or(0);
            score += req.strength * vote;
            if vote > 0 && sim.state().nudges[req.index].answer.is_none() {
                answers.push(req.index);
            }
        }
        score
    }

    fn wants_step(&self, place: Id, to: Id) -> bool {
        self.personal
            .iter()
            .flatten()
            .any(|p| p.vote(place, Some(to)) > p.vote(place, None))
            || self
                .requests
                .iter()
                .any(|r| self.vote(r, place, Some(to)) > 0 && self.vote(r, place, None) == 0)
    }
}

impl Simulation {
    /// A personal preference or a trusted request may motivate a generated move.
    pub(crate) fn directed_migration(
        &self,
        actor: Id,
        place: Id,
        p: &Process,
        q: &Query,
        known: Option<&Guidance>,
    ) -> bool {
        if !matches!(q, Query::MigrationMotive { .. }) || p.causation != Causation::Choice {
            return false;
        }
        let fresh;
        let guidance = match known {
            Some(g) => g,
            None => {
                if self.state.nudges.is_empty()
                    && !self
                        .body_at_start(actor)
                        .is_some_and(|e| e.places.is_some())
                {
                    return false;
                }
                fresh = Guidance::for_actor(
                    self,
                    actor,
                    &self.live_nudges(actor, self.state.tick),
                    None,
                );
                &fresh
            },
        };
        if guidance.is_empty() {
            return false;
        }
        let (destinations, can_stay) = movement(p);
        !can_stay
            && destinations
                .into_iter()
                .all(|to| guidance.wants_step(place, to))
    }
}
