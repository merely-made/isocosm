// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Replay;
use isocosm::Session;
use isocosm_overlay::OrdinaryAccess;

impl Replay {
    pub(crate) fn table_session(&self) -> &Session { &self.session }

    pub(crate) fn with_table_input(&self, game: Vec<u8>, access: OrdinaryAccess, bytes: &[u8])
        -> Result<(Self, Vec<u8>, String), String> {
        let mut session = self.session.clone();
        let (accepted, game) = super::submit_game_access(&mut session, game, bytes, access)?;
        Ok((Self { session }, game, accepted))
    }

    pub(crate) fn with_table_time(&self, game: Vec<u8>, access: OrdinaryAccess,
        roster: &super::time::Roster, envelope: &isocosm_overlay::vtt::VttIntentEnvelope)
        -> Result<(Self, Vec<u8>, String), String> {
        let mut session = self.session.clone();
        let (accepted, game) = super::submit_time(&mut session, game, access, roster, envelope)?;
        Ok((Self { session }, game, accepted))
    }
}
