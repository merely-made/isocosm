// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Serialize)]
struct BeforeAgreement {
    id: Key,
    tick: Tick,
    place: Id,
    subject: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<Id>,
    process: Key,
    cause: Option<Key>,
    strength: u32,
    legend: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    authored: Option<crate::asserted::HistoryLine>,
}

#[test]
fn an_unassociated_event_keeps_its_encoding_and_hashes() {
    let old = BeforeAgreement {
        id: "old-event".into(),
        tick: 0,
        place: 0,
        subject: 1,
        object: None,
        process: "deed:stood-by".into(),
        cause: None,
        strength: 1,
        legend: false,
        authored: None,
    };
    let bytes = serde_json::to_vec(&old).unwrap();
    let event: Event = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(event.agreement, None);
    assert_eq!(serde_json::to_vec(&event).unwrap(), bytes);
    assert_eq!(crate::digest(&event), crate::digest(&old));
    assert_eq!(
        state_witness::hash_value(&event).unwrap(),
        state_witness::hash_value(&old).unwrap()
    );
    let mut associated = event.clone();
    associated.agreement = Some(0);
    assert_ne!(
        state_witness::hash_value(&associated).unwrap(),
        state_witness::hash_value(&event).unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Event>(&serde_json::to_vec(&associated).unwrap()).unwrap(),
        associated
    );
}
