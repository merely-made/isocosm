// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The interim loop borrows its one native owner, including its scoring clock.

use super::{boundary, Candidate, Ending, Happening, Mode, OnCollapse, Pace, Score};
use crate::{directing::readings, history::Command, schema::*, Result, Session};

pub struct BorrowedInterim<'a> {
    pub session: &'a mut Session,
    pub participant: Id,
    pub mode: Mode,
    pub pace: Pace,
    pub on_collapse: OnCollapse,
    /// How the current epoch ends (cases 1199, 1212, 1217; lineage-rounds D7).
    pub ending: Ending,
}

impl<'a> BorrowedInterim<'a> {
    pub fn new(session: &'a mut Session, participant: Id, mode: Mode,
        pace: Pace, on_collapse: OnCollapse) -> Result<Self> {
        if !session.sim.state().population.get(participant)
            .is_some_and(crate::directing::is_participant) {
            return Err("interim participant absent".into());
        }
        Ok(Self { session, participant, mode, pace, on_collapse, ending: Ending::default() })
    }

    pub(super) fn critter(&self) -> Option<Id> { self.session.sim.plays(self.participant) }

    fn take(&mut self, critter: Id) -> Result<()> {
        self.session.command(Command::Take { participant: self.participant, critter })?;
        Ok(())
    }

    fn next_life(&self, near: Id) -> Option<Id> {
        readings::heirs(&self.session.sim, near).first().copied()
    }

    /// The played critter's round; `round` adds what a held lineage met.
    pub(super) fn round_played(&mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
        advance: &mut dyn FnMut(&mut Session) -> Result<Vec<crate::flows::Flow>>,
        score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>,
    ) -> Result<(Vec<Happening>, Vec<crate::flows::Flow>)> {
        let played = self.critter().ok_or("nobody is played")?;
        let lineage = self.session.sim.state().population.get(played)
            .ok_or("played member absent")?.lineage.clone();
        let children = |s: &Session| -> Vec<Id> {
            s.sim.state().relations.iter()
                .filter(|r| r.subject == played && r.kind == "sim:child")
                .map(|r| r.object).collect()
        };
        let before = children(self.session);
        let epoch = self.session.sim.genesis().rules.epoch_ticks;
        if epoch == 0 { return Err("interim epoch length is zero".into()); }
        let from = self.session.sim.state().tick;
        let mut flows = Vec::new();
        for _ in 0..self.pace.round { flows.extend(advance(self.session)?); }
        let now = self.session.sim.state().tick;
        let mut out: Vec<_> = children(self.session).into_iter()
            .filter(|c| !before.contains(c))
            .map(|child| Happening::Born { parent: played, child }).collect();
        if self.session.sim.global_collapse()?.is_some_and(|reading| reading.collapsed) {
            if !self.session.sim.state().population.get(played).is_some_and(|e| e.alive) {
                out.push(Happening::Died { critter: played, next: None });
            }
            out.push(Happening::WorldEnded { tick: now });
            return Ok((out, flows));
        }
        if !self.session.sim.state().population.get(played).is_some_and(|e| e.alive) {
            let next = self.next_life(played);
            if let Some(next) = next { self.take(next)?; }
            out.push(Happening::Died { critter: played, next });
            if next.is_none() { out.push(Happening::LineageEnded { lineage: lineage.clone() }); }
        }
        let e = self.ending;
        // Read only where a condition is set, so the default loop costs as before.
        let held = e.par.map(|_| boundary::standing(self.session, &lineage).0).unwrap_or_default();
        let met = e.par.is_some_and(|par| held > par);
        let due = e.due(from, now, epoch, held);
        if self.session.sim.genesis().rules.directing.as_ref().is_some_and(|d| d.regard.is_some()) {
            self.record(played, &lineage, &flows, due)?;
        }
        if e.span_passed(from, now, epoch) {
            out.push(Happening::SpanPassed { tick: now });
        }
        if !e.wait && now / epoch > from / epoch && e.extend && !due {
            self.ending.extend = false;
        }
        if due {
            if met {
                out.push(Happening::ConditionMet { tick: now, held, par: e.par.unwrap_or_default() });
            }
            let since = e.began(from, epoch);
            // The played line takes its own turn in the review; its template drifts here.
            crate::lineage::drift::commit(self.session, &lineage, since)?;
            let turns = boundary::adapt_since_using(self.session, Some(&lineage), since,
                candidates, self.pace.scoring, score)?;
            out.push(Happening::Boundary { tick: now, turns });
            self.ending.begin(now);
        }
        if let Some(c) = self.critter().filter(|c|
            self.session.sim.state().population.get(*c).is_some_and(|e| e.alive)) {
            out.extend(self.collapse(c)?);
        }
        Ok((out, flows))
    }

    fn collapse(&mut self, critter: Id) -> Result<Option<Happening>> {
        let regions = readings::regions(&self.session.sim);
        let pop = &self.session.sim.state().population;
        let Some(e) = pop.get(critter) else { return Ok(None); };
        let Some(region) = e.resident_site()
            .and_then(|site| readings::region_of(&regions, site))
            .filter(|r| r.collapsed).cloned() else { return Ok(None); };
        let moved_to = if self.on_collapse == OnCollapse::Elsewhere {
            pop.groups.iter().find(|(first, g)| {
                let e2 = &g.entity;
                e2.alive && e2.lineage == e.lineage && e2.method == Method::Deliberative
                    && **first != critter && e2.resident_site().is_some_and(|site|
                        readings::region_of(&regions, site).is_some_and(|r| !r.collapsed))
            }).map(|(first, _)| *first)
        } else { None };
        if let Some(next) = moved_to { self.take(next)?; }
        Ok(Some(Happening::Collapsed { region, moved_to }))
    }
}
