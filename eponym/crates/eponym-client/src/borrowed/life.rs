// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Life input through the existing borrowed native publication cut.

use cosmograph::Session;
pub use eponym_play::contract::mode::life::{Input, Record, KIND, prepare, previous_take, death_choices};
use cosmograph_overlay::OrdinaryAccess;

pub fn submit(session: &mut Session, game: Vec<u8>, input: &[u8], access: OrdinaryAccess)
    -> Result<(String, Vec<u8>), String> {
    eponym_play::contract::mode::life::submit_bytes(session, game, input, access)
}
