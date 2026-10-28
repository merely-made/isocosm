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

impl Simulation {
    /// Whether a member is near: every member is, in a world without
    /// attention windows; otherwise a root or one still buffered.
    pub fn near(&self, id: Id) -> bool {
        let w = &self.state.attention;
        w.is_empty() || w.foreground(id, 1) || w.buffered(self.state.tick).contains(&id)
    }

    /// Unplaces every posed member that is neither near nor buffered.
    pub(crate) fn unplace_far(&mut self) {
        let windows = &self.state.attention;
        if windows.is_empty() {
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
        }
    }

    /// Draws far members of a frame's site into the frame until it holds
    /// its share of the site's members by area, each placed by the arrival
    /// policy inside the frame (D32).
    pub(crate) fn draw_into_frames(&mut self) {
        let Some(t) = self.genesis.rules.spatial_templates.clone() else {
            return;
        };
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
                    self.keep_spatial_pose(id, site, pose);
                }
            }
        }
    }
}
