// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The interim loop at site grain (rulings 680 and 681), headless: a world
//! founded from a seed and its deep time, a start the player picks (179),
//! rounds and epochs, a birth keeping the parent (183), death handing the
//! next life from the cohort (61), the boundary's adaptation (684), regional
//! collapse (225), in either mode (180).

use super::readings::{self, Mode, Region, View};
use crate::{
    Execution, Result, Session, history::Command, rules::DeepTimeSpan, schema::*,
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};

pub mod boundary;
pub mod deep;

pub use boundary::{Candidate, Score, Turn};
pub use deep::{Handover, run_deep_time};

/// When the player steps in (179, 751): after the world's own deep time
/// (452), at the first epoch boundary where a site is habitable for the
/// played lineage, waiting at most `within` epochs for one, and then as
/// many epochs more as they like.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Start {
    pub within: u32,
    pub epochs: u32,
}

/// What to do when the played critter's region collapses: both are offered
/// until the fork is ruled (225, 181).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnCollapse {
    /// Keep playing where it is; the world goes on.
    #[default]
    Stay,
    /// Take up a member of the lineage in a region still standing.
    Elsewhere,
}

/// The loop's pacing: ticks a round runs, and how long a copy grows when
/// the boundary scores a candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pace {
    pub round: Tick,
    pub scoring: Tick,
}

/// What a round brought.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Happening {
    Born {
        parent: Id,
        child: Id,
    },
    Died {
        critter: Id,
        next: Option<Id>,
    },
    Boundary {
        tick: Tick,
        turns: Vec<Turn>,
    },
    Collapsed {
        region: Region,
        moved_to: Option<Id>,
    },
    LineageEnded {
        lineage: Key,
    },
}

#[derive(Clone, Debug)]
pub struct Interim {
    pub session: Session,
    pub participant: Id,
    pub mode: Mode,
    pub pace: Pace,
    pub on_collapse: OnCollapse,
    pub handover: Handover,
    /// The boundary at which the played lineage first had a habitable site.
    pub habitable_at: Tick,
}

/// Nothing to weigh, for a boundary that adapts no line; the default is
/// `directing::revise::revisions` (752).
pub fn no_candidates(_: &Session, _: &str) -> Vec<Candidate> {
    vec![]
}

impl Interim {
    /// Founds a world with a played lineage, runs its deep time and the
    /// player's added epochs, and takes up the played line's first member.
    pub fn found(
        genesis: Genesis,
        start: Start,
        mode: Mode,
        pace: Pace,
        execution: Execution,
    ) -> Result<Self> {
        let played = genesis.founding.as_ref().and_then(|f| f.played);
        let lineage = format!(
            "lineage:{}",
            played
                .ok_or("the interim plays a generated lineage (682)")?
                .lineage
        );
        let mut session = Session::new(genesis, execution)?;
        let epochs = |epochs| DeepTimeSpan { epochs };
        let span = session.sim.genesis().rules.deep_time;
        let first = run_deep_time(&mut session, span)?;
        let mut waited = 0;
        while readings::habitable(&session.sim, &lineage).is_empty() {
            if waited == start.within {
                return Err(format!(
                    "{lineage} found no habitable site within {waited} epochs"
                ));
            }
            run_deep_time(&mut session, epochs(1))?;
            waited += 1;
        }
        let habitable_at = session.sim.state().tick;
        let last = run_deep_time(&mut session, epochs(start.epochs))?;
        let handover = Handover {
            span: hagiograph::DeepTime {
                epochs: span.epochs + waited + start.epochs,
            },
            to_epoch: last.to_epoch,
            to_tick: last.to_tick,
            ..first
        };
        let joined = session.command(Command::Join)?;
        let participant = joined
            .strip_prefix("participant:")
            .and_then(|id| id.parse().ok())
            .ok_or("a join names its participant")?;
        let mut interim = Self {
            session,
            participant,
            mode,
            pace,
            on_collapse: OnCollapse::Stay,
            handover,
            habitable_at,
        };
        let first = interim
            .first_life(&lineage)
            .ok_or("the played lineage has no living member")?;
        interim.take(first)?;
        Ok(interim)
    }

    pub fn critter(&self) -> Option<Id> {
        self.session.sim.plays(self.participant)
    }

    pub fn take(&mut self, critter: Id) -> Result<()> {
        let participant = self.participant;
        self.session.command(Command::Take {
            participant,
            critter,
        })?;
        Ok(())
    }

    pub fn nudge(&mut self, aim: super::Aim, toward: super::Toward) -> Result<String> {
        let critter = self.critter().ok_or("nobody is played")?;
        let participant = self.participant;
        self.session.command(Command::Nudge {
            participant,
            critter,
            aim,
            toward,
        })
    }

    /// What the player sees now.
    pub fn view(&self) -> View {
        let critter = self.critter().unwrap_or(super::PLACELESS);
        readings::view(&self.session.sim, critter, self.mode)
    }

