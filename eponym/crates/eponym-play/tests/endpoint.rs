// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The next-life endpoint against the sim-host core's fixture world.

#![cfg(feature = "endpoint")]

use chirograph::{IntentInvocation, IntentResult, PortableCardV1, ProjectionSnapshot, ResourceRequest};
use eponym_play::endpoint::{EponymEndpoint, STEP, TAKE};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use cosmograph::Command;
use cosmograph_host::fixtures::fixture;

fn snapshot(endpoint: &mut EponymEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

fn items(endpoint: &mut EponymEndpoint, snapshot: &ProjectionSnapshot) -> Vec<(String, PortableCardV1, Vec<String>)> {
    snapshot.scene.active_items_in_order().into_iter().map(|(instance, item)| {
        let id = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(),
            resource: offer.resource }).unwrap().bytes;
        let intents = offer.semantics.actions.iter().map(|a| a.intent.0.clone()).collect();
        (id, serde_json::from_slice(&bytes).unwrap(), intents)
    }).collect()
}

fn invoke(snapshot: &ProjectionSnapshot, id: &str, intent: &str) -> IntentInvocation {
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id).unwrap().0;
    IntentInvocation { session: snapshot.session.clone(), target, observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision, intent: intent.into(), payload: Vec::new() }
}

fn life(items: &[(String, PortableCardV1, Vec<String>)]) -> String {
    items.iter().find(|(id, _, _)| id == "eponym").unwrap().1.values.iter()
        .find(|v| v.label == "Life").unwrap().value.clone()
}

#[test]
fn a_living_player_sees_bodies_and_steps_the_world() {
    let f = fixture();
    let mut endpoint = EponymEndpoint::open(&f.world, f.participants[0]).unwrap();
    let first = snapshot(&mut endpoint);
    let items = items(&mut endpoint, &first);
    assert_eq!(life(&items), "Living");
    assert!(items.iter().any(|(id, _, _)| id.starts_with("body:")), "the player's visible bodies are listed");
    assert!(!items.iter().any(|(id, _, _)| id == "next-life" || id.starts_with("life:")));
    let before = endpoint.world().cut().unwrap().tick;
    let step = invoke(&first, "eponym", STEP);
    assert_eq!(endpoint.invoke(step.clone()).unwrap(), IntentResult::Accepted);
    assert_eq!(endpoint.world().cut().unwrap().tick, before + 1, "without an Eponym clock a step is one tick");
    assert!(matches!(endpoint.invoke(step).unwrap(), IntentResult::Stale { .. }));
}

#[test]
fn a_dead_life_awaits_its_successor_and_says_why_none_is_offered() {
    let f = fixture();
    f.world.command(Command::Kill { entity: f.subjects[0] }).unwrap();
    let mut endpoint = EponymEndpoint::open(&f.world, f.participants[0]).unwrap();
    let first = snapshot(&mut endpoint);
    let items = items(&mut endpoint, &first);
    assert_eq!(life(&items), "Awaiting a next life");
    let (_, card, intents) = items.iter().find(|(id, _, _)| id == "next-life")
        .expect("without Eponym mode state the next life explains itself");
    assert!(intents.is_empty() && !card.values.is_empty());
    assert!(!items.iter().any(|(id, _, _)| id.starts_with("life:")));
    assert!(matches!(endpoint.invoke(invoke(&first, "next-life", TAKE)).unwrap(), IntentResult::Rejected { .. }),
        "an explanation card advertises no take");
}

#[test]
fn the_endpoint_survives_the_wire_encoding() {
    use chirograph::{Carrier, CarrierRequestBody, CarrierResponseBody};
    use graphshell_endpoint::local::LocalCarrier;
    let f = fixture();
    let endpoint = EponymEndpoint::open(&f.world, f.participants[0]).unwrap();
    let mut carrier = LocalCarrier::new(endpoint, |_, _| Err("resume is not used".to_string()));
    let CarrierResponseBody::Descriptor(descriptor) = carrier.request(CarrierRequestBody::Discover).unwrap()
        else { panic!("discovery answers a descriptor") };
    let request = descriptor.projections[0].request.clone();
    let CarrierResponseBody::Snapshot(first) = carrier.request(CarrierRequestBody::Snapshot(request)).unwrap()
        else { panic!("a snapshot request answers a snapshot") };
    let CarrierResponseBody::Intent(result) = carrier.request(CarrierRequestBody::Intent(
        invoke(&first, "eponym", STEP))).unwrap() else { panic!("an intent answers a result") };
    assert_eq!(result, IntentResult::Accepted);
}
