// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use paredros_world::{
    GameError, GameEvent, GameIntent, ItemId, ItemKind, ItemLocation, Name, Session, SessionError,
    World, WorldConfig, fixtures::three_lives::wetland_body,
};

const SUBJECT: SubjectId = SubjectId(1);

fn setup() -> (Session, Vec<ItemId>, [i32; 3]) {
    let mut game =
        paredros_world::GameState::new(World::generate(4242, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at)
                if item.kind == ItemKind::Dressing
                    && game
                        .world()
                        .ground()
                        .stands(at, mesocosm_core::places::WALKER_HEIGHT) =>
            {
                Some(at)
            },
            _ => None,
        })
        .unwrap();
    let dressings: Vec<_> = game
        .items()
        .at(at)
        .filter(|item| item.kind == ItemKind::Dressing)
        .map(|item| item.id)
        .collect();
    assert!(dressings.len() >= 2);
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject: SUBJECT,
        body_seed: 7,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject: SUBJECT,
        name: Name::new("Keeper").unwrap(),
    })
    .unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        document: Box::new(wetland_body()),
    })
    .unwrap();
    for item in &dressings {
        game.apply(GameIntent::Take {
            tick: game.next_tick(),
            subject: SUBJECT,
            item: *item,
        })
        .unwrap();
    }
    let session = Session::begin(game, SUBJECT).unwrap();
    session
        .game()
        .current_anatomy(SUBJECT)
        .expect("fixture anatomy is current");
    (session, dressings, at)
}

fn attach(session: &mut Session, item: ItemId, part: u32) {
    session
        .apply_game(GameIntent::AttachItem {
            tick: session.game().next_tick(),
            subject: SUBJECT,
            item,
            part: PartId(part),
            revision: BodyRevisionId(0),
        })
        .unwrap();
}

#[test]
fn fall_and_rest_refresh_current_anatomy_and_consume_a_dressing() {
    let (mut session, dressings, _) = setup();
    attach(&mut session, dressings[0], 1);

    let before = session.game().intents().len();
    let fall = session.fall(5).unwrap();
    assert!(fall.iter().any(|event| matches!(
        event,
        GameEvent::Injured {
            distance: 5,
            harm: 10,
            revision: BodyRevisionId(1),
            ..
        }
    )));
    assert!(fall.iter().any(|event| matches!(
        event,
        GameEvent::AnatomyReconciled {
            from_revision: BodyRevisionId(0),
            revision: BodyRevisionId(1),
            ..
        }
    )));
    assert_eq!(session.game().intents().len(), before + 2);
    assert_eq!(
        session.game().current_anatomy(SUBJECT).unwrap().revision,
        BodyRevisionId(1)
    );
    assert!(session.game().attachments(SUBJECT)[0].current);
    assert_eq!(session.game().bodies().get(SUBJECT).unwrap().wound, 10);
    assert_eq!(session.game().bodies().get(SUBJECT).unwrap().vitality, 90);

    let rest = session.rest().unwrap();
    assert!(rest.iter().any(|event| matches!(
        event,
        GameEvent::Rested {
            recovered: 10,
            revision: BodyRevisionId(2),
            ..
        }
    )));
    assert!(rest.iter().any(|event| matches!(
        event,
        GameEvent::AnatomyReconciled {
            from_revision: BodyRevisionId(1),
            revision: BodyRevisionId(2),
            ..
        }
    )));
    assert_eq!(session.game().intents().len(), before + 4);
    assert_eq!(
        session.game().current_anatomy(SUBJECT).unwrap().revision,
        BodyRevisionId(2)
    );
    assert_eq!(session.game().bodies().get(SUBJECT).unwrap().wound, 0);
    assert_eq!(session.game().bodies().get(SUBJECT).unwrap().vitality, 100);
    assert_eq!(
        session.game().items().get(dressings[0]).unwrap().location,
        ItemLocation::Consumed
    );
    assert!(session.game().attachments(SUBJECT).is_empty());
}

