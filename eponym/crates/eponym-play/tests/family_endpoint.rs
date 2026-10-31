// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "endpoint")]

use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use eponym_play::endpoint::{EponymEndpoint, STEP};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph::Execution;
use cosmograph_host::{fixtures::fixture, world::World};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};

const OPAQUE: &str = "family-test-opaque";
const SENTINEL: &[u8] = &[0, 255, 17, 42];

fn family(world: World) -> (FamilyHost, FamilyReplica) {
    let (session, mut sections) = world.into_parts().unwrap();
    sections.insert(OPAQUE.into(), SENTINEL.to_vec());
    let mut host =
        FamilyHost::new(SessionId([32; 16]), World::with_sections(session, sections)).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    let replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    (host, replica)
}

fn snapshot(endpoint: &mut EponymEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

fn invoke(endpoint: &mut EponymEndpoint, intent: &str) -> IntentInvocation {
    let snap = snapshot(endpoint);
    let target = snap
        .scene
        .active_items_in_order()
        .into_iter()
        .find(|(_, item)| {
            snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .unwrap()
                .id
                == "eponym"
        })
        .unwrap()
        .0;
    IntentInvocation {
        session: snap.session,
        target,
        observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision,
        intent: intent.into(),
        payload: Vec::new(),
    }
}

fn reproduce(
    host: &FamilyHost,
    replica: &mut FamilyReplica,
) -> Vec<isonetry::family::AcceptedRecord> {
    let records = host.drain_records();
    for record in records.iter().cloned() {
        replica.apply(record).unwrap();
    }
    let hosted = host.world().save().unwrap();
    let copied = replica.snapshot().unwrap();
    assert_eq!(copied.save, hosted);
    assert_eq!(copied.revision, host.revision().unwrap());
    assert_eq!(hosted.sections.get(OPAQUE).unwrap(), SENTINEL);
    assert_eq!(host.outbox_bytes(), 0);
    records
}

#[test]
fn next_life_endpoint_refusal_is_inert_and_step_replays_time_only_record() {
    let f = fixture();
    let (mut host, mut replica) = family(f.world);
    let mut endpoint = EponymEndpoint::open(host.world(), f.participants[0]).unwrap();
    assert!(!reproduce(&host, &mut replica).is_empty());

    let before = (
        host.sequence(),
        host.revision().unwrap(),
        host.world().save().unwrap(),
    );
    let refused = invoke(&mut endpoint, "eponym.not-offered");
    assert!(matches!(
        endpoint.invoke(refused).unwrap(),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(host.sequence(), before.0);
    assert_eq!(host.revision().unwrap(), before.1);
    assert_eq!(host.world().save().unwrap(), before.2);
    assert_eq!(host.outbox_bytes(), 0);
    assert!(host.drain_records().is_empty());

    let before_tick = endpoint.world().cut().unwrap().tick;
    let step = invoke(&mut endpoint, STEP);
    assert_eq!(
        endpoint.invoke(step.clone()).unwrap(),
        IntentResult::Accepted
    );
    let records = reproduce(&host, &mut replica);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].final_tick, before_tick + 1);
    assert!(
        records[0].native_suffix.is_empty(),
        "a time-only advance adds no native command entry"
    );
    assert_eq!(
        records[0].section_updates,
        std::collections::BTreeMap::from([("eponym".into(), Some(Vec::new()))]),
        "the existing endpoint persists its absent mode as empty bytes"
    );

    let after_step = (
        host.sequence(),
        host.revision().unwrap(),
        host.world().save().unwrap(),
    );
    assert!(matches!(
        endpoint.invoke(step).unwrap(),
        IntentResult::Stale { .. }
    ));
    assert_eq!(host.sequence(), after_step.0);
    assert_eq!(host.revision().unwrap(), after_step.1);
    assert_eq!(host.world().save().unwrap(), after_step.2);
    assert_eq!(host.outbox_bytes(), 0);
    assert!(host.drain_records().is_empty());
}
