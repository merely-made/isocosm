// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The scheduled acts a host asked to see. A host names the processes it
//! watches; each accepted act of one during an advance is kept, with the
//! matter its target held as the act began, until the host takes them.
//! Watching changes nothing the world does, and an advance that is refused
//! and put back takes its acts with it.

use crate::{anatomy::books, meaning::mass, schema::*, simulation::Simulation};
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
    ecology: Option<crate::ecology::JointCapture>,
}

impl Simulation {
    pub(crate) fn begin_ecology_witness(&mut self, spec: &crate::ecology::JointSpec) -> crate::Result<()> {
        if self.watch.ecology.is_some() { return Err("joint witness capture is already active".into()); }
        self.watch.ecology = Some(crate::ecology::JointCapture { spec: spec.clone(), acts: vec![], kept: 0 });
        Ok(())
    }

    pub(crate) fn end_ecology_witness(&mut self) -> Vec<crate::ecology::ActionProof> {
        self.watch.ecology.take().map_or_else(Vec::new, |capture| capture.acts)
    }

    pub(crate) fn ecology_spec(&self, process: &str) -> Option<&crate::ecology::JointSpec> {
        self.watch.ecology.as_ref().filter(|capture|
            capture.spec.observes(process)).map(|capture| &capture.spec)
    }

    pub(crate) fn record_ecology(&mut self, act: crate::ecology::ActionProof) {
        if let Some(capture) = &mut self.watch.ecology { capture.acts.push(act); }
    }
    /// Keeps each accepted scheduled act of `process` from now on.
    pub fn watch(&mut self, process: &str) {
        self.watch.processes.insert(process.into());
    }

    /// The watched acts kept since the last call, in the order they ran.
    pub fn take_watched(&mut self) -> Vec<Watched> {
        std::mem::take(&mut self.watch.acts)
    }

    /// The matter `target` held as the pass began, when acts of `process`
    /// are watched.
    pub(crate) fn watching(&self, process: &str, target: Option<Id>) -> Option<u128> {
        if !self.watch.processes.contains(process) {
            return None;
        }
        let held = target.and_then(|t| self.body_at_start(t));
        // A body's own matter lives in its parts (ruling 504).
        Some(held.map_or(0, |e| mass(&books(e), &self.genesis.rules)))
    }

    pub(crate) fn watched(&mut self, act: Watched) {
        self.watch.acts.push(act);
    }

    /// Marks where an advance begins, to put its acts back if it is refused.
    pub(crate) fn watch_from(&mut self) {
        self.watch.kept = self.watch.acts.len();
        if let Some(capture) = &mut self.watch.ecology { capture.kept = capture.acts.len(); }
    }

    pub(crate) fn unwatch(&mut self) {
        self.watch.acts.truncate(self.watch.kept);
        if let Some(capture) = &mut self.watch.ecology { capture.acts.truncate(capture.kept); }
    }
}
