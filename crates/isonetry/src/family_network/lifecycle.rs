//! Family host and replica lifecycle composition.
//!
//! The application owns durable identity, invitations, admission policy, the
//! transport, and the `!Send` product worlds. This module composes the already
//! authenticated Graphshell resource and signed Stickleback log helpers; it
//! does not create keys, infer participants, or replace admission policy.

use super::{
    FamilyCheckpointDescriptor, FamilyCheckpointResource, FamilyRecordBatch, FamilyRecordExt,
    FamilyRecordLane, read_stored_suffix, verify_checkpoint_resource,
};
use crate::family::{AcceptedRecord, Checkpoint, FamilyReplica, SessionId, WireLimits};
use chirograph::{ProjectionSession, ResourceResponse};
use muniment::Backend;
use murm_replication::{JoinError, MunimentStore};
use p2panda_core::{SigningKey, VerifyingKey};
use std::collections::VecDeque;
use transport::P2pandaTransport;

pub const MAX_REPLICA_RECORD_QUEUE: usize = 64;

/// Host-side ownership of one signed accepted-record lane.
///
/// The host key is supplied by the application and should be loaded from its
/// protected persistent identity store. The family world stays with the
/// application on its owner thread and is passed only to synchronous
/// checkpoint creation.
pub struct FamilyHostNetwork<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    lane: FamilyRecordLane<B>,
    store: MunimentStore<B, FamilyRecordExt>,
    signing_key: SigningKey,
    session: SessionId,
    limits: WireLimits,
    // Serializes host-log writes with checkpoint-tail validation so a
    // checkpoint cannot be signed from a World cut that races a publication.
    publication_gate: tokio::sync::Mutex<()>,
    _discarded_echoes: tokio::task::JoinHandle<()>,
}

impl<B> FamilyHostNetwork<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    /// Join the signed record lane using a caller-owned transport and store.
    /// Incoming records are persisted and verified by `FamilyRecordLane`;
    /// they are not applied to this host's authoritative world.
    pub async fn join(
        transport: &P2pandaTransport,
        store: MunimentStore<B, super::FamilyRecordExt>,
        session: SessionId,
        topic: [u8; 32],
        signing_key: SigningKey,
        limits: WireLimits,
    ) -> Result<Self, JoinError> {
        let (incoming, mut echoes) = tokio::sync::mpsc::channel(1);
        let discarded_echoes = tokio::spawn(async move { while echoes.recv().await.is_some() {} });
        let lane = match FamilyRecordLane::join(
            transport,
            store.clone(),
            session,
            topic,
            signing_key.verifying_key(),
            limits,
            incoming,
        )
        .await
        {
            Ok(lane) => lane,
            Err(error) => {
                let _ = discarded_echoes.await;
                return Err(error);
            },
        };
        Ok(Self {
            lane,
            store,
            signing_key,
            session,
            limits,
            publication_gate: tokio::sync::Mutex::new(()),
            _discarded_echoes: discarded_echoes,
        })
    }

    /// Sign a fresh full-world checkpoint resource for an admitted Graphshell
    /// projection session. The application must separately authorize full
    /// replication before exposing this resource; projection access alone is
    /// not sufficient. The checkpoint is checked against the signed lane's
    /// exact tail before it can be advertised, preventing a stale or forked
    /// World from being paired with this host identity.
    pub async fn signed_checkpoint(
        &self,
        checkpoint: Checkpoint,
        projection_session: ProjectionSession,
    ) -> Result<FamilyCheckpointResource, String> {
        let _publication = self.publication_gate.lock().await;
        checkpoint.validate()?;
        if checkpoint.session != self.session {
            return Err("family host network and World name different sessions".into());
        }
        let host_key = self.signing_key.verifying_key();
        if checkpoint.sequence == 0 {
            let stored = read_stored_suffix(
                &self.store,
                self.session,
                &host_key,
                None,
                1,
                self.limits.max_bytes,
                self.limits,
            )
            .await?;
            if !stored.records.is_empty() {
                return Err("empty host checkpoint does not match the nonempty signed log".into());
            }
        } else {
            let log_sequence = u32::try_from(checkpoint.sequence - 1)
                .map_err(|_| "checkpoint sequence exceeds the signed log range")?;
            let previous = read_stored_suffix(
                &self.store,
                self.session,
                &host_key,
                log_sequence.checked_sub(1),
                1,
                self.limits.max_bytes,
                self.limits,
            )
            .await?;
            let tail = previous
                .records
                .first()
                .ok_or("checkpoint has no matching signed log tail")?;
            if tail.sequence != checkpoint.sequence || tail.digest != checkpoint.parent_digest {
                return Err("checkpoint does not match the signed log tail".into());
            }
            let later = read_stored_suffix(
                &self.store,
                self.session,
                &host_key,
                Some(log_sequence),
                1,
                self.limits.max_bytes,
                self.limits,
            )
            .await?;
            if !later.records.is_empty() {
                return Err("checkpoint is behind the signed host log".into());
            }
        }
        FamilyCheckpointResource::new(
            &checkpoint,
            &self.signing_key,
            projection_session,
            self.limits,
        )
    }

    /// Publish an ordered batch. Existing matching records are checked through
    /// the verified stored-suffix reader, making a retry safe when `publish`
    /// persisted a record before gossip reported an error. The result counts
    /// records now durably present in the local signed log; it does not claim
    /// that every connected replica has applied them. On failure, the caller
    /// still owns the batch and can retry after lane reconciliation.
    pub async fn publish_batch(&self, batch: &FamilyRecordBatch) -> Result<usize, String> {
        let _publication = self.publication_gate.lock().await;
        if batch
            .records()
            .iter()
            .any(|record| record.session != self.session)
        {
            return Err("accepted batch belongs to another family session".into());
        }
        self.lane.resync().await;
        let mut published = 0usize;
        for record in batch.records() {
            let log_sequence = record
                .sequence
                .checked_sub(1)
                .ok_or("accepted record sequence must start at one")?;
            let log_sequence = u32::try_from(log_sequence)
                .map_err(|_| "accepted record exceeds the signed log sequence range")?;
            let after_sequence = log_sequence.checked_sub(1);
            let stored = read_stored_suffix(
                &self.store,
                self.session,
                &self.signing_key.verifying_key(),
                after_sequence,
                1,
                self.limits.max_bytes,
                self.limits,
            )
            .await?;
            if let Some(existing) = stored.records.first() {
                if existing != record {
                    return Err(format!(
                        "stored host record {} conflicts with accepted batch",
                        record.sequence
                    ));
                }
                published += 1;
                continue;
            }
            if let Err(error) = self
                .lane
                .publish(record, &self.signing_key, self.limits)
                .await
            {
                let resync = self.lane.resync().await;
                return Err(format!(
                    "publishing family record {} failed after {published} records ({error}); lane resync: {resync:?}",
                    record.sequence
                ));
            }
            published += 1;
        }
        Ok(published)
    }

    /// Reconcile stored host records with connected peers after reconnect or
    /// after a publish error.
    pub async fn resync(&self) -> murm_replication::SyncRound {
        self.lane.resync().await
    }

    pub fn host_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Leave the accepted-record topic and wait for its replication task.
    pub async fn leave_and_wait(self) -> Result<(), JoinError> {
        let Self {
            lane,
            _discarded_echoes,
            ..
        } = self;
        lane.leave_and_wait().await?;
        _discarded_echoes
            .await
            .map_err(|error| JoinError::Spawn(format!("record echo drain stopped: {error}")))
    }
}

