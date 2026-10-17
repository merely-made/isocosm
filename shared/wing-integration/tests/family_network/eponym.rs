use eponym_play::endpoint::{EponymEndpoint, SESSION, STEP};
use graphshell_endpoint::ProjectionCatalog;
use isocosm_host::fixtures::fixture;

use super::harness::{ClientIntent, Harness};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn eponym_step_crosses_admission_as_a_time_only_record() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let f = fixture();
            let participant = f.participants[0];
            let mut family = Harness::new(f.world, 41).await;
            let endpoint = EponymEndpoint::open(family.host.borrow().world(), participant)
                .expect("actual Eponym endpoint opens on family host world");
            let projection = endpoint.describe().projections[0].request.clone();
            assert_eq!(projection.session.0, SESSION);
            let before_tick = family.host.borrow().world().cut().unwrap().tick;

            assert!(
                !family.sync_pending().await.is_empty(),
                "the actual endpoint window is journaled"
            );
            family
                .serve(
                    endpoint,
                    vec![ClientIntent {
                        projection: projection.clone(),
                        target: "eponym".into(),
                        intent: STEP.into(),
                        payload: Vec::new(),
                    }],
                )
                .await;
            let accepted = family.sync_pending().await;
            assert_eq!(accepted.len(), 1);
            assert_eq!(accepted[0].final_tick, before_tick + 1);
            assert!(
                accepted[0].native_suffix.is_empty(),
                "the endpoint advances time without a command entry"
            );
            assert_eq!(
                accepted[0].section_updates,
                std::collections::BTreeMap::from([("eponym".into(), Some(Vec::new()))]),
                "the existing endpoint persists its absent mode as empty bytes"
            );
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}
