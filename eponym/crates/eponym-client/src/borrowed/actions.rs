// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual shared-host inputs use the same Eponym solver as standalone play.
//! Only derived game data is decoded; the host's candidate Session is borrowed.

use cosmograph::{Session,directing::readings::View};
use eponym_play::contract::mode::actions;
pub use eponym_play::contract::mode::actions::{Action,ActionConfig,SubjectProfile,
    MotionPolicy,CombatPolicy,FallPayment,game_source,OrdinaryAccess};

pub fn submit(session:&mut Session, game:Vec<u8>, input:&[u8],
    access:OrdinaryAccess, visible:&View) -> Result<(String,Vec<u8>),String> {
    actions::submit_bytes(session,game,input,access,visible,eponym_motion::SOLVER)
}
