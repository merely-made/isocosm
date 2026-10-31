// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read and validate the native body without letting a snapshot author it.

use cosmograph::lineage::{LineageBody, SpeciesId};
use crate::{AnatomyError, GameError, NativeAnatomySource};
use crate::identity::SubjectId;
use super::GameState;
use crate::anatomy::NativeAnatomyRead;

impl GameState {
    /// Borrows exact current native facts; the product label is supplied.
    pub fn native_anatomy_read(&self, subject: SubjectId, species: SpeciesId)
        -> Result<NativeAnatomyRead<'_>, GameError> {
        let entity = self.entity_of(subject)?;
        let sim = self.world.sim();
        let body = sim.state().population.get(entity).ok_or(GameError::Decode)?;
        let revision = self.records.get(&subject).ok_or(GameError::Decode)?.revision;
        NativeAnatomyRead::new(subject, revision, species, body, &sim.genesis().rules)
            .map_err(|_| AnatomyError::NativeMismatch.into())
    }

    /// A reading for admission, using the caller's existing species label.
    pub fn native_anatomy(&self, subject: SubjectId, species: SpeciesId) -> Result<LineageBody, GameError> {
        let entity = self.entity_of(subject)?;
        let sim = self.world.sim();
        let e = sim.state().population.get(entity).ok_or(GameError::Decode)?;
        LineageBody::snapshot(species, e, &sim.genesis().rules).map_err(|_| AnatomyError::NativeMismatch.into())
    }

    pub(super) fn native_anatomy_source(&self, subject: SubjectId) -> Result<NativeAnatomySource, GameError> {
        let entity = self.entity_of(subject)?;
        let e = self.world.sim().state().population.get(entity).ok_or(GameError::Decode)?;
        Ok(NativeAnatomySource { entity, lineage: e.lineage.clone(), body_revision: e.body_revision,
            tick: self.world.sim().state().tick, state_hash: self.world.sim().state_hash() })
    }

    pub(super) fn refresh_native_anatomy(&mut self, subject: SubjectId) -> Result<(), GameError> {
        let species = self.anatomies.get(subject).ok_or(AnatomyError::Missing(subject))?.document.species();
        let document = self.native_anatomy(subject, species)?;
        let source = self.native_anatomy_source(subject)?;
        self.anatomies.refresh_native(subject, document, source)?;
        Ok(())
    }
}
