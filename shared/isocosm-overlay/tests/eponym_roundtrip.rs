// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Byte round-trip coverage for Eponym's overlay vocabulary (D18), in the
//! same format as `core_roundtrip.rs` and `mesocosm_roundtrip.rs`.

use isocosm_overlay::eponym::{
    ActTarget, Actuation, Blow, Claim, CreativeIntent, EponymHandoff, EponymHandoffEnvelope,
    EponymIntent, EponymIntentEnvelope, FirstLife, LifeCheckpoint, LifeChoice, Manner, Motion,
    PlayerAct, PlayerActKind, Proposal, StartTime, Succession, Successor, Telling, Term, TermSide,
    TimedAct, WorkKey,
};
use isocosm_overlay::{
    ActKey, EntityHandle, EventHandle, Harm, Intent, LineageHandle, PartHandle, ParticipantHandle,
    PlaceHandle, Pointable, Tick, WorldPoint, Wound, WoundSeverity,
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

fn actuation() -> Actuation {
    Actuation {
        body: entity(1),
        motion: Motion {
            at: PlaceHandle(4),
            position: WorldPoint([12, -3, 7]),
            fell: 2,
        },
        acts: vec![
            TimedAct {
                act: ActKey("strike".into()),
                target: Some(ActTarget::Thing(entity(9))),
            },
            TimedAct {
                act: ActKey("brace".into()),
                target: None,
            },
            TimedAct {
                act: ActKey("anchor".into()),
                target: Some(ActTarget::Place(PlaceHandle(4))),
            },
        ],
    }
}

fn proposal(under: Option<EventHandle>) -> Proposal {
    Proposal {
        from: entity(1),
        to: entity(2),
        work: WorkKey("smith".into()),
        terms: vec![
            Term {
                side: TermSide::Offered,
                what: "meal".into(),
                count: 3,
            },
            Term {
                side: TermSide::Asked,
                what: "blade".into(),
                count: 1,
            },
        ],
        under,
    }
}

fn telling(manner: Manner) -> Telling {
    Telling {
        from: entity(1),
        to: entity(2),
        claim: Claim {
            about: Pointable::Event(EventHandle(77)),
            as_told: vec![1, 2, 3],
        },
        manner,
    }
}

fn harm() -> Harm {
    Harm {
        vigour_drained: 14,
        wounds: vec![
            Wound {
                part: PartHandle(3),
                severity: WoundSeverity::Minor,
            },
            Wound {
                part: PartHandle(5),
                severity: WoundSeverity::Severed,
            },
        ],
    }
}

#[test]
fn actuation_roundtrips_with_every_act_target() {
    roundtrips(&actuation());
    roundtrips(&Motion {
        at: PlaceHandle(1),
        position: WorldPoint([0, 0, 0]),
        fell: 0,
    });
    roundtrips(&WorldPoint([i32::MIN, 0, i32::MAX]));
}

#[test]
fn proposal_roundtrips_fresh_and_under_a_standing_agreement() {
    roundtrips(&proposal(None));
    roundtrips(&proposal(Some(EventHandle(21))));
}

#[test]
fn telling_roundtrips_every_manner() {
    for manner in [
        Manner::Plain,
        Manner::Intimidate,
        Manner::Persuade,
        Manner::Deceive,
    ] {
        roundtrips(&telling(manner));
    }
}

#[test]
fn player_act_roundtrips_naming_and_notes() {
    roundtrips(&PlayerAct {
        subject: entity(1),
        kind: PlayerActKind::Name {
            of: Pointable::Entity(entity(2)),
            name: "Otta".into(),
        },
    });
    roundtrips(&PlayerAct {
        subject: entity(1),
        kind: PlayerActKind::Note {
            about: Some(Pointable::Place(PlaceHandle(4))),
            text: "the bell is cracked".into(),
        },
    });
    roundtrips(&PlayerAct {
        subject: entity(1),
        kind: PlayerActKind::Note {
            about: None,
            text: "rain again".into(),
        },
    });
}

#[test]
fn first_life_roundtrips_every_start_and_choice() {
    for start in [
        StartTime::Habitability,
        StartTime::Society,
        StartTime::At(Tick(9_000)),
    ] {
        for begins_as in [
            LifeChoice::Outsider,
            LifeChoice::Birth {
                lineage: LineageHandle(6),
            },
            LifeChoice::Denizen(entity(8)),
        ] {
            roundtrips(&LifeCheckpoint::First(FirstLife { start, begins_as }));
        }
    }
}

#[test]
fn start_time_defaults_to_society() {
    assert_eq!(StartTime::default(), StartTime::Society);
}

#[test]
fn succession_roundtrips_every_successor() {
    for next in [
        Successor::Companion(entity(2)),
        Successor::Another(LifeChoice::Outsider),
        Successor::Another(LifeChoice::Birth {
            lineage: LineageHandle(6),
        }),
        Successor::Another(LifeChoice::Denizen(entity(8))),
        Successor::NewWorld,
    ] {
        roundtrips(&LifeCheckpoint::Death(Succession {
            of: entity(1),
            next,
        }));
    }
}

#[test]
fn creative_intent_roundtrips_tag_in_and_out() {
    roundtrips(&CreativeIntent::TagIn { to: entity(2) });
    roundtrips(&CreativeIntent::TagOut);
}

#[test]
fn eponym_intent_roundtrips_every_variant() {
    roundtrips(&EponymIntent::Drive(actuation()));
    roundtrips(&EponymIntent::Ask(proposal(None)));
    roundtrips(&EponymIntent::Tell(telling(Manner::Persuade)));
    roundtrips(&EponymIntent::Act(PlayerAct {
        subject: entity(1),
        kind: PlayerActKind::Name {
            of: Pointable::Entity(entity(1)),
            name: "Brin".into(),
        },
    }));
    roundtrips(&EponymIntent::Checkpoint(LifeCheckpoint::Death(
        Succession {
            of: entity(1),
            next: Successor::Companion(entity(2)),
        },
    )));
    roundtrips(&EponymIntent::Creative(CreativeIntent::TagOut));
}

#[test]
fn eponym_intent_is_creative_matches_the_creative_variant_only() {
    assert!(EponymIntent::Creative(CreativeIntent::TagOut).is_creative());
    assert!(!EponymIntent::Drive(actuation()).is_creative());
    assert!(!EponymIntent::Ask(proposal(None)).is_creative());
    assert!(!EponymIntent::Tell(telling(Manner::Plain)).is_creative());
    assert!(
        !EponymIntent::Checkpoint(LifeCheckpoint::Death(Succession {
            of: entity(1),
            next: Successor::NewWorld,
        }))
        .is_creative()
    );
}

#[test]
fn eponym_intent_envelope_roundtrips() {
    let envelope: EponymIntentEnvelope = EponymIntentEnvelope {
        tick: Tick(7),
        participant: ParticipantHandle(1),
        intent: Intent::Game(EponymIntent::Drive(actuation())),
    };
    roundtrips(&envelope);
}

/// Eponym's handoff is inhabited by ruling 232, so unlike Mesocosm's a value
/// exists to round-trip: a blow with its harm in the sim's terms.
#[test]
fn eponym_handoff_roundtrips_a_blow_with_every_wound_severity() {
    roundtrips(&EponymHandoff::Blow(Blow {
        by: entity(1),
        target: entity(9),
        act: ActKey("strike".into()),
        harm: harm(),
    }));
    for severity in [
        WoundSeverity::Minor,
        WoundSeverity::Serious,
        WoundSeverity::Crippling,
        WoundSeverity::Severed,
    ] {
        roundtrips(&Wound {
            part: PartHandle(1),
            severity,
        });
    }
    assert!(WoundSeverity::Minor < WoundSeverity::Severed);
}

#[test]
fn eponym_handoff_envelope_roundtrips() {
    let envelope: EponymHandoffEnvelope = EponymHandoffEnvelope {
        tick: Tick(8),
        subject: entity(9),
        outcome: EponymHandoff::Blow(Blow {
            by: entity(1),
            target: entity(9),
            act: ActKey("strike".into()),
            harm: harm(),
        }),
    };
    roundtrips(&envelope);
}
