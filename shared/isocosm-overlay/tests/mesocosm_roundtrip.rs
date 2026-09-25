// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Byte round-trip coverage for Mesocosm's overlay vocabulary (D18), in the
//! same format as `core_roundtrip.rs`.

use isocosm_overlay::mesocosm::{
    BirthAnswer, Boldness, CheckpointAnswer, CheckpointAnswerKind, CompetitorStance, DeathAnswer,
    DevIntent, Directive, DirectiveKind, MesocosmHandoff, MesocosmIntent, MesocosmIntentEnvelope,
    Nudge, Places, PlayerAct, PlayerActKind, Priorities, PriorityKey, RevisionAnswer, Stances,
    WorldPoint,
};
use isocosm_overlay::{
    CandidateHandle, EntityHandle, Intent, ParticipantHandle, PlaceHandle, Tick,
};
use serde::{Deserialize, Serialize};

fn roundtrips<T>(value: &T)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let bytes = serde_json::to_vec(value).expect("serializes");
    let back: T = serde_json::from_slice(&bytes).expect("deserializes");
    assert_eq!(*value, back);
}

fn entity(id: u64) -> EntityHandle {
    EntityHandle(id)
}

#[test]
fn priorities_roundtrip() {
    roundtrips(&Priorities::default());
    roundtrips(&Priorities {
        order: vec![
            PriorityKey("need:hunger".into()),
            PriorityKey("ability:graft".into()),
        ],
    });
}

#[test]
fn places_roundtrip() {
    roundtrips(&Places::default());
    roundtrips(&Places {
        range: vec![PlaceHandle(1), PlaceHandle(2)],
        avoid: vec![PlaceHandle(3)],
        home: Some(PlaceHandle(1)),
    });
}

#[test]
fn stances_roundtrip() {
    for boldness in [Boldness::Bold, Boldness::Cautious] {
        for competitor in [
            CompetitorStance::Contest,
            CompetitorStance::Yield,
            CompetitorStance::Share,
        ] {
            roundtrips(&Stances {
                boldness,
                competitor,
            });
        }
    }
}

#[test]
fn nudge_roundtrips_both_kinds() {
    roundtrips(&Nudge::GoTo(PlaceHandle(4)));
    roundtrips(&Nudge::EatThat(entity(5)));
}

#[test]
fn directive_roundtrips_every_kind() {
    roundtrips(&Directive {
        entity: entity(1),
        kind: DirectiveKind::Priorities(Priorities {
            order: vec![PriorityKey("need:hunger".into())],
        }),
    });
    roundtrips(&Directive {
        entity: entity(1),
        kind: DirectiveKind::Places(Places {
            range: vec![PlaceHandle(2)],
            avoid: vec![],
            home: None,
        }),
    });
    roundtrips(&Directive {
        entity: entity(1),
        kind: DirectiveKind::Stances(Stances {
            boldness: Boldness::Bold,
            competitor: CompetitorStance::Yield,
        }),
    });
    roundtrips(&Directive {
        entity: entity(1),
        kind: DirectiveKind::Nudge(Nudge::EatThat(entity(6))),
    });
}

#[test]
fn player_act_roundtrips_speciate() {
    roundtrips(&PlayerAct {
        entity: entity(1),
        kind: PlayerActKind::Speciate {
            name: "the long-armed".into(),
        },
    });
}

#[test]
fn checkpoint_answer_roundtrips_birth_death_and_epoch_review() {
    roundtrips(&CheckpointAnswer {
        entity: entity(1),
        kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent),
    });
    roundtrips(&CheckpointAnswer {
        entity: entity(1),
        kind: CheckpointAnswerKind::Birth(BirthAnswer::TakeOffspring),
    });
    roundtrips(&CheckpointAnswer {
        entity: entity(1),
        kind: CheckpointAnswerKind::Death(DeathAnswer { next: entity(2) }),
    });
    roundtrips(&CheckpointAnswer {
        entity: entity(1),
        kind: CheckpointAnswerKind::EpochReview(RevisionAnswer {
            revision: CandidateHandle(42),
        }),
    });
}

#[test]
fn world_point_roundtrips() {
    roundtrips(&WorldPoint([1, -2, 3]));
}

#[test]
fn dev_intent_roundtrips_all_four() {
    roundtrips(&DevIntent::EndEpoch);
    roundtrips(&DevIntent::ForceBirth {
        organism: entity(1),
    });
    roundtrips(&DevIntent::Kill {
        organism: entity(1),
    });
    roundtrips(&DevIntent::PlaceMatter {
        at: WorldPoint([0, 0, 0]),
        mass_mg: 500,
    });
}

#[test]
fn mesocosm_intent_roundtrips_every_variant() {
    roundtrips(&MesocosmIntent::Directive(Directive {
        entity: entity(1),
        kind: DirectiveKind::Nudge(Nudge::GoTo(PlaceHandle(1))),
    }));
    roundtrips(&MesocosmIntent::Act(PlayerAct {
        entity: entity(1),
        kind: PlayerActKind::Speciate {
            name: "the long-armed".into(),
        },
    }));
    roundtrips(&MesocosmIntent::Checkpoint(CheckpointAnswer {
        entity: entity(1),
        kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent),
    }));
    roundtrips(&MesocosmIntent::Dev(DevIntent::EndEpoch));
}

#[test]
fn mesocosm_intent_is_dev_matches_the_dev_variant_only() {
    assert!(
        !MesocosmIntent::Directive(Directive {
            entity: entity(1),
            kind: DirectiveKind::Nudge(Nudge::GoTo(PlaceHandle(1))),
        })
        .is_dev()
    );
    assert!(
        !MesocosmIntent::Act(PlayerAct {
            entity: entity(1),
            kind: PlayerActKind::Speciate {
                name: "the long-armed".into(),
            },
        })
        .is_dev()
    );
    assert!(MesocosmIntent::Dev(DevIntent::EndEpoch).is_dev());
}

#[test]
fn mesocosm_intent_envelope_roundtrips() {
    let envelope: MesocosmIntentEnvelope = MesocosmIntentEnvelope {
        tick: Tick(7),
        participant: ParticipantHandle(1),
        intent: Intent::Game(MesocosmIntent::Dev(DevIntent::EndEpoch)),
    };
    roundtrips(&envelope);
}

/// `MesocosmHandoff` is uninhabited by ruling (overlay plan §3: Mesocosm's
/// handoff is empty), so there is no value to round-trip. What is testable
/// is the claim itself: nothing decodes into it, because there is no valid
/// encoding of a variant that does not exist.
#[test]
fn mesocosm_handoff_never_decodes_because_it_is_uninhabited() {
    for input in ["null", "{}", "\"anything\"", "0", "[]"] {
        assert!(
            serde_json::from_str::<MesocosmHandoff>(input).is_err(),
            "unexpectedly decoded {input:?} into an uninhabited type"
        );
    }
}