/// Replica-side signed record subscription and owner-thread delivery queue.
pub struct FamilyReplicaNetwork<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    lane: FamilyRecordLane<B>,
    records: tokio::sync::mpsc::Receiver<AcceptedRecord>,
    pending: VecDeque<AcceptedRecord>,
    pending_capacity: usize,
}

impl<B> FamilyReplicaNetwork<B>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    /// Subscribe before fetching the bootstrap resource so records arriving
    /// during that request remain queued behind the bounded receiver.
    pub async fn join(
        transport: &P2pandaTransport,
        store: MunimentStore<B, super::FamilyRecordExt>,
        descriptor: &FamilyCheckpointDescriptor,
        topic: [u8; 32],
        pinned_host: VerifyingKey,
        limits: WireLimits,
        record_capacity: usize,
    ) -> Result<Self, String> {
        if record_capacity == 0 || record_capacity > MAX_REPLICA_RECORD_QUEUE {
            return Err(format!(
                "replica record queue capacity must be between 1 and {MAX_REPLICA_RECORD_QUEUE}"
            ));
        }
        let (sender, records) = tokio::sync::mpsc::channel(record_capacity);
        let lane = FamilyRecordLane::join(
            transport,
            store,
            descriptor.family_session,
            topic,
            pinned_host,
            limits,
            sender,
        )
        .await
        .map_err(|error| error.to_string())?;
        Ok(Self {
            lane,
            records,
            pending: VecDeque::new(),
            pending_capacity: record_capacity,
        })
    }

    /// Verify the full-world bootstrap resource against the invitation's
    /// descriptor and pinned host before constructing a replica.
    pub fn install_checkpoint(
        response: &ResourceResponse,
        descriptor: &FamilyCheckpointDescriptor,
        pinned_host: &VerifyingKey,
        limits: WireLimits,
        execution: cosmograph::Execution,
    ) -> Result<FamilyReplica, String> {
        let checkpoint = verify_checkpoint_resource(response, descriptor, pinned_host, limits)?;
        FamilyReplica::install(checkpoint, &[], execution)
    }

    /// Apply authenticated lane records on the replica's owner thread.
    /// Already-covered prefix records are ignored after bootstrap. A gap or
    /// failed native replay remains pending, and later records are not allowed
    /// to jump over it.
    pub fn apply_ready(&mut self, replica: &mut FamilyReplica) -> Result<usize, String> {
        while self.pending.len() < self.pending_capacity {
            match self.records.try_recv() {
                Ok(record) => self.pending.push_back(record),
                Err(_) => break,
            }
        }
        let mut applied = 0usize;
        loop {
            let Some(record) = self.pending.front() else {
                break;
            };
            if record.sequence <= replica.sequence() {
                self.pending.pop_front();
                continue;
            }
            let expected = replica
                .sequence()
                .checked_add(1)
                .ok_or("replica record sequence overflow")?;
            if record.sequence != expected {
                return Err(format!(
                    "received family sequence {}, expected {expected}",
                    record.sequence
                ));
            }
            replica.apply(record.clone())?;
            self.pending.pop_front();
            applied += 1;
        }
        Ok(applied)
    }

    pub async fn resync(&self) -> murm_replication::SyncRound {
        self.lane.resync().await
    }

    /// Leave the accepted-record topic and wait for its replication task.
    pub async fn leave_and_wait(self) -> Result<(), JoinError> {
        self.lane.leave_and_wait().await
    }
}
