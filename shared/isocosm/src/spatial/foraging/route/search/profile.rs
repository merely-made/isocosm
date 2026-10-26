// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! One bounded aggregate per physical query, not per graph visit.
use super::*;
pub(super) struct Work {
    actor: Id,
    started: Clock,
    pub trials: usize,
    pub pops: usize,
    pub candidates: usize,
    pub trials_ns: usize,
    pub neighbours_ns: usize,
    pub admit_ns: usize,
}
impl Work {
    pub fn new(actor: Id) -> Self {
        Self {
            actor,
            started: Clock::new(),
            trials: 0,
            pops: 0,
            candidates: 0,
            trials_ns: 0,
            neighbours_ns: 0,
            admit_ns: 0,
        }
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        crate::profile::counts(
            "foraging.search",
            &[
                ("actor", self.actor as usize),
                ("total_ns", self.started.ns()),
                ("trials", self.trials),
                ("pops", self.pops),
                ("candidates", self.candidates),
                ("trials_ns", self.trials_ns),
                ("neighbours_ns", self.neighbours_ns),
                ("admit_ns", self.admit_ns),
            ],
        );
    }
}
