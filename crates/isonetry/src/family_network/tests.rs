use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;

use chirograph::Carrier;
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use cosmograph::{Command, Execution, Founding, Session};
use cosmograph_host::world::World;
use muniment::MemoryBackend;
use murm_replication::MunimentStore;
use p2panda_core::SigningKey;
use personae::IdentityProvider;
use personae::delegation::Issue;
use transport::{P2pandaTransport, Transport, sync_overlay_topic};

use super::*;
use crate::family::{FamilyHost, FamilyReplica, HostAdmission};

pub(super) const SESSION: SessionId = SessionId([0x41; 16]);
pub(super) const TOPIC: [u8; 32] = [0x42; 32];

pub(super) fn host() -> FamilyHost {
    let genesis = Founding {
        seed: 17,
        sites: 2,
        population: 4,
        lineages: 2,
        ecology: false,
        ..Default::default()
    }
    .generate()
    .expect("small test genesis");
    let session = Session::new(genesis, Execution::Individuals).expect("test session");
    FamilyHost::new(
        SESSION,
        World::with_sections(
            session,
            BTreeMap::from([
                ("unknown-section".to_string(), vec![0, 0xff]),
                ("source-document".to_string(), vec![1, 2, 3]),
            ]),
        ),
    )
    .expect("family host")
}

#[derive(Debug)]
struct FamilyFixtureError(String);

impl std::fmt::Display for FamilyFixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

struct FamilyFixtureEndpoint {
    fixture: graphshell_native::canary::FixtureEndpoint,
    host: Rc<RefCell<FamilyHost>>,
    records: tokio::sync::mpsc::Sender<AcceptedRecord>,
}

impl FamilyFixtureEndpoint {
    fn new(
        host: Rc<RefCell<FamilyHost>>,
        records: tokio::sync::mpsc::Sender<AcceptedRecord>,
    ) -> Self {
        Self {
            fixture: graphshell_native::canary::FixtureEndpoint::new(),
            host,
            records,
        }
    }
}

impl ProjectionCatalog for FamilyFixtureEndpoint {
    fn describe(&self) -> chirograph::EndpointDescriptor {
        self.fixture.describe()
    }
}

impl ProjectionSource for FamilyFixtureEndpoint {
    type Error = FamilyFixtureError;

    fn snapshot(
        &mut self,
        request: chirograph::ProjectionRequest,
    ) -> Result<chirograph::ProjectionSnapshot, Self::Error> {
        self.fixture
            .snapshot(request)
            .map_err(|error| FamilyFixtureError(error.to_string()))
    }
}

impl PresentationSource for FamilyFixtureEndpoint {
    type Error = FamilyFixtureError;

    fn resource(
        &mut self,
        request: chirograph::ResourceRequest,
    ) -> Result<chirograph::ResourceResponse, Self::Error> {
        self.fixture
            .resource(request)
            .map_err(|error| FamilyFixtureError(error.to_string()))
    }
}

impl IntentSink for FamilyFixtureEndpoint {
    type Error = FamilyFixtureError;

    fn invoke(
        &mut self,
        invocation: chirograph::IntentInvocation,
    ) -> Result<chirograph::IntentResult, Self::Error> {
        let result = self
            .fixture
            .invoke(invocation)
            .map_err(|error| FamilyFixtureError(error.to_string()))?;
        if matches!(&result, chirograph::IntentResult::Accepted) {
            let permit = self.records.try_reserve().map_err(|error| {
                FamilyFixtureError(format!("accepted-record queue is full: {error:?}"))
            })?;
            let (_, _, record) = self
                .host
                .borrow_mut()
                .accept(|_, _| {
                    Ok(HostAdmission {
                        value: (),
                        commands: vec![Command::Join],
                        final_tick: None,
                        section_updates: BTreeMap::from([(
                            "unknown-section".into(),
                            Some(vec![9, 8]),
                        )]),
                    })
                })
                .map_err(FamilyFixtureError)?;
            let record = record.ok_or_else(|| {
                FamilyFixtureError("accepted intent produced no family record".into())
            })?;
            permit.send(record);
        }
        Ok(result)
    }
}

