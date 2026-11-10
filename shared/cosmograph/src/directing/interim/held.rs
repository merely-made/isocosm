// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A round for a participant who holds a lineage (cases 1216, 1217, 1224,
//! 1226). With no critter to play, the clock runs the round's pace and the
//! round says what happened to what is held: a surface's reaches, losses,
//! brims and meetings and its end, and what the world's conditions raised
//! for the holder of a world-class lineage. A participant who also plays a
//! critter has that round, and these events follow it.

use super::{BorrowedInterim, Candidate, Happening, Score};
use crate::{
    Result, Session,
    directing::{WORLD, readings},
    schema::*,
    simulation::Simulation,
    spread::{self, Identity},
};
use std::collections::BTreeSet;

impl BorrowedInterim<'_> {
    /// One round: a played critter's own round, with a held lineage's
    /// events after it; or, with no critter, the held lineages' round.
    ///
    /// # Errors
    /// Nobody is played and nothing is held, or a tick is refused.
    pub fn round(&mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
        advance: &mut dyn FnMut(&mut Session) -> Result<Vec<crate::flows::Flow>>,
        score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>,
    ) -> Result<(Vec<Happening>, Vec<crate::flows::Flow>)> {
        let held = self.session.sim.holds(self.participant);
        if held.is_empty() { return self.round_played(candidates, advance, score); }
        let before = self.session.sim.clone();
        let (mut out, flows) = if self.critter().is_some() {
            self.round_played(candidates, advance, score)?
        } else {
            let mut flows = Vec::new();
            for _ in 0..self.pace.round { flows.extend(advance(self.session)?); }
            let now = self.session.sim.state().tick;
            let mut out = Vec::new();
            if self.session.sim.global_collapse()?.is_some_and(|reading| reading.collapsed) {
                out.push(Happening::WorldEnded { tick: now });
            }
            (out, flows)
        };
        out.extend(happenings(&before, &self.session.sim, &held));
        Ok((out, flows))
    }
}

/// What befell the lineages `held` between `before` and `after`: a surface's
/// events and its end, then what the world raised if one is world-class.
pub(super) fn happenings(before: &Simulation, after: &Simulation, held: &[Key]) -> Vec<Happening> {
    let (was, now) = (before.state(), after.state());
    let surfaces: BTreeSet<Key> = held.iter().filter(|key| after.is_surface(key)).cloned().collect();
    let mut out: Vec<Happening> = spread::happened(&after.genesis().rules, was, now).into_iter()
        .filter(|event| surfaces.contains(&identity(event).key()))
        .map(Happening::Surfaced).collect();
    for lineage in &surfaces {
        if readings::held::ended(now, lineage) && !readings::held::ended(was, lineage) {
            out.push(Happening::LineageEnded { lineage: lineage.clone() });
        }
    }
    let world = held.iter().any(|key| now.lineages.get(key).is_some_and(|l| l.kingdom == WORLD));
    if world { out.extend(readings::raised(before, after).into_iter().map(Happening::Raised)); }
    out
}

fn identity(event: &spread::Surfaced) -> &Identity {
    match event {
        spread::Surfaced::Reached { identity, .. } | spread::Surfaced::Left { identity, .. }
            | spread::Surfaced::Brimmed { identity, .. } | spread::Surfaced::HostDied { identity, .. }
            | spread::Surfaced::Met { identity, .. } => identity,
    }
}
