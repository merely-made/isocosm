// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The interim loop borrows its one native owner, including its scoring clock.

use super::{boundary, Candidate, Happening, Mode, OnCollapse, Pace, Score};
use crate::{directing::readings, history::Command, schema::*, Result, Session};

pub struct BorrowedInterim<'a> {
    pub session: &'a mut Session,
    pub participant: Id,
    pub mode: Mode,
    pub pace: Pace,
    pub on_collapse: OnCollapse,
}

impl<'a> BorrowedInterim<'a> {
    pub fn new(session: &'a mut Session, participant: Id, mode: Mode,
        pace: Pace, on_collapse: OnCollapse) -> Result<Self> {
        if !session.sim.state().population.get(participant)
            .is_some_and(crate::directing::is_participant) {
            return Err("interim participant absent".into());
        }
        Ok(Self { session, participant, mode, pace, on_collapse })
    }

    fn critter(&self) -> Option<Id> { self.session.sim.plays(self.participant) }

    fn take(&mut self, critter: Id) -> Result<()> {
        self.session.command(Command::Take { participant: self.participant, critter })?;
        Ok(())
    }

    fn next_life(&self, near: Id) -> Option<Id> {
        let pop = &self.session.sim.state().population;
        let here = pop.get(near).map(|e| (&e.lineage, e.resident_site()));
        pop.groups.iter().filter_map(|(first, g)| {
            let e = &g.entity;
            if !e.alive || e.method != Method::Deliberative || *first == near
                || here.is_some_and(|(lineage, _)| *lineage != e.lineage) {
                return None;
            }
            let away = here.is_some_and(|(_, p)| p.is_none() || p != e.resident_site());
            Some((away, *first))
        }).min().map(|(_, first)| first)
    }

    pub fn round(&mut self,
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
        if now / epoch > from / epoch {
            let turns = boundary::adapt_using(self.session, Some(&lineage),
                candidates, self.pace.scoring, score)?;
            out.push(Happening::Boundary { tick: now, turns });
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
