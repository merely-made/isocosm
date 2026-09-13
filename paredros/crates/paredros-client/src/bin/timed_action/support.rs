// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored movement roles for the keeper, separate from its striking anatomy.

use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::{
    GameIntent, GameState, MOTION_SCALE, MotionEnvelope, MotionRules, MovementProfile, SupportBand,
};

pub(super) fn configure_keeper(game: &mut GameState, subject: SubjectId) {
    let body = &game.current_anatomy(subject).unwrap().document;
    // These four gripping limbs are explicitly authored as weight-bearing here.
    // Their striking reach does not define the terrain collision envelope.
    let profile = MovementProfile {
        revision: paredros_world::MOVEMENT_PROFILE_REVISION,
        source_revision: BodyRevisionId(0),
        envelope: MotionEnvelope {
            anchor: body.root,
            half_width: MOTION_SCALE / 4,
            height: 2 * MOTION_SCALE,
        },
        supports: paredros_world::fixtures::three_lives::limb_parts(body).to_vec(),
        support_band: SupportBand { min_y: 0, max_y: 2 },
    };
    game.apply(GameIntent::ConfigureMovementProfile {
        tick: game.next_tick(),
        subject,
        revision: BodyRevisionId(0),
        profile,
    })
    .expect("authored keeper movement profile");
}

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
