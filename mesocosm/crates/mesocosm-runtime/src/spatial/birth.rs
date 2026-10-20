// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored biological birth slots relative to the parent's admitted pose (848).

use super::*;
use isocosm::spatial::{BirthPending, BirthRequest, BirthGeometry, BirthResolution, BirthReport};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BirthParent {
    Entity(EntityHandle),
    /// The native parent's lineage at accepted action publication. The
    /// child's actual lineage is recorded separately; inheritance is not assumed.
    Lineage(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthSlot {
    pub offset_from_parent: [i64; 3], pub yaw_from_parent: Yaw,
    pub eye_from_child: [i64; 3], pub window: Window,
    pub operations: BTreeMap<String, Operation>,
    /// Explicit range for this accepted child only; never inherited from its parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight_range: Option<u64>,
    /// Complete authored inputs for this ordinal's actual child, never inherited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<BirthTemplates>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthInput { pub profile: String, pub parent: BirthParent, pub slots: Vec<BirthSlot> }

impl BirthInput {
    pub(super) fn applies(&self, r: &BirthRequest, sim: &Simulation) -> bool {
        sim.genesis().rules.birth_profiles.get(&self.profile).is_some_and(|p| p.process == r.process)
            && match &self.parent {
                BirthParent::Entity(parent) => parent.0 == r.parent,
                BirthParent::Lineage(lineage) => lineage == &r.parent_lineage,
            }
    }

    pub(super) fn validate(&self, sim: &Simulation) -> Result<(), String> {
        if !sim.genesis().rules.birth_profiles.contains_key(&self.profile) {
            return Err("game birth input names no native birth profile".into());
        }
        if self.slots.iter().any(|slot| slot.sight_range.is_some_and(|range| range > i64::MAX as u64)) {
            return Err("authored newborn sight range exceeds its bounded physical source".into());
        }
        for inputs in self.slots.iter().filter_map(|slot| slot.inputs.as_ref()) { inputs.validate(sim)?; }
        match &self.parent {
            BirthParent::Entity(parent) if sim.state().population.get(parent.0).is_none() =>
                Err("authored birth parent is absent".into()),
            BirthParent::Lineage(lineage) if !sim.state().lineages.contains_key(lineage) =>
                Err("authored birth parent lineage is absent".into()),
            _ => Ok(()),
        }
    }
}

impl Game {
    pub(super) fn birth_slot<'a>(&'a self, r: &BirthRequest, sim: &Simulation)
        -> Result<(&'a BirthInput, &'a BirthSlot), String> {
        let driver = self.driver.as_ref().ok_or("birth has no authored spatial driver")?;
        let mut inputs = driver.profile.births.iter().filter(|input| input.applies(r, sim));
        let input = inputs.next().ok_or("biological birth has no applicable authored profile")?;
        if inputs.next().is_some() { return Err("biological birth profile selectors are ambiguous".into()); }
        let ordinal = usize::try_from(r.ordinal).map_err(|_| "biological ordinal overflow")?;
        let slot = input.slots.get(ordinal).ok_or("biological ordinal has no explicitly authored placement slot")?;
        Ok((input, slot))
    }

    pub(super) fn resolve_birth(&self, pending: &BirthPending) -> Result<BirthGeometry, String> {
        self.resolve_birth_with_geometry(pending, &mut GeometryCache::default())
    }

    pub(super) fn resolve_birth_with_geometry(&self, pending: &BirthPending, cache: &mut GeometryCache) -> Result<BirthGeometry, String> {
        if self.generated_driver() { return self.generated_birth(pending, cache); }
        let r = &pending.request;
        if !r.parent_alive || r.site.is_none() || r.child_journey.is_some() {
            return Err("birth has no living-at-action-start resident parent pose".into());
        }
        let (input, slot) = self.birth_slot(r, &pending.sim)?;
        let parent = self.members.get(&r.parent).ok_or("birth parent has no admitted actual pose")?
            .rebound(&pending.sim)?;
        // A semelparous native act may now have killed the parent. This is
        // its unchanged admitted pose, not a corpse action or control grant.
        parent.realized(&pending.sim)?;
        if parent.site.0 != r.site.ok_or("parent is travelling")? || parent.body_revision != r.parent_revision
            || parent.resident_since != Some(r.parent_visit) || parent.terrain_revision != r.terrain_revision
        { return Err("parent birth pose differs from current native source".into()); }
        Ok(BirthGeometry { profile: input.profile.clone(),
            parent_pose: isocosm::spatial::Pose { at: parent.pose.at, yaw: parent.pose.yaw },
            parent_eye: parent.eye, parent_window: parent.window.clone(),
            parent_places: parent.places.unwrap_or_default(), offset_from_parent: slot.offset_from_parent,
            yaw_from_parent: slot.yaw_from_parent, eye_from_child: slot.eye_from_child, window: slot.window.clone() })
    }

    pub(super) fn apply_births(&mut self, inputs: &[BirthResolution], reports: &[BirthReport], sim: &Simulation)
        -> Result<Vec<profile::Transition>, String> {
        let mut changes = Vec::new();
        for report in reports {
            if self.generated_driver() {
                let driver = self.driver.as_mut().ok_or("generated birth driver absent")?;
                let child = ChildInputs { birth: report.request.clone(),
                    inputs: BirthTemplates { steps: BTreeMap::new(), arrivals: Vec::new() }, sight_range: None };
                if driver.initial.contains_key(&report.request.child)
                    || driver.children.get(&report.request.child).is_some_and(|old| *old != child) {
                    return Err("generated biological child reuses a physical source identity".into());
                }
                driver.children.insert(report.request.child, child);
                if report.patch.is_ok() {
                    let resolution = inputs.iter().find(|r| r.request == report.request).ok_or("generated birth answer absent")?;
                    changes.push(self.apply_generated_birth(resolution, report, sim)?);
                }
                continue;
            }
            self.register_birth_inputs(report, sim)?;
            let Ok(patch) = &report.patch else { continue; };
            let r = &report.request;
            let resolution = inputs.iter().find(|input| input.request == *r).ok_or("accepted birth answer absent")?;
            let geometry = resolution.geometry.as_ref().map_err(|_| "accepted birth geometry was refused")?;
            let (input, slot) = self.birth_slot(r, sim)?;
            if report.profile.as_ref() != Some(&input.profile) || geometry.profile != input.profile
                || geometry.offset_from_parent != slot.offset_from_parent || geometry.yaw_from_parent != slot.yaw_from_parent
                || geometry.eye_from_child != slot.eye_from_child || geometry.window != slot.window
            { return Err("native birth input differs from its exact authored ordinal slot".into()); }
            let parent = self.members.get(&r.parent).ok_or("accepted birth parent pose absent")?;
            if parent.pose.at != geometry.parent_pose.at || parent.pose.yaw != geometry.parent_pose.yaw
                || parent.eye != geometry.parent_eye || parent.window != geometry.parent_window
                || parent.places.unwrap_or_default() != geometry.parent_places
                || parent.resident_since != Some(r.parent_visit)
            { return Err("native birth geometry does not use the parent's admitted current pose".into()); }
            let (pose, eye) = isocosm::spatial::birth_pose(geometry)?;
            let native = sim.genesis().rules.birth_profiles.get(&input.profile).ok_or("birth profile absent")?;
            let after = Placement { actor: EntityHandle(r.child), site: PlaceHandle(r.site.ok_or("travelling birth")?),
                patch: *patch, body_revision: r.child_revision, terrain_revision: r.terrain_revision.clone(),
                pose: Pose { at: pose.at, yaw: pose.yaw }, eye, window: slot.window.clone(),
                operations: slot.operations.clone(), places: Some(native.places), resident_since: Some(r.child_visit) };
            let sight_range = slot.sight_range;
            if self.members.contains_key(&r.child) { return Err("biological birth reuses a saved physical identity".into()); }
            changes.push(profile::Transition { actor: r.child, before: None, after: after.clone(),
                admission: profile::Admission::Birth { action: r.action.clone(), parent: r.parent, ordinal: r.ordinal },
                sight_range });
            self.members.insert(r.child, after);
            if let Some(range) = sight_range {
                self.sight.get_or_insert_with(|| SightConfig { version: 1, ranges: BTreeMap::new() })
                    .ranges.insert(r.child, range);
                self.version = self.current_version();
            }
        }
        Ok(changes)
    }
}
