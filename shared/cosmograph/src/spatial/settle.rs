// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Resting bodies re-settle (the execution plan's D38). A stance is admitted
//! against the body's matter as it was posed; matter a body gains or loses
//! while it stands moves its centre, and a stance near the edge of its feet
//! stops balancing. At the end of an advance each posed member whose stance
//! no longer balances takes the nearest balanced one around it, by the search
//! arrivals use, published as a draw the game applies.

use super::*;
use crate::spatial::embodiment;

impl Simulation {
    pub(crate) fn settle_drifted(&mut self) {
        let Some(t) = self.genesis.rules.spatial_templates.clone() else {
            return;
        };
        // Only a matter-weighted balance moves with matter.
        if !embodiment::enabled(self) || !embodiment::walking_rules(self).is_some_and(|w| w.balance) || !self.tiering() {
            return;
        }
        let posed: Vec<(Id, Id, Pose)> = self.state.spatial_poses.iter()
            .map(|(&id, r)| (id, r.site, r.pose.clone())).collect();
        for (id, site, pose) in posed {
            let Some(e) = self.state.population.get(id) else { continue };
            if !e.alive || e.body.is_none() || e.resident_site() != Some(site) {
                continue;
            }
            let Ok(terrain) = self.site_geometry(site, &embodiment::site_window(&t, pose.at), t.places) else { continue };
            if embodiment::check_member(self, &terrain.volume, e, &pose).is_ok() {
                continue;
            }
            let placed: BTreeMap<Id, Pose> = self.state.spatial_poses.iter()
                .filter(|(_, r)| r.site == site).map(|(id, r)| (*id, r.pose.clone())).collect();
            let Ok(settled) = embodiment::settle(self, &t, id, site, &pose, &placed) else { continue };
            let Ok(terrain) = self.site_geometry(site, &embodiment::site_window(&t, settled.at), t.places) else { continue };
            let Some(patch) = terrain.places.place_at(settled.at) else { continue };
            let eye = std::array::from_fn(|k| settled.at[k] + t.eye[k]);
            if !embodiment::air(&terrain.volume, eye) || terrain.places.place_at(eye) != Some(patch) {
                continue;
            }
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
                self.keep_spatial_pose(id, site, settled.clone());
                if self.spatial_resolutions.enabled {
                    self.spatial_resolutions.order.push(Publication::Drawn { actor: id, site, pose: settled, patch, arrived });
                }
            }
        }
    }
}
