// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The scheduled acts a host asked to see. A host names the processes it
//! watches; each accepted act of one during an advance is kept, with the
//! matter its target held as the act began, until the host takes them.
//! Watching changes nothing the world does, and an advance that is refused
//! and put back takes its acts with it.

use crate::{meaning::mass, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One accepted scheduled act of a watched process.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Watched {
    pub tick: Tick,
    pub process: Key,
    pub actor: Id,
    pub target: Option<Id>,
    /// The members the act stood for.
    pub count: u64,
    /// The matter the target held as the act began; none without a target.
    pub target_matter: u128,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Watch {
    processes: BTreeSet<Key>,
    acts: Vec<Watched>,
    /// The acts kept when the advance under way began.
    kept: usize,
}

impl Simulation {
    /// Keeps each accepted scheduled act of `process` from now on.
    pub fn watch(&mut self, process: &str) {
        self.watch.processes.insert(process.into());
    }

    /// The watched acts kept since the last call, in the order they ran.
    pub fn take_watched(&mut self) -> Vec<Watched> {
        std::mem::take(&mut self.watch.acts)
    }

    /// The matter `target` holds now, when acts of `process` are watched.
    pub(crate) fn watching(&self, process: &str, target: Option<Id>) -> Option<u128> {
        if !self.watch.processes.contains(process) {
            return None;
        }
        let held = target.and_then(|t| self.state.population.get(t));
        Some(held.map_or(0, |e| mass(&e.accounts, &self.genesis.rules)))
    }

    pub(crate) fn watched(&mut self, act: Watched) {
        self.watch.acts.push(act);
    }

    /// Marks where an advance begins, to put its acts back if it is refused.
    pub(crate) fn watch_from(&mut self) {
        self.watch.kept = self.watch.acts.len();
    }

    pub(crate) fn unwatch(&mut self) {
        self.watch.acts.truncate(self.watch.kept);
    }
}
