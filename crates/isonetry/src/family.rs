//! One host's accepted native log and opaque section updates for the three
//! wing's products. Requests are resolved at the host; replicas apply the
//! recorded native commands and never run a product adapter to decide again.
//!
//! The core owns one accepted writer and replicas replay its native records
//! without repeating product admission. With `family-net`, records are signed
//! by the pinned host and travel over Stickleback; a bounded signed checkpoint
//! can bootstrap a fresh peer through Graphshell's admitted resource stream.
//! These mechanisms preserve a single-host authority model. They do not yet
//! make the world and operation store crash-atomic.

use std::collections::BTreeMap;
use std::rc::Rc;

use cosmograph::{Command, Execution, Session, history::Entry};
use cosmograph_host::{
    save::WorldSave,
    world::{Cut, ReplicationSnapshot, World},
};
use serde::{Deserialize, Serialize};

type Digest = String;

mod journal;
use journal::FamilyJournal;
pub use journal::FamilyLimits;
mod ledger;
use ledger::DurableState;
pub use ledger::{DEFAULT_MAX_LEDGER_BYTES, FamilyLedgerLimits};
mod durable;

/// Version of the family checkpoint and accepted-record envelope.
pub const FAMILY_FORMAT_VERSION: u16 = 1;

/// Default maximum encoded checkpoint or record size, overridable per link.
pub const DEFAULT_MAX_WIRE_BYTES: usize = 1_048_576;

/// A caller's cap for decoding or encoding one family record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireLimits {
    pub max_bytes: usize,
}

impl Default for WireLimits {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_WIRE_BYTES,
        }
    }
}

/// Caller-assigned identity for one hosted world session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub [u8; 16]);

/// A complete checkpoint. Unknown section keys remain uninterpreted bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub format: u16,
    pub session: SessionId,
    pub sequence: u64,
    pub parent_digest: Digest,
    pub native_cut: usize,
    pub native_hash: u64,
    pub final_tick: u64,
    pub section_digest: Digest,
    pub revision: u64,
    pub world: WorldSave,
    pub digest: Digest,
}

impl Checkpoint {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_with_limits(WireLimits::default())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        Self::decode_with_limits(bytes, WireLimits::default())
    }

    pub fn encode_with_limits(&self, limits: WireLimits) -> Result<Vec<u8>, String> {
        encode_bounded(self, limits)
    }

    pub fn decode_with_limits(bytes: &[u8], limits: WireLimits) -> Result<Self, String> {
        decode_bounded(bytes, limits)
    }

    fn seal(mut self) -> Self {
        self.digest.clear();
        self.digest = cosmograph::digest(&self);
        self
    }

    fn authentic(&self) -> bool {
        let mut unsigned = self.clone();
        let digest = std::mem::take(&mut unsigned.digest);
        !digest.is_empty() && digest == cosmograph::digest(&unsigned)
    }

    /// Check the digest and declared native/section cut before trusting this
    /// checkpoint as a replica starting point.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != FAMILY_FORMAT_VERSION {
            return Err("unsupported family checkpoint format".into());
        }
        if !self.authentic() {
            return Err("checkpoint digest mismatch".into());
        }
        verify_checkpoint(self)
    }
}

/// A single host-accepted transition. `native_cut` is the resulting entry
/// count; `native_suffix` begins at the receiver's preceding cut. A `None`
/// section value explicitly deletes that key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedRecord {
    pub format: u16,
    pub session: SessionId,
    pub sequence: u64,
    pub parent_digest: Digest,
    pub native_cut: usize,
    pub final_tick: u64,
    pub native_hash: u64,
    pub section_updates: BTreeMap<String, Option<Vec<u8>>>,
    pub section_digest: Digest,
    pub revision: u64,
    pub native_suffix: Vec<Entry>,
    pub digest: Digest,
}

impl AcceptedRecord {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_with_limits(WireLimits::default())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        Self::decode_with_limits(bytes, WireLimits::default())
    }

    pub fn encode_with_limits(&self, limits: WireLimits) -> Result<Vec<u8>, String> {
        encode_bounded(self, limits)
    }

    pub fn decode_with_limits(bytes: &[u8], limits: WireLimits) -> Result<Self, String> {
        decode_bounded(bytes, limits)
    }

    fn seal(mut self) -> Self {
        self.digest.clear();
        self.digest = cosmograph::digest(&self);
        self
    }

    fn authentic(&self) -> bool {
        let mut unsigned = self.clone();
        let digest = std::mem::take(&mut unsigned.digest);
        !digest.is_empty() && digest == cosmograph::digest(&unsigned)
    }
}

