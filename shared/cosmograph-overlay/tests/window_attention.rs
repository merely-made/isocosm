// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph_overlay::{
    AttentionChange, CloseWindowEnvelope, EntityHandle, ParticipantHandle,
    PlaceHandle, PlayerAttention, Pointable, Tick, WindowAttention,
    WindowAttentionEnvelope, WindowHandle,
};
use std::collections::BTreeSet;

#[test]
fn window_attention_envelope_keeps_participant_window_tick_and_change() {
    let input = WindowAttentionEnvelope {
        tick: Tick(7),
        participant: ParticipantHandle(10),
        window: WindowHandle(3),
        change: AttentionChange::Examine(PlaceHandle(4)),
    };
    let bytes = serde_json::to_vec(&input).unwrap();
    let restored: WindowAttentionEnvelope = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, input);
    let close = CloseWindowEnvelope { tick: Tick(8), participant: input.participant, window: input.window };
    let bytes = serde_json::to_vec(&close).unwrap();
    assert_eq!(serde_json::from_slice::<CloseWindowEnvelope>(&bytes).unwrap(), close);
}

#[test]
fn effective_player_attention_carries_both_close_up_regions() {
    let set = PlayerAttention {
        participant: ParticipantHandle(10),
        played: BTreeSet::from([EntityHandle(20)]),
        pinned: BTreeSet::from([Pointable::Entity(EntityHandle(21))]),
        examined: BTreeSet::from([PlaceHandle(1), PlaceHandle(2)]),
        care: BTreeSet::new(),
    };
    let bytes = serde_json::to_vec(&set).unwrap();
    let restored: PlayerAttention = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, set);
    assert_eq!(restored.examined.len(), 2, "effective attention cannot discard one close-up region");
}

#[test]
fn window_attention_input_cannot_author_played_or_native_care() {
    let input = WindowAttention {
        pinned: BTreeSet::from([Pointable::Entity(EntityHandle(21))]),
        examined: Some(PlaceHandle(1)),
    };
    let value = serde_json::to_value(&input).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 2);
    assert!(value.get("played").is_none() && value.get("care").is_none());
}