    /// The played lineage's first life: its lowest living deliberative
    /// member at a site habitable for it, or anywhere if none is.
    fn first_life(&self, lineage: &str) -> Option<Id> {
        let sim = &self.session.sim;
        let homes = readings::habitable(sim, lineage);
        let ours = sim.state().population.groups.iter().filter(|(_, g)| {
            let e = &g.entity;
            e.alive && e.lineage == lineage && e.method == Method::Deliberative
        });
        let ours: Vec<(Id, Id)> = ours.map(|(f, g)| (*f, g.entity.place)).collect();
        let home = ours.iter().find(|(_, place)| homes.contains(place));
        home.or(ours.first()).map(|(f, _)| *f)
    }

    /// The played lineage's next life: a living deliberative member, at
    /// `near`'s site first, the lowest identity among equals (61).
    fn next_life(&self, near: Option<Id>) -> Option<Id> {
        let pop = &self.session.sim.state().population;
        let here = near
            .and_then(|n| pop.get(n))
            .map(|e| (e.lineage.clone(), e.place));
        let mut best: Option<(bool, Id)> = None;
        for (first, g) in &pop.groups {
            let e = &g.entity;
            let ours = here.as_ref().is_none_or(|(l, _)| *l == e.lineage);
            if !e.alive || e.method != Method::Deliberative || !ours || Some(*first) == near {
                continue;
            }
            let away = here.as_ref().is_some_and(|(_, p)| *p != e.place);
            if best.is_none_or(|b| (away, *first) < b) {
                best = Some((away, *first));
            }
        }
        best.map(|b| b.1)
    }

    /// One round: the world runs, then births, deaths, boundaries and
    /// collapses are answered, in that order.
    pub fn round(
        &mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
    ) -> Result<Vec<Happening>> {
        let mut out = Vec::new();
        let played = self.critter().ok_or("nobody is played")?;
        let lineage = self
            .session
            .sim
            .state()
            .population
            .get(played)
            .map(|e| e.lineage.clone());
        let children = |s: &Session| -> Vec<Id> {
            let rs = &s.sim.state().relations;
            rs.iter()
                .filter(|r| r.subject == played && r.kind == "sim:child")
                .map(|r| r.object)
                .collect()
        };
        let before = children(&self.session);
        let epoch = self.session.sim.genesis().rules.epoch_ticks;
        let from = self.session.sim.state().tick;
        self.session.advance(self.pace.round)?;
        let now = self.session.sim.state().tick;
        // The player keeps the parent; the young are offered (183).
        for child in children(&self.session)
            .into_iter()
            .filter(|c| !before.contains(c))
        {
            out.push(Happening::Born {
                parent: played,
                child,
            });
        }
        if !self
            .session
            .sim
            .state()
            .population
            .get(played)
            .is_some_and(|e| e.alive)
        {
            let next = self.next_life(Some(played));
            if let Some(next) = next {
                self.take(next)?;
            }
            out.push(Happening::Died {
                critter: played,
                next,
            });
            if next.is_none() {
                out.push(Happening::LineageEnded {
                    lineage: lineage.clone().unwrap_or_default(),
                });
            }
        }
        if now / epoch > from / epoch {
            let turns = boundary::adapt(
                &mut self.session,
                lineage.as_deref(),
                candidates,
                self.pace.scoring,
            )?;
            out.push(Happening::Boundary { tick: now, turns });
        }
        if let Some(c) = self.critter().filter(|c| {
            self.session
                .sim
                .state()
                .population
                .get(*c)
                .is_some_and(|e| e.alive)
        }) {
            out.extend(self.collapse(c)?);
        }
        Ok(out)
    }

    /// The played critter's region, if it has collapsed, and where play
    /// went on.
    fn collapse(&mut self, critter: Id) -> Result<Option<Happening>> {
        let regions = readings::regions(&self.session.sim);
        let pop = &self.session.sim.state().population;
        let Some(e) = pop.get(critter) else {
            return Ok(None);
        };
        let Some(region) = readings::region_of(&regions, e.place)
            .filter(|r| r.collapsed)
            .cloned()
        else {
            return Ok(None);
        };
        let mut moved_to = None;
        if self.on_collapse == OnCollapse::Elsewhere {
            let standing =
                |site: Id| readings::region_of(&regions, site).is_some_and(|r| !r.collapsed);
            moved_to = pop
                .groups
                .iter()
                .find(|(first, g)| {
                    g.entity.alive
                        && g.entity.lineage == e.lineage
                        && g.entity.method == Method::Deliberative
                        && **first != critter
                        && standing(g.entity.place)
                })
                .map(|(first, _)| *first);
            if let Some(next) = moved_to {
                self.take(next)?;
            }
        }
        Ok(Some(Happening::Collapsed { region, moved_to }))
    }
}

#[cfg(test)]
mod tests;
