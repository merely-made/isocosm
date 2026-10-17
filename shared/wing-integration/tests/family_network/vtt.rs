use graphshell_endpoint::ProjectionCatalog;
use isocosm_overlay::Seat;
use isocosm_vtt::{TABLE, VttEndpoint};

use super::harness::{ClientIntent, Harness};

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
