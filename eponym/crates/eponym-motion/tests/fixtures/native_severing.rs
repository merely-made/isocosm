// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use eponym_play::identity::{BodyRevisionId, SubjectId};
use eponym_play::timed_action::{Direction, LimbBinding, StrikeReceipt};
use eponym_play::{CombatRules, GameEvent, GameIntent, GameState, Name, StrikeOutcome, part_bounds};
use isocosm::history::Command;
use isocosm::lineage::SpeciesId;
use isometer_core::PartId;

const ACTOR: SubjectId = SubjectId(74);

pub fn sever(game: &mut GameState, target: SubjectId, part: PartId) {
    let before = game.current_anatomy(target).unwrap().clone();
    let root = part_bounds(&before.document, before.document.root).unwrap();
    let limb = part_bounds(&before.document, part).unwrap();
    let direction = if limb.min[0] >= root.max[0] {
        Direction::Right
    } else {
        assert!(limb.max[0] <= root.min[0], "fixture cut must target a lateral leaf");
        Direction::Left
    };
    let at = game.intents().iter().find_map(|intent| match intent {
        GameIntent::Generate { subject, at, .. } if *subject == target => Some(*at),
        _ => None,
    }).unwrap();
    let motion: Vec<_> = game.intents().iter().filter(|intent| {
        intent.subject() == Some(target) && matches!(intent,
            GameIntent::ConfigureMovementProfile { .. } | GameIntent::AdvanceMotion { .. }
        )
    }).cloned().collect();
    // The same recipe supplies the striking leaf. Its outward sweep meets
    // the target leaf without crossing the native root's bounds.
    game.apply(GameIntent::Generate {
        tick: game.next_tick(), subject: ACTOR, body_seed: 7, at,
    }).unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(), subject: ACTOR, name: Name::new("Support striker").unwrap(),
    }).unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(), subject: ACTOR, revision: BodyRevisionId(0),
        document: Box::new(game.native_anatomy(ACTOR, SpeciesId(71)).unwrap()),
    }).unwrap();
    // Replay the accepted movement from the same valid founding cell. The
    // combatants' poses coincide without supplying either a final pose.
    for intent in motion {
        let intent = match intent {
            GameIntent::ConfigureMovementProfile { revision, profile, .. } => {
                GameIntent::ConfigureMovementProfile {
                    tick: game.next_tick(), subject: ACTOR, revision, profile,
                }
            },
            GameIntent::AdvanceMotion { revision, step, input, rules, .. } => {
                GameIntent::AdvanceMotion {
                    tick: game.next_tick(), subject: ACTOR, revision, step, input, rules,
                }
            },
            _ => unreachable!("only accepted motion receipts were selected"),
        };
        game.apply(intent).unwrap();
    }
    assert_eq!(game.pose(ACTOR), game.pose(target));
    let events = game.apply(GameIntent::ResolveVolley {
        tick: game.next_tick(), actor: ACTOR, target,
        strikes: vec![StrikeReceipt {
            binding: LimbBinding { part, revision: BodyRevisionId(0) },
            direction, charge: 24,
        }],
        rules: CombatRules::default(),
    }).unwrap();
    assert!(events.iter().any(|event| matches!(event,
        GameEvent::VolleyResolved { strikes, .. }
        if matches!(strikes[0].outcome, StrikeOutcome::Hit { part: hit, .. } if hit == part)
    )), "the recorded strike must contact the requested native leaf");
    let record = game.current_anatomy(target).unwrap();
    assert_eq!(record.revision, BodyRevisionId(before.revision.0 + 1));
    assert_eq!(record.document.parts.iter().filter(|p| p.severed).map(|p| p.id).collect::<Vec<_>>(), vec![part]);
    let entity = game.entity(target).unwrap();
    let native = game.world().sim().state().population.get(entity).unwrap();
    assert_eq!(native.body_revision, before.native_source.unwrap().body_revision + 1);
    assert_eq!(record.native_source.as_ref().unwrap().body_revision, native.body_revision);
    assert_eq!(record.document, game.native_anatomy(target, SpeciesId(71)).unwrap());
    assert!(game.world().session().entries.iter().any(|entry| matches!(entry.command,
        Command::Wound { entity: wounded, part: Some(cut), .. } if wounded == entity && cut == part
    )), "the support change must be backed by a logged native Wound");
}