fn encode_bounded(value: &impl Serialize, limits: WireLimits) -> Result<Vec<u8>, String> {
    // Native saves use serde defaults and omitted fields. JSON's
    // self-describing objects preserve those semantics during schema evolution;
    // postcard would decode omitted struct fields positionally and can corrupt
    // a checkpoint before its digest check.
    let bytes = serde_json::to_vec(value).map_err(|why| why.to_string())?;
    if bytes.len() > limits.max_bytes {
        return Err("family record exceeds configured wire limit".into());
    }
    Ok(bytes)
}

fn decode_bounded<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    limits: WireLimits,
) -> Result<T, String> {
    if bytes.len() > limits.max_bytes {
        return Err("family record exceeds configured wire limit".into());
    }
    serde_json::from_slice(bytes).map_err(|why| why.to_string())
}

/// One read-only adapter decision. The family host applies these commands and
/// byte updates inside the core World's atomic product transaction.
pub struct HostAdmission<T> {
    pub value: T,
    pub commands: Vec<Command>,
    /// Advances accepted native time after the commands, if present.
    pub final_tick: Option<u64>,
    /// `None` removes the section key; `Some` replaces it with these bytes.
    pub section_updates: BTreeMap<String, Option<Vec<u8>>>,
}

/// The single accepted writer. Its commit gate journals every core World
/// publication, including writes made through endpoint World handles.
pub struct FamilyHost {
    session: SessionId,
    world: World,
    journal: Rc<FamilyJournal>,
}

impl FamilyHost {
    pub fn new(session: SessionId, world: World) -> Result<Self, String> {
        Self::new_with_limits(session, world, FamilyLimits::default())
    }

    pub fn new_with_limits(
        session: SessionId,
        world: World,
        limits: FamilyLimits,
    ) -> Result<Self, String> {
        let world = world.into_exclusive()?;
        let snapshot = world.replication_snapshot()?;
        let checkpoint = checkpoint(session, 0, String::new(), snapshot).seal();
        Self::attach_journal(session, world, 0, checkpoint.digest, limits, None)
    }

    fn attach_journal(
        session: SessionId,
        world: World,
        sequence: u64,
        head_digest: Digest,
        limits: FamilyLimits,
        durable: Option<DurableState>,
    ) -> Result<Self, String> {
        let world = world.into_exclusive()?;
        let journal = Rc::new(FamilyJournal::new_at(
            session,
            sequence,
            head_digest,
            limits,
            durable,
        ));
        world.install_commit_gate(journal.clone())?;
        Ok(Self {
            session,
            world,
            journal,
        })
    }

    /// Reopen the sole host from a verified checkpoint and accepted suffix.
    /// Authentication of those records belongs to the transport layer.
    pub fn resume(
        checkpoint: Checkpoint,
        suffix: &[AcceptedRecord],
        execution: Execution,
        limits: FamilyLimits,
    ) -> Result<Self, String> {
        let replica = FamilyReplica::install(checkpoint, suffix, execution)?;
        let FamilyReplica {
            session,
            world,
            sequence,
            head_digest,
        } = replica;
        Self::attach_journal(session, world, sequence, head_digest, limits, None)
    }

    /// Shared handle used to open existing product endpoints. Their World
    /// transactions still publish through the installed commit gate.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Reads the native session and opaque sections together at one world cut.
    pub fn read<T>(
        &self,
        read: impl FnOnce(&Session, &BTreeMap<String, Vec<u8>>) -> T,
    ) -> Result<T, String> {
        self.world.read_sections(read)
    }

    pub fn revision(&self) -> Result<u64, String> {
        self.world.revision()
    }

    pub fn sequence(&self) -> u64 {
        self.journal.sequence()
    }

    pub fn outbox_bytes(&self) -> usize {
        self.journal.outbox_bytes()
    }

