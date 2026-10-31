// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Current native anatomy and an explicitly supplied product label.

use super::AnatomyView;
use crate::identity::{BodyRevisionId, SubjectId};
use cosmograph::{lineage::{BodyRead, SpeciesId}, rules::Rules, schema::Entity};

#[derive(Clone, Copy, Debug)]
pub struct NativeAnatomyRead<'a> {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    /// Labels the subject, never the origin of its foreign parts.
    pub species: SpeciesId,
    pub body: BodyRead<'a>,
}

impl<'a> NativeAnatomyRead<'a> {
    pub fn new(subject: SubjectId, revision: BodyRevisionId, species: SpeciesId,
        entity: &'a Entity, rules: &'a Rules) -> Result<Self, String> {
        Ok(Self { subject, revision, species, body: BodyRead::new(entity, rules)? })
    }

    pub fn view(&self) -> AnatomyView<'a> {
        AnatomyView { subject: self.subject, revision: self.revision,
            document: self.body.document() }
    }
}
