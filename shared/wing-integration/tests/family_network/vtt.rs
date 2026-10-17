use chirograph::{
    EndpointDescriptor, IntentInvocation, IntentResult, ProjectionRequest, ProjectionSnapshot,
    ResourceRequest, ResourceResponse,
};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm_overlay::Seat;
use isocosm_vtt::{TABLE, VttEndpoint};

use super::harness::{ClientIntent, Harness};
use isonetry::family_network::FamilyCheckpointResource;

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

#[path = "../../../../crates/isocosm-vtt/tests/support/table.rs"]
mod table;

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
