// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A generated world's declared spawn and arrival templates, bound to its actual
//! residents and saved through the ordinary authored provider (889). No cell is
//! chosen here: each stand and crossing was declared by the generator.

use super::*;
use isocosm::{Command, directing::is_participant};
use isometer_space::places::Places;

impl Game {
    /// Seats the first living embodied member of each of a site's cohorts, in
    /// id order, one per declared stand, and registers every declared crossing
    /// for them. A cohort beyond the site's stands, and a site with none, stays
    /// at site grain. Returns whether the world declares templates.
    pub fn found_generated(&mut self, session: &mut Session) -> Result<bool, String> {
        let Some(templates) = session.sim.genesis().rules.spatial_templates.clone() else { return Ok(false); };
        if self.driver.is_some() || !self.members.is_empty() {
            return Err("generated templates seat a world with no saved physical members".into());
        }
        if templates.version >= 2 { return self.found_embodied(session, &templates); }
        let mut native = session.clone();
        let mut placements = Vec::new();
        let mut arrivals = Vec::new();
        for (&site, stands) in &templates.stands {
            if stands.is_empty() { continue; }
            let volume = native.sim.volume(site, templates.window.min, templates.window.max, templates.window.depth)?;
            let places = Places::derive(&volume, templates.places)?;
            let residents: Vec<Id> = native.sim.state().population.groups.iter()
                .filter(|(_, g)| g.entity.alive && g.entity.body.is_some() && !is_participant(&g.entity)
                    && g.entity.resident_site() == Some(site))
                .map(|(&first, _)| first).collect();
            for (id, stand) in residents.into_iter().zip(stands) {
                let patch = places.place_at(stand.at).ok_or("declared stand has no realized patch")?;
                native.command(Command::Patch { entity: id, patch: Some(patch) })?;
                let member = native.sim.state().population.get(id).ok_or("seated member is absent")?;
                let eye = std::array::from_fn(|k| stand.at[k] + templates.eye[k]);
                placements.push(Placement { actor: EntityHandle(id), site: PlaceHandle(site), patch,
                    body_revision: member.body_revision, terrain_revision: native.sim.terrain_revision(),
                    pose: Pose { at:stand.at, yaw:stand.yaw, feet:stand.feet.clone() }, eye, window: templates.window.clone(),
                    operations: BTreeMap::new(), places: Some(templates.places),
                    resident_since: Some(member.arrived) });
                for crossing in &templates.crossings {
                    let to = &crossing.stand;
                    arrivals.push(ArrivalInput { actor: EntityHandle(id), from: PlaceHandle(crossing.from),
                        to: PlaceHandle(crossing.to), pose: Pose { at:to.at, yaw:to.yaw, feet:to.feet.clone() },
                        eye: std::array::from_fn(|k| to.at[k] + templates.eye[k]), window: templates.window.clone(),
                        places: templates.places, operations: BTreeMap::new() });
                }
            }
        }
        let profile = Profile { version: 4, steps: BTreeMap::new(), arrivals, births: Vec::new() };
        let mut next = self.clone();
        next.configure(&native, &Authored { placements, profile })?;
        *session = native;
        *self = next;
        Ok(true)
    }
}
