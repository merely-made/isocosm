use chirograph::{
    EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest, ProjectionSnapshot,
    ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use cosmograph_overlay::Seat;
use isocosm_native::{TABLE, VttEndpoint};

use super::harness::{ClientIntent, Harness};
use isonetry::{
    family::{FamilyHost, FamilyLedgerLimits, FamilyLimits, WireLimits},
    family_network::FamilyCheckpointResource,
};
use p2panda_core::SigningKey;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct BootstrapEndpoint {
    vtt: VttEndpoint,
    checkpoint: FamilyCheckpointResource,
}

impl ProjectionCatalog for BootstrapEndpoint {
    fn describe(&self) -> EndpointDescriptor {
        self.vtt.describe()
    }
}

impl ProjectionSource for BootstrapEndpoint {
    type Error = String;

    fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.vtt.snapshot(request)
    }
}

impl PresentationSource for BootstrapEndpoint {
    type Error = String;

    fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        if let Some(response) = self.checkpoint.respond(&request) {
            return Ok(response);
        }
        self.vtt.resource(request)
    }
}

impl IntentSink for BootstrapEndpoint {
    type Error = String;

    fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        self.vtt.invoke(intent)
    }
}

#[path = "../../../../crates/isocosm-native/tests/support/table.rs"]
mod table;

struct LedgerDirectory(PathBuf);

impl LedgerDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "isonetry-vtt-restart-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create unique host-ledger test directory");
        Self(path)
    }

    fn ledger(&self) -> PathBuf {
        self.0.join("host.ledger")
    }
}

