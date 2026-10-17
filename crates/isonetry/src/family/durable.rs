use std::path::Path;

use isocosm::Execution;
use isocosm_host::world::World;

use super::{
    FamilyHost, FamilyLedgerLimits, FamilyLimits, SessionId, checkpoint,
    ledger::{DurableState, FileLedger, LedgerSnapshot},
};

impl FamilyHost {
    /// Create a host whose checkpoint and accepted records are kept in one
    /// bounded, atomically replaced file. The staged file is synced before
    /// replacement; the parent directory is not synced, so this does not claim
    /// power-loss durability. Existing paths are never reset.
    pub fn create_file(
        path: impl AsRef<Path>,
        session: SessionId,
        world: World,
        limits: FamilyLimits,
        ledger_limits: FamilyLedgerLimits,
    ) -> Result<Self, String> {
        let world = world.into_exclusive()?;
        let snapshot = world.replication_snapshot()?;
        let checkpoint = checkpoint(session, 0, String::new(), snapshot).seal();
        let storage = FileLedger::create(path, checkpoint.clone(), ledger_limits)?;
        let durable = DurableState {
            storage,
            snapshot: LedgerSnapshot {
                checkpoint: checkpoint.clone(),
                records: Vec::new(),
            },
        };
        Self::attach_journal(session, world, 0, checkpoint.digest, limits, Some(durable))
    }

    /// Reopen the sole writer from its bounded file ledger. The stored suffix
    /// is verified and replayed before the host World becomes available.
    pub fn resume_file(
        path: impl AsRef<Path>,
        execution: Execution,
        limits: FamilyLimits,
        ledger_limits: FamilyLedgerLimits,
    ) -> Result<Self, String> {
        let (storage, snapshot) = FileLedger::open(path, ledger_limits)?;
        let host = Self::resume(
            snapshot.checkpoint.clone(),
            &snapshot.records,
            execution,
            limits,
        )?;
        host.journal
            .attach_durable(DurableState { storage, snapshot })?;
        Ok(host)
    }
}
