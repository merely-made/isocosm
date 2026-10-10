// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl TimedActionSession {
    /// Admit a fall, refresh surviving anatomy and repair prepared contributions.
    pub fn fall(&mut self, distance: i32) -> Result<Vec<GameEvent>, TimedActionError> {
        self.body_change(|session| session.fall(distance))
    }

    /// Rest using the world's treatment rules, preserving lost parts and charge.
    pub fn rest(&mut self) -> Result<Vec<GameEvent>, TimedActionError> {
        self.body_change(Session::rest)
    }

    fn body_change(
        &mut self,
        change: impl FnOnce(&mut Session) -> Result<Vec<GameEvent>, SessionError>,
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        let mut candidate = self.clone();
        let events = change(&mut candidate.session)?;
        candidate.reconcile_action()?;
        *self = candidate;
        Ok(events)
    }
}