impl Drop for LedgerDirectory {
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.0) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir(&self.0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn vtt_party_selection_and_join_cross_admission_and_publish_native_event() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let table::Table {
                world, dm, bodies, ..
            } = table::table();
            let mut family = Harness::new(world, 61).await;
            let endpoint = VttEndpoint::open(family.host.borrow().world(), dm, Seat::Dm)
                .expect("actual VTT endpoint opens on family host world");
            let projection = endpoint
                .describe()
                .projections
                .into_iter()
                .find(|offer| offer.request.session.0 == TABLE)
                .unwrap()
                .request;
            assert!(
                !family.sync_pending().await.is_empty(),
                "the actual table window is journaled"
            );

            let body = format!("body:{}", bodies[0]);
            let accepted = vec![
                ClientIntent {
                    projection: projection.clone(),
                    target: "party:party:band".into(),
                    intent: "vtt.read-party".into(),
                    payload: Vec::new(),
                },
                ClientIntent {
                    projection: projection.clone(),
                    target: body,
                    intent: "vtt.join".into(),
                    payload: Vec::new(),
                },
            ];
            family.serve(endpoint, accepted).await;
            let records = family.sync_pending().await;
            assert_eq!(
                records.len(),
                1,
                "local party selection is record-free; join commits once"
            );
            assert!(
                !records[0].native_suffix.is_empty(),
                "join submits a native party event"
            );
            assert!(matches!(
                records[0].section_updates.get("vtt"),
                Some(Some(_))
            ));
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fresh_peer_bootstraps_over_admitted_resource_then_applies_signed_suffix() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let table::Table {
                world, dm, bodies, ..
            } = table::table();
            let mut family = Harness::new(world, 67).await;
            let endpoint = VttEndpoint::open(family.host.borrow().world(), dm, Seat::Dm)
                .expect("actual VTT endpoint opens on family host world");
            let projection = endpoint
                .describe()
                .projections
                .into_iter()
                .find(|offer| offer.request.session.0 == TABLE)
                .unwrap()
                .request;
            family.sync_pending().await;

            let checkpoint = family.bootstrap_resource(projection.session.clone());
            let descriptor = checkpoint.descriptor().clone();
            let endpoint = BootstrapEndpoint {
                vtt: endpoint,
                checkpoint,
            };
            let body = format!("body:{}", bodies[0]);
            family
                .serve_with_bootstrap(endpoint, Some(descriptor), Vec::new())
                .await;
            assert!(family.has_bootstrapped_peer());

            let endpoint = VttEndpoint::open(family.host.borrow().world(), dm, Seat::Dm)
                .expect("reopen actual VTT endpoint after checkpoint install");
            family
                .serve(
                    endpoint,
                    vec![
                        ClientIntent {
                            projection: projection.clone(),
                            target: "party:party:band".into(),
                            intent: "vtt.read-party".into(),
                            payload: Vec::new(),
                        },
                        ClientIntent {
                            projection,
                            target: body,
                            intent: "vtt.join".into(),
                            payload: Vec::new(),
                        },
                    ],
                )
                .await;
            assert!(!family.sync_pending().await.is_empty());
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn vtt_host_restart_reconnects_with_signed_checkpoint_and_republished_ledger_suffix() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let directory = LedgerDirectory::new();
            let path = directory.ledger();
            let seed = 73;
            let key_bytes = [0x73; 32];
            let table::Table {
                world, dm, bodies, ..
            } = table::table();
            let mut family =
                Harness::new_file(world, seed, &path, SigningKey::from_bytes(&key_bytes)).await;

            let invitation_checkpoint = family
                .host
                .borrow_mut()
                .checkpoint()
                .expect("capture initial checkpoint before opening endpoint");
            let endpoint = VttEndpoint::open(family.host.borrow().world(), dm, Seat::Dm)
                .expect("actual VTT endpoint opens on durable family host");
            let projection = endpoint
                .describe()
                .projections
                .into_iter()
                .find(|offer| offer.request.session.0 == TABLE)
                .unwrap()
                .request;
            let invitation_checkpoint = FamilyCheckpointResource::new(
                &invitation_checkpoint,
                &SigningKey::from_bytes(&key_bytes),
                projection.session.clone(),
                WireLimits::default(),
            )
            .expect("sign initial host checkpoint for its invitation");
            let invitation_descriptor = invitation_checkpoint.descriptor().clone();
            let first_body = format!("body:{}", bodies[0]);
            family
                .serve(
                    endpoint,
                    vec![
                        ClientIntent {
                            projection: projection.clone(),
                            target: "party:party:band".into(),
                            intent: "vtt.read-party".into(),
                            payload: Vec::new(),
                        },
                        ClientIntent {
                            projection,
                            target: first_body,
                            intent: "vtt.join".into(),
                            payload: Vec::new(),
                        },
                    ],
                )
                .await;
            let original_records = family.sync_pending().await;
            assert_eq!(original_records.len(), 2);
            assert_eq!(original_records[0].sequence, 1);
            assert_eq!(original_records[1].sequence, 2);
            family.shutdown().await;

            // The product host and its transports are recreated from the
            // bounded family ledger. The caller retains the original host key.
            let recovered_host = FamilyHost::resume_file(
                &path,
                cosmograph::Execution::Individuals,
                FamilyLimits::default(),
                FamilyLedgerLimits::default(),
            )
            .expect("restart sole family host from persisted accepted records");
            let endpoint = VttEndpoint::open(recovered_host.world(), dm, Seat::Dm)
                .expect("reopen actual VTT endpoint before reconnect checkpoint");
            let mut family =
                Harness::from_host(recovered_host, seed, SigningKey::from_bytes(&key_bytes)).await;
            let projection = endpoint
                .describe()
                .projections
                .into_iter()
                .find(|offer| offer.request.session == invitation_descriptor.projection_session)
                .unwrap()
                .request;
            family
                .serve_with_bootstrap(
                    BootstrapEndpoint {
                        vtt: endpoint,
                        checkpoint: invitation_checkpoint,
                    },
                    Some(invitation_descriptor),
                    vec![ClientIntent {
                        projection: projection.clone(),
                        target: "party:party:band".into(),
                        intent: "vtt.read-party".into(),
                        payload: Vec::new(),
                    }],
                )
                .await;
            assert!(family.has_bootstrapped_peer());
            let republished = family.republish_persisted_suffix(0).await;
            assert!(republished.len() >= original_records.len());
            assert_eq!(
                &republished[..original_records.len()],
                original_records.as_slice()
            );
            let prior_sequence = family.host.borrow().sequence();
            let prior_digest = republished.last().unwrap().digest.clone();

            let endpoint = VttEndpoint::open(family.host.borrow().world(), dm, Seat::Dm)
                .expect("reopened endpoint accepts another actual VTT action");
            let next_projection = endpoint
                .describe()
                .projections
                .into_iter()
                .find(|offer| offer.request.session.0 == TABLE)
                .unwrap()
                .request;
            family
                .serve(
                    endpoint,
                    vec![
                        ClientIntent {
                            projection: next_projection.clone(),
                            target: "party:party:band".into(),
                            intent: "vtt.read-party".into(),
                            payload: Vec::new(),
                        },
                        ClientIntent {
                            projection: next_projection,
                            target: format!("member:{}", bodies[0]),
                            intent: "vtt.leave".into(),
                            payload: Vec::new(),
                        },
                    ],
                )
                .await;
            let continued = family.sync_pending().await;
            assert!(!continued.is_empty());
            assert!(
                continued
                    .iter()
                    .any(|record| !record.native_suffix.is_empty()),
                "the offered VTT leave action creates a native accepted record"
            );
            assert_eq!(continued[0].sequence, prior_sequence + 1);
            assert_eq!(continued[0].parent_digest, prior_digest);
            for pair in continued.windows(2) {
                assert_eq!(pair[1].sequence, pair[0].sequence + 1);
                assert_eq!(pair[1].parent_digest, pair[0].digest);
            }
            assert_eq!(
                family.host.borrow().sequence(),
                prior_sequence + continued.len() as u64
            );
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}
