// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "support/combat_fixture.rs"]
mod combat_fixture;

use paredros_identity::Tick;
use paredros_world::{
    AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment, CombatRules, GameEvent, GameIntent,
    GameState, ItemLocation, PartCapability, PartFunction, ResourceKind, ResourceReserve, Session,
    StrikeOutcome, SubjectSheet, SubjectSheetInput, TechniqueId, TechniqueInputs,
    TechniqueKnowledge,
};

use combat_fixture::{TARGET, prepared};

fn target_sheet(game: &GameState) -> SubjectSheet {
    let target = game.current_anatomy(TARGET).unwrap();
    let knowledge = TechniqueKnowledge {
        subject: TARGET,
        learned: vec![TechniqueId::ArrestFall],
    };
    let inputs = TechniqueInputs {
        occupied_parts: Vec::new(),
        part_capabilities: vec![PartCapability {
            part: mesocosm_core::PartId(2),
            function: PartFunction::Adhesion,
            reach_voxels: 8,
            load_capacity_mg: 100_000,
        }],
        equipment: Vec::new(),
        resources: vec![ResourceReserve {
            kind: ResourceKind::Adhesive,
            available_units: 1,
        }],
        environment: ArrestFallEnvironment {
            support_present: true,
            support_distance_voxels: 1,
            arrest_load_mg: 1,
            support_load_capacity_mg: 100_000,
            adhesive_surface: AdhesiveSurface::Suitable,
            adhesive_resource: AdhesiveResource::Available,
        },
    };
    SubjectSheet::from_input(SubjectSheetInput {
        subject: TARGET,
        revision: target.revision,
        current_revision: target.revision,
        body: &target.document,
        knowledge: &knowledge,
        inputs: &inputs,
        part_names: &[],
        selected_part: Some(mesocosm_core::PartId(2)),
    })
}

#[test]
fn released_limbs_hit_then_sever_release_and_kill_in_one_replayable_cut() {
    let (mut session, item) = prepared();
    let action_tick = Tick(session.action().unwrap().last_tick.0 + 1);
    session.charge(action_tick).unwrap();
    let action_tick = Tick(action_tick.0 + 1);
    session.charge(action_tick).unwrap();
    let events = session
        .release_against(
            TARGET,
            action_tick,
            CombatRules {
                base_harm: 100,
                sever_threshold: 1,
                ..CombatRules::default()
            },
        )
        .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::VolleyResolved { strikes, harm, .. }
            if *harm > 0
                && strikes.len() == 2
                && strikes.iter().all(|strike| matches!(strike.outcome, StrikeOutcome::Hit { .. }))
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::ItemReleased { item: released, .. } if *released == item
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::Died { subject, .. } if *subject == TARGET
    )));
    assert!(
        !session
            .session()
            .game()
            .bodies()
            .get(TARGET)
            .unwrap()
            .alive()
    );
    assert!(
        session
            .session()
            .game()
            .anatomies()
            .get(TARGET)
            .unwrap()
            .document
            .part(mesocosm_core::PartId(2))
            .unwrap()
            .severed
    );
    assert!(matches!(
        session.session().game().items().get(item).unwrap().location,
        ItemLocation::At(_)
    ));
    assert_eq!(
        paredros_world::timed_action::TimedActionSession::restore(&session.save().unwrap())
            .unwrap(),
        session
    );
}

#[test]
fn short_reach_records_a_miss_without_target_mutation() {
    let (mut session, _) = prepared();
    let before = session.session().game().clone();
    let action_tick = Tick(session.action().unwrap().last_tick.0 + 1);
    session.charge(action_tick).unwrap();
    let action_tick = Tick(action_tick.0 + 1);
    session.charge(action_tick).unwrap();
    let events = session
        .release_against(
            TARGET,
            action_tick,
            CombatRules {
                max_reach: 1,
                ..CombatRules::default()
            },
        )
        .unwrap();
    assert!(matches!(
        events.as_slice(),
        [GameEvent::VolleyResolved { harm: 0, strikes, .. }]
            if strikes.iter().all(|strike| strike.outcome == StrikeOutcome::Miss)
    ));
    assert_eq!(
        session.session().game().bodies().get(TARGET),
        before.bodies().get(TARGET)
    );
    assert_eq!(
        session.session().game().anatomies().get(TARGET),
        before.anatomies().get(TARGET)
    );
}

#[test]
fn surviving_target_loses_the_part_from_its_sheet_then_can_resume_after_restore() {
    let (mut session, _) = prepared();
    assert!(
        target_sheet(session.session().game())
            .actions
            .iter()
            .any(|action| action.available)
    );
    let action_tick = Tick(session.action().unwrap().last_tick.0 + 1);
    session.charge(action_tick).unwrap();
    let action_tick = Tick(action_tick.0 + 1);
    session.charge(action_tick).unwrap();
    session
        .release_against(
            TARGET,
            action_tick,
            CombatRules {
                base_harm: 1,
                sever_threshold: 1,
                ..CombatRules::default()
            },
        )
        .unwrap();
    let sheet = target_sheet(session.session().game());
    assert!(
        sheet
            .parts
            .iter()
            .any(|part| part.id == mesocosm_core::PartId(2) && part.severed)
    );
    assert!(sheet.actions.iter().all(|action| !action.available));

    let restored = GameState::restore(&session.session().game().save().unwrap()).unwrap();
    assert!(restored.bodies().get(TARGET).unwrap().alive());
    let mut resumed = Session::begin(restored, TARGET).unwrap();
    let tick = resumed.game().next_tick();
    assert!(matches!(
        resumed
            .apply_game(GameIntent::Wait {
                tick,
                subject: TARGET,
            })
            .unwrap()
            .as_slice(),
        [GameEvent::Waited { subject, .. }, ..] if *subject == TARGET
    ));
}
