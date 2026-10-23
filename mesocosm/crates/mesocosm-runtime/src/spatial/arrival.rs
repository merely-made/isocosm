// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::spatial::{ArrivalRequest, ArrivalGeometry, ArrivalResolution, ArrivalReport};

impl Game {
    pub(super) fn arrival_inputs(&self, session: &Session) -> Result<Vec<ArrivalResolution>, String> {
        let _span = crate::profile::Span::new("arrival.inputs");
        if self.generated_driver() { return self.generated_arrivals(session); }
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
                geometry: ArrivalGeometry { window: input.window.clone(), pose: input.pose.clone(), eye: input.eye, places: input.places },
            });
        }
        Ok(out)
    }

    pub(super) fn arrival_placement(&self, input: &ArrivalResolution, patch: PlaceId) -> Result<Placement, String> {
        if self.generated_driver() {
            let driver = self.driver.as_ref().ok_or("generated arrival driver absent")?;
            let held = self.members.get(&input.request.actor);
            // The native report validates actual identity and journey, including
            // an initially unplaced founding member taken up at a death hold.
            let source = held.or_else(|| driver.initial.values().next()).ok_or("generated arrival source absent")?;
            // A failed physical landing can leave a historical pose several
            // native visits behind. Native arrival admission validates the
            // actual journey; this projection retains only its input contract.
            if (source.window != input.geometry.window && driver.profile.version != 6)
                || source.places.unwrap_or_default() != input.geometry.places {
                return Err("generated arrival differs from its source window and route".into());
            }
            return Ok(Placement { actor: EntityHandle(input.request.actor), site: PlaceHandle(input.request.to), patch,
                body_revision: input.request.body_revision, terrain_revision: input.request.terrain_revision.clone(),
                pose: input.geometry.pose.clone(), eye: input.geometry.eye,
                window: input.geometry.window.clone(), operations: BTreeMap::new(), places: Some(input.geometry.places),
                resident_since: Some(input.request.arrives) });
        }
        let authored = self.driver.as_ref().ok_or("arrival driver absent")?.arrival_inputs()?;
        let held = authored.iter().find(|held| held.actor.0 == input.request.actor && held.from.0 == input.request.from
            && held.to.0 == input.request.to).ok_or("accepted arrival has no authored destination")?;
        if input.geometry.window != held.window || input.geometry.pose != held.pose
            || input.geometry.eye != held.eye
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_arrival_can_replace_history_from_before_an_unplaced_visit() {
        let runtime = crate::Runtime::bench_placed(
            crate::Founded::generated(16_003_851_315_348_448_882, 60).unwrap(), 2).unwrap();
        let game = runtime.spatial();
        let held = game.members.values().next().expect("generated physical resident");
        let input = ArrivalResolution {
            request: ArrivalRequest { actor: held.actor.0, from: held.site.0 + 1,
                to: held.site.0, departed: 12, arrives: 16,
                body_revision: held.body_revision, terrain_revision: held.terrain_revision.clone() },
            geometry: ArrivalGeometry { window: held.window.clone(),
                pose: held.pose.clone(),
                eye: held.eye, places: held.places.unwrap_or_default() },
        };
        let next = game.arrival_placement(&input, held.patch).unwrap();
        assert_eq!(next.resident_since, Some(16));
        assert_eq!(next.pose, held.pose);
        let founder = runtime.sim().genesis().population.groups.iter()
            .filter(|(_, group)| group.entity.body.is_some())
            .flat_map(|(&first, group)| first..first + group.count)
            .find(|id| !game.members.contains_key(id)).expect("unplaced native founding member");
        let mut founding_arrival = input.clone();
        founding_arrival.request.actor = founder;
        assert_eq!(game.arrival_placement(&founding_arrival, held.patch).unwrap().actor.0, founder);
        let mut mismatched = input;
        mismatched.geometry.window.depth += 1;
        assert!(game.arrival_placement(&mismatched, held.patch).is_err());
    }
}
