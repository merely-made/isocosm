// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Movement rules for the native profile admitted by the shared fixture.

use eponym_play::identity::SubjectId;
use eponym_play::{GameState, MotionRules};

pub(super) fn rules(game: &GameState, subject: SubjectId) -> MotionRules {
    MotionRules {
        revision: if game.movement_profile(subject).is_some() {
            2
        } else {
            1
        },
        ..MotionRules::default()
    }
}
