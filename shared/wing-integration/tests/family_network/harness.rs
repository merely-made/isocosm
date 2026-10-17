use std::fmt::Display;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use chirograph::{
    Carrier, CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult,
    ProjectionRequest, ProjectionSnapshot, ProtocolVersion, ResourceResponse, SessionOpen,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm::Execution;
use isocosm_host::world::World;
use isonetry::family::{AcceptedRecord, FamilyHost, FamilyReplica, SessionId, WireLimits};
use isonetry::family_network::{
    FamilyCheckpointDescriptor, FamilyCheckpointResource, FamilyRecordLane,
    verify_checkpoint_resource,
};
use muniment::MemoryBackend;
use murm_replication::MunimentStore;
use p2panda_core::SigningKey;
use personae::{IdentityProvider, InMemoryProvider};
use std::cell::RefCell;
use transport::{P2pandaTransport, Transport, sync_overlay_topic};

use isonetry::family_network::stream::{
    admission, carrier as host_carrier, lifecycle, network_carrier, session_loop,
};

pub const OPAQUE: &str = "family-network-opaque";
pub const SENTINEL: &[u8] = &[0, 255, 17, 42, 88];
#[derive(Clone)]
pub struct ClientIntent {
    pub projection: ProjectionRequest,
    pub target: String,
    pub intent: String,
    pub payload: Vec<u8>,
}

pub struct Harness {
    pub host: Rc<RefCell<FamilyHost>>,
    pub replica: FamilyReplica,
    fresh_replica: Option<FamilyReplica>,
    owner_lane: FamilyRecordLane<MemoryBackend>,
    guest_lane: FamilyRecordLane<MemoryBackend>,
    owner_rx: tokio::sync::mpsc::Receiver<AcceptedRecord>,
    guest_rx: tokio::sync::mpsc::Receiver<AcceptedRecord>,
    owner: Arc<P2pandaTransport>,
    guest: Arc<P2pandaTransport>,
    host_key: SigningKey,
    seed: u8,
}

impl Harness {
    pub async fn new(world: World, seed: u8) -> Self {
        let (session, mut sections) = world.into_parts().expect("fixture world is unshared");
        sections.insert(OPAQUE.into(), SENTINEL.to_vec());
        let session_id = SessionId([seed; 16]);
        let topic = [seed.wrapping_add(1); 32];
        let host = Rc::new(RefCell::new(
            FamilyHost::new(session_id, World::with_sections(session, sections))
                .expect("family host"),
        ));
        let checkpoint = host.borrow_mut().checkpoint().expect("family checkpoint");
        let replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals)
            .expect("replica installs same-world checkpoint");

        let alpn = host_carrier::projection_alpn();
        let owner = Arc::new(
            P2pandaTransport::builder_from_seed([seed.wrapping_add(2); 32])
                .alpns(vec![alpn.clone()])
                .gossip()
                .bind()
                .await
                .expect("family host transport"),
        );
        let guest = Arc::new(
            P2pandaTransport::builder_from_seed([seed.wrapping_add(3); 32])
                .alpns(vec![alpn])
                .gossip()
                .bind()
                .await
                .expect("family guest transport"),
        );
        let overlay = sync_overlay_topic(topic);
        let owner_id = owner.local_peer_id();
        let guest_id = guest.local_peer_id();
        owner
            .add_peer(guest.endpoint_addr().await.expect("guest address"))
            .await
            .unwrap();
        owner.set_topics(guest_id, &[overlay]).await.unwrap();
        guest
            .add_peer(owner.endpoint_addr().await.expect("owner address"))
            .await
            .unwrap();
        guest.set_topics(owner_id, &[overlay]).await.unwrap();

        let host_key = SigningKey::from_bytes(&[seed.wrapping_add(2); 32]);
        let (owner_tx, owner_rx) = tokio::sync::mpsc::channel(8);
        let (guest_tx, guest_rx) = tokio::sync::mpsc::channel(8);
        let owner_lane = FamilyRecordLane::join(
            owner.as_ref(),
            MunimentStore::new(MemoryBackend::new()),
            session_id,
            topic,
            host_key.verifying_key(),
            WireLimits::default(),
            owner_tx,
        )
        .await
        .expect("host accepted-record lane");
        let guest_lane = FamilyRecordLane::join(
            guest.as_ref(),
            MunimentStore::new(MemoryBackend::new()),
            session_id,
            topic,
            host_key.verifying_key(),
            WireLimits::default(),
            guest_tx,
        )
        .await
        .expect("guest accepted-record lane");

        Self {
            host,
            replica,
            fresh_replica: None,
            owner_lane,
            guest_lane,
            owner_rx,
            guest_rx,
            owner,
            guest,
            host_key,
            seed,
        }
    }

    pub async fn sync_pending(&mut self) -> Vec<AcceptedRecord> {
        let records = self.host.borrow().drain_records();
        for record in &records {
            self.owner_lane
                .publish(record, &self.host_key, WireLimits::default())
                .await
                .expect("host signs and publishes the accepted world record");
            let received = wait_for_record(&mut self.guest_rx, record.sequence).await;
            assert_eq!(
                &received, record,
                "transported signed record decodes to exact world record"
            );
            self.replica
                .apply(received.clone())
                .expect("replica applies host accepted record");
            if let Some(fresh) = &mut self.fresh_replica {
                fresh
                    .apply(received)
                    .expect("fresh peer applies signed accepted suffix");
            }
        }
        self.assert_converged();
        records
    }

    pub fn assert_converged(&self) {
        let hosted = self.host.borrow().world().save().expect("host full save");
        let replicated = self.replica.snapshot().expect("replica full snapshot");
        assert_eq!(
            replicated.save, hosted,
            "native save and every opaque section converge"
        );
        assert_eq!(replicated.revision, self.host.borrow().revision().unwrap());
        assert_eq!(self.replica.sequence(), self.host.borrow().sequence());
        if let Some(fresh) = &self.fresh_replica {
            assert_eq!(fresh.snapshot().unwrap().save, hosted);
            assert_eq!(fresh.sequence(), self.host.borrow().sequence());
        }
        assert_eq!(hosted.sections.get(OPAQUE).unwrap().as_slice(), SENTINEL);
        assert_eq!(self.host.borrow().outbox_bytes(), 0);
    }

    pub fn has_bootstrapped_peer(&self) -> bool {
        self.fresh_replica.is_some()
    }

    pub fn bootstrap_resource(
        &mut self,
        projection_session: chirograph::ProjectionSession,
    ) -> FamilyCheckpointResource {
        let checkpoint = self
            .host
            .borrow_mut()
            .checkpoint()
            .expect("host checkpoint");
        FamilyCheckpointResource::new(
            &checkpoint,
            &self.host_key,
            projection_session,
            WireLimits::default(),
        )
        .expect("host signs bounded checkpoint")
    }

    pub async fn serve<E>(&mut self, endpoint: E, intents: Vec<ClientIntent>)
    where
        E: ProjectionCatalog + ProjectionSource + PresentationSource + IntentSink + 'static,
        <E as ProjectionSource>::Error: Display,
        <E as PresentationSource>::Error: Display,
        <E as IntentSink>::Error: Display,
    {
        self.serve_with_bootstrap(endpoint, None, intents).await;
    }

    pub async fn serve_with_bootstrap<E>(
        &mut self,
        mut endpoint: E,
        descriptor: Option<FamilyCheckpointDescriptor>,
        intents: Vec<ClientIntent>,
    ) where
        E: ProjectionCatalog + ProjectionSource + PresentationSource + IntentSink + 'static,
        <E as ProjectionSource>::Error: Display,
        <E as PresentationSource>::Error: Display,
        <E as IntentSink>::Error: Display,
    {
        let (request, host_policy, dial_policy) =
            admission_setup(self.seed, self.guest.local_peer_id());
        let host_side = Arc::clone(&self.owner);
        let accepting_host_side = Arc::clone(&host_side);
        let (client_closed_tx, client_closed_rx) = tokio::sync::oneshot::channel();
        let server = tokio::task::spawn_local(async move {
            let mut admitted = host_carrier::accept_projection_session(
                accepting_host_side.as_ref(),
                &host_policy,
                &notochord::RevocationLedger::default(),
                50,
                0,
            )
            .await
            .expect("admitted product session")
            .expect("viewer admitted");
            let authority = lifecycle::SessionAuthority::retain_admitted(&admitted);
            let mut resume = |_: &mut E, _| Err("resume not used by this test".into());
            let summary = session_loop::serve_admitted_session(
                &mut admitted,
                &authority,
                &std::sync::RwLock::new(notochord::RevocationLedger::default()),
                &mut endpoint,
                &mut resume,
                || 50,
            )
            .await
            .expect("actual product endpoint serves admitted stream");
            client_closed_rx
                .await
                .expect("client shuts down carrier before session is dropped");
            summary
        });

        let guest_side = Arc::clone(&self.guest);
        let runtime = tokio::runtime::Handle::current();
        let dial_peer = host_side.local_peer_id();
        let host_key = self.host_key.verifying_key();
        let expected_descriptor = descriptor.clone();
        let driven = tokio::task::spawn_blocking(move || {
            let stream = runtime
                .block_on(network_carrier::dial_projection_session(
                    guest_side.as_ref(),
                    dial_peer,
                    &request,
                    &dial_policy.limits,
                ))
                .expect("client dials admitted stream")
                .expect("host grants projection session");
            let mut carrier = network_carrier::NetworkCarrier::over(
                stream,
                network_carrier::CarrierRuntime::borrowed(runtime.clone()),
            );
            let bootstrap = run_client(&mut carrier, &intents, descriptor);
            carrier
                .shutdown()
                .expect("client carrier shuts down cleanly");
            client_closed_tx
                .send(())
                .expect("host awaits clean client carrier shutdown");
            bootstrap
        });
        let bootstrap = driven.await.expect("client driver task");
        server.await.expect("host session task");
        if let (Some(descriptor), Some(response)) = (expected_descriptor, bootstrap) {
            let checkpoint = verify_checkpoint_resource(
                &response,
                &descriptor,
                &host_key,
                WireLimits::default(),
            )
            .expect("fresh peer verifies checkpoint from admitted resource stream");
            self.fresh_replica = Some(
                FamilyReplica::install(checkpoint, &[], Execution::Individuals)
                    .expect("fresh peer installs authenticated checkpoint"),
            );
        }
    }

    pub async fn shutdown(self) {
        self.owner_lane
            .leave_and_wait()
            .await
            .expect("host lane shutdown");
        self.guest_lane
            .leave_and_wait()
            .await
            .expect("guest lane shutdown");
        self.owner.close().await.expect("host transport shutdown");
        self.guest.close().await.expect("guest transport shutdown");
        drop(self.owner_rx);
        drop(self.guest_rx);
    }
}

