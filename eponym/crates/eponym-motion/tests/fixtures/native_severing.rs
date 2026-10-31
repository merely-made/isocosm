// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::identity::{BodyRevisionId, SubjectId};
use eponym_play::{GameEvent, GameState, StrikeOutcome};
use cosmograph::history::Command;
use isometer_core::PartId;

pub fn sever(game: &mut GameState, target: SubjectId, part: PartId) {
    let before = game.current_anatomy(target).unwrap().clone();
    let events: Vec<_> = eponym_play::fixtures::native_severing::volley(game, target, part)
        .into_iter()
        .flat_map(|intent| game.apply(intent).unwrap())
        .collect();
    assert!(
        events.iter().any(|event| matches!(event,
            GameEvent::VolleyResolved { strikes, .. }
            if matches!(strikes[0].outcome, StrikeOutcome::Hit { part: hit, .. } if hit == part)
        )),
        "the recorded strike must contact the requested native leaf"
    );
    let record = game.current_anatomy(target).unwrap();
    assert_eq!(record.revision, BodyRevisionId(before.revision.0 + 1));
    assert_eq!(
        record
            .document
            .parts
            .iter()
            .filter(|p| p.severed)
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        vec![part]
    );
    let entity = game.entity(target).unwrap();
    let native = game.world().sim().state().population.get(entity).unwrap();
    assert_eq!(
        native.body_revision,
        before.native_source.unwrap().body_revision + 1
    );
    assert_eq!(
        record.native_source.as_ref().unwrap().body_revision,
        native.body_revision
    );
    assert_eq!(
        record.document,
        game.native_anatomy(target, before.document.species())
            .unwrap()
    );
    assert!(game.world().session().entries.iter().any(|entry| matches!(entry.command,
        Command::Wound { entity: wounded, part: Some(cut), .. } if wounded == entity && cut == part
    )), "the support change must be backed by a logged native Wound");
}
