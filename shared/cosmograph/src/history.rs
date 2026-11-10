// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, schema::*, simulation::*};
use serde::{Deserialize, Serialize};
use state_witness::{Witness, first_divergence};
use std::collections::BTreeMap;

mod read;
mod movement;
mod time; pub use time::LifeHistory;
pub use read::{CheckpointV1, CheckpointV2, SavedV1, SavedV2, LegacyCohort, LegacyEntity, LegacyGenesis, LegacyPart, LegacyPopulation};

mod command;
mod dispatch;
pub use command::Command;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: Key,
    pub tick: Tick,
    pub order: u64,
    pub command: Command,
    pub outcome: String,
}

/// Saves have their own version: `crate::VERSION` also gates every genesis.
/// v5 records how the world's life history was founded (case 1073).
pub const SAVE_VERSION: u32 = 5;

/// An epoch boundary's per-field entries (rulings 633 and 651).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub tick: Tick,
    pub witness: Witness,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved {
    pub version: u32,
    /// v4 records the exact movement semantics; absent only in genuine v3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement: Option<Movement>,
    #[serde(default, skip_serializing_if = "Option::is_none")] // v5 on (1073)
    pub life_history: Option<LifeHistory>,
    pub genesis: Genesis,
    pub genesis_digest: Key,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub tick: Tick,
    /// The final entries' digest (ruling 652).
    pub state_hash: u64,
    pub witness: Witness,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub sim: Simulation,
    pub branch: Key,
    pub entries: Vec<Entry>,
    pub checkpoints: Vec<Checkpoint>,
    pub life_history: LifeHistory, // kept when it saves again (1073)
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
            life_history: LifeHistory::WorldTime,
        })
    }
    pub fn advance(&mut self, ticks: Tick) -> Result<Work> {
        let end = self
            .sim
            .state
            .tick
            .checked_add(ticks)
            .ok_or("clock overflow")?;
        // The world keeps what the advance changes and puts it back if any
        // part is refused, as a copy of the whole session once did.
        let checkpoints = self.checkpoints.len();
        let outer = self.sim.begin();
        let work = self.epochs(end);
        self.sim.finish(outer, work.is_ok());
        if work.is_err() {
            self.checkpoints.truncate(checkpoints);
        }
        work
    }
    /// Advances one tick at a time, handing `each` the world after each: the
    /// per-tick traces the tools write on demand (rulings 610 and 653).
    pub fn advance_traced(
        &mut self,
        ticks: Tick,
        mut each: impl FnMut(&Simulation),
    ) -> Result<Work> {
        let mut work = Work::default();
        for _ in 0..ticks {
            let next = self.advance(1)?;
            work.evaluations += next.evaluations;
            work.represented += next.represented;
            work.arrivals += next.arrivals;
            work.accepted += next.accepted;
            work.blocked += next.blocked;
            each(&self.sim);
        }
        Ok(work)
    }
    /// Advances to `end` epoch by epoch, checkpointing each boundary; a rule
    /// that ends epochs otherwise runs one unbounded epoch (ruling 451).
    fn epochs(&mut self, end: Tick) -> Result<Work> {
        let mut work = Work::default();
        let budget = self.sim.genesis.rules.epoch_budget();
        while self.sim.state.tick < end {
            let now = self.sim.state.tick;
            let boundary = budget
                .and_then(|epoch| now.checked_add(epoch - now % epoch))
                .unwrap_or(end)
                .min(end);
            let next = self.sim.advance(boundary - now)?;
            work.evaluations += next.evaluations;
            work.represented += next.represented;
            work.arrivals += next.arrivals;
            work.accepted += next.accepted;
            work.blocked += next.blocked;
            // The budget counts members (ruling 285), the same in both modes.
            if work.represented > self.sim.genesis.rules.limits.events_per_advance as u64 {
                return Err("advance exceeds configured operation budget".into());
            }
            if budget.is_some_and(|epoch| boundary.is_multiple_of(epoch)) {
                self.checkpoints.push(Checkpoint {
                    tick: boundary,
                    witness: self.sim.witness(),
                });
            }
        }
        Ok(work)
    }
    fn replay_until(&mut self, tick: Tick) -> Result<()> {
        while self.sim.state.tick < tick {
            let budget = self.sim.genesis.rules.epoch_budget();
            let mut step = (tick - self.sim.state.tick).min(budget.unwrap_or(Tick::MAX));
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
        let tick = self.sim.state.tick;
        let outcome = self.recorded(&command)?;
        let order = self.entries.len() as u64;
        let id = format!(
            "intent:{}",
            crate::digest(&(&self.branch, tick, order, &command))
        );
        self.entries.push(Entry {
            id,
            tick,
            order,
            command,
            outcome: outcome.clone(),
        });
        Ok(outcome)
    }
    fn recorded(&mut self, command: &Command) -> Result<String> {
        match command {
            Command::Owned { command, selections } => crate::society().and_then(|()| self.owned_command(command,selections)),
            Command::AdvanceGeometry { ticks, resolutions } => self.advance_geometry(*ticks, resolutions),
            Command::AdvanceSpatial { ticks, terrain, steps, arrivals } =>
                self.advance_spatial(*ticks, terrain, steps, arrivals),
            Command::AdvanceBirthSpatial { ticks, terrain, steps, arrivals, births, handbacks } =>
                self.advance_birth_spatial(*ticks, terrain, steps, arrivals, births, handbacks),
            Command::BirthSpatial { command, births } => self.birth_spatial(command, births),
            _ => run(&mut self.sim, command),
        }
    }
    /// Whether a dev placed matter or edited a volume in this run: the run is assisted
    /// (ruling 271), read from the history, which is the record (ruling 344).
    pub fn assisted(&self) -> bool {
        let placed = |e: &Entry| {
            matches!(
                e.command,
                Command::PlaceMatter { .. }
                    | Command::Edit { by: None, .. }
                    | Command::ForceBirth { .. }
                    | Command::Kill { .. }
                    | Command::AttachSite { attachment: crate::site_authoring::Attachment::New { .. }, .. }
            ) || matches!(&e.command, Command::BirthSpatial { command, .. }
                if matches!(command.as_ref(), Command::ForceBirth { .. }))
        };
        self.entries.iter().any(placed)
    }
    pub fn save(&self) -> Saved {
        let witness = self.sim.witness();
        Saved {
            version: SAVE_VERSION,
            movement: Some(self.sim.state.movement),
            life_history: Some(self.life_history),
            genesis: self.sim.genesis.as_ref().clone(),
            genesis_digest: crate::digest(&self.sim.genesis),
            branch: self.branch.clone(),
            entries: self.entries.clone(),
            tick: self.sim.state.tick,
            state_hash: witness.digest(),
            witness,
            checkpoints: self.checkpoints.clone(),
        }
    }
    /// Loads a save of any version, read by its `version` field.
    pub fn load_json(json: &[u8], mode: Execution) -> Result<Self> {
        #[derive(Deserialize)]
        struct Version {
            version: u32,
        }
        let parse = |e: serde_json::Error| e.to_string();
        match serde_json::from_slice::<Version>(json)
            .map_err(parse)?
            .version
        {
            1 => Self::load_v1(serde_json::from_slice(json).map_err(parse)?, mode),
            2 => Self::load_v2(serde_json::from_slice(json).map_err(parse)?, mode),
            _ => Self::load(serde_json::from_slice(json).map_err(parse)?, mode),
        }
    }
    pub fn load(saved: Saved, mode: Execution) -> Result<Self> {
        let (movement, life_history) = match (saved.version, saved.movement, saved.life_history) {
            (3, None, None) => (Movement::ImmediateV3, LifeHistory::Ticks),
            (4, Some(movement), None) => (movement, LifeHistory::Ticks),
            (SAVE_VERSION, Some(movement), Some(life)) => (movement, life),
            _ => return Err("save version, movement or life-history semantics mismatch".into()),
        };
        if crate::digest(&saved.genesis) != saved.genesis_digest {
            return Err("save version or genesis digest mismatch".into());
        }
        let session = Self::replay(
            saved.genesis,
            saved.branch,
            saved.entries,
            saved.tick,
            mode,
            movement,
            None,
            &[],
            |_, _| Ok(()),
        )?;
        // The earliest checkpoint that diverges names the first entry that
        // did (rulings 609 and 610); then the final state.
        let ticks = |c: &[Checkpoint]| c.iter().map(|c| c.tick).collect::<Vec<_>>();
        if ticks(&session.checkpoints) != ticks(&saved.checkpoints) {
            return Err("checkpoint history mismatch".into());
        }
        for (ours, theirs) in session.checkpoints.iter().zip(&saved.checkpoints) {
            if let Some(d) = first_divergence(&theirs.witness, &ours.witness) {
                return Err(format!(
                    "checkpoint at tick {} diverges first at {}",
                    ours.tick, d.label
                ));
            }
        }
        if saved.state_hash != saved.witness.digest() {
            return Err("save state hash mismatch".into());
        }
        if let Some(d) = first_divergence(&saved.witness, &session.sim.witness()) {
            return Err(format!("save state diverges first at {}", d.label));
        }
        Ok(Self { life_history, ..session })
    }
    /// Replays a saved history to `tick`, calling `at` on reaching each of
    /// `stops` (ascending), before any entry at that tick.
    fn replay(
        genesis: Genesis,
        branch: Key,
        entries: Vec<Entry>,
        tick: Tick,
        mode: Execution,
        movement: Movement,
        caches: Option<&Simulation>,
        stops: &[Tick],
        mut at: impl FnMut(usize, &Simulation) -> Result<()>,
    ) -> Result<Self> {
        let mut session = Self::new(genesis, mode)?;
        if let Some(source) = caches {
            session.sim.spatial_geometry = source.spatial_geometry.clone();
            session.sim.foraging_routes = source.foraging_routes.clone();
        }
        session.sim.state.movement = movement;
        session.branch = branch;
        let mut next = 0;
        let mut reach = |session: &mut Self, until: Tick| -> Result<()> {
            while next < stops.len() && stops[next] <= until {
                session.replay_until(stops[next])?;
                at(next, &session.sim)?;
                next += 1;
            }
            session.replay_until(until)
        };
        let mut ids = std::collections::BTreeSet::new();
        for entry in entries {
            if !ids.insert(entry.id.clone())
                || entry.tick < session.sim.state.tick
                || entry.tick > tick
            {
                return Err("invalid intent order or identity".into());
            }
            reach(&mut session, entry.tick)?;
            let outcome = session.recorded(&entry.command)?;
            if outcome != entry.outcome {
                return Err(format!("replay outcome mismatch for {}", entry.id));
            }
            session.entries.push(entry);
        }
        reach(&mut session, tick)?;
        if next < stops.len() {
            return Err("checkpoint history mismatch".into());
        }
        Ok(session)
    }
    /// Replays the exact accepted prefix before an entry, including earlier
    /// entries at its tick. This read does not create another played branch.
    pub fn before_entry(&self, key: &str) -> Result<Self> {
        let index = self.entries.iter().position(|entry| entry.id == key)
            .ok_or("history entry is absent")?;
        Self::replay(self.sim.genesis.as_ref().clone(), self.branch.clone(),
            self.entries[..index].to_vec(), self.entries[index].tick,
            self.sim.mode, self.sim.state.movement, Some(&self.sim), &[], |_, _| Ok(()))
    }
    /// Read an exact accepted prefix, excluding later entries at the same tick.
    /// This authenticates a finite witness without creating a played branch.
    pub fn prefix(&self, entry_count: usize, tick: Tick) -> Result<Self> {
        if entry_count > self.entries.len() || tick > self.sim.state.tick {
            return Err("history prefix exceeds the accepted source".into());
        }
        if self.entries[..entry_count].last().is_some_and(|entry| entry.tick > tick)
            || self.entries.get(entry_count).is_some_and(|entry| entry.tick < tick) {
            return Err("history prefix and requested tick disagree".into());
        }
        let prefix = Self::replay(self.sim.genesis.as_ref().clone(), self.branch.clone(),
            self.entries[..entry_count].to_vec(), tick, self.sim.mode,
            self.sim.state.movement, Some(&self.sim), &[], |_, _| Ok(()))?;
        if prefix.sim.state.tick != tick {
            return Err("history prefix splits a multi-tick command".into());
        }
        Ok(prefix)
    }
    pub fn fork_at(&self, tick: Tick, branch: Key) -> Result<Self> {
        if tick > self.sim.state.tick {
            return Err("cannot fork an unplayed future".into());
        }
        let mut fork = Self::new(self.sim.genesis.as_ref().clone(), self.sim.mode)?;
        // Replaying still authenticates every command. The derivative caches
        // match exact terrain/physical inputs and re-read biology before use.
        fork.sim.spatial_geometry = self.sim.spatial_geometry.clone();
        fork.sim.foraging_routes = self.sim.foraging_routes.clone();
        fork.sim.state.movement = self.sim.state.movement;
        fork.branch = branch;
        for entry in self.entries.iter().filter(|e| e.tick <= tick && match &e.command {
            Command::AdvanceGeometry { ticks, .. } | Command::AdvanceSpatial { ticks, .. }
                | Command::AdvanceBirthSpatial { ticks, .. } =>
                e.tick.checked_add(*ticks).is_some_and(|end| end <= tick),
            _ => true,
        }) {
            fork.replay_until(entry.tick)?;
            let outcome = fork.recorded(&entry.command)?;
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
        if self.sim.state.movement != other.sim.state.movement {
            return Err("histories use different movement semantics".into());
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
        proposed.sim.state.movement = self.sim.state.movement;
        proposed.branch = format!("branch:{}", crate::digest(&entries));
        let mut changes = Vec::new();
        for mut entry in entries {
            proposed.replay_until(entry.tick)?;
            let outcome = proposed.recorded(&entry.command)?;
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

pub(crate) fn run(sim: &mut Simulation, command: &Command) -> Result<String> {
    let outcome = dispatch::dispatch(sim, command)?;
    sim.refresh_attention();
    sim.protect_attention();
    Ok(outcome)
}
