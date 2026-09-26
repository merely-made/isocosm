// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Byte round-trip coverage for the VTT's overlay vocabulary (D18), in the
//! same format as `core_roundtrip.rs` and `mesocosm_roundtrip.rs`.

use isocosm_overlay::vtt::{
    ActionKey, Assertion, Calibration, Cell, CellEdit, Condition, Fact, Harm, HookIntent, MapEdit,
    NewCharacter, Pace, PartHandle, RequestId, Resolved, RulesetKey, TableAct, TableActKind,
    TableBatch, TimeIntent, Transfer, Travel, VttHandoff, VttHandoffEnvelope, VttIntent,
    VttIntentEnvelope, WorldPoint, Wound, WoundSeverity,
};
use isocosm_overlay::{
    EntityHandle, EventHandle, FactionHandle, Intent, ParticipantHandle, PlaceHandle, Pointable,
    Tick,
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

fn cell(x: i32, y: i32) -> Cell {
    Cell {
        site: PlaceHandle(3),
        at: WorldPoint([x, y, 0]),
    }
}

fn batch() -> TableBatch {
    TableBatch {
        acts: vec![
            TableAct {
                actor: entity(1),
                kind: TableActKind::Move { to: cell(4, 5) },
            },
            TableAct {
                actor: entity(1),
                kind: TableActKind::Act {
                    action: ActionKey("attack".into()),
                    target: Some(entity(9)),
                },
            },
            TableAct {
                actor: entity(2),
                kind: TableActKind::Act {
                    action: ActionKey("perception".into()),
                    target: None,
                },
            },
        ],
    }
}

fn fact() -> Fact {
    Fact {
        kind: "reveal".into(),
        about: Some(Pointable::Entity(entity(9))),
        text: "the tower-beast nests on the north ledge".into(),
        tags: vec!["faction:scavengers".into()],
    }
}

fn resolved(calibration: Calibration) -> Resolved {
    Resolved {
        request: RequestId(41),
        ruleset: RulesetKey("pf2e".into()),
        actor: entity(1),
        target: entity(9),
        action: ActionKey("attack".into()),
        harm: Harm {
            vigour_drained: 7,
            wounds: vec![Wound {
                part: PartHandle(2),
                severity: WoundSeverity::Serious,
            }],
        },
        defeated: vec![entity(9)],
        dead: vec![],
        conditions: vec![Condition {
            subject: entity(9),
            condition: "prone".into(),
            magnitude: 1,
        }],
        displaced: vec![(entity(9), cell(6, 5))],
        transferred: vec![Transfer {
            item: entity(30),
            from: entity(9),
            to: entity(1),
        }],
        calibration,
    }
}

#[test]
fn table_batch_roundtrips_acts_and_moves() {
    roundtrips(&batch());
    roundtrips(&TableBatch { acts: vec![] });
    roundtrips(&WorldPoint([i32::MIN, 0, i32::MAX]));
}

#[test]
fn assertion_roundtrips_every_variant_and_nests() {
    roundtrips(&Assertion::Fact(fact()));
    roundtrips(&Assertion::Edit(MapEdit {
        site: PlaceHandle(3),
        cells: vec![
            CellEdit {
                at: WorldPoint([0, 0, 0]),
                kind: "stone".into(),
                height: 2,
            },
            CellEdit {
                at: WorldPoint([1, 0, 0]),
                kind: "water".into(),
                height: 0,
            },
        ],
    }));
    roundtrips(&Assertion::Character(NewCharacter {
        name: "Mira".into(),
        faction: Some(FactionHandle(5)),
        at: cell(2, 2),
        owner: Some(ParticipantHandle(2)),
    }));
    roundtrips(&Assertion::Character(NewCharacter {
        name: "Elian".into(),
        faction: None,
        at: cell(8, 8),
        owner: None,
    }));
    roundtrips(&Assertion::Storylet {
        storylet: "conclusion".into(),
        asserts: vec![Assertion::Fact(fact())],
    });
    roundtrips(&Assertion::PackForced {
        pack: "watchtower".into(),
        asserts: vec![
            Assertion::Fact(fact()),
            Assertion::Storylet {
                storylet: "arrival".into(),
                asserts: vec![],
            },
        ],
    });
}

#[test]
fn time_intent_roundtrips_all_four() {
    roundtrips(&TimeIntent::Downtime { ticks: 10_000 });
    roundtrips(&TimeIntent::Consent {
        downtime: EventHandle(50),
    });
    roundtrips(&TimeIntent::SimOff);
    roundtrips(&TimeIntent::SimOn);
}

#[test]
fn travel_roundtrips_and_pace_defaults_to_normal() {
    roundtrips(&Travel {
        party: FactionHandle(1),
        to: PlaceHandle(12),
        pace: Pace(50),
    });
    assert_eq!(Pace::default(), Pace(100));
}

#[test]
fn hook_intent_roundtrips_all_three() {
    roundtrips(&HookIntent::TakeUp {
        arc: EventHandle(60),
    });
    roundtrips(&HookIntent::Drop {
        arc: EventHandle(60),
    });
    roundtrips(&HookIntent::Reshape {
        arc: EventHandle(60),
        note: "the heir is Brin".into(),
    });
}

#[test]
fn vtt_intent_roundtrips_every_variant() {
    roundtrips(&VttIntent::Table(batch()));
    roundtrips(&VttIntent::Assert(Assertion::Fact(fact())));
    roundtrips(&VttIntent::Time(TimeIntent::SimOff));
    roundtrips(&VttIntent::Travel(Travel {
        party: FactionHandle(1),
        to: PlaceHandle(12),
        pace: Pace::default(),
    }));
    roundtrips(&VttIntent::Hook(HookIntent::Drop {
        arc: EventHandle(60),
    }));
}

#[test]
fn vtt_intent_is_assertion_matches_the_assert_variant_only() {
    assert!(VttIntent::Assert(Assertion::Fact(fact())).is_assertion());
    assert!(!VttIntent::Table(batch()).is_assertion());
    assert!(!VttIntent::Time(TimeIntent::SimOn).is_assertion());
}

#[test]
fn vtt_intent_envelope_roundtrips() {
    let envelope: VttIntentEnvelope = VttIntentEnvelope {
        tick: Tick(7),
        participant: ParticipantHandle(1),
        intent: Intent::Game(VttIntent::Table(batch())),
    };
    roundtrips(&envelope);
}

#[test]
fn vtt_handoff_roundtrips_calibrated_and_uncalibrated() {
    roundtrips(&VttHandoff::Resolved(resolved(Calibration::Calibrated)));
    roundtrips(&VttHandoff::Resolved(resolved(Calibration::Uncalibrated)));
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
}

#[test]
fn vtt_handoff_envelope_roundtrips() {
    let envelope: VttHandoffEnvelope = VttHandoffEnvelope {
        tick: Tick(8),
        subject: entity(9),
        outcome: VttHandoff::Resolved(resolved(Calibration::Uncalibrated)),
    };
    roundtrips(&envelope);
}
