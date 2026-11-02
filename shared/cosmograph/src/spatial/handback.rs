// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round's foreground hands back each minute (cases 1080, 1084; doctrine
//! Processes 1). In view, a product steps bodies sixty times a real second,
//! finding paths, posing bodies and placing feet; at the tick boundary it
//! hands back where each body stands and what it touches. The sim checks
//! each against its own invariants before the minute's passes run: the body
//! and terrain it was resolved against, a supported stance inside its tile,
//! clear of every other resident, and contacts within feeding reach. An
//! accepted pose is published as a draw the game applies; a refused one
//! leaves the body where the sim had it, and the refusal is recorded. A
//! handback moves no matter: meals and wounds stay ordinary acts and
//! commands on the minute.

use super::*;

/// One body's end of a foreground minute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handback {
    pub actor: Id,
    pub site: Id,
    pub body_revision: u64,
    pub terrain_revision: Key,
    pub geometry: ArrivalGeometry,
    /// Residents the body touches, each within feeding reach of the pose.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandbackReport {
    pub actor: Id,
    pub patch: std::result::Result<PlaceId, String>,
}

impl Simulation {
    /// Checks and applies a minute's handbacks, in actor order.
    pub(crate) fn hand_back(&mut self, inputs: &[Handback]) -> Result<()> {
        let mut seen = BTreeSet::new();
        if !inputs.iter().all(|h| seen.insert(h.actor)) {
            return Err("duplicate foreground handback".into());
        }
        let matter = self.matter();
        let mut ordered: Vec<&Handback> = inputs.iter().collect();
        ordered.sort_by_key(|h| h.actor);
        for h in ordered {
            let patch = self.check_handback(h);
            if let Ok(patch) = &patch {
                self.publish_handback(h, *patch)?;
            }
            self.spatial_resolutions.handbacks.push(HandbackReport { actor: h.actor, patch });
        }
        if self.matter() != matter {
            return Err("a foreground handback moved matter".into());
        }
        Ok(())
    }

    fn check_handback(&self, h: &Handback) -> Result<PlaceId> {
        let e = self.state.population.get(h.actor).ok_or("handback actor absent")?;
        let held = self.state.spatial_poses.get(&h.actor).ok_or("handback actor has no pose")?;
        if !e.alive || e.body.is_none() || e.resident_site() != Some(h.site) || held.site != h.site
            || e.body_revision != h.body_revision || self.terrain_revision() != h.terrain_revision {
            return Err("handback source changed".into());
        }
        let g = &h.geometry;
        let patch = geometry::check_pose(self, e, h.site, g)?;
        let t = self.genesis.rules.spatial_templates.as_ref().ok_or("handback needs templates")?;
        let doc = e.body.as_ref().ok_or("handback body absent")?;
        let own = embodiment::boxes(doc, &g.pose)?;
        for (&other, r) in &self.state.spatial_poses {
            if other == h.actor || r.site != h.site {
                continue;
            }
            let Some(body) = self.state.population.get(other).and_then(|o| o.body.as_ref()) else { continue };
            if embodiment::intersects(&own, &embodiment::boxes(body, &r.pose)?) {
                return Err("handback pose intersects another resident".into());
            }
        }
        let reach = t.foraging.values().map(|f| f.reach).max().unwrap_or(0);
        for &c in &h.contacts {
            let r = self.state.spatial_poses.get(&c).filter(|r| r.site == h.site)
                .ok_or("handback contact is not posed at its site")?;
            let body = self.state.population.get(c).and_then(|o| o.body.as_ref())
                .ok_or("handback contact has no body")?;
            if foraging::gap(&own, &embodiment::boxes(body, &r.pose)?) > reach {
                return Err("handback contact is beyond feeding reach".into());
            }
        }
        Ok(patch)
    }

    fn publish_handback(&mut self, h: &Handback, patch: PlaceId) -> Result<()> {
        if let Some(j) = &mut self.journal {
            let population = &self.state.population;
            if let Some((&first, _)) = population.groups.range(..=h.actor).next_back() {
                for key in [first, h.actor, h.actor.saturating_add(1)] {
                    j.group(key, population.groups.get(&key));
                }
            }
        }
        let e = self.state.population.lift(h.actor)?;
        e.patch = Some(patch);
        let arrived = e.arrived;
        self.keep_spatial_pose(h.actor, h.site, h.geometry.pose.clone());
        self.spatial_resolutions.order.push(Publication::Drawn {
            actor: h.actor, site: h.site, pose: h.geometry.pose.clone(), patch, arrived });
        Ok(())
    }

    /// The handbacks the last advance checked, accepted or refused.
    pub fn spatial_handbacks(&self) -> &[HandbackReport] {
        &self.spatial_resolutions.handbacks
    }
}
