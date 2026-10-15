// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::endpoint::{Curation, EXAMINE, INSPECT, SAVE, STEP, SimEndpoint};
use crate::founding::{Purpose, found};
use crate::save::WorldSave;
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, PortableCardV1,
    ProjectionSnapshot, ResourceRequest, Revision,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm::Founding;

fn bench() -> World {
    let genesis = Founding { seed: 7, population: 48, sites: 4, lineages: 2, ..Default::default() }
        .generate().unwrap();
    found(genesis, Execution::Grouped, Purpose::Bench, None).unwrap()
}

fn snapshot(endpoint: &mut SimEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

/// Each disclosed item's source id and its decoded card.
fn cards(endpoint: &mut SimEndpoint, snapshot: &ProjectionSnapshot) -> Vec<(String, PortableCardV1)> {
    snapshot.scene.active_items_in_order().into_iter().map(|(instance, item)| {
        let id = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id.clone();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(),
            resource: offer.resource }).unwrap().bytes;
        (id, serde_json::from_slice(&bytes).unwrap())
    }).collect()
}

fn target(snapshot: &ProjectionSnapshot, id: &str) -> sceno::InstanceId {
    snapshot.scene.active_items_in_order().into_iter().find(|(_, item)|
        snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == id)
        .map(|(instance, _)| instance).unwrap()
}

fn invoke(snapshot: &ProjectionSnapshot, id: &str, intent: &str, payload: &[u8]) -> IntentInvocation {
    IntentInvocation { session: snapshot.session.clone(), target: target(snapshot, id),
        observed_epoch: snapshot.scene.epoch, observed_revision: snapshot.scene.revision,
        intent: intent.into(), payload: payload.to_vec() }
}

#[test]
fn the_world_projection_discloses_a_world_card_and_one_card_per_site() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    assert_eq!(endpoint.describe().projections.len(), 1);
    let first = snapshot(&mut endpoint);
    let cards = cards(&mut endpoint, &first);
    let sites = endpoint.world().read(|s| s.sim.state().sites.len()).unwrap();
    assert_eq!(cards.len(), 1 + sites, "no group is disclosed before a site is examined");
    let (_, world) = cards.iter().find(|(id, _)| id == "world").unwrap();
    assert!(world.values.iter().any(|v| v.label == "Tick" && v.value == "0"));
    assert!(cards.iter().filter(|(id, _)| id.starts_with("site:")).all(|(_, c)|
        c.values.iter().any(|v| v.label == "Alive")));
    assert!(!first.scene.tables.relations.is_empty(), "routes link the sites");
}

#[test]
fn examining_a_site_discloses_its_groups_and_stales_the_old_revision() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let first = snapshot(&mut endpoint);
    let site = endpoint.world().read(|s| s.sim.state().population.groups.values()
        .find(|g| g.entity.alive).map(|g| g.entity.place).unwrap()).unwrap();
    let examine = invoke(&first, &format!("site:{site}"), EXAMINE, &[]);
    assert_eq!(endpoint.invoke(examine.clone()).unwrap(), IntentResult::Accepted);
    assert_eq!(endpoint.curation(), Curation { site: Some(site), group: None });
    assert!(matches!(endpoint.invoke(examine).unwrap(), IntentResult::Stale { current_revision: Revision(2), .. }));
    let second = snapshot(&mut endpoint);
    let groups: Vec<_> = cards(&mut endpoint, &second).into_iter().filter(|(id, _)| id.starts_with("group:")).collect();
    assert!(!groups.is_empty(), "the examined site's stored groups are disclosed");
    let (group, _) = &groups[0];
    assert_eq!(endpoint.invoke(invoke(&second, group, INSPECT, &[])).unwrap(), IntentResult::Accepted);
    let third = snapshot(&mut endpoint);
    let inspected = cards(&mut endpoint, &third).into_iter().find(|(id, _)| id == group).unwrap().1;
    assert!(inspected.badges.contains(&"inspected".to_string()));
}

