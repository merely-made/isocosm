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
    if span.is_bare() {
        let (tick, epoch) = (session.sim.state().tick, rules.epoch_ticks);
        let at = tick / epoch;
        return Ok(Handover {
            span: hagiograph::DeepTime { epochs: 0 },
            from_epoch: at,
            to_epoch: at,
            from_tick: tick,
            to_tick: tick,
        });
    }
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

/// Continue this actual session through authored deep time with its supplied
/// one-tick provider. Configured physical clocks are never replaced by site time.
pub fn run_deep_time_using(session: &mut Session, span: DeepTimeSpan,
    advance: &mut dyn FnMut(&mut Session) -> Result<()>) -> Result<Handover> {
    let rules = &session.sim.genesis().rules;
    let epoch = rules.epoch_ticks;
    if epoch == 0 { return Err("deep-time epoch length is zero".into()); }
    if span.is_bare() {
        let tick = session.sim.state().tick;
        return Ok(Handover { span: hagiograph::DeepTime { epochs: 0 },
            from_epoch: tick / epoch, to_epoch: tick / epoch,
            from_tick: tick, to_tick: tick });
    }
    if !rules.epoch.is_timed() { return Err("deep time needs a timed epoch".into()); }
    crate::rules::deep_time_ceiling(rules.epoch, epoch, span)?;
    let genesis = session.sim.genesis.clone();
    let branch = session.branch.clone();
    let mode = session.sim.mode();
    let movement = session.sim.state().movement;
    let mut driven = Driven { session, advance, epoch, genesis, branch, mode, movement, failed: None };
    let done = hagiograph::run(&mut driven,
        hagiograph::DeepTime { epochs: span.epochs }, u64::from(span.epochs) + 1);
    if let Some(why) = driven.failed { return Err(why); }
    done.map_err(|why| why.to_string())
}

struct Driven<'a> {
    session: &'a mut Session,
    advance: &'a mut dyn FnMut(&mut Session) -> Result<()>,
    epoch: Tick,
    genesis: std::sync::Arc<crate::simulation::Genesis>,
    branch: crate::schema::Key,
    mode: crate::Execution,
    movement: crate::schema::Movement,
    failed: Option<String>,
}

impl Driven<'_> {
    fn step(&mut self) -> Result<()> {
        let now = self.session.sim.state().tick;
        let end = now.checked_add(self.epoch - now % self.epoch)
            .ok_or("deep-time boundary overflow")?;
        while self.session.sim.state().tick < end {
            let before = self.session.sim.state().tick;
            let next = before.checked_add(1).ok_or("deep-time clock overflow")?;
            let entries = self.session.entries.clone();
            let checkpoints = self.session.checkpoints.clone();
            (self.advance)(self.session)?;
            if self.session.sim.state().tick != next || self.session.sim.mode() != self.mode
                || self.session.sim.state().movement != self.movement
                || self.session.sim.genesis() != self.genesis.as_ref()
                || self.session.branch != self.branch || !self.session.entries.starts_with(&entries)
                || !self.session.checkpoints.starts_with(&checkpoints) {
                return Err("deep-time provider changed its native source, prefix or one-tick boundary".into());
            }
            for entry in &self.session.entries[entries.len()..] {
                if !matches!(&entry.command,
                    crate::Command::AdvanceGeometry { ticks: 1, .. }
                    | crate::Command::AdvanceSpatial { ticks: 1, .. }
                    | crate::Command::AdvanceBirthSpatial { ticks: 1, .. }
                    | crate::Command::Patch { .. }) {
                    return Err("deep-time provider permits only clocks and physical patch inputs".into());
                }
            }
        }
        Ok(())
    }
}

impl hagiograph::Epochal for Driven<'_> {
    fn advance(&mut self) {
        if self.failed.is_none() && let Err(why) = self.step() { self.failed = Some(why); }
    }
    fn epochs(&self) -> u64 { self.session.sim.state().tick / self.epoch }
    fn tick(&self) -> u64 { self.session.sim.state().tick }
}
