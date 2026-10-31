// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world-graph projection's invariants (H9): scrubbing never changes a
//! line, a branch with no change replays as its main line, a change inside a
//! branch is its own and survives its save, and folds and filters only
//! narrow what is shown.

use super::*;
use crate::endpoint::graph::{CLOSE, OPEN};
use crate::endpoint::{BRANCH, DEATH, FILTER, GRAPH, SCRUB, STEP, SimEndpoint};
use crate::founding::{Purpose, found};
use crate::save::WorldSave;
use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};

fn bench() -> World {
    let genesis = cosmograph::Founding { seed: 7, population: 48, sites: 4, lineages: 2, ecology: true,
        ..Default::default() }.generate().unwrap();
    found(genesis, Execution::Grouped, Purpose::Bench, None).unwrap()
}

fn snapshot(endpoint: &mut SimEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections.iter().find(|p| p.request.session.0 == GRAPH)
        .unwrap().request.clone();
    endpoint.snapshot(request).unwrap()
}

fn ids(snapshot: &ProjectionSnapshot) -> Vec<String> {
    snapshot.scene.active_items_in_order().into_iter()
        .map(|(_, item)| snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone())
        .collect()
}

fn hidden(snapshot: &ProjectionSnapshot) -> usize {
    snapshot.scene.tables.folds.iter().flatten().map(|f| f.members.len() - 1).sum()
}

fn send(endpoint: &mut SimEndpoint, id: &str, intent: &str, payload: &[u8]) -> IntentResult {
    let snapshot = snapshot(endpoint);
    let target = snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id)
        .map(|(instance, _)| instance).unwrap_or_else(|| panic!("{id} is not shown"));
    endpoint.invoke(IntentInvocation { session: snapshot.session.clone(), target,
        observed_epoch: snapshot.scene.epoch, observed_revision: snapshot.scene.revision,
        intent: intent.into(), payload: payload.to_vec() }).unwrap()
}

fn hash(endpoint: &SimEndpoint) -> u64 {
    endpoint.graph().timeline().world().read(|s| s.sim.state_hash()).unwrap()
}

#[test]
fn sites_fold_their_residents_until_opened() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let first = snapshot(&mut endpoint);
    let shown = ids(&first);
    assert!(shown.iter().any(|id| id.starts_with("region:")));
    assert!(first.scene.tables.relations.iter().flatten().any(|r| r.kind.as_deref() == Some("route")));
    let site = shown.iter().find(|id| id.starts_with("site:")
        && first.scene.tables.folds.iter().flatten().any(|f| f.label.as_deref()
            == Some(&format!("site {}'s residents", &id[5..])))).cloned().expect("a site holds residents");
    assert_eq!(send(&mut endpoint, &site, OPEN, &[]), IntentResult::Accepted);
    let opened = snapshot(&mut endpoint);
    assert!(hidden(&opened) < hidden(&first), "opening a site unfolds its residents");
    assert_eq!(ids(&opened), shown, "folding hides; it never adds or removes an item");
    assert_eq!(send(&mut endpoint, &site, CLOSE, &[]), IntentResult::Accepted);
    assert_eq!(hidden(&snapshot(&mut endpoint)), hidden(&first));
}

