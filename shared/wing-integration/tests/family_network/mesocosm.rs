use graphshell_endpoint::ProjectionCatalog;
use cosmograph::{
    directing::interim::{OnCollapse, Pace},
    directing::readings::Mode,
};
use cosmograph_host::fixtures::fixture;
use mesocosm_runtime::{
    continuation::LoopSettings,
    endpoint::{CONFIGURE, MODE, MesocosmEndpoint, SESSION},
};

use super::harness::{ClientIntent, Harness};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mesocosm_configure_crosses_admission_and_publishes_live_section_bytes() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let f = fixture();
            let participant = f.participants[0];
            let mut family = Harness::new(f.world, 21).await;
            let endpoint = MesocosmEndpoint::open(family.host.borrow().world(), participant)
                .expect("actual Mesocosm endpoint opens on family host world");
            let projection = endpoint.describe().projections[0].request.clone();
            assert_eq!(projection.session.0, SESSION);

            let opened = family.sync_pending().await;
            assert!(
                !opened.is_empty(),
                "the actual endpoint window opens on the hosted world"
            );
            family
                .serve(
                    endpoint,
                    vec![ClientIntent {
                        projection: projection.clone(),
                        target: "turn".into(),
                        intent: CONFIGURE.into(),
                        payload: serde_json::to_vec(&LoopSettings {
                            mode: Mode::Survival,
                            pace: Pace {
                                round: 1,
                                scoring: 6,
                            },
                            on_collapse: OnCollapse::Stay,
                        })
                        .unwrap(),
                    }],
                )
                .await;
            let accepted = family.sync_pending().await;
            assert_eq!(accepted.len(), 1);
            assert!(
                accepted[0].native_suffix.is_empty(),
                "configure is a game-section update"
            );
            assert!(matches!(
                accepted[0].section_updates.get(MODE),
                Some(Some(_))
            ));
            family.assert_converged();
            family.shutdown().await;
        })
        .await;
}
