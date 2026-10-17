use super::super::acceptance::{accept_incoming, sign_next_record};
use super::super::{FamilyRecordExt, read_stored_suffix, sign_record, verify_host_record};
use super::{SESSION, TOPIC, host};
use crate::family::{
    AcceptedRecord, FAMILY_FORMAT_VERSION, FamilyHost, HostAdmission, SessionId, WireLimits,
};
use isocosm::Command;
use muniment::MemoryBackend;
use murm_replication::MunimentStore;
use p2panda_core::{Body, Header, Operation, Signature, SigningKey, Topic};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

fn accepted_pair() -> (AcceptedRecord, AcceptedRecord) {
    fn accept(
        host: &mut FamilyHost,
        commands: Vec<Command>,
        section_updates: BTreeMap<String, Option<Vec<u8>>>,
    ) -> AcceptedRecord {
        host.accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands,
                final_tick: None,
                section_updates,
            })
        })
        .expect("family host accepts fixture record")
        .2
        .expect("accepted transition produces a record")
    }

    let mut family_host = host();
    let first = accept(
        &mut family_host,
        vec![Command::Join],
        BTreeMap::from([("unknown-section".into(), Some(vec![7, 8]))]),
    );
    let second = accept(
        &mut family_host,
        Vec::new(),
        BTreeMap::from([("source-document".into(), Some(vec![4, 5, 6]))]),
    );
    (first, second)
}

fn sign_with_format(
    record: &AcceptedRecord,
    key: &SigningKey,
    format: u16,
) -> Operation<FamilyRecordExt> {
    let bytes = record.encode().expect("encode fixture record");
    let header = Header::builder()
        .body(&bytes)
        .seq_num(0)
        .backlink(None)
        .build(
            key,
            FamilyRecordExt {
                session: record.session,
                format,
            },
        );
    Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(&bytes)),
    }
}

#[test]
fn signed_records_reject_tampering_wrong_envelopes_positions_and_caps() {
    let (record, next) = accepted_pair();
    let key = SigningKey::from_bytes(&[0x51; 32]);
    let author = key.verifying_key();
    let valid = sign_record(&record, &key, 0, None, WireLimits::default()).unwrap();
    assert_eq!(
        verify_host_record(&valid, &author, SESSION, WireLimits::default()).unwrap(),
        record
    );

    let mut tampered_body = valid.clone();
    let mut body = tampered_body.body.as_ref().unwrap().to_bytes().to_vec();
    body[0] ^= 1;
    tampered_body.body = Some(Body::from_bytes(&body));
    assert!(verify_host_record(&tampered_body, &author, SESSION, WireLimits::default()).is_err());

    let mut tampered_signature = valid.clone();
    let mut signature = tampered_signature.header.signature.to_bytes();
    signature[0] ^= 1;
    tampered_signature.header.signature = Signature::from_bytes(&signature);
    tampered_signature.hash = p2panda_core::Hash::digest(tampered_signature.header.encode());
    assert!(
        verify_host_record(&tampered_signature, &author, SESSION, WireLimits::default()).is_err()
    );

    let mut other_session = record.clone();
    other_session.session = SessionId([0x99; 16]);
    other_session.digest.clear();
    other_session.digest = isocosm::digest(&other_session);
    let wrong_session = sign_record(&other_session, &key, 0, None, WireLimits::default()).unwrap();
    assert!(verify_host_record(&wrong_session, &author, SESSION, WireLimits::default()).is_err());

    let wrong_version = sign_with_format(&record, &key, FAMILY_FORMAT_VERSION + 1);
    assert!(verify_host_record(&wrong_version, &author, SESSION, WireLimits::default()).is_err());
    assert!(sign_record(&next, &key, 0, None, WireLimits::default()).is_err());
    assert!(sign_record(&record, &key, 0, None, WireLimits { max_bytes: 1 }).is_err());
    assert!(verify_host_record(&valid, &author, SESSION, WireLimits { max_bytes: 1 }).is_err());
}

async fn insert(
    store: &MunimentStore<MemoryBackend, FamilyRecordExt>,
    operation: &Operation<FamilyRecordExt>,
) {
    store
        .insert_indexed_operation(&Topic::from(TOPIC), operation, &SESSION.0)
        .await
        .expect("fixture operation indexes");
}