fn run_client(
    carrier: &mut impl Carrier,
    intents: &[ClientIntent],
    descriptor: Option<FamilyCheckpointDescriptor>,
) -> Option<ResourceResponse> {
    let opened = carrier
        .request(CarrierRequestBody::Open(Box::new(SessionOpen {
            version: ProtocolVersion { major: 1, minor: 0 },
            capabilities: chirograph::CapabilityProfile::default(),
        })))
        .expect("client opens admitted session");
    assert!(matches!(opened, CarrierResponseBody::Opened(_)));
    let bootstrap = descriptor.map(|descriptor| {
        let response = carrier
            .request(CarrierRequestBody::Resource(descriptor.request()))
            .expect("fetch join-invitation checkpoint by exact content address");
        let CarrierResponseBody::Resource(response) = response else {
            panic!("expected signed checkpoint resource");
        };
        response
    });
    for action in intents {
        let projection = &action.projection;
        let response = carrier
            .request(CarrierRequestBody::Snapshot(projection.clone()))
            .expect("product projection snapshot");
        let CarrierResponseBody::Snapshot(snapshot) = response else {
            panic!("expected actual endpoint snapshot");
        };
        let target = item_id(&snapshot, &action.target);
        let response = carrier
            .request(CarrierRequestBody::Intent(IntentInvocation {
                session: projection.session.clone(),
                target,
                observed_epoch: snapshot.scene.epoch,
                observed_revision: snapshot.scene.revision,
                intent: action.intent.clone(),
                payload: action.payload.clone(),
            }))
            .expect("actual product intent");
        assert!(
            matches!(
                response,
                CarrierResponseBody::Intent(IntentResult::Accepted)
            ),
            "actual product intent {} was refused",
            action.intent
        );
    }
    carrier
        .request(CarrierRequestBody::Close)
        .expect("client closes admitted session");
    bootstrap
}

