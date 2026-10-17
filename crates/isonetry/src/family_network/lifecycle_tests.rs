use super::{SESSION, TOPIC, host, transports};
use crate::family::{HostAdmission, WireLimits};
use crate::family_network::{FamilyHostNetwork, FamilyRecordOutbox, FamilyReplicaNetwork};
use isocosm::{Command, Execution};
use muniment::MemoryBackend;
use murm_replication::MunimentStore;
use p2panda_core::SigningKey;
use std::collections::BTreeMap;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn lifecycle_wrappers_checkpoint_publish_retry_apply_and_leave() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let (owner_transport, guest_transport) = transports().await;
            let key = SigningKey::from_bytes(&[0x51; 32]);
            let host_key = key.verifying_key();
            let limits = WireLimits::default();
            let mut host = host();
            let initial_checkpoint = host.checkpoint().expect("host checkpoint");
            let projection_session = chirograph::ProjectionSession("family-projection".into());

            let host_store = MunimentStore::new(MemoryBackend::new());
            let host_network =
                FamilyHostNetwork::join(&owner_transport, host_store, SESSION, TOPIC, key, limits)
                    .await
                    .expect("host lifecycle joins signed lane");
            let bootstrap = host_network
                .signed_checkpoint(initial_checkpoint.clone(), projection_session)
                .await
                .expect("host checkpoint matches empty signed lane");

            let guest_store = MunimentStore::new(MemoryBackend::new());
            let mut guest_network = FamilyReplicaNetwork::join(
                &guest_transport,
                guest_store,
                bootstrap.descriptor(),
                TOPIC,
                host_key,
                limits,
                8,
            )
            .await
            .expect("guest lifecycle subscribes before bootstrap install");
            let mut replica = FamilyReplicaNetwork::<MemoryBackend>::install_checkpoint(
                bootstrap.response(),
                bootstrap.descriptor(),
                &host_key,
                limits,
                Execution::Individuals,
            )
            .expect("guest verifies and installs signed checkpoint");
            assert_eq!(replica.sequence(), 0);

            let (outbox, mut batches) = FamilyRecordOutbox::channel(2, limits.max_bytes).unwrap();
            let (_, _, accepted) = host
                .accept(|_, _| {
                    Ok(HostAdmission {
                        value: (),
                        commands: vec![Command::Join],
                        final_tick: None,
                        section_updates: BTreeMap::from([(
                            "unknown-section".into(),
                            Some(vec![0xfa, 0xce]),
                        )]),
                    })
                })
                .expect("host accepts native and opaque section update");
            let accepted = accepted.expect("accepted state change is journaled");
            assert!(!accepted.native_suffix.is_empty());
            assert_eq!(
                accepted.section_updates.get("unknown-section"),
                Some(&Some(vec![0xfa, 0xce]))
            );

            assert_eq!(outbox.try_forward(&host).unwrap(), 1);
            let batch = tokio::time::timeout(Duration::from_secs(2), batches.recv())
                .await
                .expect("outbox delivers accepted batch")
                .expect("batch channel remains open");
            assert_eq!(host_network.publish_batch(&batch).await.unwrap(), 1);
            assert_eq!(
                host_network.publish_batch(&batch).await.unwrap(),
                1,
                "retry of the exact persisted record is idempotent"
            );

            let stale = host_network
                .signed_checkpoint(
                    initial_checkpoint,
                    bootstrap.descriptor().projection_session.clone(),
                )
                .await
                .expect_err("old World cut cannot be paired with a later signed log");
            assert!(stale.contains("empty host checkpoint"));

            tokio::time::timeout(Duration::from_secs(10), async {
                while replica.sequence() < 1 {
                    let applied = guest_network
                        .apply_ready(&mut replica)
                        .expect("contiguous signed record applies");
                    if applied == 0 {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                }
            })
            .await
            .expect("guest applies the accepted family record");
            assert_eq!(replica.sequence(), 1);
            assert_eq!(
                replica
                    .read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                host.read(|session, sections| (session.save(), sections.clone()))
                    .unwrap(),
                "native state and opaque section converge through lifecycle wrappers"
            );

            host_network
                .leave_and_wait()
                .await
                .expect("host lane leaves cleanly");
            guest_network
                .leave_and_wait()
                .await
                .expect("guest lane leaves cleanly");
            owner_transport
                .close()
                .await
                .expect("host transport closes");
            guest_transport
                .close()
                .await
                .expect("guest transport closes");
        })
        .await;
}