#[tokio::test]
async fn stored_suffix_rejects_broken_chains_gaps_bad_cursors_and_limits() {
    let (first, second) = accepted_pair();
    let key = SigningKey::from_bytes(&[0x51; 32]);
    let author = key.verifying_key();
    let first_op = sign_record(&first, &key, 0, None, WireLimits::default()).unwrap();
    let broken_link =
        sign_record(&second, &key, 1, Some([0xab; 32]), WireLimits::default()).unwrap();
    let store = MunimentStore::new(MemoryBackend::new());
    insert(&store, &first_op).await;
    insert(&store, &broken_link).await;
    assert!(
        read_stored_suffix(
            &store,
            SESSION,
            &author,
            None,
            2,
            2 * WireLimits::default().max_bytes,
            WireLimits::default(),
        )
        .await
        .is_err()
    );

    let gap_store = MunimentStore::new(MemoryBackend::new());
    insert(&gap_store, &broken_link).await;
    assert!(
        read_stored_suffix(
            &gap_store,
            SESSION,
            &author,
            None,
            1,
            WireLimits::default().max_bytes,
            WireLimits::default(),
        )
        .await
        .is_err()
    );

    let valid_store = MunimentStore::new(MemoryBackend::new());
    insert(&valid_store, &first_op).await;
    assert!(
        read_stored_suffix(
            &valid_store,
            SESSION,
            &author,
            Some(1),
            1,
            WireLimits::default().max_bytes,
            WireLimits::default(),
        )
        .await
        .is_err()
    );
    assert!(
        read_stored_suffix(
            &valid_store,
            SESSION,
            &author,
            None,
            1,
            1,
            WireLimits::default(),
        )
        .await
        .is_err()
    );
    assert!(
        read_stored_suffix(
            &valid_store,
            SESSION,
            &author,
            None,
            257,
            WireLimits::default().max_bytes,
            WireLimits::default(),
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn wrong_family_parent_is_refused_before_store_queue_and_recovery_return() {
    let (first, second) = accepted_pair();
    let key = SigningKey::from_bytes(&[0x51; 32]);
    let author = key.verifying_key();
    let first_op = sign_record(&first, &key, 0, None, WireLimits::default()).unwrap();
    let mut wrong_parent = second;
    wrong_parent.parent_digest = "wrong-prior-family-digest".into();
    wrong_parent.digest.clear();
    wrong_parent.digest = isocosm::digest(&wrong_parent);
    let wrong_op = sign_record(
        &wrong_parent,
        &key,
        1,
        Some(*first_op.hash.as_bytes()),
        WireLimits::default(),
    )
    .unwrap();
    assert_eq!(
        verify_host_record(&wrong_op, &author, SESSION, WireLimits::default()).unwrap(),
        wrong_parent,
        "negative control is a validly signed, internally digested wrong-parent record"
    );
    let store = MunimentStore::new(MemoryBackend::new());
    insert(&store, &first_op).await;

    let (received_tx, mut received_rx) = tokio::sync::mpsc::channel(2);
    let rejected = Arc::new(AtomicU64::new(0));
    assert!(
        !accept_incoming(
            &store,
            wrong_op.clone(),
            TOPIC,
            SESSION,
            &author,
            WireLimits::default(),
            received_tx,
            Arc::clone(&rejected),
        )
        .await
    );
    assert_eq!(rejected.load(Ordering::Relaxed), 1);
    assert!(store.get_operation(&wrong_op.hash).await.unwrap().is_none());
    assert_eq!(
        store
            .get_latest_entry(&author, &SESSION.0)
            .await
            .unwrap()
            .unwrap()
            .header
            .seq_num,
        0,
        "refusal leaves the durable host log at the accepted predecessor"
    );
    assert!(received_rx.try_recv().is_err());

    assert!(
        sign_next_record(
            &store,
            &wrong_parent,
            &key,
            &author,
            SESSION,
            WireLimits::default(),
        )
        .await
        .is_err(),
        "local signing refuses a broken family parent before FamilyRecordLane inserts"
    );
    assert!(store.get_operation(&wrong_op.hash).await.unwrap().is_none());

    let persisted_invalid = MunimentStore::new(MemoryBackend::new());
    insert(&persisted_invalid, &first_op).await;
    insert(&persisted_invalid, &wrong_op).await;
    assert!(
        read_stored_suffix(
            &persisted_invalid,
            SESSION,
            &author,
            Some(0),
            1,
            WireLimits::default().max_bytes,
            WireLimits::default(),
        )
        .await
        .is_err(),
        "recovery page checks its cursor predecessor's family digest"
    );
}
