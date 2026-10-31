// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One door (ruling 925): the owned runtime and the played-turn endpoint
//! drive one core world and one game in its `mesocosm` section, each
//! carrying on from the other's rounds.

#![cfg(feature = "endpoint")]

#[path = "runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph_overlay::ParticipantHandle;
use mesocosm_runtime::endpoint::{ANSWER, MesocosmEndpoint, STEP};

fn snapshot(endpoint: &mut MesocosmEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

fn invoke(snapshot: &ProjectionSnapshot, id: &str, intent: &str) -> IntentInvocation {
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id).unwrap().0;
    IntentInvocation { session: snapshot.session.clone(), target, observed_epoch: snapshot.scene.epoch,
        observed_revision: snapshot.scene.revision, intent: intent.into(), payload: Vec::new() }
}

fn step(snapshot: &ProjectionSnapshot) -> IntentInvocation { invoke(snapshot, "turn", STEP) }

fn ids(snapshot: &ProjectionSnapshot) -> Vec<String> {
    snapshot.scene.active_items_in_order().into_iter().map(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone()).collect()
}

#[test]
fn the_runtime_and_the_endpoint_carry_on_from_each_others_rounds() {
    let mut runtime = worlds::offered();
    let mut endpoint = MesocosmEndpoint::open(runtime.world(), ParticipantHandle(runtime.interim().participant)).unwrap();
    let tick = runtime.tick();
    let first = snapshot(&mut endpoint);
    assert_eq!(endpoint.invoke(step(&first)).unwrap(), IntentResult::Accepted, "the endpoint steps the runtime's game");
    assert_eq!(runtime.tick(), tick + 1);
    assert_eq!(runtime.step(1), 1, "the runtime adopts the endpoint's game and carries on");
    assert_eq!((runtime.tick(), runtime.fault()), (tick + 2, None));
    let seen = snapshot(&mut endpoint);
    assert_eq!(runtime.step(1), 1);
    assert!(matches!(endpoint.invoke(step(&seen)).unwrap(), IntentResult::Stale { .. }),
        "the runtime's round stales the endpoint's snapshot");
    let fresh = snapshot(&mut endpoint);
    assert_eq!(endpoint.invoke(step(&fresh)).unwrap(), IntentResult::Accepted);
    assert_eq!(runtime.step(1), 1);
    assert_eq!((runtime.tick(), runtime.fault()), (tick + 5, None));
    let saved = runtime.save_game();
    assert_eq!(mesocosm_runtime::Runtime::replayed_game(saved).unwrap().state_hash, runtime.state_hash(),
        "the save holds both doors' rounds");
}

#[test]
fn an_answer_through_the_endpoint_releases_the_runtimes_hold() {
    let mut runtime = worlds::held();
    assert_eq!(runtime.step(1), 1);
    assert!(runtime.checkpoint().is_some(), "a birth question holds the runtime");
    assert_eq!(runtime.step(1), 0);
    let mut endpoint = MesocosmEndpoint::open(runtime.world(), ParticipantHandle(runtime.interim().participant)).unwrap();
    let held = snapshot(&mut endpoint);
    let answer = ids(&held).into_iter().find(|id| id.starts_with("answer:")).expect("the endpoint offers the answers");
    assert_eq!(endpoint.invoke(invoke(&held, &answer, ANSWER)).unwrap(), IntentResult::Accepted);
    assert_eq!(runtime.step(1), 1, "the runtime adopts the endpoint's answer and carries on");
    assert!(runtime.checkpoint().is_none());
    assert_eq!(runtime.fault(), None);
}
