// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::fixtures::session::timed_action_world_with_practice_striker_at_target;
use crate::{GameEvent, GameIntent, ItemLocation, Name, Session, World, WorldConfig};
use cosmograph::lineage::SpeciesId;

#[test]
fn two_native_severings_keep_one_game_revision_and_the_actual_native_snapshot_source() {
    let subject = SubjectId(819);
    let world = World::generate(7, WorldConfig { side: 2, extent: 8 }).unwrap();
    let top = world.ground().surface(0,0).unwrap();
    let mut game = GameState::new(world);
    game.apply(GameIntent::Generate { tick: game.next_tick(), subject, body_seed: 9,
        at: [0,top+1,0] }).unwrap();
    game.apply(GameIntent::Name { tick: game.next_tick(), subject, name: Name::new("Two leaves").unwrap() }).unwrap();
    let doc = game.native_anatomy(subject, SpeciesId(71)).unwrap();
    game.apply(GameIntent::AdmitAnatomy { tick: game.next_tick(), subject,
        revision: BodyRevisionId(0), document: Box::new(doc) }).unwrap();
    let before = game.current_anatomy(subject).unwrap().native_source.as_ref().unwrap().body_revision;
    let handoff = EponymHandoff::Hurt(Harm { vigour_drained: 0,
        wounds: [1,4].map(|part| Wound { part: PartHandle(part), severity: WoundSeverity::Severed }).to_vec() });
    let (revision, died, _) = game.receive_harm(game.next_tick(), subject, handoff).unwrap();
    assert!(!died);
    assert_eq!(revision,BodyRevisionId(1),"one game revision for the admitted severing batch");
    let id = game.entity(subject).unwrap();
    let e = game.world.sim().state().population.get(id).unwrap();
    let record = game.anatomies.get(subject).unwrap();
    assert_eq!(e.body_revision,before+2,"native commands each moved the native body revision");
    let source = record.native_source.as_ref().unwrap();
    assert_eq!(source.body_revision,e.body_revision,"snapshot source must name the native result");
    assert_eq!(source.state_hash,game.world.sim().state_hash());
    assert_eq!(record.document.doc(),e.body.as_ref().unwrap());
    assert_eq!(record.document,game.native_anatomy(subject,SpeciesId(71)).unwrap());
    for part in [1,4] { assert!(record.document.part(PartId(part)).unwrap().severed); }
}

#[test]
fn severing_a_non_played_worn_part_drops_the_same_item_at_the_wearer() {
    let mut fixture = timed_action_world_with_practice_striker_at_target();
    crate::fixtures::native_severing::sever_target(&mut fixture.action, fixture.target, PartId(2)).unwrap();
    let game = fixture.action.session().game();
    assert!(
        game.current_anatomy(fixture.target)
            .unwrap()
            .document
            .part(PartId(2))
            .unwrap()
            .severed
    );
    assert_eq!(game.played(), Some(fixture.keeper));
    let item = game
        .items()
        .get(fixture.target_item)
        .expect("target dressing remains");
    let ItemLocation::At(at) = item.location else {
        panic!("dressing on the severed part must be on the ground")
    };
    assert_eq!(at, game.at(fixture.target).unwrap());
    let released_at = game
        .events()
        .iter()
        .find_map(|event| match event {
            GameEvent::ItemReleased {
                subject, item, at, ..
            } if *subject == fixture.target && *item == fixture.target_item => Some(*at),
            _ => None,
        })
        .expect("the target's dressing release is recorded");
    assert_eq!(at, released_at);
    let saved = fixture.action.session().save().unwrap();
    assert_eq!(Session::restore(&saved).unwrap().save().unwrap(), saved);
}

#[test]
fn an_external_actor_keeps_distinct_native_and_game_revision_counters() {
    let fixture = crate::fixtures::session::timed_action_world_with_practice_striker();
    let mut game = fixture.action.session().game().clone();
    let actor = crate::fixtures::native_severing::STRIKER;
    let injury = EponymHandoff::Hurt(Harm { vigour_drained: 0,
        wounds: [1, 4].map(|part| Wound { part: PartHandle(part),
            severity: WoundSeverity::Severed }).to_vec() });
    game.receive_harm(game.next_tick(), actor, injury).unwrap();
    let record = game.current_anatomy(actor).unwrap();
    let revision = record.revision;
    assert_ne!(revision.0, record.native_source.as_ref().unwrap().body_revision);
    game.apply(GameIntent::ExternalVolley { tick: game.next_tick(), actor,
        actor_revision: revision, target: fixture.keeper,
        strikes: vec![crate::timed_action::StrikeReceipt {
            binding: crate::timed_action::LimbBinding { part: PartId(2), revision },
            direction: crate::timed_action::Direction::Right, charge: 24 }],
        rules: crate::CombatRules::default() }).unwrap();
}
