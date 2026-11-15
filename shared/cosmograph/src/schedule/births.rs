// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Generated site-grain reproduction can contribute to a population row
//! without writing personal history. Direct acts and physical birth offers
//! still use the individual path.

use crate::{rules::{Causation, Effect, Process}, schema::{Entity, Id, Method}, Simulation};

impl Simulation {
    pub(super) fn birth_pass(&self, process: &Process) -> Option<super::pass::BirthPass> {
        let rules = &self.genesis.rules;
        (rules.cohort_births
            && process.id.strip_prefix("ecology:birth-").is_some_and(|i| i.parse::<u32>().is_ok())
            && process.causation == Causation::Transition
            && process.target.is_none()
            && process.risk.is_none()
            && process.commitments.is_empty()
            && process.goal_directives.is_empty()
            && matches!(&process.effects[..], [Effect::Birth { scatter: None, .. }])
            && rules.ecology_witness.as_ref().is_none_or(|spec| !spec.observes(&process.id))
            && (!self.state.attention.is_empty()
                || !rules.birth_profiles.values().any(|profile| profile.process == process.id)))
            .then(|| super::pass::BirthPass {
                protected: self.band_identities(true),
                population: self.state.population.count(),
                counted: false,
            })
    }

    pub(crate) fn counts_birth(&self, actor: Id, entity: &Entity) -> bool {
        self.pass.as_ref().and_then(|pass| pass.births.as_ref())
            .is_some_and(|pass| !pass.protected.contains(&actor))
            && entity.method == Method::Reactive
            && entity.authored.is_none()
            && entity.patch.is_none()
            && entity.resident_site().is_some()
            && !self.state.spatial_poses.contains_key(&actor)
            && (self.state.attention.is_empty() || !self.near(actor))
    }
}

#[cfg(test)]
mod tests;
