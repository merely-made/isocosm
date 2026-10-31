// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{physical, Game};
use cosmograph::{Session, Result, lineage::{Review, boundary::{self, Candidate, Score}}};

#[derive(Default)]
pub(crate) struct Scorer { source: Option<boundary::ScoringSource> }

impl Scorer {
    pub(crate) fn score(&mut self, session: &Session, game: &Game, lineage: &str,
        candidate: &Candidate, ticks: u64) -> Result<Score> {
        let _span = crate::profile::Span::new("forecast.total");
        if self.source.as_ref().is_none_or(|source| !source.matches(session)) {
            self.source = Some(crate::profile::measure("forecast.source_replay", ||
                boundary::ScoringSource::new(session))?);
        }
        let mut driver = physical(game);
        // A forecast is an ephemeral trial, never an offered/admitted world.
        // Keep its actual physical source while withdrawing the live branch's seal.
        driver.ecology=None;
        driver.version=driver.current_version();
        self.source.as_ref().ok_or("forecast source absent")?
            .score_using(lineage, candidate, ticks, &mut |copy, ticks| {
            let _span = crate::profile::Span::new("forecast.advance");
            driver.refresh(copy)?;
            // Scoring reads the resulting trial state, not its transient matter moves.
            for _ in 0..ticks { let _flows = driver.advance_clock(copy)?; }
            Ok(())
        })
    }
}

pub(crate) fn review(session: &Session, game: &Game, participant: u64,
    ticks: u64) -> Result<Review> {
    let played = session.sim.plays(participant).ok_or("review has no played member")?;
    let lineage = &session.sim.state().population.get(played)
        .ok_or("review member absent")?.lineage;
    let mut scorer = Scorer::default();
    Review::of_using(session, lineage, ticks,
        &mut |session, lineage, candidate, ticks| scorer.score(session, game, lineage, candidate, ticks))
}
