// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's motion and contact, solved on the game side over conatus
//! (rulings 233 and 597). The sim admits each fixed step its state's solver
//! returns and links no solver of its own; a host hands it [`SOLVER`].

mod contact;
mod solver;

pub use contact::{
    BodyId, BodyKind, BodyProfile as ContactBodyProfile, BodyState, BoxCollider, ContactEffect,
    ContactError, ContactSave, ContactWorld, FIXED_DT_SECONDS, HeldInput, Impairment, Input,
    InputFrame, MAX_RECORDED_FRAMES, MovableBoard, Position, TriggeredInput,
};
pub use solver::{SOLVER, advance};
