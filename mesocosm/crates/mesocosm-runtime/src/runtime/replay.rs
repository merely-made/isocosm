// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A run's record is its session's log: saving it and loading it back
//! replays every command and checks every witness on the way.

use isocosm::history::Saved;
use isocosm::{Execution, Session};
use serde::{Deserialize, Serialize};

use super::{Founded, Runtime};

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

impl Runtime {
    /// The run's save: its genesis and its log.
    pub fn save(&self) -> Saved {
        self.interim.session.save()
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

    /// The founding a run began from, beside its save.
    pub fn founding(&self) -> &Founded {
        &self.founded
    }
}
