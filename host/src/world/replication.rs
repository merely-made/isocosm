// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Atomic installation of host-accepted authority. The session layer owns
//! peer admission, record ordering and authentication, outside this core.

use super::*;
use isocosm::history::Entry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationSnapshot {
    pub save: WorldSave,
    pub revision: u64,
}

impl World {
    pub fn replication_snapshot(&self) -> Result<ReplicationSnapshot, String> {
        let world = self.0.try_borrow().map_err(|_| "world is being advanced")?;
        Ok(ReplicationSnapshot {
            save: WorldSave { format: FORMAT_VERSION, native: world.session.save(),
                sections: world.sections.clone() },
            revision: world.revision,
        })
    }

    /// Unlike ordinary disk resume, an admitted replication checkpoint keeps
    /// the host's revision. Session identity is checked by the session layer.
    pub fn resume_replica(snapshot: ReplicationSnapshot, execution: Execution) -> Result<Self, String> {
        let world = Self::resume(snapshot.save, execution)?;
        world.0.borrow_mut().revision = snapshot.revision;
        Ok(world)
    }

    /// Replay accepted native entries, time and opaque updates on one candidate.
    /// A caller verifies its authenticated record before anything is published.
    pub fn apply_replica_record(
        &self, expected_revision: u64, expected_native_cut: usize,
        native_suffix: &[Entry], final_tick: u64,
        section_updates: &BTreeMap<String, Option<Vec<u8>>>,
        verify: impl FnOnce(&Session, &BTreeMap<String, Vec<u8>>, u64) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut world = self.0.try_borrow_mut().map_err(|_| "world is being read")?;
        if world.commit_gate.is_some() {
            return Err("hosted world refuses replica replay mutation".into());
        }
        if world.revision != expected_revision || world.session.entries.len() != expected_native_cut {
            return Err("replica predecessor is stale".into());
        }
        let revision = world.revision.checked_add(1).ok_or("world revision overflow")?;
        let mut candidate = world.session.clone();
        let mut sections = world.sections.clone();
        for entry in native_suffix {
            let ticks = entry.tick.checked_sub(candidate.sim.state().tick)
                .ok_or("replica entry moves backwards")?;
            if ticks > 0 { candidate.advance(ticks)?; }
            candidate.command(entry.command.clone())?;
            if candidate.entries.last() != Some(entry) {
                return Err("replica native entry diverges".into());
            }
        }
        let ticks = final_tick.checked_sub(candidate.sim.state().tick)
            .ok_or("replica clock moves backwards")?;
        if ticks > 0 { candidate.advance(ticks)?; }
        for (key, update) in section_updates {
            match update {
                Some(bytes) => { sections.insert(key.clone(), bytes.clone()); }
                None => { sections.remove(key); }
            }
        }
        if same_authority(&candidate, &world.session) && sections == world.sections {
            return Err("empty replica record".into());
        }
        verify(&candidate, &sections, revision)?;
        world.session = candidate;
        world.sections = sections;
        world.revision = revision;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
