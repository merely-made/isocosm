// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    /// A meal already in reach can wait for its feeding cadence. Only weaker
    /// departures are suppressed; paid upkeep, healing and stronger nudges remain.
    pub(crate) fn foraging_wait_score(&self, actor: Id) -> i64 {
        if !self.foraging_enabled() || !self.state.spatial_poses.contains_key(&actor) {
            return 0;
        }
        let templates = self.genesis.rules.spatial_templates.as_ref().unwrap();
        templates.foraging.values().filter_map(|f| {
            let feed = &self.genesis.rules.processes[&f.feed];
            if feed.period.is_none_or(|p| self.state.tick % p == 0) {
                return None;
            }
            let score = self.food_need_score(actor, feed);
            (score > 0 && self.food_candidates(actor, feed).iter().any(|(_, d)| *d <= f.reach))
                .then_some(score)
        }).max().unwrap_or(0)
    }

    fn food_candidates(&self, actor: Id, feed: &Process) -> Vec<(Id, u64)> {
        let Some(body) = self.body_at_start(actor) else {
            return Vec::new();
        };
        if feed.need_account.as_ref().is_some_and(|key| {
            crate::anatomy::held(body, &self.genesis.rules, key) >= feed.need_below
        }) || feed.feeding_part(body).is_err()
            || !self.route_ready(actor, feed)
            || self.direction_ready(actor, feed).is_err()
        {
            return Vec::new();
        }
        let part = self.bind_part(actor, feed);
        let ids: BTreeSet<_> = self
            .state
            .spatial_poses
            .keys()
            .copied()
            .chain(
                self.frame
                    .iter()
                    .flat_map(|f| f.spatial_poses.keys().copied()),
            )
            .collect();
        ids.into_iter()
            .filter_map(|target| {
                let d = self.food_gap(actor, target)?;
                (self.target_matches_without_reach(actor, Some(target), feed)
                    && self.owned_process_ready(actor, Some(target), feed)
                    && feed.requires.iter().all(|q| {
                        self.process_query(actor, Some(target), body.place, part, feed, q)
                            .is_ok()
                    }))
                .then_some((target, d))
            })
            .collect()
    }

    /// Some(None) is a physical search with no eligible target. None keeps the
    /// ordinary cohort-weighted selector for unplaced/coarse actors unchanged.
    pub(crate) fn foraging_target(&self, actor: Id, p: &Process) -> Option<Option<Id>> {
        if !self.foraging_enabled() {
            return None;
        }
        let t = self.genesis.rules.spatial_templates.as_ref()?;
        if let Some(f) = t.foraging.get(&p.id) {
            if !self.state.spatial_poses.contains_key(&actor) {
                return Some(None);
            }
            let feed = &self.genesis.rules.processes[&f.feed];
            if self.food_need_score(actor, feed) <= 0 {
                return Some(None);
            }
            let food = self.food_candidates(actor, feed);
            if food.iter().any(|(_, d)| *d <= f.reach) {
                return Some(None);
            }
            return Some(
                food.into_iter()
                    .filter(|(_, d)| *d <= f.search)
                    .min_by_key(|(id, d)| (*d, *id))
                    .map(|(id, _)| id),
            );
        }
        if !self.state.spatial_poses.contains_key(&actor) {
            return None;
        }
        let f = t.foraging.values().find(|f| f.feed == p.id)?;
        let food: Vec<_> = self
            .food_candidates(actor, p)
            .into_iter()
            .filter(|(_, d)| *d <= f.reach)
            .map(|(id, _)| id)
            .collect();
        if !p.target.as_ref()?.weighted {
            return Some(food.first().copied());
        }
        // Only the actual physically placed identities enter this draw. A
        // representative's pose must never place all its cohort's other IDs.
        let weights: Vec<_> = food
            .into_iter()
            .map(|id| {
                (
                    id,
                    crate::meaning::mass(
                        &self.books_at_start(id).expect("placed prey"),
                        &self.genesis.rules,
                    ),
                )
            })
            .filter(|(_, held)| *held > 0)
            .collect();
        let total: u128 = weights.iter().map(|(_, held)| held).sum();
        if total == 0 {
            return Some(None);
        }
        let mut pick = u128::from(crate::draw(
            self.genesis.dynamics_seed(),
            &format!("target:{}", p.id),
            &[self.state.tick, actor],
        )) * total
            >> 64;
        for (id, held) in weights {
            if pick < held {
                return Some(Some(id));
            }
            pick -= held;
        }
        Some(None)
    }

    pub(crate) fn foraging_walk_matches(&self, actor: Id, target: Option<Id>, p: &Process) -> bool {
        let Some(f) = self.foraging_policy(&p.id) else {
            return true;
        };
        if !self.state.spatial_poses.contains_key(&actor) {
            return false;
        }
        let feed = &self.genesis.rules.processes[&f.feed];
        self.food_need_score(actor, feed) > 0
            && self
                .food_candidates(actor, feed)
                .iter()
                .any(|(id, d)| Some(*id) == target && *d > f.reach && *d <= f.search)
    }
}
