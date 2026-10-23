// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native resident poses and opt-in food-directed steps. Older spatial worlds
//! keep their site-grain feeding and game-owned root contract.

use super::*;
pub const FOOD_SEARCH: u64 = 32;
use crate::{rules::Process, simulation::Genesis};

mod food;
mod route;
mod senses;
pub use route::geometry;
pub use senses::{Detection, FoodDetection};
pub(crate) use senses::declare as declare_senses;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foraging {
    /// Existing feeding process whose selector, intake and needs walking serves.
    pub feed: Key,
    /// Maximum occupied-box gap for feeding, in base cells.
    pub reach: u64,
    /// Maximum food-search distance, in the same cells.
    pub search: u64,
    /// Maximum supported root positions examined per route search.
    pub nodes: usize,
    /// Remote food needs living sensory tissue and a clear terrain ray.
    /// Absent in older recorded rules, which keep their original choices.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sight: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resident {
    pub site: Id,
    pub arrived: Tick,
    pub pose: Pose,
}

pub(super) fn validate(t: &Templates, rules: &Rules) -> Result<()> {
    if t.version < 3 && !t.foraging.is_empty() {
        return Err("food-directed walking needs version-three spatial templates".into());
    }
    let mut feeds = BTreeSet::new();
    for (walk, f) in &t.foraging {
        let step = rules
            .step_profiles
            .get(walk)
            .ok_or("foraging step profile absent")?;
        let p = rules
            .processes
            .get(walk)
            .ok_or("foraging walking process absent")?;
        let feed = rules
            .processes
            .get(&f.feed)
            .ok_or("foraging feeding process absent")?;
        if step.process != *walk
            || p.target != feed.target
            || p.target.is_none()
            || p.causation != Causation::Choice
            || feed.causation != Causation::Choice
            || feed.port_feeding.is_none()
            || !feeds.insert(&f.feed)
            || f.search <= f.reach
            || f.search > 128
            || f.nodes == 0
            || f.nodes > 1024
            || p.period.is_none()
            || feed.period.is_none()
        {
            return Err(
                "foraging needs one bounded step per existing targeted feeding choice".into(),
            );
        }
    }
    Ok(())
}

pub(crate) fn founding_poses(g: &Genesis) -> BTreeMap<Id, Resident> {
    let Some(t) = g
        .rules
        .spatial_templates
        .as_ref()
        .filter(|t| t.version >= 3)
    else {
        return BTreeMap::new();
    };
    t.stands
        .iter()
        .flat_map(|(&site, stands)| {
            stands.iter().filter_map(move |stand| {
                let id = stand.actor?;
                Some((
                    id,
                    Resident {
                        site,
                        arrived: g.population.get(id)?.arrived,
                        pose: Pose { at: stand.at, yaw: stand.yaw, feet: stand.feet.clone() },
                    },
                ))
            })
        })
        .collect()
}

/// Manhattan gap between occupied native part boxes. Touching boxes have gap
/// zero; negative coordinates and separated attachment parts retain their shape.
pub fn gap(a: &[embodiment::Box3], b: &[embodiment::Box3]) -> u64 {
    a.iter()
        .flat_map(|[lo, hi]| {
            b.iter().map(move |[other_lo, other_hi]| {
                (0..3)
                    .map(|k| {
                        if hi[k] < other_lo[k] {
                            hi[k].abs_diff(other_lo[k])
                        } else if other_hi[k] < lo[k] {
                            other_hi[k].abs_diff(lo[k])
                        } else {
                            0
                        }
                    })
                    .fold(0u64, u64::saturating_add)
            })
        })
        .min()
        .unwrap_or(u64::MAX)
}

impl Simulation {
    pub fn foraging_enabled(&self) -> bool {
        self.genesis
            .rules
            .spatial_templates
            .as_ref()
            .is_some_and(|t| t.version >= 3)
    }

    pub fn foraging_policy(&self, walk: &str) -> Option<&Foraging> {
        self.genesis
            .rules
            .spatial_templates
            .as_ref()?
            .foraging
            .get(walk)
    }

    pub(crate) fn keep_spatial_pose(&mut self, id: Id, site: Id, pose: Pose) {
        if let Some(frame) = &mut self.frame {
            frame
                .spatial_poses
                .entry(id)
                .or_insert_with(|| self.state.spatial_poses.get(&id).cloned());
        }
        let arrived = self
            .state
            .population
            .get(id)
            .expect("published spatial actor")
            .arrived;
        self.state.spatial_poses.insert(
            id,
            Resident {
                site,
                arrived,
                pose,
            },
        );
    }

    /// The same immutable pass-start source as native body/target selection.
    pub(crate) fn spatial_pose_at_start(&self, id: Id) -> Option<&Resident> {
        let kept = self.frame.as_ref().and_then(|f| f.spatial_poses.get(&id));
        let at = match kept {
            Some(at) => at.as_ref(),
            None => self.state.spatial_poses.get(&id),
        }?;
        let body = self.body_at_start(id)?;
        (body.resident_site() == Some(at.site)
            && body.arrived == at.arrived
            && body.body.is_some()
            && body.patch.is_some())
        .then_some(at)
    }

    pub(crate) fn placed_boxes(&self, id: Id) -> Option<Vec<embodiment::Box3>> {
        let at = self.spatial_pose_at_start(id)?;
        embodiment::boxes(self.body_at_start(id)?.body.as_ref()?, &at.pose).ok()
    }

    /// Collision admission reads positions already published by earlier actors,
    /// while choosing a target continues to read the immutable pass start.
    pub(crate) fn current_placed_boxes(&self, id: Id, site: Id) -> Option<Vec<embodiment::Box3>> {
        let at = self.state.spatial_poses.get(&id)?;
        let member = self.state.population.get(id)?;
        if at.site != site || member.resident_site() != Some(site) || member.arrived != at.arrived {
            return None;
        }
        embodiment::boxes(member.body.as_ref()?, &at.pose).ok()
    }

    pub(crate) fn spatial_destination_clear(&self, actor: Id, site: Id, pose: &Pose) -> bool {
        let Some(document) = self
            .state
            .population
            .get(actor)
            .and_then(|e| e.body.as_ref())
        else {
            return false;
        };
        let Ok(boxes) = embodiment::boxes(document, pose) else {
            return false;
        };
        self.state
            .spatial_poses
            .keys()
            .filter(|id| **id != actor)
            .filter_map(|id| self.current_placed_boxes(*id, site))
            .all(|occupied| !embodiment::intersects(&boxes, &occupied))
    }

    pub(crate) fn feeding_reachable(&self, actor: Id, target: Id, p: &Process) -> bool {
        let Some(t) = self.genesis.rules.spatial_templates.as_ref() else {
            return true;
        };
        let Some(f) = t.foraging.values().find(|f| f.feed == p.id) else {
            return true;
        };
        if !self.state.spatial_poses.contains_key(&actor) {
            return true;
        }
        self.food_gap(actor, target).is_some_and(|d| d <= f.reach)
    }

    pub(crate) fn food_gap(&self, actor: Id, target: Id) -> Option<u64> {
        let a = self.spatial_pose_at_start(actor)?;
        let b = self.spatial_pose_at_start(target)?;
        if a.site != b.site {
            return None;
        }
        Some(gap(&self.placed_boxes(actor)?, &self.placed_boxes(target)?))
    }
}