async fn transports() -> (P2pandaTransport, P2pandaTransport) {
    let alpn = graphshell_native::carrier::projection_alpn();
    let owner = P2pandaTransport::builder_from_seed([0x51; 32])
        .alpns(vec![alpn.clone()])
        .gossip()
        .bind()
        .await
        .expect("owner transport");
    let guest = P2pandaTransport::builder_from_seed([0x75; 32])
        .alpns(vec![alpn])
        .gossip()
        .bind()
        .await
        .expect("guest transport");
    let owner_id = owner.local_peer_id();
    let guest_id = guest.local_peer_id();
    let overlay = sync_overlay_topic(TOPIC);
    owner
        .add_peer(guest.endpoint_addr().await.expect("guest address"))
        .await
        .expect("owner adds guest");
    owner
        .set_topics(guest_id, &[overlay])
        .await
        .expect("owner subscribes guest");
    guest
        .add_peer(owner.endpoint_addr().await.expect("owner address"))
        .await
        .expect("guest adds owner");
    guest
        .set_topics(owner_id, &[overlay])
        .await
        .expect("guest subscribes owner");
    (owner, guest)
}

async fn wait_for_record(
    rx: &mut tokio::sync::mpsc::Receiver<AcceptedRecord>,
    sequence: u64,
) -> AcceptedRecord {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Some(record) = rx.recv().await {
                if record.sequence == sequence {
                    return record;
                }
            }
        }
    })
    .await
    .expect("host-authored family record arrives")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn admitted_graphshell_stream_and_host_signed_lane_converge_and_shutdown() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let (owner_transport, guest_transport) = transports().await;
            let host_key = SigningKey::from_bytes(&[0x51; 32]);
            let other_key = SigningKey::from_bytes(&[0x62; 32]);
            let host_author = host_key.verifying_key();

            let family_host = Rc::new(RefCell::new(host()));
            let checkpoint = family_host
                .borrow_mut()
                .checkpoint()
                .expect("initial checkpoint");
            let recovery_checkpoint = checkpoint.clone();
            let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals)
                .expect("replica joins initial checkpoint");

            let owner_store = MunimentStore::new(MemoryBackend::new());
            let guest_store = MunimentStore::new(MemoryBackend::new());
            let (_owner_received_tx, mut owner_received_rx) = tokio::sync::mpsc::channel(4);
            let (guest_received_tx, mut guest_received_rx) = tokio::sync::mpsc::channel(4);
            let owner_lane = FamilyRecordLane::join(
                &owner_transport,
                owner_store,
                SESSION,
                TOPIC,
                host_author,
                WireLimits::default(),
                _owner_received_tx,
            )
            .await
            .expect("owner joins accepted-record lane");
            let guest_lane = FamilyRecordLane::join(
                &guest_transport,
                guest_store,
                SESSION,
                TOPIC,
                host_author,
                WireLimits::default(),
                guest_received_tx,
            )
            .await
            .expect("guest joins accepted-record lane");

            let viewer = personae::InMemoryProvider::from_seed([0x75; 32]);
            let request = graphshell_native::admission::open_session(
                &viewer,
                notochord::NetworkId([0x72; 32]),
                notochord::ProfileRef {
                    id: "mere.base".into(),
                    revision: 1,
                },
                notochord::TrafficClass::Interactive,
                [0x73; 32],
                &graphshell_native::network_carrier::projection_binding(
                    guest_transport.local_peer_id(),
                ),
                vec![test_grant()],
            )
            .expect("signed Graphshell hello");
            let owner_peer = owner_transport.local_peer_id();
            let host_policy = graphshell_native::carrier::projection_policy(
                notochord::NetworkId([0x72; 32]),
                vec![notochord::TrustedRoot {
                    authority: [0x74; 32],
                    issuer: personae::InMemoryProvider::from_seed([0x71; 32])
                        .master_public_key()
                        .to_bytes(),
                }],
                vec![notochord::ProfileRef {
                    id: "mere.base".into(),
                    revision: 1,
                }],
                None,
            );
            let dial_policy = graphshell_native::carrier::projection_policy(
                notochord::NetworkId([0x72; 32]),
                vec![notochord::TrustedRoot {
                    authority: [0x74; 32],
                    issuer: personae::InMemoryProvider::from_seed([0x71; 32])
                        .master_public_key()
                        .to_bytes(),
                }],
                vec![notochord::ProfileRef {
                    id: "mere.base".into(),
                    revision: 1,
                }],
                None,
            );
            // Keep both endpoints alive outside their one-shot session tasks.
            // P2pandaTransport documents that dropping an open endpoint aborts
            // QUIC connections and makes the peer observe a lost connection.
            let host_side = std::sync::Arc::new(owner_transport);
            let guest_side = std::sync::Arc::new(guest_transport);
            let accepting_host_side = std::sync::Arc::clone(&host_side);
            let (client_shutdown_tx, client_shutdown_rx) = tokio::sync::oneshot::channel();
            let (accepted_record_tx, mut accepted_record_rx) = tokio::sync::mpsc::channel(4);
            let endpoint_host = Rc::clone(&family_host);
            let accepting = tokio::task::spawn_local(async move {
                let mut session = graphshell_native::carrier::accept_projection_session(
                    accepting_host_side.as_ref(),
                    &host_policy,
                    &notochord::RevocationLedger::default(),
                    50,
                    0,
                )
                .await
                .expect("host accepts the stream")
                .expect("viewer admitted");
                let authority =
                    graphshell_native::lifecycle::SessionAuthority::retain_admitted(&session);
                let mut endpoint = FamilyFixtureEndpoint::new(endpoint_host, accepted_record_tx);
                let mut resume = |_: &mut FamilyFixtureEndpoint, _| {
                    Err("resume not needed in this gate".to_string())
                };
                let summary = graphshell_native::session_loop::serve_admitted_session(
                    &mut session,
                    &authority,
                    &std::sync::RwLock::new(notochord::RevocationLedger::default()),
                    &mut endpoint,
                    &mut resume,
                    || 50,
                )
                .await
                .expect("Graphshell requests served");
                client_shutdown_rx
                    .await
                    .expect("client confirms stream shutdown before host drops its session");
                drop(session);
                summary
            });

            let runtime = tokio::runtime::Handle::current();
            let client = std::sync::Arc::clone(&guest_side);
            let driven = tokio::task::spawn_blocking(move || {
                let stream = runtime
                    .block_on(graphshell_native::network_carrier::dial_projection_session(
                        client.as_ref(),
                        owner_peer,
                        &request,
                        &dial_policy.limits,
                    ))
                    .expect("dial")
                    .expect("admission");
                let mut carrier = graphshell_native::network_carrier::NetworkCarrier::over(
                    stream,
                    graphshell_native::network_carrier::CarrierRuntime::borrowed(runtime.clone()),
                );
                let opened = carrier
                    .request(chirograph::CarrierRequestBody::Open(Box::new(
                        chirograph::SessionOpen {
                            version: chirograph::ProtocolVersion { major: 1, minor: 0 },
                            capabilities: chirograph::CapabilityProfile::default(),
                        },
                    )))
                    .expect("open admitted session");
                assert!(matches!(opened, chirograph::CarrierResponseBody::Opened(_)));
                let fixture = graphshell_native::canary::FixtureEndpoint::new();
                let projection = fixture.request();
                let snapshot = carrier
                    .request(chirograph::CarrierRequestBody::Snapshot(projection.clone()))
                    .expect("request fixture projection");
                let chirograph::CarrierResponseBody::Snapshot(snapshot) = snapshot else {
                    panic!("expected projection snapshot");
                };
                assert_eq!(snapshot.session, projection.session);
                let accepted = carrier
                    .request(chirograph::CarrierRequestBody::Intent(
                        chirograph::IntentInvocation {
                            session: projection.session,
                            target: sceno::InstanceId(0),
                            observed_epoch: snapshot.scene.epoch,
                            observed_revision: snapshot.scene.revision,
                            intent: "fixture.open-note".into(),
                            payload: Vec::new(),
                        },
                    ))
                    .expect("invoke admitted Graphshell intent");
                assert!(matches!(
                    accepted,
                    chirograph::CarrierResponseBody::Intent(chirograph::IntentResult::Accepted)
                ));
                carrier
                    .request(chirograph::CarrierRequestBody::Close)
                    .expect("close admitted session");
                carrier.shutdown().expect("client carrier shutdown");
                client_shutdown_tx
                    .send(())
                    .expect("host session remains alive until client shutdown");
            });
            driven.await.expect("client driver");
            accepting.await.expect("host session completes");

            let record = tokio::time::timeout(Duration::from_secs(5), accepted_record_rx.recv())
                .await
                .expect("accepted Graphshell intent queues its family record")
                .expect("family record is queued");
            assert_eq!(record.sequence, 1, "the intent produced the first record");
            owner_lane
                .publish(&record, &host_key, WireLimits::default())
                .await
                .expect("signed host record published");
            replica
                .apply(wait_for_record(&mut guest_received_rx, 1).await)
                .expect("native and section record applies");
            assert_eq!(replica.sequence(), 1);
            assert_eq!(
                replica
                    .read(|session, sections| (session.entries.len(), sections.clone()))
                    .unwrap(),
                family_host
                    .borrow()
                    .read(|session, sections| (session.entries.len(), sections.clone()))
                    .unwrap(),
            );

            let (_, _, section_only) = family_host
                .borrow_mut()
                .accept(|_, _| {
                    Ok(HostAdmission {
                        value: (),
                        commands: Vec::new(),
                        final_tick: None,
                        section_updates: BTreeMap::from([(
                            "unknown-section".into(),
                            Some(vec![7, 6, 5]),
                        )]),
                    })
                })
                .expect("section-only host acceptance");
            let section_only = section_only.expect("opaque change is recorded");
            assert!(section_only.native_suffix.is_empty());
            owner_lane
                .publish(&section_only, &host_key, WireLimits::default())
                .await
                .expect("section-only host record published");
            replica
                .apply(wait_for_record(&mut guest_received_rx, 2).await)
                .expect("section-only record applies");

            let forged = sign_record(&record, &other_key, 0, None, WireLimits::default())
                .expect("other author can sign a valid envelope");
            assert_eq!(
                verify_host_record(&forged, &host_author, SESSION, WireLimits::default())
                    .unwrap_err(),
                "accepted record was not authored by the pinned host"
            );
            guest_lane
                .store
                .insert_indexed_operation(&p2panda_core::Topic::from(TOPIC), &forged, &SESSION.0)
                .await
                .expect("store forged incoming test operation");
            guest_lane
                .joined
                .publish(forged)
                .expect("publish forged test operation");
            tokio::time::timeout(Duration::from_secs(10), async {
                while owner_lane.rejected_records() == 0 {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await
            .expect("owner lane processed and rejected the wrong-author operation");
            assert_eq!(replica.sequence(), 2, "wrong author never reaches replica");
            assert!(
                owner_received_rx.try_recv().is_err(),
                "wrong author is not queued"
            );
            assert_eq!(family_host.borrow().sequence(), 2);
            assert_eq!(
                replica
                    .read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                family_host
                    .borrow()
                    .read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                "wrong-author traffic leaves the complete hosted and replica state equal"
            );
            assert_eq!(
                replica.snapshot().unwrap().revision,
                family_host.borrow().revision().unwrap()
            );

            // The replication callback persists before bounded local delivery.
            // A restarted owner can page the verified stored suffix rather
            // than silently losing records that were already persisted.
            let first_page = read_stored_suffix(
                &guest_lane.store,
                SESSION,
                &host_author,
                None,
                1,
                WireLimits::default().max_bytes,
                WireLimits::default(),
            )
            .await
            .expect("first stored recovery page");
            assert_eq!(first_page.records.len(), 1);
            assert_eq!(first_page.records[0].sequence, 1);
            assert_eq!(first_page.through_sequence, Some(0));
            assert!(first_page.has_more);
            let second_page = read_stored_suffix(
                &guest_lane.store,
                SESSION,
                &host_author,
                first_page.through_sequence,
                1,
                WireLimits::default().max_bytes,
                WireLimits::default(),
            )
            .await
            .expect("second stored recovery page");
            assert_eq!(second_page.records.len(), 1);
            assert_eq!(second_page.records[0].sequence, 2);
            assert_eq!(second_page.through_sequence, Some(1));
            assert!(!second_page.has_more);
            let mut recovered =
                FamilyReplica::install(recovery_checkpoint, &[], Execution::Individuals)
                    .expect("recovery replica installs original checkpoint");
            for record in first_page.records.into_iter().chain(second_page.records) {
                recovered.apply(record).expect("stored record applies");
            }
            assert_eq!(
                recovered
                    .read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                family_host
                    .borrow()
                    .read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                "verified stored suffix recovers the same native and section state"
            );

            owner_lane
                .leave_and_wait()
                .await
                .expect("owner lane shutdown");
            guest_lane
                .leave_and_wait()
                .await
                .expect("guest lane shutdown");
            host_side.close().await.expect("host transport shutdown");
            guest_side.close().await.expect("guest transport shutdown");
            drop(owner_received_rx);
        })
        .await;
}

fn test_grant() -> insigne::delegation::SignedDelegationCertificate {
    let owner = personae::InMemoryProvider::from_seed([0x71; 32]);
    insigne::delegation::SignedDelegationCertificate::issue(
        &owner,
        insigne::delegation::DelegationCertificate::new(
            insigne::delegation::DelegationParent::Root([0x74; 32]),
            owner.master_public_key().to_bytes(),
            personae::InMemoryProvider::from_seed([0x75; 32])
                .master_public_key()
                .to_bytes(),
            insigne::delegation::CapabilityScope {
                domain: graphshell_native::admission::GRAPHSHELL_DOMAIN.into(),
                resource: notochord::NetworkId([0x72; 32]).0.to_vec(),
                path_prefix: graphshell_native::admission::PROJECTION_SERVICE.into(),
                actions: [graphshell_native::admission::CONNECT_ACTION.to_string()]
                    .into_iter()
                    .collect(),
            },
            5,
            10,
            Some(3_600_050),
            1,
            [0x76; 32],
        ),
    )
    .expect("issue test delegation")
}

#[path = "validation.rs"]
mod validation;

#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