    /// Remove accepted records ready for transport, freeing the bounded
    /// local outbox. The operation log remains the durable external layer.
    pub fn drain_records(&self) -> Vec<AcceptedRecord> {
        self.journal.drain_records()
    }

    /// Return persisted accepted records after a transport cursor so a
    /// restarted host can republish its verified suffix.
    pub fn persisted_suffix_after(&self, sequence: u64) -> Result<Vec<AcceptedRecord>, String> {
        self.journal.persisted_suffix_after(sequence)
    }

    /// Current join point for a new or reconnecting replica.
    pub fn checkpoint(&mut self) -> Result<Checkpoint, String> {
        let snapshot = self.world.replication_snapshot()?;
        let sequence = self.journal.sequence();
        let parent_digest = if sequence == 0 {
            String::new()
        } else {
            self.journal.head_digest()
        };
        let checkpoint = checkpoint(self.session, sequence, parent_digest, snapshot).seal();
        Ok(checkpoint)
    }

    /// Runs host admission and publishes its native log suffix and section
    /// changes as one record. Returning an error leaves the world and sequence
    /// untouched; identical resulting bytes return no record.
    pub fn accept<T>(
        &mut self,
        admit: impl FnOnce(&Session, &BTreeMap<String, Vec<u8>>) -> Result<HostAdmission<T>, String>,
    ) -> Result<(T, Vec<String>, Option<AcceptedRecord>), String> {
        let before = self.world.replication_snapshot()?;
        let before_cut = cut(&before.save);
        let previous_sequence = self.journal.sequence();
        let (value, outcomes) = self
            .world
            .product_sections(before_cut, |session, sections| {
                let plan = admit(session, sections)?;
                let mut outcomes = Vec::with_capacity(plan.commands.len());
                for command in plan.commands {
                    outcomes.push(session.command(command)?);
                }
                if let Some(tick) = plan.final_tick {
                    let ticks = tick
                        .checked_sub(session.sim.state().tick)
                        .ok_or("host admission moves time backwards")?;
                    if ticks > 0 {
                        session.advance(ticks)?;
                    }
                }
                let mut updated = sections.clone();
                for (key, bytes) in plan.section_updates {
                    match bytes {
                        Some(bytes) => {
                            updated.insert(key, bytes);
                        },
                        None => {
                            updated.remove(&key);
                        },
                    }
                }
                Ok(((plan.value, outcomes), updated))
            })?;
        let after = self.world.replication_snapshot()?;
        if after.save.native == before.save.native && after.save.sections == before.save.sections {
            return Ok((value, outcomes, None));
        }
        let sequence = self.journal.sequence();
        if sequence
            != previous_sequence
                .checked_add(1)
                .ok_or("sequence overflow")?
        {
            return Err("world commit was not journaled".into());
        }
        let record = self
            .journal
            .record(sequence)
            .ok_or("accepted record left the bounded outbox during admission")?;
        Ok((value, outcomes, Some(record)))
    }
}

/// A replica accepts only a contiguous, digest-linked stream. Checkpoint plus
/// suffix installation is staged in a private `World` and exposed only after
/// every record verifies.
pub struct FamilyReplica {
    session: SessionId,
    world: World,
    sequence: u64,
    head_digest: Digest,
}

