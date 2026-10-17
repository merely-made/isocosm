//! Signed host-only family records over JoinedSpace. The `!Send` family world
//! stays on its owner thread; accepted records cross by Tokio channel.

use muniment::Backend;
use murm_replication::{JoinError, JoinedSpace, MunimentStore, lane_id};
use p2panda_core::{
    Body, Hash, Header, Operation, SigningKey, Topic, VerifyingKey, operation::validate_operation,
};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use transport::P2pandaTransport;

use crate::family::{AcceptedRecord, FAMILY_FORMAT_VERSION, SessionId, WireLimits};

mod bootstrap;
pub use bootstrap::{
    FamilyCheckpointDescriptor, FamilyCheckpointResource, verify_checkpoint_resource,
};

/// Signed extension used by the single-host accepted-record log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyRecordExt {
    pub session: SessionId,
    pub format: u16,
}

impl Default for FamilyRecordExt {
    fn default() -> Self {
        Self {
            session: SessionId([0; 16]),
            format: 0,
        }
    }
}

/// Sign one bounded record at its host log position (record sequence is log + 1).
pub fn sign_record(
    record: &AcceptedRecord,
    key: &SigningKey,
    sequence: u32,
    backlink: Option<[u8; 32]>,
    limits: WireLimits,
) -> Result<Operation<FamilyRecordExt>, String> {
    if record.format != FAMILY_FORMAT_VERSION || record.sequence != u64::from(sequence) + 1 {
        return Err("family and host log sequences do not match".into());
    }
    let bytes = record.encode_with_limits(limits)?;
    let body = Body::from_bytes(&bytes);
    let header = Header::builder()
        .body(&bytes)
        .seq_num(sequence)
        .backlink(backlink.map(Hash::from))
        .build(
            key,
            FamilyRecordExt {
                session: record.session,
                format: FAMILY_FORMAT_VERSION,
            },
        );
    let hash = header.hash();
    Ok(Operation {
        hash,
        header,
        body: Some(body),
    })
}

/// Verify p2panda authorship and the family envelope before insert.
pub fn verify_host_record(
    operation: &Operation<FamilyRecordExt>,
    host: &VerifyingKey,
    session: SessionId,
    limits: WireLimits,
) -> Result<AcceptedRecord, String> {
    // Header's cached in-memory verifier is intentionally permissive in
    // production builds. Decode its signed bytes at this trust boundary: the
    // p2panda decoder performs strict signature verification and recomputes
    // the header hash from those bytes.
    let header = Header::<FamilyRecordExt>::decode(&operation.header.encode())
        .map_err(|error| format!("accepted record header is invalid: {error}"))?;
    if operation.hash != header.hash() {
        return Err("accepted record operation hash does not match its signed header".into());
    }
    let signed_operation = Operation {
        hash: operation.hash,
        header,
        body: operation.body.clone(),
    };
    validate_operation(&signed_operation).map_err(|error| error.to_string())?;
    if &signed_operation.header.verifying_key != host {
        return Err("accepted record was not authored by the pinned host".into());
    }
    if signed_operation.header.extensions.session != session
        || signed_operation.header.extensions.format != FAMILY_FORMAT_VERSION
    {
        return Err("accepted record extension names another family session".into());
    }
    let body = signed_operation
        .body
        .as_ref()
        .ok_or_else(|| "accepted record has no body".to_string())?;
    let record = AcceptedRecord::decode_with_limits(body.to_bytes().as_slice(), limits)?;
    if record.format != FAMILY_FORMAT_VERSION || record.session != session {
        return Err("accepted record body names another family session".into());
    }
    if record.sequence != u64::from(signed_operation.header.seq_num) + 1 {
        return Err("accepted record sequence does not match its signed log position".into());
    }
    let mut unsigned = record.clone();
    let digest = std::mem::take(&mut unsigned.digest);
    if digest.is_empty() || digest != isocosm::digest(&unsigned) {
        return Err("accepted record digest mismatch".into());
    }
    Ok(record)
}

