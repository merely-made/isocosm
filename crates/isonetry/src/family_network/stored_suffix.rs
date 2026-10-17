//! Bounded recovery reads for records persisted before owner-thread delivery.

use super::{FamilyRecordExt, acceptance::verify_parent_link, verify_host_record};
use crate::family::{AcceptedRecord, SessionId, WireLimits};
use muniment::Backend;
use murm_replication::MunimentStore;

const MAX_BATCH_RECORDS: usize = 256;

/// One verified page of a host's stored accepted-record log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSuffix {
    /// Decoded accepted records in ascending host log order.
    pub records: Vec<AcceptedRecord>,
    /// Last included host log sequence, suitable as the next exclusive cursor.
    pub through_sequence: Option<u32>,
    /// Whether records remain after this page.
    pub has_more: bool,
}

/// Read and verify at most one bounded page after `after_sequence`.
///
/// `None` starts at host log sequence zero. A cursor is exclusive. The stored
/// predecessor is checked so the first returned record must backlink to it.
/// Both operation count and encoded header/body bytes are bounded.
pub async fn read_stored_suffix<B>(
    store: &MunimentStore<B, FamilyRecordExt>,
    session: SessionId,
    host: &p2panda_core::VerifyingKey,
    after_sequence: Option<u32>,
    max_records: usize,
    max_wire_bytes: usize,
    limits: WireLimits,
) -> Result<StoredSuffix, String>
where
    B: Backend + Clone + Send + Sync + 'static,
{
    if max_records == 0 || max_records > MAX_BATCH_RECORDS || max_wire_bytes == 0 {
        return Err("stored suffix limits are outside the supported bounds".into());
    }
    let log_id = session.0;
    let (mut previous_hash, mut previous_record) = match after_sequence {
        None => (None, None),
        Some(sequence) => {
            let previous = store
                .get_log_entries(host, &log_id, sequence.checked_sub(1), Some(sequence))
                .await
                .map_err(|error| error.to_string())?
                .and_then(|entries| {
                    entries
                        .into_iter()
                        .find(|(operation, _)| operation.header.seq_num == sequence)
                })
                .ok_or("stored suffix cursor has no signed predecessor")?;
            let record = verify_host_record(&previous.0, host, session, limits)?;
            (Some(previous.0.hash), Some(record))
        },
    };
    let first_sequence = after_sequence.map_or(0, |sequence| sequence.saturating_add(1));
    if after_sequence == Some(u32::MAX) {
        return Ok(StoredSuffix {
            records: Vec::new(),
            through_sequence: after_sequence,
            has_more: false,
        });
    }
    let last_sequence =
        first_sequence.saturating_add(u32::try_from(max_records - 1).unwrap_or(u32::MAX));
    let found = store
        .get_log_entries(host, &log_id, after_sequence, Some(last_sequence))
        .await
        .map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    let mut expected_sequence = u64::from(first_sequence);
    let mut used_bytes = 0usize;
    for (operation, encoded_header) in found.unwrap_or_default() {
        if u64::from(operation.header.seq_num) != expected_sequence {
            return Err("stored accepted-record log contains a sequence gap".into());
        }
        if operation.header.backlink != previous_hash {
            return Err("stored accepted-record log contains a broken backlink".into());
        }
        let body_bytes = operation
            .body
            .as_ref()
            .map_or(0, |body| body.to_bytes().len());
        let record_bytes = encoded_header
            .len()
            .checked_add(body_bytes)
            .ok_or("stored suffix byte count overflow")?;
        used_bytes = used_bytes
            .checked_add(record_bytes)
            .ok_or("stored suffix byte count overflow")?;
        if used_bytes > max_wire_bytes {
            return Err("stored suffix exceeded its wire-byte limit".into());
        }
        let record = verify_host_record(&operation, host, session, limits)?;
        verify_parent_link(&record, previous_record.as_ref())?;
        previous_hash = Some(operation.hash);
        previous_record = Some(record.clone());
        records.push(record);
        expected_sequence += 1;
    }
    let through_sequence = records
        .last()
        .map(|record| u32::try_from(record.sequence - 1).expect("verified log sequence"))
        .or(after_sequence);
    let latest = store
        .get_latest_entry(host, &log_id)
        .await
        .map_err(|error| error.to_string())?;
    let has_more = latest.is_some_and(|latest| {
        through_sequence.is_none_or(|through| latest.header.seq_num > through)
    });
    if records.is_empty() && has_more {
        return Err("stored accepted-record log is missing the next sequence".into());
    }
    Ok(StoredSuffix {
        records,
        through_sequence,
        has_more,
    })
}
