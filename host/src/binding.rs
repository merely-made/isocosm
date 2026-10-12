// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm_overlay::{AttentionChange, AttentionSet, Intent, IntentEnvelope, ParticipantHandle, Tick};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    pub mode: String,
    pub controls: Vec<String>,
    pub view_defaults: Vec<u8>,
    pub attention: AttentionSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BindingPlan {
    pub requested: Binding,
    pub changes: Vec<IntentEnvelope<()>>,
}

pub(crate) fn plan(
    current: &Binding,
    requested: Binding,
    participant: ParticipantHandle,
    tick: u64,
) -> BindingPlan {
    let mut changes = Vec::new();
    let mut push = |change| changes.push(envelope(participant, tick, change));
    for point in current.attention.pinned.difference(&requested.attention.pinned) {
        push(AttentionChange::Unpin(*point));
    }
    for point in requested.attention.pinned.difference(&current.attention.pinned) {
        push(AttentionChange::Pin(*point));
    }
    if current.attention.examined != requested.attention.examined {
        push(match requested.attention.examined {
            Some(place) => AttentionChange::Examine(place),
            None => AttentionChange::StopExamining,
        });
    }
    // Played and care are native/profile-derived, not AttentionChange operations.
    // This is a requested cut; no accepted native watch or window switch is claimed.
    BindingPlan { requested, changes }
}

pub(crate) fn envelope(
    participant: ParticipantHandle,
    tick: u64,
    change: AttentionChange,
) -> IntentEnvelope<()> {
    IntentEnvelope {
        tick: Tick(tick),
        participant,
        intent: Intent::Attention(change),
    }
}
