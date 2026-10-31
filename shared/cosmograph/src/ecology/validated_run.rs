// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Call-scoped proof objects for a fresh native witness. They are deliberately
//! not serializable; restored witnesses use the public replaying validators.

use super::{Admission, JointWitness};
use crate::Session;

pub struct ValidatedJointRun {
    witness: JointWitness,
    session: Session,
}

impl ValidatedJointRun {
    pub(super) fn new(witness: JointWitness, session: Session) -> Self {
        Self { witness, session }
    }

    pub fn witness(&self) -> &JointWitness {
        &self.witness
    }
    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn into_parts(self) -> (JointWitness, Session) {
        (self.witness, self.session)
    }
}

pub struct AdmittedJointRun {
    witness: JointWitness,
    admission: Admission,
    session: Session,
}

impl AdmittedJointRun {
    pub(super) fn new(witness: JointWitness, admission: Admission, session: Session) -> Self {
        Self {
            witness,
            admission,
            session,
        }
    }

    pub fn into_parts(self) -> (JointWitness, Admission, Session) {
        (self.witness, self.admission, self.session)
    }
}
