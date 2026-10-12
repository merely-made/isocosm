// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Finite old Generate and admission grammar for authenticated pre-recipe worlds.

use isocosm::Command;
use isocosm::lineage::LineageBody;
use crate::bodies::{BodyError, BodyProfile, BodyRecord};
use crate::founding::{self, ENERGY, FULL, RESERVE};
use crate::identity::{BodyRevisionId, SubjectId, Tick};
use crate::{GameError, GameEvent, MovementError};
use super::GameState;

#[cfg(test)]
mod tests;

impl GameState {
    pub(super) fn generate_legacy_body(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        body_seed: u64,
        at: [i32; 3],
    ) -> Result<Vec<GameEvent>, GameError> {
        if self.records.contains_key(&subject) {
            return Err(BodyError::SubjectExists(subject).into());
        }
        if self.movement.position(subject).is_some() {
            return Err(MovementError::SubjectExists(subject).into());
        }
        if !crate::walking::stands(self.world.volume(), at) {
            return Err(MovementError::InvalidStart(at).into());
        }
        let profile = BodyProfile::draw(self.world.seed(), body_seed, subject);
        let accounts = [
            (founding::TISSUE, u64::from(profile.mass_mg)),
            (RESERVE, FULL),
            (ENERGY, FULL),
        ];
        let arrival = isocosm::arrival::Arrival {
            lineage: founding::SOPHONT.into(),
            site: self.world.site(),
            method: isocosm::schema::Method::Normative,
            accounts: accounts.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            traits: Default::default(),
            skills: Default::default(),
            disposition: [0; 5],
            character: None,
        };
        let outcome = self.world.found(Command::Arrive(arrival))?;
        let entity = outcome
            .strip_prefix("entity:")
            .and_then(|s| s.parse().ok())
            .ok_or(GameError::Decode)?;
        // A body of one part until its anatomy is admitted.
        let body =
            isometer_core::BodyDocument::new(isometer_core::VolumeRef::from_tag(0), [2, 4, 2]);
        self.world.found(Command::Embody { entity, body })?;
        self.world.bind(subject, entity);
        let revision = BodyRevisionId(0);
        self.records.insert(
            subject,
            BodyRecord {
                entity,
                revision,
                body_seed,
                named_at: None,
                died_at: None,
            },
        );
        self.movement.spawn(&self.world, subject, at)?;
        Ok(vec![GameEvent::Generated {
            tick,
            subject,
            revision,
        }])
    }

    pub(super) fn admit_legacy_body(
        &mut self, tick: Tick, subject: SubjectId, revision: BodyRevisionId, document: &LineageBody,
    ) -> Result<Vec<GameEvent>, GameError> {
        let entity = self.entity_of(subject)?;
        self.anatomies.admit(subject, revision, document.clone())?;
        let body = document.doc().clone();
        self.world.found(Command::Embody { entity, body })?;
        Ok(vec![GameEvent::AnatomyAdmitted { tick, subject, revision }])
    }
}
