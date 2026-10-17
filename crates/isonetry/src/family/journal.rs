use std::{cell::RefCell, collections::VecDeque};

use isocosm::Session;
use isocosm_host::{
    save::{FORMAT_VERSION as WORLD_FORMAT_VERSION, WorldSave},
    world::CommitGate,
};

use super::{
    AcceptedRecord, FAMILY_FORMAT_VERSION, SessionId, WireLimits, ledger::DurableState,
    section_diff,
};

/// Bounds both one serialized record and the total waiting host outbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FamilyLimits {
    pub wire: WireLimits,
    pub max_outbox_bytes: usize,
}

impl Default for FamilyLimits {
    fn default() -> Self {
        Self {
            wire: WireLimits::default(),
            max_outbox_bytes: 16 * 1_048_576,
        }
    }
}

struct JournalState {
    sequence: u64,
    head_digest: String,
    outbox: VecDeque<(AcceptedRecord, usize)>,
    outbox_bytes: usize,
    durable: Option<DurableState>,
}

/// Commit gate installed on the one shared core World.
pub(super) struct FamilyJournal {
    session: SessionId,
    limits: FamilyLimits,
    state: RefCell<JournalState>,
}

impl FamilyJournal {
    pub(super) fn new_at(
        session: SessionId,
        sequence: u64,
        head_digest: String,
        limits: FamilyLimits,
        durable: Option<DurableState>,
    ) -> Self {
        Self {
            session,
            limits,
            state: RefCell::new(JournalState {
                sequence,
                head_digest,
                outbox: VecDeque::new(),
                outbox_bytes: 0,
                durable,
            }),
        }
    }

    pub(super) fn sequence(&self) -> u64 {
        self.state.borrow().sequence
    }

    pub(super) fn head_digest(&self) -> String {
        self.state.borrow().head_digest.clone()
    }

    pub(super) fn outbox_bytes(&self) -> usize {
        self.state.borrow().outbox_bytes
    }

    pub(super) fn record(&self, sequence: u64) -> Option<AcceptedRecord> {
        self.state
            .borrow()
            .outbox
            .iter()
            .find(|(record, _)| record.sequence == sequence)
            .map(|(record, _)| record.clone())
    }

    pub(super) fn drain_records(&self) -> Vec<AcceptedRecord> {
        let mut state = self.state.borrow_mut();
        let drained = std::mem::take(&mut state.outbox);
        state.outbox_bytes = 0;
        drained.into_iter().map(|(record, _)| record).collect()
    }

    pub(super) fn attach_durable(&self, durable: DurableState) -> Result<(), String> {
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| "family journal is already publishing")?;
        let last = durable.snapshot.records.last();
        let sequence = last.map_or(durable.snapshot.checkpoint.sequence, |record| {
            record.sequence
        });
        let digest = last.map_or_else(
            || durable.snapshot.checkpoint.digest.clone(),
            |record| record.digest.clone(),
        );
        if sequence != state.sequence || digest != state.head_digest {
            return Err("family ledger does not match the resumed journal head".into());
        }
        if state.durable.is_some() {
            return Err("family journal already has a durable ledger".into());
        }
        state.durable = Some(durable);
        Ok(())
    }

    pub(super) fn persisted_suffix_after(
        &self,
        sequence: u64,
    ) -> Result<Vec<AcceptedRecord>, String> {
        self.state
            .borrow()
            .durable
            .as_ref()
            .ok_or("family host has no file ledger")?
            .suffix_after(sequence)
    }
}

impl CommitGate for FamilyJournal {
    fn before_publish(
        &self,
        before: &Session,
        before_sections: &std::collections::BTreeMap<String, Vec<u8>>,
        before_revision: u64,
        candidate: &Session,
        candidate_sections: &std::collections::BTreeMap<String, Vec<u8>>,
        candidate_revision: u64,
    ) -> Result<(), String> {
        let expected_revision = before_revision
            .checked_add(1)
            .ok_or("world revision overflow")?;
        if candidate_revision != expected_revision {
            return Err("family commit revision is not contiguous".into());
        }
        if candidate.branch != before.branch || candidate.sim.genesis() != before.sim.genesis() {
            return Err("family commit changes the native branch or genesis".into());
        }
        if !candidate.entries.starts_with(&before.entries) {
            return Err("family commit replaces or truncates native history".into());
        }

        let suffix = &candidate.entries[before.entries.len()..];
        let final_tick = candidate.sim.state().tick;
        let replayed = replay_suffix(before, suffix, final_tick)?;
        if replayed.save() != candidate.save() {
            return Err("family commit native suffix does not reproduce its candidate".into());
        }

        let before_save = WorldSave {
            format: WORLD_FORMAT_VERSION,
            native: before.save(),
            sections: before_sections.clone(),
        };
        let candidate_save = WorldSave {
            format: WORLD_FORMAT_VERSION,
            native: candidate.save(),
            sections: candidate_sections.clone(),
        };
        if before_save.native == candidate_save.native
            && before_save.sections == candidate_save.sections
        {
            return Err("empty family commit".into());
        }

        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| "family journal is already publishing")?;
        let sequence = state.sequence.checked_add(1).ok_or("sequence overflow")?;
        let record = AcceptedRecord {
            format: FAMILY_FORMAT_VERSION,
            session: self.session,
            sequence,
            parent_digest: state.head_digest.clone(),
            native_cut: candidate.entries.len(),
            final_tick,
            native_hash: candidate.sim.state_hash(),
            section_updates: section_diff(&before_save.sections, &candidate_save.sections),
            section_digest: isocosm::digest(&candidate_save.sections),
            revision: candidate_revision,
            native_suffix: suffix.to_vec(),
            digest: String::new(),
        }
        .seal();
        let encoded_len = record.encode_with_limits(self.limits.wire)?.len();
        let next_outbox_bytes = state
            .outbox_bytes
            .checked_add(encoded_len)
            .ok_or("family outbox size overflow")?;
        if next_outbox_bytes > self.limits.max_outbox_bytes {
            return Err("family accepted-record outbox is full".into());
        }

        if let Some(durable) = state.durable.as_mut() {
            durable.append(&record)?;
        }

        // Core publication is an infallible assignment after this callback.
        state.outbox.push_back((record.clone(), encoded_len));
        state.outbox_bytes = next_outbox_bytes;
        state.sequence = sequence;
        state.head_digest = record.digest;
        Ok(())
    }
}

fn replay_suffix(
    before: &Session,
    suffix: &[isocosm::history::Entry],
    final_tick: u64,
) -> Result<Session, String> {
    let mut replayed = before.clone();
    for expected in suffix {
        let ticks = expected
            .tick
            .checked_sub(replayed.sim.state().tick)
            .ok_or("native suffix entry moves time backwards")?;
        if ticks > 0 {
            replayed.advance(ticks)?;
        }
        replayed.command(expected.command.clone())?;
        if replayed.entries.last() != Some(expected) {
            return Err("native suffix entry differs from replay".into());
        }
    }
    let ticks = final_tick
        .checked_sub(replayed.sim.state().tick)
        .ok_or("native suffix final tick moves backwards")?;
    if ticks > 0 {
        replayed.advance(ticks)?;
    }
    Ok(replayed)
}
