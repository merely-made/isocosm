// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph::Execution;
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use isocosm_native::{AUTHORING, TABLE, VttEndpoint};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};

#[path = "support/table.rs"]
mod table;

const OPAQUE: &str = "family-test-opaque";
const SENTINEL: &[u8] = &[0, 255, 17, 42];

fn family(world: World) -> (FamilyHost, FamilyReplica) {
    let (session, mut sections) = world.into_parts().unwrap();
    sections.insert(OPAQUE.into(), SENTINEL.to_vec());
    let mut host =
        FamilyHost::new(SessionId([33; 16]), World::with_sections(session, sections)).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    let replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    (host, replica)
}

fn snapshot_for(endpoint: &mut VttEndpoint, session: &str) -> ProjectionSnapshot {
    let request = endpoint
        .describe()
        .projections
        .into_iter()
        .find(|offer| offer.request.session.0 == session)
        .unwrap()
        .request;
    endpoint.snapshot(request).unwrap()
}

fn snapshot(endpoint: &mut VttEndpoint) -> ProjectionSnapshot {
    snapshot_for(endpoint, TABLE)
}

fn invoke_in(
    endpoint: &mut VttEndpoint,
    session: &str,
    id: &str,
    intent: &str,
    payload: &[u8],
) -> IntentInvocation {
    let snap = snapshot_for(endpoint, session);
    let target = snap
        .scene
        .active_items_in_order()
        .into_iter()
        .find(|(_, item)| {
            snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .unwrap()
                .id
                == id
        })
        .unwrap()
        .0;
    IntentInvocation {
        session: snap.session,
        target,
        observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision,
        intent: intent.into(),
        payload: payload.to_vec(),
    }
}

fn invoke(endpoint: &mut VttEndpoint, id: &str, intent: &str) -> IntentInvocation {
    invoke_in(endpoint, TABLE, id, intent, b"")
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
fn vtt_local_selection_is_record_free_then_native_party_join_replays() {
    let table::Table {
        world, dm, bodies, ..
    } = table::table();
    let (mut host, mut replica) = family(world);
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    assert!(
        !reproduce(&host, &mut replica).is_empty(),
        "opening a table window is recorded"
    );

    let before_refusal = (
        host.sequence(),
        host.revision().unwrap(),
        host.world().save().unwrap(),
    );
    let malformed = invoke_in(&mut endpoint, AUTHORING, "authoring", "vtt.apply", b"{}");
    assert!(matches!(
        endpoint.invoke(malformed).unwrap(),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(host.sequence(), before_refusal.0);
    assert_eq!(host.revision().unwrap(), before_refusal.1);
    assert_eq!(host.world().save().unwrap(), before_refusal.2);
    assert_eq!(host.outbox_bytes(), 0);
    assert!(host.drain_records().is_empty());

    let before = (
        host.sequence(),
        host.revision().unwrap(),
        host.world().save().unwrap(),
    );
    let select = invoke(&mut endpoint, "party:party:band", "vtt.read-party");
    assert!(matches!(
        endpoint.invoke(select).unwrap(),
        IntentResult::Accepted
    ));
    assert_eq!(
        host.sequence(),
        before.0,
        "endpoint-local selection writes no world event"
    );
    assert_eq!(host.revision().unwrap(), before.1);
    assert_eq!(host.world().save().unwrap(), before.2);
    assert_eq!(host.outbox_bytes(), 0);
    assert!(host.drain_records().is_empty());

    let body = format!("body:{}", bodies[0]);
    let join = invoke(&mut endpoint, &body, "vtt.join");
    assert_eq!(endpoint.invoke(join).unwrap(), IntentResult::Accepted);
    let records = reproduce(&host, &mut replica);
    assert_eq!(records.len(), 1);
    assert!(
        !records[0].native_suffix.is_empty(),
        "join submits a native party event"
    );
    assert!(
        matches!(records[0].section_updates.get("vtt"), Some(Some(_))),
        "the VTT game section is committed with it"
    );
}
