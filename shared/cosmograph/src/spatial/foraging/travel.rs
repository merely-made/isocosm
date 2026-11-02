// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Out of view a forager travels (case 1084; doctrine Bodies 16, Stack 12).
//! In a world with a walking speed, a posed forager no round walks chooses
//! its meal at the place graph's grain: among the food it detects, the one
//! nearest by the route over its site's Places that its body fits, and no
//! nearer than its gap. It
//! departs with the route's travel time at its speed and keeps its stance
//! until it arrives, then stands at its meal, or near where the meal stood
//! when it is gone. No step or route search runs on the way. Bodies a round
//! walks keep the game's steps.

use super::*;

/// A forager on its way to a meal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Travel {
    pub target: Id,
    /// Where the meal stood when the forager left.
    pub at: [i64; 3],
    pub arrives: Tick,
}

impl Simulation {
    /// Whether this world's out-of-view foragers travel.
    pub(crate) fn travels(&self) -> bool {
        self.genesis.rules.speed.is_some() && self.foraging_enabled()
    }

    /// Whether `id` travels: in a travelling world, posed and in no
    /// participant's round foreground.
    pub(crate) fn travels_out_of_view(&self, id: Id) -> bool {
        self.travels() && self.state.spatial_poses.contains_key(&id) && !self.in_view().contains(&id)
    }

    /// The bodies some participant's round walks.
    pub(crate) fn in_view(&self) -> BTreeSet<Id> {
        self.state.attention.participants().into_iter().flat_map(|p| self.foreground(p)).collect()
    }

    pub(crate) fn travelling(&self, id: Id) -> bool {
        self.state.spatial_poses.get(&id).is_some_and(|r| r.travel.is_some())
    }

    /// The cost of the cheapest route between `actor`'s place and
    /// `target`'s that its walking body fits, in base units.
    fn place_route(&self, actor: Id, target: Id, walk: &str) -> Option<u64> {
        let from = self.spatial_pose_at_start(actor)?;
        let to = self.spatial_pose_at_start(target)?;
        if from.site != to.site {
            return None;
        }
        let t = self.genesis.rules.spatial_templates.as_ref()?;
        let profile = self.genesis.rules.step_profiles.get(walk)?;
        let terrain = self.site_geometry(from.site, &embodiment::site_window(t, from.pose.at), profile.places).ok()?;
        let places = &terrain.places;
        let start = self.body_at_start(actor)?.patch?;
        let end = places.place_at(to.pose.at)?;
        Some(places.route(start, end, &profile.body)?.0)
    }

    /// A traveller's meal: the detected food beyond reach and within
    /// search, nearest by place route, then by identity.
    pub(super) fn travel_target(&self, actor: Id, walk: &str, f: &Foraging) -> Option<Id> {
        let feed = &self.genesis.rules.processes[&f.feed];
        let food = self.food_candidates(actor, feed, f);
        if food.iter().any(|(_, d)| *d <= f.reach) {
            return None;
        }
        food.into_iter()
            .filter(|(_, d)| *d <= f.search)
            .filter_map(|(id, d)| Some((self.place_route(actor, id, walk)?.max(d), id)))
            .min()
            .map(|(_, id)| id)
    }

    /// The journey a chosen walk to `target` makes: its route's cost at
    /// the body's speed, in whole ticks.
    pub(crate) fn departure(&self, actor: Id, target: Id, walk: &str) -> Result<Travel> {
        let speed = self.genesis.rules.speed.ok_or("travel needs a world speed")?;
        // Within a place the graph charges nothing, so the way is at least
        // the gap to the meal.
        let gap = self.food_gap(actor, target).ok_or("chosen meal is not posed")?;
        let cost = self.place_route(actor, target, walk).ok_or("no route to the chosen meal")?.max(gap);
        let doc = self.body_at_start(actor).and_then(|b| b.body.as_ref()).ok_or("traveller has no body")?;
        let tick = self.genesis.rules.tick_microseconds.unwrap_or(crate::rules::MINUTE_MICROSECONDS);
        let ticks = crate::speed::Speed::ticks(cost, speed.of(doc), tick).ok_or("the traveller cannot move")?;
        let at = self.spatial_pose_at_start(target).ok_or("chosen meal is not posed")?.pose.at;
        Ok(Travel { target, at, arrives: self.state.tick.saturating_add(ticks) })
    }

    pub(crate) fn commit_travel(&mut self, travel: Option<(Id, Travel)>) {
        let Some((id, travel)) = travel else { return };
        if let Some(frame) = &mut self.frame {
            frame.spatial_poses.entry(id).or_insert_with(|| self.state.spatial_poses.get(&id).cloned());
        }
        if let Some(r) = self.state.spatial_poses.get_mut(&id) {
            r.travel = Some(travel);
        }
    }

    /// Lands every traveller due by now, at the end of an advance.
    pub(crate) fn arrive_travellers(&mut self) {
        let Some(t) = self.genesis.rules.spatial_templates.clone() else { return };
        if !self.travels() || !self.tiering() {
            return;
        }
        let tick = self.state.tick;
        let due: Vec<(Id, Id, isometer_core::Yaw, Travel)> = self.state.spatial_poses.iter()
            .filter_map(|(&id, r)| Some((id, r.site, r.pose.yaw, r.travel.clone().filter(|v| v.arrives <= tick)?)))
            .collect();
        let reach = t.foraging.values().map(|f| f.reach).max().unwrap_or(0);
        for (id, site, yaw, travel) in due {
            if let Some(r) = self.state.spatial_poses.get_mut(&id) {
                r.travel = None;
            }
            let Some(e) = self.state.population.get(id) else { continue };
            if !e.alive || e.body.is_none() || e.resident_site() != Some(site) {
                continue;
            }
            let placed: BTreeMap<Id, Pose> = self.state.spatial_poses.iter()
                .filter(|(other, r)| **other != id && r.site == site).map(|(id, r)| (*id, r.pose.clone())).collect();
            let Ok(pose) = embodiment::arrive_near(self, &t, id, site, travel.target, travel.at, reach, yaw, &placed) else { continue };
            let Ok(terrain) = self.site_geometry(site, &embodiment::site_window(&t, pose.at), t.places) else { continue };
            let Some(patch) = terrain.places.place_at(pose.at) else { continue };
            if let Some(j) = &mut self.journal {
                let population = &self.state.population;
                if let Some((&first, _)) = population.groups.range(..=id).next_back() {
                    for key in [first, id, id.saturating_add(1)] {
                        j.group(key, population.groups.get(&key));
                    }
                }
            }
            if let Ok(e) = self.state.population.lift(id) {
                e.patch = Some(patch);
                let arrived = e.arrived;
                self.keep_spatial_pose(id, site, pose.clone());
                if self.spatial_resolutions.enabled {
                    self.spatial_resolutions.order.push(Publication::Drawn { actor: id, site, pose, patch, arrived });
                }
            }
        }
    }
}