fn item_id(snapshot: &ProjectionSnapshot, id: &str) -> sceno::InstanceId {
    snapshot
        .scene
        .active_items_in_order()
        .into_iter()
        .find(|(_, item)| {
            snapshot.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .is_some_and(|source| source.id == id)
        })
        .unwrap_or_else(|| panic!("projection does not offer item {id}"))
        .0
}

async fn wait_for_record(
    rx: &mut tokio::sync::mpsc::Receiver<AcceptedRecord>,
    sequence: u64,
) -> AcceptedRecord {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let record = rx.recv().await.expect("accepted-record lane remains open");
            if record.sequence == sequence {
                return record;
            }
        }
    })
    .await
    .expect("host-signed accepted record arrives")
}

fn admission_setup(
    seed: u8,
    guest_peer: transport::PeerID,
) -> (
    notochord::SessionHello,
    notochord::LocalNetworkPolicy,
    notochord::LocalNetworkPolicy,
) {
    let network = notochord::NetworkId([seed.wrapping_add(10); 32]);
    let root_authority = [seed.wrapping_add(11); 32];
    let owner = InMemoryProvider::from_seed([seed.wrapping_add(12); 32]);
    let viewer = InMemoryProvider::from_seed([seed.wrapping_add(3); 32]);
    let trusted = notochord::TrustedRoot {
        authority: root_authority,
        issuer: owner.master_public_key().to_bytes(),
    };
    let profile = notochord::ProfileRef {
        id: "mere.base".into(),
        revision: 1,
    };
    let policy = || {
        host_carrier::projection_policy(network, vec![trusted.clone()], vec![profile.clone()], None)
    };
    let request = admission::open_session(
        &viewer,
        network,
        profile.clone(),
        notochord::TrafficClass::Interactive,
        [seed.wrapping_add(13); 32],
        &network_carrier::projection_binding(guest_peer),
        vec![grant(seed, network, root_authority, &owner, &viewer)],
    )
    .expect("Graphshell viewer admission request");
    (request, policy(), policy())
}

fn grant(
    seed: u8,
    network: notochord::NetworkId,
    authority: [u8; 32],
    owner: &InMemoryProvider,
    viewer: &InMemoryProvider,
) -> insigne::delegation::SignedDelegationCertificate {
    use personae::delegation::Issue;
    insigne::delegation::SignedDelegationCertificate::issue(
        owner,
        insigne::delegation::DelegationCertificate::new(
            insigne::delegation::DelegationParent::Root(authority),
            owner.master_public_key().to_bytes(),
            viewer.master_public_key().to_bytes(),
            insigne::delegation::CapabilityScope {
                domain: admission::GRAPHSHELL_DOMAIN.into(),
                resource: network.0.to_vec(),
                path_prefix: admission::PROJECTION_SERVICE.into(),
                actions: [admission::CONNECT_ACTION.to_string()]
                    .into_iter()
                    .collect(),
            },
            5,
            10,
            Some(3_600_050),
            1,
            [seed.wrapping_add(14); 32],
        ),
    )
    .expect("issue test viewer delegation")
}
