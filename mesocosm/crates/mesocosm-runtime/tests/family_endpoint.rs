// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "endpoint")]

use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph::{
    Execution,
    directing::interim::{OnCollapse, Pace},
    directing::readings::Mode,
};
use cosmograph_host::{fixtures::fixture, world::World};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};
use mesocosm_runtime::{
    continuation::LoopSettings,
    endpoint::{CONFIGURE, MODE as GAME_SECTION, MesocosmEndpoint, STEP},
};

const OPAQUE: &str = "family-test-opaque";
const SENTINEL: &[u8] = &[0, 255, 17, 42];

fn family(world: World) -> (FamilyHost, FamilyReplica) {
    let (session, mut sections) = world.into_parts().unwrap();
    sections.insert(OPAQUE.into(), SENTINEL.to_vec());
    let mut host =
        FamilyHost::new(SessionId([31; 16]), World::with_sections(session, sections)).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    let replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    (host, replica)
}

fn snapshot(endpoint: &mut MesocosmEndpoint) -> ProjectionSnapshot {
    let request = endpoint.describe().projections[0].request.clone();
    endpoint.snapshot(request).unwrap()
}

fn invoke(
    endpoint: &mut MesocosmEndpoint,
    id: &str,
    intent: &str,
    payload: &[u8],
) -> IntentInvocation {
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

fn unchanged(
    host: &FamilyHost,
    endpoint: &mut MesocosmEndpoint,
    before: (u64, u64, cosmograph_host::save::WorldSave),
) {
    assert_eq!(host.sequence(), before.0);
    assert_eq!(host.revision().unwrap(), before.1);
    assert_eq!(host.outbox_bytes(), 0);
    assert!(host.drain_records().is_empty());
    assert_eq!(endpoint.world().save().unwrap(), before.2);
    assert_eq!(
        endpoint
            .world()
            .save()
            .unwrap()
            .sections
            .get(OPAQUE)
            .unwrap(),
        SENTINEL
    );
}

#[test]
fn played_turn_records_section_only_setup_and_native_step_and_refusal_is_inert() {
    let f = fixture();
    let (mut host, mut replica) = family(f.world);
    let mut endpoint = MesocosmEndpoint::open(host.world(), f.participants[0]).unwrap();
    assert!(
        !reproduce(&host, &mut replica).is_empty(),
        "opening a product window is recorded"
    );

    let before = (
        host.sequence(),
        host.revision().unwrap(),
        host.world().save().unwrap(),
    );
    let refused = invoke(&mut endpoint, "turn", CONFIGURE, b"{}");
    assert!(matches!(
        endpoint.invoke(refused).unwrap(),
        IntentResult::Rejected { .. }
    ));
    unchanged(&host, &mut endpoint, before);

    let settings = serde_json::to_vec(&LoopSettings {
        mode: Mode::Survival,
        pace: Pace {
            round: 1,
            scoring: 6,
        },
        on_collapse: OnCollapse::Stay,
    })
    .unwrap();
    let configure = invoke(&mut endpoint, "turn", CONFIGURE, &settings);
    assert_eq!(endpoint.invoke(configure).unwrap(), IntentResult::Accepted);
    let configured = reproduce(&host, &mut replica);
    assert_eq!(configured.len(), 1);
    assert!(
        configured[0].native_suffix.is_empty(),
        "configuration only replaces the game section"
    );
    assert!(matches!(
        configured[0].section_updates.get(GAME_SECTION),
        Some(Some(_))
    ));

    let snap = snapshot(&mut endpoint);
    let step = invoke(&mut endpoint, "turn", STEP, b"");
    assert_eq!(endpoint.invoke(step).unwrap(), IntentResult::Accepted);
    let stepped = reproduce(&host, &mut replica);
    assert_eq!(stepped.len(), 1);
    assert!(
        !stepped[0].native_suffix.is_empty(),
        "the played step advances native time"
    );
    assert!(matches!(
        stepped[0].section_updates.get(GAME_SECTION),
        Some(Some(_))
    ));
    assert_eq!(
        snap.scene.revision.0 + 1,
        endpoint.world().revision().unwrap()
    );
}
