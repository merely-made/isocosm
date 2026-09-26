// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Byte round-trip coverage for the wing-level, game-neutral types (D18:
//! every contract type round-trips through bytes). Uses `serde_json`, the
//! format `shared/isocosm` already serializes saves and receipts with
//! (`serde_json::to_vec` / `from_slice`, `isocosm::history::Session::save`).
//!
//! A small local payload stands in for a game's own intent and handoff
//! vocabulary, so these tests exercise [`IntentEnvelope`] and
//! [`HandoffEnvelope`] without reaching into [`isocosm_overlay::mesocosm`] —
//! that module has its own round-trip file.

use std::collections::BTreeSet;

use isocosm_overlay::{
    ActKey, AttentionChange, AttentionSet, CandidateHandle, EntityHandle, EventHandle, EventRecord,
    EventTopic, FactionHandle, HandoffEnvelope, Harm, Intent, IntentEnvelope, LineageHandle,
    PartHandle, ParticipantHandle, PlaceHandle, Pointable, Tick, ViewHandle, WorldPoint, Wound,
    WoundSeverity,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum DummyPayload {
    Ping,
    Value(u32),
}

#[test]
fn tick_roundtrips() {
    roundtrips(&Tick(0));
    roundtrips(&Tick(u64::MAX));
}

#[test]
fn tick_next_saturates_instead_of_wrapping() {
    assert_eq!(Tick(u64::MAX).next(), Tick(u64::MAX));
    assert_eq!(Tick(0).next(), Tick(1));
}

#[test]
fn handles_roundtrip() {
    roundtrips(&EntityHandle(7));
    roundtrips(&PlaceHandle(9));
    roundtrips(&CandidateHandle(11));
    roundtrips(&ParticipantHandle(13));
    roundtrips(&LineageHandle(15));
    roundtrips(&FactionHandle(17));
    roundtrips(&EventHandle(19));
}

#[test]
fn intent_envelope_roundtrips_a_generic_payload_and_attention() {
    roundtrips(&IntentEnvelope {
        tick: Tick(3),
        participant: ParticipantHandle(1),
        intent: Intent::Game(DummyPayload::Ping),
    });
    roundtrips(&IntentEnvelope {
        tick: Tick(4),
        participant: ParticipantHandle(2),
        intent: Intent::Game(DummyPayload::Value(42)),
    });
    roundtrips(&IntentEnvelope::<DummyPayload> {
        tick: Tick(5),
        participant: ParticipantHandle(1),
        intent: Intent::Attention(AttentionChange::Examine(PlaceHandle(3))),
    });
}

#[test]
fn event_topic_roundtrips() {
    roundtrips(&EventTopic("mesocosm:born".into()));
}

#[test]
fn attention_changes_roundtrip() {
    for pointable in [
        Pointable::Entity(EntityHandle(1)),
        Pointable::Place(PlaceHandle(2)),
        Pointable::Lineage(LineageHandle(3)),
        Pointable::Faction(FactionHandle(4)),
        Pointable::Event(EventHandle(5)),
    ] {
        roundtrips(&AttentionChange::Pin(pointable));
        roundtrips(&AttentionChange::Unpin(pointable));
    }
    roundtrips(&AttentionChange::Examine(PlaceHandle(6)));
    roundtrips(&AttentionChange::StopExamining);
}

#[test]
fn attention_set_roundtrips() {
    roundtrips(&AttentionSet::default());
    roundtrips(&AttentionSet {
        played: BTreeSet::from([EntityHandle(1), EntityHandle(2)]),
        pinned: BTreeSet::from([
            Pointable::Place(PlaceHandle(3)),
            Pointable::Event(EventHandle(4)),
        ]),
        examined: Some(PlaceHandle(3)),
        care: BTreeSet::from([Pointable::Lineage(LineageHandle(5))]),
    });
}

#[test]
fn event_record_roundtrips_with_an_opaque_payload() {
    roundtrips(&EventRecord {
        tick: Tick(5),
        topic: EventTopic("mesocosm:born".into()),
        payload: vec![1, 2, 3, 4],
    });
    roundtrips(&EventRecord {
        tick: Tick(6),
        topic: EventTopic("mesocosm:died".into()),
        payload: vec![],
    });
}

#[test]
fn view_handle_roundtrips() {
    roundtrips(&ViewHandle {
        tick: Tick(10),
        generation: 0,
    });
}

#[test]
fn handoff_envelope_roundtrips_a_generic_outcome() {
    roundtrips(&HandoffEnvelope {
        tick: Tick(1),
        subject: EntityHandle(2),
        outcome: DummyPayload::Value(9),
    });
}

/// The shapes lifted from the game modules on 2026-09-26: a voxel point, an
/// act key, and harm in the sim's terms (ruling 123).
#[test]
fn lifted_shapes_roundtrip() {
    roundtrips(&WorldPoint([i32::MIN, 0, i32::MAX]));
    roundtrips(&ActKey("strike".into()));
    roundtrips(&PartHandle(21));
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
    roundtrips(&Harm {
        vigour_drained: 14,
        wounds: vec![Wound {
            part: PartHandle(3),
            severity: WoundSeverity::Crippling,
        }],
    });
    assert!(WoundSeverity::Minor < WoundSeverity::Severed);
}