#[test]
fn stale_or_missing_anatomy_rejects_body_change_atomically() {
    let (session, _, _) = setup();
    let mut stale_game = session.game().clone();
    stale_game
        .apply(GameIntent::Fall {
            tick: stale_game.next_tick(),
            subject: SUBJECT,
            distance: 5,
        })
        .unwrap();
    let mut stale = Session::begin(stale_game, SUBJECT).unwrap();
    let before = stale.clone();
    assert!(matches!(
        stale.fall(1),
        Err(SessionError::Game(GameError::Anatomy(_)))
    ));
    assert_eq!(stale, before);

    let mut missing_game = paredros_world::GameState::new(session.game().world().clone());
    // Remove the only admitted record by using a fresh generated and named body.
    let at = session.game().movement().position(SUBJECT).unwrap();
    missing_game
        .apply(GameIntent::Generate {
            tick: Tick(0),
            subject: SUBJECT,
            body_seed: 7,
            at,
        })
        .unwrap();
    missing_game
        .apply(GameIntent::Name {
            tick: missing_game.next_tick(),
            subject: SUBJECT,
            name: Name::new("Keeper").unwrap(),
        })
        .unwrap();
    let mut missing = Session::begin(missing_game, SUBJECT).unwrap();
    let before = missing.clone();
    assert!(matches!(
        missing.rest(),
        Err(SessionError::Game(GameError::Anatomy(_)))
    ));
    assert_eq!(missing, before);
}

#[test]
fn save_restore_continues_body_change_and_lethal_fall_cannot_resurrect() {
    let (mut session, dressings, _) = setup();
    attach(&mut session, dressings[0], 1);
    session.fall(5).unwrap();
    let restored = Session::restore(&session.save().unwrap()).unwrap();
    assert_eq!(restored, session);

    let mut straight = session.clone();
    let mut resumed = restored;
    assert_eq!(straight.rest().unwrap(), resumed.rest().unwrap());
    assert_eq!(straight, resumed);

    let lethal = straight.fall(20).unwrap();
    assert!(
        lethal
            .iter()
            .any(|event| matches!(event, GameEvent::Died { .. }))
    );
    let dead = straight.clone();
    assert!(matches!(
        straight.rest(),
        Err(SessionError::Game(GameError::Body(
            paredros_world::BodyError::Dead(SUBJECT)
        )))
    ));
    assert_eq!(straight, dead);
    assert_eq!(
        Session::restore(&straight.save().unwrap()).unwrap(),
        straight
    );
}

#[test]
fn safe_fall_does_not_create_a_revision_or_synthetic_anatomy_change() {
    let (mut session, _, _) = setup();
    let before = session.game().clone();
    let events = session.fall(2).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::Injured {
            distance: 2,
            harm: 0,
            revision: BodyRevisionId(0),
            ..
        }
    )));
    assert_eq!(session.game().intents().len(), before.intents().len() + 1);
    assert_eq!(
        session.game().bodies().get(SUBJECT).unwrap().revision,
        BodyRevisionId(0)
    );
    assert_eq!(
        session.game().current_anatomy(SUBJECT).unwrap().revision,
        BodyRevisionId(0)
    );
}

#[test]
fn rest_without_a_dressing_does_not_heal_or_bump_revision() {
    let (mut session, dressings, _) = setup();
    attach(&mut session, dressings[0], 1);
    session.fall(5).unwrap();
    session.rest().unwrap();
    session.fall(5).unwrap();
    session.rest().unwrap();
    session.fall(5).unwrap();

    let before = session.game().bodies().get(SUBJECT).unwrap().clone();
    let events = session.rest().unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::Rested {
            recovered: 0,
            revision: BodyRevisionId(5),
            ..
        }
    )));
    let after = session.game().bodies().get(SUBJECT).unwrap();
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.wound, before.wound);
    assert_eq!(after.vitality, before.vitality);
}

#[test]
fn non_controlled_body_change_is_rejected_without_mutation() {
    let (mut session, _, _) = setup();
    let before = session.clone();
    assert!(matches!(
        session.apply_game(GameIntent::Fall {
            tick: session.game().next_tick(),
            subject: SubjectId(9),
            distance: 5,
        }),
        Err(SessionError::NotControlled { .. })
    ));
    assert_eq!(session, before);
}
