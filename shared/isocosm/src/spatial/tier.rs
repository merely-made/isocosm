// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line's physical side (the execution plan's E5; D25, D26, D32).
//! Where a world has attention windows, near is attention's roots (the
//! played, pinned and cared-for, and every member posed in a camera's
//! frame) and those still inside the collection buffer; a member that is
//! neither gives up its pose and patch at the end of an advance, and from
//! then forages at site grain. A world without windows keeps every pose.

use crate::{Simulation, attention::Region, operations::Window, schema::*, spatial::{Pose, embodiment}};
use std::collections::BTreeMap;

/// Foragers per site and feed process (D35), read once an advance.
pub(crate) type FarFood = BTreeMap<(Id, Key), u64>;

impl Simulation {
    /// Whether a member is near: every member is, in a world without
    /// attention windows; otherwise a root or one still buffered.
    pub fn near(&self, id: Id) -> bool {
        let w = &self.state.attention;
        w.is_empty() || w.foreground(id, 1) || w.buffered(self.state.tick).contains(&id)
    }

    /// Unplaces every posed member that is neither near nor buffered.
    /// Whether this advance may move members on or off the physical path:
    /// any but an older spatial command, whose record publishes no tier.
    pub(super) fn tiering(&self) -> bool {
        !self.spatial_resolutions.enabled || self.spatial_resolutions.birth_enabled
    }

    pub(crate) fn unplace_far(&mut self) {
        let windows = &self.state.attention;
        if windows.is_empty() || !self.tiering() {
            return;
        }
        let buffered = windows.buffered(self.state.tick);
        let far: Vec<Id> = self
            .state
            .spatial_poses
            .keys()
            .copied()
            .filter(|id| !windows.foreground(*id, 1) && !buffered.contains(id))
            .collect();
        for id in far {
            if let Some(j) = &mut self.journal {
                let population = &self.state.population;
                if let Some((&first, _)) = population.groups.range(..=id).next_back() {
                    for key in [first, id, id.saturating_add(1)] {
                        j.group(key, population.groups.get(&key));
                    }
                }
            }
            self.state.spatial_poses.remove(&id);
            if let Ok(e) = self.state.population.lift(id) {
                e.patch = None;
            }
            if self.spatial_resolutions.enabled {
                self.spatial_resolutions.order.push(super::Publication::Unplaced(id));
            }
        }
    }

