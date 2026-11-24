// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::ecology::soil::SeedRain;

/// Land a seed on supported, unoccupied ground after the biological birth.
impl Simulation {
    pub(crate) fn land_seed(
        &mut self,
        parent: Id,
        child: Id,
        site: Id,
        rain: &SeedRain,
    ) -> Result<()> {
        let t = self
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("seed bed needs terrain")?;
        let root = self
            .state()
            .spatial_poses
            .get(&parent)
            .filter(|r| r.site == site)
            .ok_or("seed source has no root position")?;
        let mut here = root.pose.clone();
        let spread = u64::from(rain.jitter);
        for (axis, drift) in rain.drift.iter().enumerate() {
            let offset = crate::draw(
                self.genesis().dynamics_seed(),
                "local-seed-rain",
                &[child, axis as u64],
            ) % (spread * 2 + 1);
            here.at[axis * 2] += i64::from(*drift) + offset as i64 - spread as i64;
            if !(t.window.min[axis]..t.window.max[axis]).contains(&here.at[axis * 2]) {
                return Err("seed left the surveyed bed".into());
            }
        }
        let placed = self
            .state()
            .spatial_poses
            .iter()
            .filter(|(id, r)| {
                r.site == site
                    && self.state().population.get(**id).is_some_and(|e| {
                        e.alive && e.resident_site() == Some(site) && e.arrived == r.arrived
                    })
            })
            .map(|(&id, r)| (id, r.pose.clone()))
            .collect();
        let landing = embodiment::settle(self, t, child, site, &here, &placed)?;
        let terrain =
            self.site_geometry(site, &embodiment::site_window(t, landing.at), t.places)?;
        let patch = terrain
            .places
            .place_at(landing.at)
            .ok_or("no seed landing patch")?;
        self.state.population.lift(child)?.patch = Some(patch);
        self.keep_spatial_pose(child, site, landing.clone());
        if self.spatial_resolutions.enabled {
            let arrived = self.state.population.get(child).unwrap().arrived;
            self.spatial_resolutions
                .order
                .push(crate::spatial::Publication::Drawn {
                    actor: child,
                    site,
                    pose: landing,
                    patch,
                    arrived,
                });
        }
        Ok(())
    }
}
