// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A bounded, disposable run of an exact specimen world. The baseline world
//! plus the applied trace is its replay source; seed/count alone cannot replay
//! a proportion edit or resized content palette. This exposes no shipping
//! Runtime receipt and never commits its result back to the specimen.

use super::Runtime;
use crate::Checkpoint;
use mesocosm_core::{History, Intent, OrganismId, World, history::Event, state_hash};
use serde::Serialize;
use std::collections::BTreeMap;

pub const MAX_TRIAL_STEPS: u32 = 128;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialActivity {
    /// Zero-based ordinal in this disposable trial's accepted history.
    pub sequence: u64,
    /// Core event tick, not a wall clock or frame number.
    pub tick: u64,
    pub event: Event,
    /// Mover or eater. This record does not identify a responsible organ.
    pub source: OrganismId,
    /// The donor for feeding; absent for movement.
    pub target: Option<OrganismId>,
    /// Movement's recorded departure, or the donor's pre-step position.
    pub from: [i32; 3],
    /// Movement's recorded arrival, or the eater's post-step position.
    pub to: [i32; 3],
}

pub struct Trial {
    baseline: World,
    runtime: Runtime,
    steps: u32,
    activities: Vec<TrialActivity>,
}

impl Trial {
    /// The bench currently supplies fresh admitted worlds. A later snapshot
    /// needs its runtime checkpoint/history too, so refuse it rather than
    /// silently reconstructing a missing decision or past.
    pub fn new(source: &World) -> Result<Self, String> {
        if source.tick != 0 || source.epoch != 0 || source.at_boundary() {
            return Err("specimen trial requires a fresh tick-zero world".into());
        }
        if source.controlled().is_none_or(|body| !body.is_alive()) {
            return Err("specimen trial requires a living controlled body".into());
        }
        // Pending founding events are serialized world state. Preserve them
        // exactly; the ordinary runtime absorbs them on its first step, while
        // the activity projection below excludes Born and other founding facts.
        let baseline = source.clone();
        let runtime = driver(&baseline);
        Ok(Self {
            baseline,
            runtime,
            steps: 0,
            activities: Vec::new(),
        })
    }

    pub fn world(&self) -> &World {
        self.runtime.world()
    }
    pub fn history(&self) -> &History {
        self.runtime.history()
    }
    pub fn steps(&self) -> u32 {
        self.steps
    }
    pub fn state_hash(&self) -> u64 {
        self.runtime.state_hash()
    }
    pub fn baseline_hash(&self) -> u64 {
        state_hash(&self.baseline)
    }
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.runtime.checkpoint()
    }
    pub fn trace(&self) -> &[Intent] {
        self.runtime.trace()
    }
    /// Latest successful step's batch. Reading or redrawing never drains it;
    /// process it only when `step()` returns true, keyed by tick/sequence.
    pub fn activities(&self) -> &[TrialActivity] {
        &self.activities
    }
    pub fn drain_ground_dirty(&mut self) -> Vec<[i16; 3]> {
        self.runtime.drain_ground_dirty()
    }

    pub fn reset(&mut self) {
        self.runtime = driver(&self.baseline);
        self.steps = 0;
        self.activities.clear();
    }

    /// One ordinary idle step. Checkpoints retain their existing runtime
    /// semantics; the trial never answers a question on the user's behalf.
    pub fn step(&mut self) -> bool {
        if self.steps >= MAX_TRIAL_STEPS || self.checkpoint().is_some() {
            return false;
        }
        let before: BTreeMap<_, _> = self
            .world()
            .organisms
            .iter()
            .map(|o| (o.id, o.position))
            .collect();
        let start = self.history().len();
        if self.runtime.step(1) == 0 {
            return false;
        }
        self.steps += 1;
        self.activities.clear();
        for (offset, recorded) in self.runtime.history().log().entries()[start..]
            .iter()
            .enumerate()
        {
            let endpoints = match recorded.record {
                Event::Moved { organism, from, to } if from != to => {
                    Some((organism, None, from, to))
                },
                Event::Fed {
                    eater,
                    from: donor,
                    mass_mg,
                    ..
                } if mass_mg > 0 => {
                    let old = |id| before.get(&id).copied();
                    let now = |id| {
                        self.runtime
                            .world()
                            .organisms
                            .iter()
                            .find(|o| o.id == id)
                            .map(|o| o.position)
                    };
                    old(donor)
                        .or_else(|| now(donor))
                        .zip(now(eater).or_else(|| old(eater)))
                        .map(|(from, to)| (eater, Some(donor), from, to))
                },
                _ => None,
            };
            if let Some((source, target, from, to)) = endpoints {
                self.activities.push(TrialActivity {
                    sequence: (start + offset) as u64,
                    tick: recorded.tick,
                    event: recorded.record,
                    source,
                    target,
                    from,
                    to,
                });
            }
        }
        true
    }
}

fn driver(baseline: &World) -> Runtime {
    // Runtime's legacy seed/count fields are deliberately unexposed here.
    // The owned baseline is this trial's exact and only reconstruction source.
    Runtime::from_world(baseline.clone(), 0, 0, 1)
}

#[cfg(test)]
mod tests;
