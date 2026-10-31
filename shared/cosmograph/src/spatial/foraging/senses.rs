// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Physical detection over the same pass-start bodies used by native choices.
//! This is a current reading, not a second knowledge or position store.
use super::*;
use crate::rules::Role;
use std::sync::Arc;

/// Generated ecology carries function cells but has not declared organ systems.
/// New physical foraging adopts the existing sensory networks, with the default
/// nervous system only where the world declares none. Existing saves skip this.
pub(crate) fn declare(g: &mut crate::simulation::Genesis) {
    if !g
        .rules
        .systems
        .values()
        .any(|s| s.routes("function:sense", Role::Source))
    {
        let nervous = crate::rules::default_systems()
            .remove("system:nervous")
            .expect("native nervous system");
        g.rules
            .systems
            .entry("system:nervous".into())
            .or_insert(nervous);
    }
    let sensory: BTreeMap<_, _> = g
        .rules
        .systems
        .iter()
        .filter(|(_, s)| s.routes("function:sense", Role::Source))
        .map(|(key, s)| (key.clone(), s.clone()))
        .collect();
    for lineage in g.lineages.values_mut() {
        if let Some(d) = &mut lineage.development {
            for (key, s) in &sensory {
                d.systems.entry(key.clone()).or_insert_with(|| s.clone());
            }
        }
    }
    for group in g.population.groups.values_mut() {
        if group.entity.body.is_none() {
            continue;
        }
        let e = group.entity_mut();
        for (key, s) in &sensory {
            if crate::systems::realizes(e, s) {
                e.systems.entry(key.clone()).or_insert_with(|| s.clone());
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detection {
    pub eye: [i64; 3],
    pub cell: [i64; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodDetection {
    Contact,
    Sight(Detection),
}

pub(super) struct Sight {
    site: Id,
    range: u64,
    eyes: Vec<[i64; 3]>,
    terrain: Arc<super::super::site_geometry::SiteGeometry>,
}

impl Sight {
    pub(super) fn read(sim: &Simulation, actor: Id, range: u64) -> Option<Self> {
        let _span = crate::profile::Span::new("foraging.sight.read");
        let body = sim.body_at_start(actor)?;
        if !body.alive || !crate::systems::routes(body, "function:sense", Role::Source) {
            return None;
        }
        let at = sim.spatial_pose_at_start(actor)?;
        let doc = body.body.as_ref()?;
        let bounds = embodiment::part_boxes(doc, &at.pose).ok()?;
        let mut eyes = Vec::new();
        for (id, _) in body
            .living()
            .filter(|(_, p)| p.cells.get("function:sense").is_some_and(|n| *n > 0))
        {
            let point = if let Some([lo, hi]) = bounds.get(&id) {
                [0, 1, 2].map(|k| lo[k] + (hi[k] - lo[k] - 1) / 2)
            } else {
                let geo = body.geo(id)?;
                let local = doc.place(id, geo.half_extent)?.map(i64::from);
                let offset = super::super::birth::rotate(at.pose.yaw, local).ok()?;
                let point = [0, 1, 2].map(|k| at.pose.at[k].checked_add(offset[k]));
                [point[0]?, point[1]?, point[2]?]
            };
            eyes.push(point);
        }
        if eyes.is_empty() {
            return None;
        }
        let t = sim.genesis.rules.spatial_templates.as_ref()?;
        let window = embodiment::site_window(t, at.pose.at);
        let terrain = sim.site_geometry(at.site, &window, t.places).ok()?;
        eyes.retain(|eye| clear(&terrain.volume, *eye) && terrain.places.place_at(*eye).is_some());
        (!eyes.is_empty()).then_some(Self {
            site: at.site,
            range,
            eyes,
            terrain,
        })
    }

    pub(super) fn detects(&self, sim: &Simulation, target: Id) -> Option<Detection> {
        let _span = crate::profile::Span::new("foraging.sight.detect");
        if sim.spatial_pose_at_start(target)?.site != self.site {
            return None;
        }
        let boxes = sim.placed_boxes(target)?;
        for &eye in &self.eyes {
            for &[lo, hi] in &boxes {
                // Closest occupied cell, centre, then corners. A visible leaf
                // or shell can reveal food whose root is behind terrain.
                let nearest = [0, 1, 2].map(|k| eye[k].clamp(lo[k], hi[k] - 1));
                let centre = [0, 1, 2].map(|k| lo[k] + (hi[k] - lo[k] - 1) / 2);
                let corners = (0..8).map(|bits| {
                    [0, 1, 2].map(|k| {
                        if bits & (1 << k) == 0 {
                            lo[k]
                        } else {
                            hi[k] - 1
                        }
                    })
                });
                for cell in [nearest, centre].into_iter().chain(corners) {
                    if (0..3).all(|k| eye[k].abs_diff(cell[k]) <= self.range)
                        && clear(&self.terrain.volume, cell)
                        && self
                            .terrain
                            .places
                            .sees(&self.terrain.volume, eye, cell, self.range)
                    {
                        return Some(Detection { eye, cell });
                    }
                }
            }
        }
        None
    }
}

fn clear(volume: &isometer_space::volume::Volume, cell: [i64; 3]) -> bool {
    volume
        .material(cell)
        .is_some_and(|m| m == volume.materials.air || m == volume.materials.water)
}

impl Simulation {
    /// Physical detection only; appetite, diet and process readiness remain the
    /// existing meal selector's. No observation or knowledge fact is added.
    pub fn food_detection(&self, actor: Id, target: Id, walk: &str) -> Option<FoodDetection> {
        if !self.body_at_start(actor)?.alive {
            return None;
        }
        let f = self.foraging_policy(walk)?;
        if !f.sight {
            return None;
        }
        let gap = self.food_gap(actor, target)?;
        if gap <= f.reach {
            return Some(FoodDetection::Contact);
        }
        if gap > f.search {
            return None;
        }
        Sight::read(self, actor, f.search)?
            .detects(self, target)
            .map(FoodDetection::Sight)
    }
}