#[test]
fn a_filter_keeps_only_its_domains() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    assert_eq!(send(&mut endpoint, "world", FILTER, br#"{"domains":["geography"]}"#), IntentResult::Accepted);
    let shown = ids(&snapshot(&mut endpoint));
    assert!(shown.iter().any(|id| id.starts_with("site:")));
    assert!(!shown.iter().any(|id| id.starts_with("group:")), "ecology is filtered out");
    assert!(matches!(send(&mut endpoint, "world", FILTER, br#"{"domains":["lens"]}"#),
        IntentResult::Rejected { .. }));
}

#[test]
fn scrubbing_reads_the_past_without_changing_the_line() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let mut at = vec![hash(&endpoint)];
    for _ in 0..3 {
        assert_eq!(send(&mut endpoint, "world", STEP, br#"{"ticks":2}"#), IntentResult::Accepted);
        at.push(hash(&endpoint));
    }
    let (cut, revision) = (endpoint.world().cut().unwrap(), endpoint.world().revision().unwrap());
    assert_eq!(send(&mut endpoint, "world", SCRUB, br#"{"tick":2}"#), IntentResult::Accepted);
    let past = snapshot(&mut endpoint);
    let world = past.presentation.offers.get(&chirograph::PresentationKey("world".into())).unwrap();
    assert_eq!(world[0].semantics.label, "World");
    assert_eq!(endpoint.world().cut().unwrap(), cut, "scrubbing changes no line");
    assert_eq!(endpoint.world().revision().unwrap(), revision);
    // Tick 4 was never kept by a step of two, so it is replayed; both agree.
    for (tick, expected) in [(2, at[1]), (4, at[2]), (0, at[0])] {
        assert_eq!(send(&mut endpoint, "world", SCRUB, format!(r#"{{"tick":{tick}}}"#).as_bytes()),
            IntentResult::Accepted);
        let seen = endpoint.graph().timeline().world().read(|s| s.sim.state().tick).unwrap();
        assert_eq!(seen, 6, "the line's head stays at 6");
        let _ = snapshot(&mut endpoint);
        assert_eq!(viewed_hash(&mut endpoint), expected, "tick {tick} reads as it was played");
    }
    assert!(matches!(send(&mut endpoint, "world", SCRUB, br#"{"tick":7}"#), IntentResult::Rejected { .. }));
}

fn viewed_hash(endpoint: &mut SimEndpoint) -> u64 {
    let snapshot = snapshot(endpoint);
    let offer = &snapshot.presentation.offers[&chirograph::PresentationKey("world".into())][0];
    let bytes = graphshell_endpoint::PresentationSource::resource(endpoint, chirograph::ResourceRequest {
        session: snapshot.session.clone(), resource: offer.resource }).unwrap().bytes;
    let card: chirograph::PortableCardV1 = serde_json::from_slice(&bytes).unwrap();
    let hex = &card.values.iter().find(|v| v.label == "State hash").unwrap().value;
    u64::from_str_radix(hex, 16).unwrap()
}

#[test]
fn a_branch_is_its_own_only_once_changed_inside_it() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    assert_eq!(send(&mut endpoint, "world", STEP, br#"{"ticks":6}"#), IntentResult::Accepted);
    let main = hash(&endpoint);
    assert_eq!(send(&mut endpoint, "world", SCRUB, br#"{"tick":2}"#), IntentResult::Accepted);
    assert_eq!(send(&mut endpoint, "world", BRANCH, &[]), IntentResult::Accepted);
    assert_eq!(endpoint.graph().timeline().line(), 1);
    // Unchanged, the branch replays its main line exactly: the key seeds nothing.
    assert_eq!(send(&mut endpoint, "world", STEP, br#"{"ticks":4}"#), IntentResult::Accepted);
    assert_eq!(hash(&endpoint), main);
    let group = ids(&snapshot(&mut endpoint)).into_iter().find(|id| id.starts_with("group:")
        && endpoint.graph().timeline().world().read(|s| s.sim.state().population
            .get(id[6..].parse().unwrap()).is_some_and(|e| e.alive)).unwrap()).unwrap();
    assert_eq!(send(&mut endpoint, &group, DEATH, &[]), IntentResult::Accepted);
    assert_eq!(send(&mut endpoint, "world", STEP, br#"{"ticks":2}"#), IntentResult::Accepted);
    let branch = hash(&endpoint);
    assert_ne!(branch, endpoint.world().read(|s| s.sim.state_hash()).unwrap(), "the change is the branch's");
    assert_eq!(endpoint.world().read(|s| s.sim.state().tick).unwrap(), 6, "the main line did not move");
    let saved = endpoint.graph().timeline().world().save().unwrap();
    let resumed = World::resume(WorldSave::decode(&saved.encode().unwrap()).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(resumed.read(|s| s.sim.state_hash()).unwrap(), branch, "its replay is its own");
}

#[test]
fn the_main_line_takes_no_change_from_the_viewer() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let shown = ids(&snapshot(&mut endpoint));
    let group = shown.iter().find(|id| id.starts_with("group:")).unwrap();
    assert!(matches!(send(&mut endpoint, group, DEATH, &[]), IntentResult::Rejected { .. }),
        "a death is offered only inside a branch");
}
