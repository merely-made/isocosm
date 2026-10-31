//! Bounded owner-thread handoff for `FamilyHost` accepted records.
//!
//! Reserve one batch slot before draining the host journal. A full or closed
//! queue therefore leaves every accepted record in `FamilyHost`'s bounded
//! outbox. The receiver publishes each record through `FamilyRecordLane`;
//! keep a received batch alive until publication or recovery is resolved.

use crate::family::{AcceptedRecord, FamilyHost};

pub const MAX_FAMILY_RECORD_BATCHES: usize = 16;
pub const MAX_FAMILY_RECORD_BATCH_BYTES: usize = 16 * 1_048_576;

/// One ordered, already-authenticated family journal batch.
#[derive(Debug)]
pub struct FamilyRecordBatch {
    records: Vec<AcceptedRecord>,
    encoded_bytes: usize,
}

impl FamilyRecordBatch {
    pub fn records(&self) -> &[AcceptedRecord] {
        &self.records
    }

    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub fn into_records(self) -> Vec<AcceptedRecord> {
        self.records
    }
}

/// Owner-thread sender. Queue batch count and each batch's encoded bytes are
/// bounded; worst-case queued bytes are `capacity * max_batch_bytes`, plus a
/// batch currently being published by the receiver.
pub struct FamilyRecordOutbox {
    sender: tokio::sync::mpsc::Sender<FamilyRecordBatch>,
    max_batch_bytes: usize,
}

impl FamilyRecordOutbox {
    pub fn channel(
        capacity: usize,
        max_batch_bytes: usize,
    ) -> Result<(Self, tokio::sync::mpsc::Receiver<FamilyRecordBatch>), String> {
        if capacity == 0 || max_batch_bytes == 0 {
            return Err("family record queue capacity and batch limit must be nonzero".into());
        }
        if capacity > MAX_FAMILY_RECORD_BATCHES {
            return Err(format!(
                "family record queue capacity exceeds {MAX_FAMILY_RECORD_BATCHES}"
            ));
        }
        if max_batch_bytes > MAX_FAMILY_RECORD_BATCH_BYTES {
            return Err(format!(
                "family record batch limit exceeds {MAX_FAMILY_RECORD_BATCH_BYTES} bytes"
            ));
        }
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        Ok((
            Self {
                sender,
                max_batch_bytes,
            },
            receiver,
        ))
    }

    /// Transfer the current accepted journal records to the transport worker.
    /// This call is synchronous and nonblocking, so it is safe in the UI pump.
    /// When the queue is full, no host records are drained.
    pub fn try_forward(&self, host: &FamilyHost) -> Result<usize, String> {
        let expected_bytes = host.outbox_bytes();
        if expected_bytes == 0 {
            return Ok(0);
        }
        if expected_bytes > self.max_batch_bytes {
            return Err(format!(
                "accepted batch is {expected_bytes} bytes; queue limit is {}",
                self.max_batch_bytes
            ));
        }

        let permit = self.sender.try_reserve().map_err(|error| match error {
            tokio::sync::mpsc::error::TrySendError::Full(()) => {
                "family record queue is full".to_string()
            },
            tokio::sync::mpsc::error::TrySendError::Closed(()) => {
                "family record queue is closed".to_string()
            },
        })?;
        let records = host.drain_records();
        if records.is_empty() {
            drop(permit);
            return Err("host outbox byte count was nonzero but contained no records".into());
        }
        let count = records.len();
        permit.send(FamilyRecordBatch {
            records,
            encoded_bytes: expected_bytes,
        });
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::HostAdmission;
    use cosmograph::{Command, Execution, Founding, Session};
    use cosmograph_host::world::World;
    use std::collections::BTreeMap;

    fn host() -> FamilyHost {
        let genesis = Founding {
            seed: 21,
            sites: 2,
            population: 4,
            lineages: 2,
            ecology: false,
            ..Default::default()
        }
        .generate()
        .expect("small test genesis");
        let session = Session::new(genesis, Execution::Individuals).expect("test session");
        FamilyHost::new(crate::family::SessionId([0x31; 16]), World::new(session))
            .expect("family host")
    }

    fn update(host: &mut FamilyHost, value: u8) {
        host.accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: Vec::<Command>::new(),
                final_tick: None,
                section_updates: BTreeMap::from([(
                    "fixture-section".to_string(),
                    Some(vec![value]),
                )]),
            })
        })
        .expect("opaque section update");
    }

    #[test]
    fn full_transport_queue_does_not_drain_host_records() {
        let (outbox, mut receiver) = FamilyRecordOutbox::channel(1, 4 * 1_048_576).unwrap();
        let mut host = host();
        update(&mut host, 1);
        assert_eq!(outbox.try_forward(&host).unwrap(), 1);
        assert_eq!(host.outbox_bytes(), 0);

        update(&mut host, 2);
        let queued_bytes = host.outbox_bytes();
        assert!(queued_bytes > 0);
        assert!(outbox.try_forward(&host).unwrap_err().contains("full"));
        assert_eq!(host.outbox_bytes(), queued_bytes);

        let first = receiver.try_recv().unwrap();
        assert_eq!(first.records().len(), 1);
        assert_eq!(outbox.try_forward(&host).unwrap(), 1);
        let second = receiver.try_recv().unwrap();
        assert_eq!(second.records().len(), 1);
    }

    #[test]
    fn closed_transport_queue_does_not_drain_host_records() {
        let (outbox, receiver) = FamilyRecordOutbox::channel(1, 4 * 1_048_576).unwrap();
        let mut host = host();
        update(&mut host, 1);
        let queued_bytes = host.outbox_bytes();
        drop(receiver);

        assert_eq!(
            outbox.try_forward(&host).unwrap_err(),
            "family record queue is closed"
        );
        assert_eq!(host.outbox_bytes(), queued_bytes);
    }
}