    /// Draws far members of a frame's site into the frame until it holds
    /// its share of the site's members by area, each placed by the arrival
    /// policy inside the frame (D32).
    pub(crate) fn draw_into_frames(&mut self) {
        let Some(t) = self.genesis.rules.spatial_templates.clone() else {
            return;
        };
        if !self.tiering() {
            return;
        }
        for frame in self.state.attention.frames() {
            let Region::Frame { site, min, max } = frame else { continue };
            let area = |a: [i64; 2], b: [i64; 2]| ((b[0] - a[0]).max(0) * (b[1] - a[1]).max(0)) as u128;
            let (whole, seen) = (area(t.window.min, t.window.max), area(min, max));
            if whole == 0 {
                continue;
            }
            let s = &self.state;
            let mut members: Vec<Id> = Vec::new();
            for (&first, g) in &s.population.groups {
                let e = &g.entity;
                if e.alive && e.body.is_some() && e.journey.is_none() && e.resident_site() == Some(site)
                    && !crate::directing::is_participant(e) {
                    members.extend(first..first + g.count);
                }
            }
            let inside = members.iter().filter(|id| s.spatial_poses.get(id)
                .is_some_and(|r| frame.frames(r.site, r.pose.at))).count() as u128;
            let share = (members.len() as u128 * seen + whole / 2) / whole;
            let want = share.saturating_sub(inside) as usize;
            if want == 0 {
                continue;
            }
            let (seed, tick) = (self.genesis.seed, s.tick);
            let mut far: Vec<Id> = members.into_iter().filter(|id| !s.spatial_poses.contains_key(id)).collect();
            far.sort_by_key(|id| (crate::draw(seed, "frame-draw", &[*id, tick]), *id));
            let mut placed: BTreeMap<Id, Pose> = s.spatial_poses.iter()
                .filter(|(_, r)| r.site == site).map(|(id, r)| (*id, r.pose.clone())).collect();
            let within = Window { min, max, depth: t.window.depth };
            let mut drawn = Vec::new();
            // At most twice the shortfall tried an advance: a frame on hard
            // ground fills over several advances, not by searching for every
            // far member each tick.
            for id in far.into_iter().take(want.saturating_mul(2)) {
                if drawn.len() == want {
                    break;
                }
                let Ok(pose) = embodiment::place_within(self, &t, id, site, &within, &placed) else { continue };
                if !frame.frames(site, pose.at) {
                    continue;
                }
                let Ok(terrain) = self.site_geometry(site, &embodiment::site_window(&t, pose.at), t.places) else { continue };
                let Some(patch) = terrain.places.place_at(pose.at) else { continue };
                placed.insert(id, pose.clone());
                drawn.push((id, pose, patch));
            }
            for (id, pose, patch) in drawn {
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
                        self.spatial_resolutions.order.push(super::Publication::Drawn { actor: id, site, pose, patch, arrived });
                    }
                }
            }
        }
    }

    /// The far-food table, where a world has attention windows and
    /// foraging: each feed's foragers, by site.
    pub(crate) fn far_food_table(&self) -> Option<FarFood> {
        let t = self.genesis.rules.spatial_templates.as_ref()?;
        if self.state.attention.is_empty() || t.foraging.is_empty() {
            return None;
        }
        let rules = &self.genesis.rules;
        let mut table = FarFood::new();
        for f in t.foraging.values() {
            let Some(p) = rules.processes.get(&f.feed) else { continue };
            let needs: Vec<&Key> = p.requires.iter().filter_map(|q| match q {
                crate::rules::Query::Trait { who: crate::rules::Binding::Actor, key } => Some(key),
                _ => None,
            }).collect();
            for g in self.state.population.groups.values() {
                let e = &g.entity;
                let Some(site) = e.resident_site() else { continue };
                if e.alive && needs.iter().all(|k| e.traits.contains(*k)) {
                    *table.entry((site, f.feed.clone())).or_default() += g.count;
                }
            }
        }
        Some(table)
    }

    /// Whether a far forager finds food at this decision (D35): with chance
    /// M / (M + K), M the prey matter its own intake admits at its site
    /// (D37) shared among that feed's foragers there, K one meal's worth (its
    /// meal times its decision period) times the policy's half-saturation
    /// constant.
    pub(crate) fn far_meal(&self, actor: Id, feed: &str) -> bool {
        let Some(table) = &self.far_food else { return true };
        let Some(e) = self.state.population.get(actor) else { return false };
        let Some(site) = e.resident_site() else { return false };
        let rules = &self.genesis.rules;
        let Some(p) = rules.processes.get(feed) else { return false };
        let prey = self.prey_matter(actor, p);
        if prey == 0 {
            return false;
        }
        let foragers = table.get(&(site, feed.to_string())).copied().unwrap_or(0);
        let meal = p.effects.iter().find_map(|x| match x {
            crate::rules::Effect::Eat { amount: crate::rules::Amount::Fixed(n), .. } => Some(*n),
            _ => None,
        }).unwrap_or(1);
        let period = self.state.lineages.get(&e.lineage).and_then(|l| l.decides).unwrap_or(1);
        let half = rules.spatial_templates.as_ref().and_then(|t| t.foraging.values().find(|f| f.feed == feed))
            .map_or(1, |f| f.far_half);
        let k = u128::from(meal.max(1) * period * half.max(1));
        let draw = crate::draw(self.genesis.seed, "far-meal", &[actor, self.state.tick]) % 1_000_000;
        u128::from(draw) < encounter(prey / u128::from(foragers.max(1)), k)
    }
}

/// A far forager's chance in a million to find food (D35): its share of the
/// prey matter against that share plus K.
pub(crate) fn encounter(share: u128, k: u128) -> u128 {
    share * 1_000_000 / (share + k.max(1))
}
