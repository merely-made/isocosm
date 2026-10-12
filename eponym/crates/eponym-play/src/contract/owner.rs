// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl World {
    pub fn submit(&mut self, participant: Option<Id>, intent: &EponymIntent)
        -> Result<Lowered, WorldError> {
        self.borrowed_contract().submit(participant, intent)
    }
    pub fn hand_back(&mut self, subject: Id, outcome: &EponymHandoff)
        -> Result<Lowered, WorldError> {
        self.borrowed_contract().hand_back(subject, outcome)
    }
}
