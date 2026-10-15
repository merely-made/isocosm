// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The played-turn endpoint against the sim-host core's fixture world.

#![cfg(feature = "endpoint")]

use chirograph::{IntentInvocation, IntentResult, PortableCardV1, ProjectionSnapshot, ResourceRequest};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm::directing::interim::{OnCollapse, Pace};
use isocosm::directing::readings::Mode;
use isocosm_host::fixtures::fixture;
use mesocosm_runtime::continuation::LoopSettings;
use mesocosm_runtime::endpoint::{ANSWER, CONFIGURE, MesocosmEndpoint, STEP};

fn snapshot(endpoint: &mut MesocosmEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

/// Each disclosed item's id, its card, and the intents it offers.
fn items(endpoint: &mut MesocosmEndpoint, snapshot: &ProjectionSnapshot) -> Vec<(String, PortableCardV1, Vec<String>)> {
    snapshot.scene.active_items_in_order().into_iter().map(|(instance, item)| {
        let id = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(),
            resource: offer.resource }).unwrap().bytes;
        let intents = offer.semantics.actions.iter().map(|a| a.intent.0.clone()).collect();
        (id, serde_json::from_slice(&bytes).unwrap(), intents)
    }).collect()
}

fn invoke(snapshot: &ProjectionSnapshot, id: &str, intent: &str, payload: Vec<u8>) -> IntentInvocation {
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id).unwrap().0;
    IntentInvocation { session: snapshot.session.clone(), target, observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision, intent: intent.into(), payload }
}

fn settings() -> Vec<u8> {
    serde_json::to_vec(&LoopSettings { mode: Mode::Survival, pace: Pace { round: 1, scoring: 6 },
        on_collapse: OnCollapse::Stay }).unwrap()
}

fn configured() -> (MesocosmEndpoint, ProjectionSnapshot) {
    let f = fixture();
    let mut endpoint = MesocosmEndpoint::open(&f.world, f.participants[0]).unwrap();
    let first = snapshot(&mut endpoint);
    assert_eq!(endpoint.invoke(invoke(&first, "turn", CONFIGURE, settings())).unwrap(), IntentResult::Accepted);
    let second = snapshot(&mut endpoint);
    (endpoint, second)
}

#[test]
fn an_unconfigured_turn_offers_only_its_setup() {
    let f = fixture();
    let mut endpoint = MesocosmEndpoint::open(&f.world, f.participants[0]).unwrap();
    let first = snapshot(&mut endpoint);
    let items = items(&mut endpoint, &first);
    assert_eq!(items.len(), 1, "no answer before the loop runs");
    let (id, card, intents) = &items[0];
    assert_eq!(id, "turn");
    assert!(card.values.iter().any(|v| v.label == "Settings" && v.value.starts_with("Choose a loop setup")));
    assert_eq!(intents, &[CONFIGURE.to_string()]);
    assert!(matches!(endpoint.invoke(invoke(&first, "turn", CONFIGURE, b"{}".to_vec())).unwrap(),
        IntentResult::Rejected { .. }), "a setup names mode, pace and collapse choice");
}

#[test]
fn a_configured_turn_steps_and_stales_the_old_revision() {
    let (mut endpoint, second) = configured();
    let items = items(&mut endpoint, &second);
    let (_, card, intents) = &items[0];
    assert!(card.values.iter().any(|v| v.label == "Settings" && v.value.contains("ticks per round")));
    assert_eq!(intents, &[STEP.to_string()]);
    let before = endpoint.world().cut().unwrap().tick;
    let step = invoke(&second, "turn", STEP, Vec::new());
    assert_eq!(endpoint.invoke(step.clone()).unwrap(), IntentResult::Accepted);
    assert!(endpoint.world().cut().unwrap().tick > before, "a round advances the native world");
    assert!(matches!(endpoint.invoke(step).unwrap(), IntentResult::Stale { .. }));
    let third = snapshot(&mut endpoint);
    assert!(matches!(endpoint.invoke(invoke(&third, "turn", ANSWER, Vec::new())).unwrap(),
        IntentResult::Rejected { .. }), "the turn card advertises no answer");
}

#[test]
fn a_held_turn_offers_answers_and_takes_one() {
    let (mut endpoint, mut current) = configured();
    let mut answered = false;
    for _ in 0..400 {
        let items = items(&mut endpoint, &current);
        if let Some((id, _, _)) = items.iter().find(|(id, _, _)| id.starts_with("answer:")) {
            assert_eq!(endpoint.invoke(invoke(&current, id, ANSWER, Vec::new())).unwrap(), IntentResult::Accepted);
            answered = true;
            break;
        }
        let stepping = items[0].2.contains(&STEP.to_string());
        assert!(stepping, "a turn with no answer can step");
        assert_eq!(endpoint.invoke(invoke(&current, "turn", STEP, Vec::new())).unwrap(), IntentResult::Accepted);
        current = snapshot(&mut endpoint);
    }
    assert!(answered, "the loop reaches a held or ended turn within 400 rounds");
}

#[test]
fn the_endpoint_survives_the_wire_encoding() {
    use chirograph::{Carrier, CarrierRequestBody, CarrierResponseBody};
    use graphshell_endpoint::local::LocalCarrier;
    let f = fixture();
    let endpoint = MesocosmEndpoint::open(&f.world, f.participants[0]).unwrap();
    let mut carrier = LocalCarrier::new(endpoint, |_, _| Err("resume is not used".to_string()));
    let CarrierResponseBody::Descriptor(descriptor) = carrier.request(CarrierRequestBody::Discover).unwrap()
        else { panic!("discovery answers a descriptor") };
    let request = descriptor.projections[0].request.clone();
    let CarrierResponseBody::Snapshot(first) = carrier.request(CarrierRequestBody::Snapshot(request)).unwrap()
        else { panic!("a snapshot request answers a snapshot") };
    let CarrierResponseBody::Intent(result) = carrier.request(CarrierRequestBody::Intent(
        invoke(&first, "turn", CONFIGURE, settings()))).unwrap() else { panic!("an intent answers a result") };
    assert_eq!(result, IntentResult::Accepted);
}
