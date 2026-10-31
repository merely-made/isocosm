use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{AcceptedRecord, Checkpoint};

const LEDGER_FORMAT: u16 = 1;
pub const DEFAULT_MAX_LEDGER_BYTES: usize = 64 * 1_048_576;

/// Bound the complete checkpoint and accepted-record history kept by a file
/// host. Reaching the cap refuses the next World publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyLedgerLimits {
    pub max_bytes: usize,
}

impl Default for FamilyLedgerLimits {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_LEDGER_BYTES,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EncodedLedger {
    format: u16,
    checkpoint: Checkpoint,
    records: Vec<AcceptedRecord>,
}

#[derive(Clone, Debug)]
pub(super) struct LedgerSnapshot {
    pub checkpoint: Checkpoint,
    pub records: Vec<AcceptedRecord>,
}

/// One atomically replaced ledger file and its process-lifetime writer lock.
pub(super) struct FileLedger {
    path: PathBuf,
    _lock: File,
    limits: FamilyLedgerLimits,
}

impl FileLedger {
    pub(super) fn create(
        path: impl AsRef<Path>,
        checkpoint: Checkpoint,
        limits: FamilyLedgerLimits,
    ) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let ledger = Self::lock(path, limits)?;
        if ledger.path.exists() {
            return Err("family ledger already exists; resume it instead".into());
        }
        checkpoint.validate()?;
        ledger.write(&LedgerSnapshot {
            checkpoint,
            records: Vec::new(),
        })?;
        Ok(ledger)
    }

    pub(super) fn open(
        path: impl AsRef<Path>,
        limits: FamilyLedgerLimits,
    ) -> Result<(Self, LedgerSnapshot), String> {
        let ledger = Self::lock(path.as_ref().to_path_buf(), limits)?;
        let bytes = ledger.read_bounded()?;
        let encoded: EncodedLedger =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if encoded.format != LEDGER_FORMAT {
            return Err("unsupported family ledger format".into());
        }
        if encoded.checkpoint.sequence != 0 || !encoded.checkpoint.parent_digest.is_empty() {
            return Err("family ledger anchor is not its initial checkpoint".into());
        }
        encoded.checkpoint.validate()?;
        Ok((
            ledger,
            LedgerSnapshot {
                checkpoint: encoded.checkpoint,
                records: encoded.records,
            },
        ))
    }

    fn lock(path: PathBuf, limits: FamilyLedgerLimits) -> Result<Self, String> {
        if limits.max_bytes == 0 {
            return Err("family ledger byte limit must be nonzero".into());
        }
        let mut lock_name = path
            .file_name()
            .ok_or("family ledger path has no file name")?
            .to_os_string();
        lock_name.push(".family-lock");
        let lock_path = path.with_file_name(lock_name);
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|error| error.to_string())?;
        lock.try_lock()
            .map_err(|error| format!("family ledger already has a writer: {error}"))?;
        Ok(Self {
            path,
            _lock: lock,
            limits,
        })
    }

    fn read_bounded(&self) -> Result<Vec<u8>, String> {
        let file = File::open(&self.path).map_err(|error| error.to_string())?;
        let max_bytes = u64::try_from(self.limits.max_bytes)
            .map_err(|_| "family ledger byte limit cannot be represented")?;
        if file.metadata().map_err(|error| error.to_string())?.len() > max_bytes {
            return Err("family ledger exceeds configured byte limit".into());
        }
        let mut bytes = Vec::new();
        file.take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() > self.limits.max_bytes {
            return Err("family ledger exceeds configured byte limit".into());
        }
        Ok(bytes)
    }

    pub(super) fn write(&self, snapshot: &LedgerSnapshot) -> Result<(), String> {
        let bytes = serde_json::to_vec(&EncodedLedger {
            format: LEDGER_FORMAT,
            checkpoint: snapshot.checkpoint.clone(),
            records: snapshot.records.clone(),
        })
        .map_err(|error| error.to_string())?;
        if bytes.len() > self.limits.max_bytes {
            return Err("family ledger reached configured byte limit".into());
        }
        cosmograph_host::file_save::replace(&self.path, &bytes)
            .map_err(|error| format!("family ledger replacement failed: {error}"))
    }
}

pub(super) struct DurableState {
    pub storage: FileLedger,
    pub snapshot: LedgerSnapshot,
}

impl DurableState {
    pub(super) fn append(&mut self, record: &AcceptedRecord) -> Result<(), String> {
        let mut next = self.snapshot.clone();
        next.records.push(record.clone());
        self.storage.write(&next)?;
        self.snapshot = next;
        Ok(())
    }

    pub(super) fn suffix_after(&self, sequence: u64) -> Result<Vec<AcceptedRecord>, String> {
        let latest = self
            .snapshot
            .records
            .last()
            .map_or(self.snapshot.checkpoint.sequence, |record| record.sequence);
        if sequence < self.snapshot.checkpoint.sequence || sequence > latest {
            return Err("family ledger cursor is outside its retained sequence".into());
        }
        Ok(self
            .snapshot
            .records
            .iter()
            .filter(|record| record.sequence > sequence)
            .cloned()
            .collect())
    }
}
