// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The native deep-time runner (ruling 452, legacy D7a): a world lives its
//! span with no hand, epoch by epoch through hagiograph, and is handed over
//! on the boundary its last epoch closed.

use crate::{Result, Session, rules::DeepTimeSpan, schema::Tick};
pub use hagiograph::Handover;

/// A session as hagiograph advances it: one call runs to the next epoch
/// boundary, in steps the operation budget allows.
struct Unheld<'a> {
    session: &'a mut Session,
    epoch: Tick,
    failed: Option<String>,
}

impl Unheld<'_> {
    fn step(&mut self) -> Result<()> {
        let now = self.session.sim.state().tick;
        let mut step = self.epoch - now % self.epoch;
        loop {
            match self.session.advance(step) {
                Ok(_) if self.session.sim.state().tick % self.epoch == 0 => return Ok(()),
                Ok(_) => step = self.epoch - self.session.sim.state().tick % self.epoch,
                Err(why) if step > 1 && why.contains("operation budget") => step = step.div_ceil(2),
                Err(why) => return Err(why),
            }
        }
    }
}

impl hagiograph::Epochal for Unheld<'_> {
    fn advance(&mut self) {
        if self.failed.is_none()
            && let Err(why) = self.step()
        {
            self.failed = Some(why);
        }
    }
    fn epochs(&self) -> u64 {
        self.session.sim.state().tick / self.epoch
    }
    fn tick(&self) -> u64 {
        self.session.sim.state().tick
    }
}

/// Runs `span` epochs of the session's world with nobody in it. Refused
/// before any tick under a rule that never closes an epoch on its own.
pub fn run_deep_time(session: &mut Session, span: DeepTimeSpan) -> Result<Handover> {
    let rules = &session.sim.genesis().rules;
    rules
        .epoch
        .is_timed()
        .then_some(())
        .ok_or("deep time needs a timed epoch")?;
    // Checks the span against the rule; each call closes an epoch.
    crate::rules::deep_time_ceiling(rules.epoch, rules.epoch_ticks, span)?;
    let mut unheld = Unheld {
        epoch: rules.epoch_ticks,
        session,
        failed: None,
    };
    let calls = u64::from(span.epochs).saturating_add(1);
    let done = hagiograph::run(
        &mut unheld,
        hagiograph::DeepTime {
            epochs: span.epochs,
        },
        calls,
    );
    if let Some(why) = unheld.failed {
        return Err(why);
    }
    done.map_err(|e| e.to_string())
}