/// One continuously reconciling host-author log and its local store.
pub struct FamilyRecordLane<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    store: MunimentStore<B, FamilyRecordExt>,
    joined: JoinedSpace<FamilyRecordExt>,
    session: SessionId,
    topic: [u8; 32],
    host: VerifyingKey,
    rejected_records: Arc<AtomicU64>,
}

impl<B> FamilyRecordLane<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    /// Start the host-only record lane. Authenticated records are persisted in
    /// Stickleback before they are sent to the owner's local world thread.
    pub async fn join(
        transport: &P2pandaTransport,
        store: MunimentStore<B, FamilyRecordExt>,
        session: SessionId,
        topic: [u8; 32],
        host: VerifyingKey,
        limits: WireLimits,
        received: tokio::sync::mpsc::Sender<AcceptedRecord>,
    ) -> Result<Self, JoinError> {
        let (endpoint, gossip) = transport
            .sync_parts()
            .ok_or_else(|| JoinError::Spawn("transport gossip is disabled".into()))?;
        let receiving_store = store.clone();
        let topic_for_accept = topic;
        let rejected_records = Arc::new(AtomicU64::new(0));
        let rejects = Arc::clone(&rejected_records);
        let joined = JoinedSpace::join::<_, [u8; 16], _, _>(
            lane_id("isocosm/family-records/v1", topic),
            store.clone(),
            endpoint,
            gossip,
            topic,
            move |operation: Operation<FamilyRecordExt>| {
                let store = receiving_store.clone();
                let received = received.clone();
                let host = host.clone();
                let rejects = Arc::clone(&rejects);
                let limits = limits;
                async move {
                    acceptance::accept_incoming(
                        &store,
                        operation,
                        topic_for_accept,
                        session,
                        &host,
                        limits,
                        received,
                        rejects,
                    )
                    .await
                }
            },
        )
        .await?;
        Ok(Self {
            store,
            joined,
            session,
            topic,
            host,
            rejected_records,
        })
    }

    /// Persist a host-signed record, then publish it to connected replicas.
    /// A publish failure leaves the operation in the log for later resync.
    pub async fn publish(
        &self,
        record: &AcceptedRecord,
        key: &SigningKey,
        limits: WireLimits,
    ) -> Result<Operation<FamilyRecordExt>, String> {
        let author = key.verifying_key();
        if author != self.host {
            return Err("record signing key differs from pinned family host".into());
        }
        if record.session != self.session {
            return Err("record belongs to another family session".into());
        }
        let log_id = self.session.0;
        let operation = acceptance::sign_next_record(
            &self.store,
            record,
            key,
            &self.host,
            self.session,
            limits,
        )
        .await?;
        let topic = Topic::from(self.topic);
        let inserted = self
            .store
            .insert_indexed_operation(&topic, &operation, &log_id)
            .await
            .map_err(|error| error.to_string())?;
        if !inserted {
            return Err("host record already exists in the operation store".into());
        }
        self.joined
            .publish(operation.clone())
            .map_err(|error| error.to_string())?;
        Ok(operation)
    }

    pub fn sync_status(&self) -> murm_replication::SyncStatus {
        self.joined.sync_status()
    }

    pub fn rejected_records(&self) -> u64 {
        self.rejected_records.load(Ordering::Relaxed)
    }

    pub async fn resync(&self) -> murm_replication::SyncRound {
        self.joined.resync().await
    }

    pub async fn leave_and_wait(self) -> Result<(), JoinError> {
        self.joined.leave_and_wait().await
    }
}

#[cfg(test)]
#[path = "family_network/tests.rs"]
mod tests;

#[path = "family_network/acceptance.rs"]
mod acceptance;

#[path = "family_network/stored_suffix.rs"]
mod stored_suffix;

pub use stored_suffix::{StoredSuffix, read_stored_suffix};

/// Graphshell's admitted stream path, re-exported at the Isocosm family seam.
///
/// Admission and request semantics remain owned by Graphshell. Isocosm owns
/// the family host, signed accepted-record log, and replica application.
pub mod stream {
    pub use graphshell_native::{admission, carrier, lifecycle, network_carrier, session_loop};
}
