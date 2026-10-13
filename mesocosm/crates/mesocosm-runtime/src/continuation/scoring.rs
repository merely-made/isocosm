// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{physical, Game};
use isocosm::{Session, Result, lineage::{Review, boundary::{self, Candidate, Score}}};

pub(crate) fn score(session: &Session, game: &Game, lineage: &str,
    candidate: &Candidate, ticks: u64) -> Result<Score> {
    let mut driver = physical(game);
    // A forecast is an ephemeral trial, never an offered/admitted world.
    // Keep its actual physical source while withdrawing the live branch's seal.
    driver.ecology=None;
    driver.version=driver.current_version();
    boundary::score_using(session, lineage, candidate, ticks, &mut |copy, ticks| {
        driver.refresh(copy)?;
        for _ in 0..ticks { driver.advance_clock(copy)?; }
        Ok(())
    })
}

pub(crate) fn review(session: &Session, game: &Game, participant: u64,
    ticks: u64) -> Result<Review> {
    let played = session.sim.plays(participant).ok_or("review has no played member")?;
    let lineage = &session.sim.state().population.get(played)
        .ok_or("review member absent")?.lineage;
    Review::of_using(session, lineage, ticks,
        &mut |session, lineage, candidate, ticks| score(session, game, lineage, candidate, ticks))
}
