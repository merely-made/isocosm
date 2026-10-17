// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use eponym_play::{World, WorldIntent};
use eponym_sortie::{scene, sortie::SortieEvent};

#[test]
fn native_scarp_is_the_fall_and_survives_world_replay() {
    let (_, sortie) = scene::played_through();
    let world = sortie.society.world();

    assert_eq!(
        world.intents().iter().filter(|intent| matches!(intent, WorldIntent::MaterialEdit { .. })).count(),
        1,
    );
    assert!(sortie.events.iter().any(|event| matches!(event, SortieEvent::Fell { .. })));

    let restored = World::restore(&world.save().expect("world save encodes"))
        .expect("native material edit replays");
    assert_eq!(restored.ground(), world.ground());
}
