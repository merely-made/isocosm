// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::spatial::{ArrivalRequest, ArrivalGeometry, ArrivalResolution, ArrivalReport};

impl Game {
    pub(super) fn arrival_inputs(&self, session: &Session) -> Result<Vec<ArrivalResolution>, String> {
        let Some(driver) = &self.driver else { return Ok(Vec::new()); };
        let end = session.sim.state().tick.checked_add(1).ok_or("spatial clock overflows")?;
        let mut out = Vec::new();
        for input in &driver.arrival_inputs()? {
            let Some(member) = session.sim.state().population.get(input.actor.0) else { continue; };
            let Some(journey) = &member.journey else { continue; };
            if journey.from != input.from.0 || journey.to != input.to.0
                || journey.arrives < session.sim.state().tick || journey.arrives > end { continue; }
            out.push(ArrivalResolution {
                request: ArrivalRequest { actor: input.actor.0, from: journey.from, to: journey.to,
                    departed: journey.departed, arrives: journey.arrives, body_revision: member.body_revision,
                    terrain_revision: session.sim.terrain_revision() },
                geometry: ArrivalGeometry { window: input.window.clone(), pose: isocosm::spatial::Pose {
                    at: input.pose.at, yaw: input.pose.yaw }, eye: input.eye, places: input.places },
            });
        }
        Ok(out)
    }

    pub(super) fn arrival_placement(&self, input: &ArrivalResolution, patch: PlaceId) -> Result<Placement, String> {
        let authored = self.driver.as_ref().ok_or("arrival driver absent")?.arrival_inputs()?;
        let held = authored.iter().find(|held| held.actor.0 == input.request.actor && held.from.0 == input.request.from
            && held.to.0 == input.request.to).ok_or("accepted arrival has no authored destination")?;
        if input.geometry.window != held.window || input.geometry.pose.at != held.pose.at
            || input.geometry.pose.yaw != held.pose.yaw || input.geometry.eye != held.eye
            || input.geometry.places != held.places
        { return Err("native arrival geometry differs from the saved authored destination".into()); }
        Ok(Placement { actor: held.actor, site: held.to, patch, body_revision: input.request.body_revision,
            terrain_revision: input.request.terrain_revision.clone(), pose: held.pose.clone(), eye: held.eye,
            window: held.window.clone(), operations: held.operations.clone(), places: Some(held.places),
            resident_since: Some(input.request.arrives) })
    }

    pub(super) fn apply_arrivals(&mut self, inputs: &[ArrivalResolution], reports: &[ArrivalReport])
        -> Result<Vec<profile::Transition>, String> {
        let mut changes = Vec::new();
        for report in reports {
            let Ok(patch) = &report.patch else { continue; };
            let input = inputs.iter().find(|r| r.request.actor == report.actor).ok_or("arrival input missing")?;
            let after = self.arrival_placement(input, *patch)?;
            let sight_range = self.arrival_sight_range(report.actor);
            changes.push(profile::Transition { actor: report.actor, before: self.members.get(&report.actor).cloned(),
                after: after.clone(), admission: profile::Admission::Arrival, sight_range });
            self.members.insert(report.actor, after);
            if let Some(range) = sight_range {
                self.sight.get_or_insert_with(|| SightConfig { version: 1, ranges: BTreeMap::new() })
                    .ranges.insert(report.actor, range);
                self.version = self.current_version();
            }
        }
        Ok(changes)
    }
}
