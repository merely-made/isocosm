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
        Self { version: crate::spatial::UNMARKED_VERSION, native, spatial: crate::spatial::Game::default() }
    }
    pub fn check(&self) -> Result<(), String> {
        let current = self.spatial.current_version();
        let supported = match self.version {
            1..=crate::spatial::UNMARKED_VERSION => current == crate::spatial::UNMARKED_VERSION,
            crate::spatial::SIGHT_VERSION => current == crate::spatial::SIGHT_VERSION,
            crate::spatial::BIRTH_SIGHT_VERSION => current == crate::spatial::BIRTH_SIGHT_VERSION,
            crate::spatial::VERSION => current == crate::spatial::VERSION,
            crate::spatial::LOOP_VERSION => current == crate::spatial::LOOP_VERSION,
            crate::spatial::ECOLOGY_VERSION => current == crate::spatial::ECOLOGY_VERSION
                && self.spatial.continuation.is_some(),
            _ => false,
        };
        if !supported {
            return Err("unsupported Mesocosm game save".into());
        }
        self.spatial.version()
    }
}

impl Runtime {
    /// The run's save: its genesis and its log.
    pub fn save(&self) -> Saved {
        self.session().save()
    }

    pub fn save_game(&self) -> GameSave {
        let spatial = self.driver.product().loop_game(self.driver.world(), self.driver.queued().cloned().collect());
        GameSave { version: spatial.current_version(), native: self.save(), spatial }
    }

    pub fn replayed_game(saved: GameSave) -> Result<Replayed, String> {
        saved.check()?;
        let execution=saved.spatial.ecology.as_ref().map(|seal|seal.witness.execution)
            .unwrap_or(Execution::Grouped);
        let session=Session::load(saved.native,execution)?;
        let replayed=Replayed {state_hash:session.sim.state_hash(),session};
        saved.spatial.validate_replay(&replayed.session)?;
        Ok(replayed)
    }

    pub fn resume_game(saved: GameSave, mode: Mode, pace: Pace, ticks_per_second: u32) -> Result<Self, String> {
        saved.check()?;
        if saved.spatial.continuation.is_some() {
            return Self::resume_game_with_mode(saved, ticks_per_second, Some(mode));
        }
        let mut runtime = Self::resume(saved.native, mode, pace, ticks_per_second)?;
        saved.spatial.validate_replay(&runtime.session())?;
        runtime.with_product(|product, _| product.spatial = saved.spatial);
        Ok(runtime)
    }

    /// A marked save restores its own controller, pace and outstanding hold.
    /// Only an explicitly supplied mode overrides its presentation mode.
    pub fn resume_game_with_mode(saved: GameSave, ticks_per_second: u32,
        mode: Option<Mode>) -> Result<Self, String> {
        saved.check()?;
        let Some(continuation) = &saved.spatial.continuation else {
            // The old standalone resume grammar had these explicit settings;
            // it contained no saved loop or question to recover.
            return Self::resume_game(saved, mode.unwrap_or(Mode::Creative),
                Pace { round: 1, scoring: 6 }, ticks_per_second);
        };
        let execution=saved.spatial.ecology.as_ref().map(|seal|seal.witness.execution)
            .unwrap_or(Execution::Grouped);
        let session = Session::load(saved.native, execution)?;
        saved.spatial.validate_replay(&session)?;
        let settings = continuation.settings;
        let founded = Founded { genesis: session.sim.genesis().clone(),
            start: continuation.origin.as_ref().map(|o| o.start).unwrap_or_default(),
            mode: settings.mode, pace: settings.pace, on_collapse: settings.on_collapse };
        let interim = Interim::resume_as(session, continuation.participant, settings.mode, settings.pace)?;
        let mut runtime = Self::over(interim, founded, ticks_per_second);
        let (queued, fault) = runtime.with_product(|product, world| {
            product.spatial = saved.spatial;
            product.restore_loop(world)?;
            if let Some(mode) = mode {
                product.lp.mode = mode; product.founded.mode = mode;
                product.sync_loop(world);
            }
            let queued: Vec<_> = product.spatial.continuation.as_ref().map(|c| c.queued.clone()).unwrap_or_default();
            Ok::<_, String>((queued, product.fault.clone()))
        })?;
        runtime.driver.restore_queue(queued);
        runtime.driver.restore_fault(fault);
        runtime.publish();
        Ok(runtime)
    }

    pub fn receipt(&self) -> Receipt {
        let session = self.session();
        Receipt {
            genesis: isocosm::digest(session.sim.genesis()),
            tick: session.sim.state().tick,
            entries: session.entries.len() as u64,
            envelopes: self.trace().len() as u64,
            state_hash: session.sim.state_hash(),
            assisted: session.assisted(),
            dev_intents: self.dev_intents(),
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
        self.founded()
    }
}
