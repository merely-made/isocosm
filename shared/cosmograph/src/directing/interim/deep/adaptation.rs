// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit adaptation between epochs of unattended time (Ecology 1, 1147).
//! Clock providers keep their existing contract; lineage commands are admitted
//! at the boundary, outside that provider. Forecast copies advance without
//! opening adaptation rounds of their own.

use super::{Handover, run_deep_time, run_deep_time_using};
use crate::{
    Result, Session,
    directing::interim::Happening,
    lineage::{
        boundary::{self, Candidate, Score},
        revise,
    },
    rules::DeepTimeSpan,
    schema::Tick,
};

/// Runs unattended epochs, then weighs the native revision candidates for every
/// living lineage at each boundary. Returns the handover and boundary readings.
/// `scoring` is the forecast horizon, in this world's ticks; it must be positive
/// for a nonempty span. This exact site-grain clock makes no coarse-step claim.
pub fn run_deep_time_adapting(
    session: &mut Session,
    span: DeepTimeSpan,
    scoring: Tick,
) -> Result<(Handover, Vec<Happening>)> {
    run_adapting(
        session,
        span,
        scoring,
        &revise::revisions,
        &mut run_deep_time,
        &mut boundary::score,
    )
}

/// Runs the same boundary operation through an explicit physical clock and
/// scorer. The clock remains restricted to native one-tick physical inputs;
/// accepted revisions are recorded separately by the ordinary command path.
/// The scorer must advance its disposable forecast without opening further
/// candidate searches; `boundary::score_using` supplies that scoring door.
pub fn run_deep_time_adapting_using(
    session: &mut Session,
    span: DeepTimeSpan,
    scoring: Tick,
    advance: &mut dyn FnMut(&mut Session) -> Result<()>,
    score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>,
) -> Result<(Handover, Vec<Happening>)> {
    run_adapting(
        session,
        span,
        scoring,
        &revise::revisions,
        &mut |session, span| run_deep_time_using(session, span, advance),
        score,
    )
}

fn run_adapting(
    session: &mut Session,
    span: DeepTimeSpan,
    scoring: Tick,
    candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
    advance: &mut dyn FnMut(&mut Session, DeepTimeSpan) -> Result<Handover>,
    score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>,
) -> Result<(Handover, Vec<Happening>)> {
    let rules = &session.sim.genesis().rules;
    let epoch = rules.epoch_ticks;
    if epoch == 0 {
        return Err("deep-time epoch length is zero".into());
    }
    let tick = session.sim.state().tick;
    let mut handover = Handover {
        span: hagiograph::DeepTime {
            epochs: span.epochs,
        },
        from_epoch: tick / epoch,
        to_epoch: tick / epoch,
        from_tick: tick,
        to_tick: tick,
    };
    if span.is_bare() {
        return Ok((handover, vec![]));
    }
    if scoring == 0 {
        return Err("adaptive deep time needs positive scoring ticks".into());
    }
    if !rules.epoch.is_timed() {
        return Err("deep time needs a timed epoch".into());
    }
    crate::rules::deep_time_ceiling(rules.epoch, epoch, span)?;
    let mut boundaries = Vec::new();
    for _ in 0..span.epochs {
        let next = advance(session, DeepTimeSpan { epochs: 1 })?;
        let turns = boundary::adapt_using(session, None, candidates, scoring, score)?;
        boundaries.push(Happening::Boundary {
            tick: session.sim.state().tick,
            turns,
        });
        handover.to_epoch = next.to_epoch;
        handover.to_tick = next.to_tick;
    }
    Ok((handover, boundaries))
}

#[cfg(test)]
mod tests;