#[test]
fn stepping_is_domain_truth_and_advances_the_world() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let first = snapshot(&mut endpoint);
    let before = endpoint.world().cut().unwrap();
    let step = invoke(&first, "world", STEP, br#"{"ticks":3}"#);
    assert_eq!(endpoint.invoke(step).unwrap(), IntentResult::Accepted);
    let after = endpoint.world().cut().unwrap();
    assert_eq!(after.tick, before.tick + 3);
    let second = snapshot(&mut endpoint);
    assert!(matches!(endpoint.invoke(invoke(&second, "world", STEP, br#"{"ticks":0}"#)).unwrap(),
        IntentResult::Rejected { .. }), "a step names a positive number of ticks");
}

#[test]
fn undisclosed_and_unknown_intents_are_refused() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let first = snapshot(&mut endpoint);
    let site = endpoint.world().read(|s| *s.sim.state().sites.keys().next().unwrap()).unwrap();
    // Step and save read no target kind, so only the advertisement keeps
    // them off a site's card.
    for (id, intent) in [(format!("site:{site}"), INSPECT), ("world".to_string(), EXAMINE),
        ("world".to_string(), "isocosm.found"), (format!("site:{site}"), STEP), (format!("site:{site}"), SAVE)] {
        assert!(matches!(endpoint.invoke(invoke(&first, &id, intent, &[])).unwrap(),
            IntentResult::Rejected { .. }), "{intent} on {id}");
    }
    assert_eq!(endpoint.curation(), Curation::default());
    assert_eq!(endpoint.world().cut().unwrap().tick, 0, "no refused intent moved the world");
}

#[test]
fn saving_needs_a_path_and_writes_a_resumable_world() {
    let mut endpoint = SimEndpoint::new(bench(), None);
    let first = snapshot(&mut endpoint);
    assert!(matches!(endpoint.invoke(invoke(&first, "world", SAVE, &[])).unwrap(), IntentResult::Rejected { .. }));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("world.save");
    let mut endpoint = SimEndpoint::new(bench(), Some(path.clone()));
    let first = snapshot(&mut endpoint);
    assert_eq!(endpoint.invoke(invoke(&first, "world", SAVE, &[])).unwrap(), IntentResult::Accepted);
    let resumed = World::resume(WorldSave::decode(&std::fs::read(&path).unwrap()).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(resumed.cut().unwrap(), endpoint.world().cut().unwrap());
}

#[test]
fn the_endpoint_survives_the_wire_encoding() {
    use graphshell_endpoint::local::LocalCarrier;
    use chirograph::Carrier;
    let mut carrier = LocalCarrier::new(SimEndpoint::new(bench(), None), |_, _| Err("resume is not used".to_string()));
    let CarrierResponseBody::Descriptor(descriptor) = carrier.request(CarrierRequestBody::Discover).unwrap()
        else { panic!("discovery answers a descriptor") };
    let request = descriptor.projections[0].request.clone();
    let CarrierResponseBody::Snapshot(first) = carrier.request(CarrierRequestBody::Snapshot(request)).unwrap()
        else { panic!("a snapshot request answers a snapshot") };
    let step = invoke(&first, "world", STEP, &[]);
    let CarrierResponseBody::Intent(result) = carrier.request(CarrierRequestBody::Intent(step)).unwrap()
        else { panic!("an intent answers a result") };
    assert_eq!(result, IntentResult::Accepted);
    assert_eq!(carrier.endpoint().world().cut().unwrap().tick, 1);
}

#[test]
fn a_change_through_one_endpoint_stales_another_over_the_same_world() {
    let world = bench();
    let mut a = SimEndpoint::new(world.share(), None);
    let mut b = SimEndpoint::new(world.share(), None);
    snapshot(&mut a);
    let seen = snapshot(&mut b);
    let first = snapshot(&mut a);
    assert_eq!(a.invoke(invoke(&first, "world", STEP, b"")).unwrap(), IntentResult::Accepted);
    let IntentResult::Stale { current_revision, .. } = b.invoke(invoke(&seen, "world", STEP, b"")).unwrap()
        else { panic!("a step through another endpoint stales this one") };
    assert!(current_revision.0 > seen.scene.revision.0);
    let fresh = snapshot(&mut b);
    assert_eq!(fresh.scene.revision, current_revision);
    assert_eq!(b.invoke(invoke(&fresh, "world", STEP, b"")).unwrap(), IntentResult::Accepted);
    // A change that rewrites only a mode section leaves the cut alone and still stales.
    let seen = snapshot(&mut a);
    let before = world.cut().unwrap();
    world.product_sections(before, |_, sections| {
        let mut sections = sections.clone();
        sections.insert("probe".into(), vec![1]);
        Ok(((), sections))
    }).unwrap();
    assert_eq!(world.cut().unwrap(), before);
    assert!(matches!(a.invoke(invoke(&seen, "world", STEP, b"")).unwrap(), IntentResult::Stale { .. }));
    assert!(snapshot(&mut a).scene.revision.0 > seen.scene.revision.0, "the next snapshot is a new revision");
}
