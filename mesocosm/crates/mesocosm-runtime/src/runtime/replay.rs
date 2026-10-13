// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A run's record is its session's log: saving it and loading it back
//! replays every command and checks every witness on the way.

use isocosm::history::Saved;
use isocosm::{Execution, Session};
use serde::{Deserialize, Serialize};

use super::{Founded, Runtime};
use isocosm::directing::interim::{Interim, OnCollapse, Pace, Start};
use isocosm::directing::readings::Mode;

/// What a receipt says about a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub genesis: String,
    pub tick: u64,
    pub entries: u64,
    pub envelopes: u64,
    pub state_hash: u64,
    /// A dev placed matter or edited a volume (271).
    pub assisted: bool,
    pub dev_intents: u64,
}

/// A run read back from its save.
pub struct Replayed {
    pub session: Session,
    pub state_hash: u64,
}

/// The native record and its product-owned physical inputs share one save.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameSave {
    pub version: u32,
    pub native: Saved,
    pub spatial: crate::spatial::Game,
}

impl GameSave {
    pub fn from_native(native: Saved) -> Self {
        Self { version: crate::spatial::VERSION, native, spatial: crate::spatial::Game::default() }
    }
    pub fn check(&self) -> Result<(), String> {
        if self.version != 1 && self.version != 2 && self.version != crate::spatial::VERSION {
            return Err("unsupported Mesocosm game save".into());
        }
        self.spatial.version()
    }
}

impl Runtime {
    /// The run's save: its genesis and its log.
    pub fn save(&self) -> Saved {
        self.interim.session.save()
    }

    pub fn save_game(&self) -> GameSave {
        GameSave { version: crate::spatial::VERSION, native: self.save(), spatial: self.spatial.clone() }
    }

    pub fn replayed_game(saved: GameSave) -> Result<Replayed, String> {
        saved.check()?;
        let replayed = Self::replayed(saved.native)?;
        saved.spatial.validate_replay(&replayed.session)?;
        Ok(replayed)
    }

    pub fn resume_game(saved: GameSave, mode: Mode, pace: Pace, ticks_per_second: u32) -> Result<Self, String> {
        saved.check()?;
        let mut runtime = Self::resume(saved.native, mode, pace, ticks_per_second)?;
        saved.spatial.validate_replay(&runtime.interim.session)?;
        runtime.spatial = saved.spatial;
        Ok(runtime)
    }

    pub fn receipt(&self) -> Receipt {
        let session = &self.interim.session;
        Receipt {
            genesis: isocosm::digest(session.sim.genesis()),
            tick: self.tick(),
            entries: session.entries.len() as u64,
            envelopes: self.trace.len() as u64,
            state_hash: self.state_hash(),
            assisted: session.assisted(),
            dev_intents: self.dev_intents,
        }
    }

    /// Replays a save, refusing one that does not replay to its witness.
    pub fn replayed(saved: Saved) -> Result<Replayed, String> {
        let session = Session::load(saved, Execution::Grouped)?;
        let state_hash = session.sim.state_hash();
        Ok(Replayed {
            session,
            state_hash,
        })
    }

    /// Takes a saved run up where it stands, so a host can watch it go on
    /// (786), under `mode` and `pace`.
    pub fn resume(saved: Saved, mode: Mode, pace: Pace, ticks_per_second: u32) -> Result<Self, String> {
        let session = Session::load(saved, Execution::Grouped)?;
        let founded = Founded {
            genesis: session.sim.genesis().clone(),
            start: Start::default(),
            mode,
            pace,
            on_collapse: OnCollapse::Stay,
        };
        let interim = Interim::resume(session, mode, pace)?;
        Ok(Self::over(interim, founded, ticks_per_second))
    }

    /// The founding a run began from, beside its save.
    pub fn founding(&self) -> &Founded {
        &self.founded
    }
}
