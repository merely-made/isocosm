//! Shared chain checks for received and locally signed host records.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use muniment::Backend;
use murm_replication::MunimentStore;
use p2panda_core::{Operation, SigningKey, Topic};

use super::{FamilyRecordExt, sign_record, verify_host_record};
use crate::family::{AcceptedRecord, SessionId, WireLimits};

pub(super) fn verify_parent_link(
    record: &AcceptedRecord,
    previous: Option<&AcceptedRecord>,
) -> Result<(), String> {
    if previous.is_some_and(|previous| record.parent_digest != previous.digest) {
        return Err("accepted record parent digest does not match its predecessor".into());
    }
    Ok(())
}

pub(super) async fn accept_incoming<B>(
    store: &MunimentStore<B, FamilyRecordExt>,
    operation: Operation<FamilyRecordExt>,
    topic: [u8; 32],
    session: SessionId,
    host: &p2panda_core::VerifyingKey,
    limits: WireLimits,
    received: tokio::sync::mpsc::Sender<AcceptedRecord>,
    rejected: Arc<AtomicU64>,
) -> bool
where
    B: Backend + Clone + Send + Sync + 'static,
{
    let record = match verify_host_record(&operation, host, session, limits) {
        Ok(record) => record,
        Err(_) => {
            rejected.fetch_add(1, Ordering::Relaxed);
            return false;
        },
    };
    match store.get_operation(&operation.hash).await {
        Ok(None) => {},
        Ok(Some(_)) => return false,
        Err(_) => {
            rejected.fetch_add(1, Ordering::Relaxed);
            return false;
        },
    }
    let previous = match store.get_latest_entry(host, &session.0).await {
        Ok(previous) => previous,
        Err(_) => {
            rejected.fetch_add(1, Ordering::Relaxed);
            return false;
        },
    };
    let (expected_sequence, expected_backlink, previous_record) = match previous {
        Some(previous) => {
            let Some(sequence) = previous.header.seq_num.checked_add(1) else {
                rejected.fetch_add(1, Ordering::Relaxed);
                return false;
            };
            let stored = match store.get_operation(&previous.hash).await {
                Ok(Some(stored)) => stored,
                _ => {
                    rejected.fetch_add(1, Ordering::Relaxed);
                    return false;
                },
            };
            let previous_record = match verify_host_record(&stored, host, session, limits) {
                Ok(record) => record,
                Err(_) => {
                    rejected.fetch_add(1, Ordering::Relaxed);
                    return false;
                },
            };
            (sequence, Some(previous.hash), Some(previous_record))
        },
        None => (0, None, None),
    };
    if operation.header.seq_num != expected_sequence
        || operation.header.backlink != expected_backlink
        || verify_parent_link(&record, previous_record.as_ref()).is_err()
    {
        rejected.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    match store
        .insert_indexed_operation(&Topic::from(topic), &operation, &session.0)
        .await
    {
        Ok(true) => received.send(record).await.is_ok(),
        Ok(false) | Err(_) => false,
    }
}

pub(super) async fn sign_next_record<B>(
    store: &MunimentStore<B, FamilyRecordExt>,
    record: &AcceptedRecord,
    key: &SigningKey,
    host: &p2panda_core::VerifyingKey,
    session: SessionId,
    limits: WireLimits,
) -> Result<Operation<FamilyRecordExt>, String>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    let previous = store
        .get_latest_entry(host, &session.0)
        .await
        .map_err(|error| error.to_string())?;
    let (sequence, backlink, previous_record) = match previous {
        Some(previous) => {
            let sequence = previous
                .header
                .seq_num
                .checked_add(1)
                .ok_or("host log sequence overflow")?;
            let stored = store
                .get_operation(&previous.hash)
                .await
                .map_err(|error| error.to_string())?
                .ok_or("host log predecessor is missing from operation store")?;
            let previous_record = verify_host_record(&stored, host, session, limits)?;
            (
                sequence,
                Some(*previous.hash.as_bytes()),
                Some(previous_record),
            )
        },
        None => (0, None, None),
    };
    verify_parent_link(record, previous_record.as_ref())?;
    let operation = sign_record(record, key, sequence, backlink, limits)?;
    verify_host_record(&operation, host, session, limits)?;
    Ok(operation)
}
