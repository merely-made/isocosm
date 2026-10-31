// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Campaign;
use cosmograph::Session;
use cosmograph_overlay::OrdinaryAccess;

impl Campaign {
    pub fn with_table_pack(&self, game: Vec<u8>, access: OrdinaryAccess,
        private: &std::collections::BTreeSet<cosmograph::schema::Key>, input: &crate::native::pack::PackEnvelope)
        -> Result<(Self, Vec<u8>, String), String> {
        let (replay,game,accepted) = self.replay.with_table_pack(game,access,private,input)?;
        Ok((Self { replay,..self.clone() },game,accepted))
    }
    /// Borrow the same authenticated native replay; this creates no new owner.
    pub fn table_session(&self) -> &Session { self.replay.table_session() }

    /// A product table input changes native history and game bindings together.
    /// The caller publishes the returned pair as one accepted table result.
    pub fn with_table_input(&self, game: Vec<u8>, access: OrdinaryAccess, bytes: &[u8])
        -> Result<(Self, Vec<u8>, String), String> {
        let (replay, game, accepted) = self.replay.with_table_input(game, access, bytes)?;
        Ok((Self { replay, ..self.clone() }, game, accepted))
    }

    /// The roster comes from the trusted table host, separately from the
    /// authenticated player's typed request and the immutable campaign content.
    pub fn with_table_time(&self, game: Vec<u8>, access: OrdinaryAccess,
        roster: &crate::native::time::Roster, envelope: &cosmograph_overlay::vtt::VttIntentEnvelope)
        -> Result<(Self, Vec<u8>, String), String> {
        let (replay, game, accepted) = self.replay.with_table_time(game, access, roster, envelope)?;
        Ok((Self { replay, ..self.clone() }, game, accepted))
    }

    pub fn with_table_journey(&self, game: Vec<u8>, access: OrdinaryAccess,
        roster: &crate::native::time::Roster, input: &crate::native::joint_journey::JourneyEnvelope)
        -> Result<(Self, Vec<u8>, String), String> {
        let (replay, game, accepted) = self.replay.with_table_journey(game,access,roster,input)?;
        Ok((Self { replay, ..self.clone() },game,accepted))
    }
}
