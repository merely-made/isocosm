// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, schema::*, simulation::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Act {
        actor: Id,
        target: Option<Id>,
        process: Key,
        cause: Option<Key>,
    },
    Inspect(Id),
    Release(Id),
    Collect,
    Learn {
        entity: Id,
        event: Key,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: Key,
    pub tick: Tick,
    pub order: u64,
    pub command: Command,
    pub outcome: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub tick: Tick,
    pub state_hash: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved {
    pub version: u32,
    pub genesis: Genesis,
    pub genesis_digest: Key,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub tick: Tick,
    pub state_hash: Key,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub sim: Simulation,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub entry: Key,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug)]
pub struct Merge {
    pub proposed: Session,
    pub changes: Vec<Conflict>,
}

impl Session {
    pub fn new(genesis: Genesis, mode: Execution) -> Result<Self> {
        Ok(Self {
            sim: Simulation::new(genesis, mode)?,
            branch: "branch:trunk".into(),
            entries: vec![],
            checkpoints: vec![],
        })
    }
    pub fn advance(&mut self, ticks: Tick) -> Result<Work> {
        let end = self
            .sim
            .state
            .tick
            .checked_add(ticks)
            .ok_or("clock overflow")?;
        if ticks > self.sim.genesis.rules.limits.advance_ticks {
            return Err("advance budget".into());
        }
        let mut candidate = self.clone();
        let mut work = Work::default();
        let epoch = self.sim.genesis.rules.epoch_ticks;
        while candidate.sim.state.tick < end {
            let now = candidate.sim.state.tick;
            let boundary = now.checked_add(epoch - now % epoch).unwrap_or(end).min(end);
            let next = candidate.sim.advance(boundary - now)?;
            work.evaluations += next.evaluations;
            work.represented += next.represented;
            work.accepted += next.accepted;
            work.blocked += next.blocked;
            if work.evaluations > self.sim.genesis.rules.limits.events_per_advance as u64 {
                return Err("advance exceeds configured operation budget".into());
            }
            if boundary % epoch == 0 {
                candidate.checkpoints.push(Checkpoint {
                    tick: boundary,
                    state_hash: candidate.sim.state_hash(),
                });
            }
        }
        *self = candidate;
        Ok(work)
    }
    fn replay_until(&mut self, tick: Tick) -> Result<()> {
        while self.sim.state.tick < tick {
            let mut step = (tick - self.sim.state.tick)
                .min(self.sim.genesis.rules.limits.advance_ticks)
                .min(self.sim.genesis.rules.epoch_ticks);
            loop {
                match self.advance(step) {
                    Ok(_) => break,
                    Err(why) if step > 1 && why.contains("operation budget") => {
                        step = step.div_ceil(2)
                    },
                    Err(why) => return Err(why),
                }
            }
        }
        Ok(())
    }
    pub fn command(&mut self, command: Command) -> Result<String> {
        if self.entries.len() >= self.sim.genesis.rules.limits.history {
            return Err("command log limit".into());
        }
        let outcome = run(&mut self.sim, &command)?;
        let order = self.entries.len() as u64;
        let id = format!(
            "intent:{}",
            crate::digest(&(&self.branch, self.sim.state.tick, order, &command))
        );
        self.entries.push(Entry {
            id,
            tick: self.sim.state.tick,
            order,
            command,
            outcome: outcome.clone(),
        });
        Ok(outcome)
    }
    pub fn save(&self) -> Saved {
        Saved {
            version: crate::VERSION,
            genesis: self.sim.genesis.as_ref().clone(),
            genesis_digest: crate::digest(&self.sim.genesis),
            branch: self.branch.clone(),
            entries: self.entries.clone(),
            tick: self.sim.state.tick,
            state_hash: self.sim.state_hash(),
            checkpoints: self.checkpoints.clone(),
        }
    }
    pub fn load(saved: Saved, mode: Execution) -> Result<Self> {
        if saved.version != crate::VERSION || crate::digest(&saved.genesis) != saved.genesis_digest
        {
            return Err("save version or genesis digest mismatch".into());
        }
        let mut session = Self::new(saved.genesis, mode)?;
        session.branch = saved.branch;
        let mut ids = std::collections::BTreeSet::new();
        for entry in saved.entries {
            if !ids.insert(entry.id.clone())
                || entry.tick < session.sim.state.tick
                || entry.tick > saved.tick
            {
                return Err("invalid intent order or identity".into());
            }
            session.replay_until(entry.tick)?;
            let outcome = run(&mut session.sim, &entry.command)?;
            if outcome != entry.outcome {
                return Err(format!("replay outcome mismatch for {}", entry.id));
            }
            session.entries.push(entry);
        }
        session.replay_until(saved.tick)?;
        if session.sim.state_hash() != saved.state_hash {
            return Err("save state hash mismatch".into());
        }
        if session.checkpoints != saved.checkpoints {
            return Err("checkpoint history mismatch".into());
        }
        Ok(session)
    }
    pub fn fork_at(&self, tick: Tick, branch: Key) -> Result<Self> {
        if tick > self.sim.state.tick {
            return Err("cannot fork an unplayed future".into());
        }
        let mut fork = Self::new(self.sim.genesis.as_ref().clone(), self.sim.mode)?;
        fork.branch = branch;
        for entry in self.entries.iter().filter(|e| e.tick <= tick) {
            fork.replay_until(entry.tick)?;
            let outcome = run(&mut fork.sim, &entry.command)?;
            if outcome != entry.outcome {
                return Err("source replay changed".into());
            }
            fork.entries.push(entry.clone());
        }
        fork.replay_until(tick)?;
        Ok(fork)
    }
    /// A proposal: never mutates either played branch. Includes changed accepted
    /// receipts as well as refusals, because legal replay can change consequences.
    pub fn merge(&self, other: &Self) -> Result<Merge> {
        if self.sim.state.tick != other.sim.state.tick {
            return Err("different spans require realignment".into());
        }
        if crate::digest(&self.sim.genesis) != crate::digest(&other.sim.genesis) {
            return Err("worlds have different founding facts".into());
        }
        let mut union = BTreeMap::new();
        for entry in self.entries.iter().chain(&other.entries) {
            if let Some(previous) = union.insert(entry.id.clone(), entry.clone())
                && previous != *entry
            {
                return Err("intent identity has conflicting contents".into());
            }
        }
        let mut entries: Vec<_> = union.into_values().collect();
        entries.sort_by(|a, b| (a.tick, a.order, &a.id).cmp(&(b.tick, b.order, &b.id)));
        let mut proposed = Self::new(self.sim.genesis.as_ref().clone(), self.sim.mode)?;
        proposed.branch = format!("branch:{}", crate::digest(&entries));
        let mut changes = Vec::new();
        for mut entry in entries {
            proposed.replay_until(entry.tick)?;
            let outcome = run(&mut proposed.sim, &entry.command)?;
            if outcome != entry.outcome {
                changes.push(Conflict {
                    entry: entry.id.clone(),
                    before: entry.outcome.clone(),
                    after: outcome.clone(),
                });
            }
            entry.outcome = outcome;
            proposed.entries.push(entry);
        }
        proposed.replay_until(self.sim.state.tick)?;
        Ok(Merge { proposed, changes })
    }
}

fn run(sim: &mut Simulation, command: &Command) -> Result<String> {
    match command {
        Command::Act {
            actor,
            target,
            process,
            cause,
        } => {
            let receipt = sim.execute(*actor, *target, process, cause.clone());
            serde_json::to_string(&receipt).map_err(|e| e.to_string())
        },
        Command::Inspect(id) => {
            sim.inspect(*id)?;
            Ok("inspected".into())
        },
        Command::Release(id) => {
            sim.release(*id);
            Ok("released".into())
        },
        Command::Collect => {
            sim.collect();
            Ok("collected".into())
        },
        Command::Learn { entity, event } => Ok(sim.learn(*entity, event)?.to_string()),
    }
}
