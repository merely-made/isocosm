// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit ordinal-slot inputs acquire only an actual reported biological child ID.

use super::*;
use isocosm::spatial::{BirthReport, BirthRequest};

/// Destination data has no guessed actor. Birth admission supplies that identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrivalTemplate {
    pub from: PlaceHandle, pub to: PlaceHandle,
    pub pose: Pose, pub eye: [i64; 3], pub window: Window,
    pub places: isometer_space::places::Rules,
    pub operations: BTreeMap<String, Operation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthTemplates {
    pub steps: BTreeMap<String, StepInput>, pub arrivals: Vec<ArrivalTemplate>,
}

/// Product input provenance only. Native choice, Journey and control stay native.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildInputs {
    pub birth: BirthRequest, pub inputs: BirthTemplates,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sight_range: Option<u64>,
}

impl BirthTemplates {
    pub(super) fn validate(&self, sim: &Simulation) -> Result<(), String> {
        for (key, step) in &self.steps {
            if !sim.genesis().rules.step_profiles.contains_key(key) {
                return Err("birth walking template names no native step profile".into());
            }
            if step.target.is_some_and(|target| sim.state().population.get(target.0).is_none()) {
                return Err("birth walking template names an absent explicit target".into());
            }
        }
        let mut routes = BTreeSet::new();
        for input in &self.arrivals {
            if !routes.insert((input.from.0, input.to.0)) || !sim.state().sites.contains_key(&input.to.0)
                || !sim.state().sites.get(&input.from.0).is_some_and(|site|
                    site.routes.iter().any(|route| route.to == input.to.0)) {
                return Err("birth destination template names a duplicate or absent native route".into());
            }
        }
        Ok(())
    }
}

impl ArrivalTemplate {
    fn bind(&self, child: Id) -> ArrivalInput {
        ArrivalInput { actor: EntityHandle(child), from: self.from, to: self.to,
            pose: self.pose.clone(), eye: self.eye, window: self.window.clone(),
            places: self.places, operations: self.operations.clone() }
    }
}

impl Driver {
    pub(super) fn step_input(&self, actor: Id, profile: &str) -> Result<&StepInput, String> {
        let initial = self.profile.steps.get(&actor).and_then(|steps| steps.get(profile));
        let born = self.children.get(&actor).and_then(|child| child.inputs.steps.get(profile));
        match (initial, born) {
            (Some(_), Some(_)) => Err("walking inputs collide for one actual native identity".into()),
            (Some(input), None) | (None, Some(input)) => Ok(input),
            (None, None) => Err("chosen step has no explicitly authored game input".into()),
        }
    }

    pub(super) fn arrival_inputs(&self) -> Result<Vec<ArrivalInput>, String> {
        let mut inputs = self.profile.arrivals.clone();
        let mut identities = BTreeSet::new();
        for input in &inputs {
            if !identities.insert((input.actor.0, input.from.0, input.to.0)) {
                return Err("destination inputs collide for one actual native identity".into());
            }
        }
        for (&id, child) in &self.children {
            for template in &child.inputs.arrivals {
                let input = template.bind(id);
                if !identities.insert((id, input.from.0, input.to.0)) {
                    return Err("birth destination inputs collide with another authored destination".into());
                }
                inputs.push(input);
            }
        }
        Ok(inputs)
    }
}

impl Game {
    pub(super) fn register_birth_inputs(&mut self, report: &BirthReport, sim: &Simulation) -> Result<(), String> {
        // Reports originate after native biological insertion even when placement
        // is refused or the child shares a journey. No pose or child ID is guessed.
        let Ok((_, slot)) = self.birth_slot(&report.request, sim) else { return Ok(()); };
        let Some(inputs) = &slot.inputs else { return Ok(()); };
        let child = ChildInputs { birth: report.request.clone(), inputs: inputs.clone(), sight_range: slot.sight_range };
        let driver = self.driver.as_mut().ok_or("reported child inputs have no authored driver")?;
        if driver.initial.contains_key(&report.request.child)
            || driver.profile.steps.contains_key(&report.request.child)
            || driver.profile.arrivals.iter().any(|input| input.actor.0 == report.request.child) {
            return Err("reported biological child reuses an existing authored identity".into());
        }
        if let Some(old) = driver.children.get(&report.request.child) {
            if old != &child { return Err("native child has conflicting birth input provenance".into()); }
        } else { driver.children.insert(report.request.child, child); }
        Ok(())
    }

    pub(super) fn arrival_sight_range(&self, actor: Id) -> Option<u64> {
        if self.members.contains_key(&actor) { return None; }
        self.driver.as_ref()?.children.get(&actor)?.sight_range
    }
}
