// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A native practice striker that records a real severing volley.

use crate::identity::{BodyRevisionId, SubjectId};
use crate::timed_action::{
    Direction, LimbBinding, StrikeReceipt, TimedActionError, TimedActionSession,
};
use crate::{CombatRules, GameEvent, GameIntent, GameState, part_bounds};
use cosmograph::lineage::SpeciesId;
use isometer_core::PartId;

pub const STRIKER: SubjectId = SubjectId(74);

/// Pre-found native practice actor for hosts and fixtures that explicitly
/// need an external volley. The ordinary shared scene stays unchanged.
pub fn found_practice_striker(game: &mut GameState, at: [i32; 3]) {
    let mut tick = game.next_tick();
    game.apply(GameIntent::Generate {
        tick,
        subject: STRIKER,
        body_seed: 7,
        at,
    })
    .expect("found practice striker");
    tick = game.next_tick();
    game.apply(GameIntent::Name {
        tick,
        subject: STRIKER,
        name: crate::Name::new("Practice striker").unwrap(),
    })
    .expect("name practice striker");
    let anatomy = game.native_anatomy(STRIKER, SpeciesId(71)).unwrap();
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject: STRIKER,
        revision: BodyRevisionId(0),
        document: Box::new(anatomy),
    })
    .expect("admit practice striker anatomy");
}

fn strike(game: &GameState, target: SubjectId, part: PartId) -> StrikeReceipt {
    let before = game
        .current_anatomy(target)
        .expect("target anatomy")
        .clone();
    let root = part_bounds(&before.document, before.document.root).expect("root bounds");
    let limb = part_bounds(&before.document, part).expect("target limb bounds");
    let direction = if limb.min[0] >= root.max[0] {
        Direction::Right
    } else {
        assert!(
            limb.max[0] <= root.min[0],
            "fixture cut must target a lateral leaf"
        );
        Direction::Left
    };
    StrikeReceipt {
        binding: LimbBinding {
            part,
            revision: BodyRevisionId(0),
        },
        direction,
        charge: 24,
    }
}

/// Build the motion suite's native founding and player-volley sequence. Its
/// direct GameState fixture predates controlled Session saves.
pub fn volley(game: &GameState, target: SubjectId, part: PartId) -> Vec<GameIntent> {
    let at = game.intents().iter().find_map(|intent| match intent {
        GameIntent::Generate { subject, at, .. } if *subject == target => Some(*at),
        _ => None,
    }).expect("target founding position");
    let motion: Vec<_> = game.intents().iter().filter(|intent| {
        intent.subject() == Some(target) && matches!(intent,
            GameIntent::ConfigureMovementProfile { .. } | GameIntent::AdvanceMotion { .. })
    }).cloned().collect();
    let mut candidate = game.clone();
    found_practice_striker(&mut candidate, at);
    let mut intents = candidate.intents()[game.intents().len()..].to_vec();
    // Reproduce the target's accepted fixed-point trajectory from its founding
    // cell. Integer position alone loses the fractional contact witness.
    for intent in motion {
        let intent = match intent {
            GameIntent::ConfigureMovementProfile { revision, profile, .. } =>
                GameIntent::ConfigureMovementProfile { tick: candidate.next_tick(),
                    subject: STRIKER, revision, profile },
            GameIntent::AdvanceMotion { revision, step, input, rules, .. } =>
                GameIntent::AdvanceMotion { tick: candidate.next_tick(), subject: STRIKER,
                    revision, step, input, rules },
            _ => unreachable!("only motion intents selected"),
        };
        candidate.apply(intent.clone()).expect("accepted practice trajectory");
        intents.push(intent);
    }
    assert_eq!(candidate.pose(STRIKER), game.pose(target), "native practice contact pose");
    intents.push(GameIntent::ResolveVolley {
        tick: candidate.next_tick(), actor: STRIKER, target,
        strikes: vec![strike(game, target, part)], rules: CombatRules::default(),
    });
    intents
}

fn external_volley(game: &GameState, target: SubjectId, part: PartId) -> GameIntent {
    GameIntent::ExternalVolley {
        tick: game.next_tick(),
        actor: STRIKER,
        actor_revision: game
            .bodies()
            .get(STRIKER)
            .expect("pre-founded striker")
            .revision,
        target,
        strikes: vec![strike(game, target, part)],
        rules: CombatRules::default(),
    }
}

/// Apply a fixed native volley to the nominated body, including a non-played
/// target used by the native-source fixture.
pub fn sever_target(
    action: &mut TimedActionSession,
    target: SubjectId,
    part: PartId,
) -> Result<Vec<GameEvent>, TimedActionError> {
    let game = action.session().game();
    if game
        .current_anatomy(target)
        .ok()
        .and_then(|body| body.document.part(part))
        .is_some_and(|part| part.severed)
    {
        return Ok(Vec::new());
    }
    if game.bodies().get(STRIKER).is_none() {
        return Err(TimedActionError::Session(crate::SessionError::Ineligible {
            subject: STRIKER,
        }));
    }
    let GameIntent::ExternalVolley {
        actor,
        actor_revision,
        target,
        strikes,
        rules,
        ..
    } = external_volley(game, target, part)
    else {
        unreachable!("fixture constructs an external volley")
    };
    action.apply_external_volley(actor, actor_revision, target, strikes, rules)
}

/// Apply the fixture's native practice strike to the played subject while
/// preserving timed-action reconciliation and atomic failure behavior.
pub fn injure_played(action: &mut TimedActionSession) -> Result<Vec<GameEvent>, TimedActionError> {
    let target = action.session().control().played();
    if action
        .session()
        .game()
        .current_anatomy(target)
        .ok()
        .and_then(|body| body.document.part(PartId(1)))
        .is_some_and(|part| part.severed)
    {
        return Ok(Vec::new());
    }
    sever_target(action, target, PartId(1))
}
