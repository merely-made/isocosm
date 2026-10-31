// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// One replay-authenticated current cut for independent candidate trials.
/// Private ownership prevents a trial from changing the next trial's source.
pub struct ScoringSource { key: Key, session: Session }

impl ScoringSource {
    pub fn new(session: &Session) -> Result<Self> {
        let _span = crate::profile::Span::new("forecast.source_replay");
        let key = source_key(session);
        let session = session.fork_at(session.sim.state().tick, "branch:score:source".into())?;
        Ok(Self { key, session })
    }

    /// Covers exactly the inputs read by `fork_at`, including mutable history.
    /// A lineage commit, clock change or changed recorded outcome invalidates it.
    pub fn matches(&self, session: &Session) -> bool { self.key == source_key(session) }

    pub fn score_using(&self, lineage: &str, candidate: &Candidate, ticks: Tick,
        advance: &mut dyn FnMut(&mut Session, Tick) -> Result<()>) -> Result<Score> {
        let mut copy = self.session.clone();
        copy.branch = format!("branch:score:{}", candidate.name);
        super::grow_score(copy, lineage, candidate, ticks, advance)
    }
}

fn source_key(session: &Session) -> Key {
    crate::digest(&(session.sim.genesis(), session.sim.mode(),
        session.sim.state().movement, session.sim.state().tick, &session.entries))
}
