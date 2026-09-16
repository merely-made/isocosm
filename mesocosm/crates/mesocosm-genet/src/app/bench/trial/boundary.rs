// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Advance-to-boundary state (D5): step the trial until its epoch closes, a
//! checkpoint holds it, or the ceiling is reached. Only this small run state
//! lives here; the actual stepping stays in `trial.rs`, since only
//! `WorldTrial::step` knows how to fold a tick into marks, activity and
//! dirty ground.

use mesocosm_runtime::{MAX_TRIAL_STEPS, Trial};

/// Ticks applied per frame while advancing, without Play's wall-clock rate
/// limit. A whole 1,000-tick epoch (`rules::DEFAULT_EPOCH_TICKS`) then
/// finishes in roughly ten frames — well under a second even in debug for
/// the one lone specimen a trial steps — while no single frame's chunk runs
/// long enough on its own to look like a hang.
pub(super) const STEP_BUDGET: u32 = 100;

/// An advance-to-boundary run, kept beside the trial it drives.
#[derive(Default)]
pub(super) struct Boundary {
    /// The epoch noted when a run started. `None` when idle.
    from: Option<u64>,
    /// The world tick of the most recent boundary this trial crossed, by
    /// Step, Play or an advance run alike. `None` until the first one.
    /// Cleared by reset.
    last_tick: Option<u64>,
    /// The epoch the last step left the world in, so a crossing is seen
    /// whichever control made it. A bench trial starts at epoch zero.
    seen_epoch: u64,
}
impl Boundary {
    pub(super) fn new() -> Self {
        Self::default()
    }
    pub(super) fn advancing(&self) -> bool {
        self.from.is_some()
    }
    pub(super) fn reset(&mut self, driver: &Trial) {
        self.from = None;
        self.last_tick = None;
        self.seen_epoch = driver.world().epoch;
    }
    /// Called after every successful trial step, from any control, so the
    /// probe's last boundary does not depend on how the boundary was reached.
    pub(super) fn note_step(&mut self, driver: &Trial) {
        let world = driver.world();
        if world.epoch > self.seen_epoch {
            self.seen_epoch = world.epoch;
            self.last_tick = Some(world.tick);
        }
    }
    /// Notes the trial's current epoch and arms a run. A no-op if one is
    /// already in progress or the trial has nothing left to spend — the same
    /// ceiling/checkpoint guard `play_trial` already applies before playing.
    pub(super) fn start(&mut self, driver: &Trial) {
        if self.from.is_none() && driver.steps() < MAX_TRIAL_STEPS && driver.checkpoint().is_none()
        {
            self.from = Some(driver.world().epoch);
        }
    }
    /// Called after every step attempted for this run, successful or not:
    /// past the noted epoch, the run is done (`note_step` already kept the
    /// crossing tick); refused (checkpoint, or the ceiling just reached), the
    /// run is done with no crossing; otherwise it keeps going.
    pub(super) fn observe(&mut self, driver: &Trial) {
        let Some(from) = self.from else { return };
        if driver.world().epoch > from {
            self.from = None;
        } else if driver.steps() >= MAX_TRIAL_STEPS || driver.checkpoint().is_some() {
            self.from = None;
        }
    }
    pub(super) fn probe_fields(&self, driver: &Trial) -> Vec<(&'static str, String)> {
        vec![
            ("trial-epoch", driver.world().epoch.to_string()),
            (
                "trial-last-boundary-tick",
                self.last_tick
                    .map_or_else(|| "none".to_string(), |t| t.to_string()),
            ),
            ("trial-advancing", self.advancing().to_string()),
        ]
    }
}