impl FamilyReplica {
    pub fn install(
        checkpoint: Checkpoint,
        suffix: &[AcceptedRecord],
        execution: Execution,
    ) -> Result<Self, String> {
        checkpoint.validate()?;
        let head_digest = if checkpoint.sequence == 0 {
            checkpoint.digest.clone()
        } else {
            checkpoint.parent_digest.clone()
        };
        let world = World::resume_replica(
            ReplicationSnapshot {
                save: checkpoint.world.clone(),
                revision: checkpoint.revision,
            },
            execution,
        )?;
        let mut replica = Self {
            session: checkpoint.session,
            world,
            sequence: checkpoint.sequence,
            head_digest,
        };
        for record in suffix {
            replica.apply(record.clone())?;
        }
        Ok(replica)
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn read<T>(
        &self,
        read: impl FnOnce(&Session, &BTreeMap<String, Vec<u8>>) -> T,
    ) -> Result<T, String> {
        self.world.read_sections(read)
    }

    pub fn snapshot(&self) -> Result<ReplicationSnapshot, String> {
        self.world.replication_snapshot()
    }

    /// Applies one serialized host record atomically. A record is never
    /// partially visible if native replay or a section check fails.
    pub fn apply(&mut self, record: AcceptedRecord) -> Result<(), String> {
        if record.format != FAMILY_FORMAT_VERSION {
            return Err("unsupported family record format".into());
        }
        if !record.authentic() {
            return Err("accepted record digest mismatch".into());
        }
        if record.session != self.session {
            return Err("record belongs to another session".into());
        }
        let next_sequence = self.sequence.checked_add(1).ok_or("sequence overflow")?;
        if record.sequence != next_sequence {
            return Err(if record.sequence <= self.sequence {
                "duplicate or stale record".into()
            } else {
                "accepted record gap or reordering".into()
            });
        }
        if record.parent_digest != self.head_digest {
            return Err("accepted record parent digest mismatch".into());
        }
        let before = self.world.replication_snapshot()?;
        let expected_cut = before.save.native.entries.len();
        if record.native_cut
            != expected_cut
                .checked_add(record.native_suffix.len())
                .ok_or("native cut overflow")?
        {
            return Err("accepted record native cut mismatch".into());
        }
        if record.revision
            != before
                .revision
                .checked_add(1)
                .ok_or("world revision overflow")?
        {
            return Err("accepted record revision is not contiguous".into());
        }
        let verify = |native: &Session, sections: &BTreeMap<String, Vec<u8>>, revision| {
            if native.entries.len() != record.native_cut
                || native.sim.state().tick != record.final_tick
                || native.sim.state_hash() != record.native_hash
                || cosmograph::digest(sections) != record.section_digest
                || revision != record.revision
            {
                return Err("accepted record postcondition mismatch".into());
            }
            Ok(())
        };
        self.world.apply_replica_record(
            before.revision,
            expected_cut,
            &record.native_suffix,
            record.final_tick,
            &record.section_updates,
            verify,
        )?;
        self.sequence = next_sequence;
        self.head_digest = record.digest;
        Ok(())
    }
}

fn checkpoint(
    session: SessionId,
    sequence: u64,
    parent_digest: Digest,
    snapshot: ReplicationSnapshot,
) -> Checkpoint {
    Checkpoint {
        format: FAMILY_FORMAT_VERSION,
        session,
        sequence,
        parent_digest,
        native_cut: snapshot.save.native.entries.len(),
        native_hash: snapshot.save.native.state_hash,
        final_tick: snapshot.save.native.tick,
        section_digest: cosmograph::digest(&snapshot.save.sections),
        revision: snapshot.revision,
        world: snapshot.save,
        digest: String::new(),
    }
}

fn verify_checkpoint(checkpoint: &Checkpoint) -> Result<(), String> {
    if checkpoint.format != FAMILY_FORMAT_VERSION {
        return Err("unsupported family checkpoint format".into());
    }
    if (checkpoint.sequence == 0 && !checkpoint.parent_digest.is_empty())
        || (checkpoint.sequence > 0 && checkpoint.parent_digest.is_empty())
    {
        return Err("checkpoint chain anchor is invalid".into());
    }
    if checkpoint.world.native.entries.len() != checkpoint.native_cut
        || checkpoint.world.native.state_hash != checkpoint.native_hash
        || checkpoint.world.native.tick != checkpoint.final_tick
        || cosmograph::digest(&checkpoint.world.sections) != checkpoint.section_digest
    {
        return Err("checkpoint contents do not match their declared cut".into());
    }
    Ok(())
}

fn cut(save: &WorldSave) -> Cut {
    Cut {
        tick: save.native.tick,
        entries: save.native.entries.len(),
        state_hash: save.native.state_hash,
    }
}

fn section_diff(
    before: &BTreeMap<String, Vec<u8>>,
    after: &BTreeMap<String, Vec<u8>>,
) -> BTreeMap<String, Option<Vec<u8>>> {
    let keys: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
    keys.into_iter()
        .filter_map(|key| match after.get(key) {
            Some(bytes) if before.get(key) != Some(bytes) => {
                Some((key.clone(), Some(bytes.clone())))
            },
            None if before.contains_key(key) => Some((key.clone(), None)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[path = "family/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "family/durable_tests.rs"]
mod durable_tests;
