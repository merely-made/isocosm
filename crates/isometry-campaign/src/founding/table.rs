// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Campaign;
use isocosm::Session;
use isocosm_overlay::OrdinaryAccess;

impl Campaign {
    /// Borrow the same authenticated native replay; this creates no new owner.
    pub fn table_session(&self) -> &Session { self.replay.table_session() }

    /// A product table input changes native history and game bindings together.
    /// The caller publishes the returned pair as one accepted table result.
    pub fn with_table_input(&self, game: Vec<u8>, access: OrdinaryAccess, bytes: &[u8])
        -> Result<(Self, Vec<u8>, String), String> {
        let (replay, game, accepted) = self.replay.with_table_input(game, access, bytes)?;
        Ok((Self { replay, ..self.clone() }, game, accepted))
    }
}
