// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    /// Edible resident identities for physical admission, including an actor
    /// arriving at `site`. Reuse native meal/trait/stock queries; placement
    /// ignores only the actor's current location and momentary appetite.
    pub fn placement_food(&self, actor: Id, site: Id) -> BTreeMap<Id, u64> {
        let Some(body) = self.body_at_start(actor) else {
            return BTreeMap::new();
        };
        let mut out = BTreeMap::new();
        for feed in self
            .genesis
            .rules
            .processes
            .values()
            .filter(|p| p.port_feeding.is_some() && p.target.is_some())
        {
            if feed.feeding_part(body).is_err() {
                continue;
            }
            let mut selector = feed.clone();
            selector.target.as_mut().unwrap().same_place = false;
            let search = self
                .genesis
                .rules
                .spatial_templates
                .as_ref()
                .into_iter()
                .flat_map(|t| t.foraging.values())
                .filter(|f| f.feed == feed.id)
                .map(|f| f.search)
                .max()
                .unwrap_or(FOOD_SEARCH);
            for (&id, group) in &self.state.population.groups {
                if group.entity.resident_site() != Some(site) {
                    continue;
                }
                let count = if embodiment::whole_site(self) { group.count } else { 1 };
                for target in id..id+count {
                if self.food_queries(actor, target, &selector, site) {
                    out.entry(target)
                        .and_modify(|range: &mut u64| *range = (*range).max(search))
                        .or_insert(search);
                }
                }
            }
        }
        out
    }

    fn food_queries(&self, actor: Id, target: Id, feed: &Process, place: Id) -> bool {
        let part = self.bind_part(actor, feed);
        self.target_matches_without_reach(actor, Some(target), feed)
            && feed.requires.iter().all(|q| {
                self.process_query(actor, Some(target), place, part, feed, q)
                    .is_ok()
            })
    }

    /// A meal already in reach can wait for its feeding cadence. Only weaker
    /// departures are suppressed; paid upkeep, healing and stronger nudges remain.
    pub(crate) fn foraging_wait_score(&self, actor: Id) -> i64 {
        if !self.foraging_enabled() || !self.state.spatial_poses.contains_key(&actor) {
            return 0;
        }
        let templates = self.genesis.rules.spatial_templates.as_ref().unwrap();
        templates
            .foraging
            .values()
            .filter_map(|f| {
                let feed = &self.genesis.rules.processes[&f.feed];
                if feed.period.is_none_or(|p| self.state.tick % p == 0) {
                    return None;
                }
                let score = self.food_need_score(actor, feed);
                (score > 0
                    && self
                        .food_candidates(actor, feed, f)
                        .iter()
                        .any(|(_, d)| *d <= f.reach))
                .then_some(score)
            })
            .max()
            .unwrap_or(0)
    }

    pub(super) fn food_candidates(&self, actor: Id, feed: &Process, policy: &Foraging) -> Vec<(Id, u64)> {
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
        // Build at most one eye/terrain reading, only if an eligible remote
        // meal needs it. Contact feeding and inedible residents need no rays.
        let sight = std::cell::OnceCell::new();
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
                if (policy.sight && d > policy.search)
                    || !self.food_queries(actor, target, feed, body.place) {
                    return None;
                }
                let detected = !policy.sight || d <= policy.reach || sight
                    .get_or_init(|| senses::Sight::read(self, actor, policy.search))
                    .as_ref().is_some_and(|s| s.detects(self, target).is_some());
                (detected && self.owned_process_ready(actor, Some(target), feed)).then_some((target, d))
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
            if f.reachable {
                return Some(self.food_route(actor, &p.id).ok()
                    .and_then(|found| found.route.map(|r| r.target)));
            }
            let food = self.food_candidates(actor, feed, f);
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
            .food_candidates(actor, p, f)
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
                .food_candidates(actor, feed, f)
                .iter()
                .any(|(id, d)| Some(*id) == target && *d > f.reach && *d <= f.search)
    }
}
