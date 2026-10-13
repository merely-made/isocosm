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

pub use crate::lineage::boundary;
pub mod deep;
mod borrowed;
pub use borrowed::BorrowedInterim;

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

    /// Takes up a loop from a session already played (786): its first
    /// participant, the handover read from where the session stands.
    pub fn resume(session: Session, mode: Mode, pace: Pace) -> Result<Self> {
        let state = session.sim.state();
        let participant = state
            .population
            .groups
            .iter()
            .find(|(_, g)| super::is_participant(&g.entity))
            .map(|(first, _)| *first)
            .ok_or("the session has no participant")?;
        Self::resume_as(session, participant, mode, pace)
    }

    /// Restore the explicitly saved controller without selecting another one.
    pub fn resume_as(session: Session, participant: Id, mode: Mode, pace: Pace) -> Result<Self> {
        let state = session.sim.state();
        if !state.population.get(participant).is_some_and(super::is_participant) {
            return Err("saved interim participant absent".into());
        }
        let tick = state.tick;
        let epoch = session.sim.genesis().rules.epoch_ticks.max(1);
        let handover = Handover {
            span: hagiograph::DeepTime { epochs: 0 },
            from_epoch: 0,
            to_epoch: tick / epoch,
            from_tick: 0,
            to_tick: tick,
        };
        Ok(Self {
            session,
            participant,
            mode,
            pace,
            on_collapse: OnCollapse::Stay,
            handover,
            habitable_at: 0,
        })
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

    pub fn nudge(
        &mut self,
        aim: super::Aim,
        toward: super::Toward,
        act: Option<Key>,
    ) -> Result<String> {
        let critter = self.critter().ok_or("nobody is played")?;
        let participant = self.participant;
        self.session.command(Command::Nudge {
            participant,
            critter,
            aim,
            toward,
            act,
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
        let ours: Vec<(Id, Option<Id>)> = ours.map(|(f, g)| (*f, g.entity.resident_site())).collect();
        let home = ours.iter().find(|(_, place)| place.is_some_and(|place| homes.contains(&place)));
        home.or(ours.first()).map(|(f, _)| *f)
    }

    /// One round: the world runs, then births, deaths, boundaries and
    /// collapses are answered, in that order.
    pub fn round(
        &mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
    ) -> Result<Vec<Happening>> {
        self.round_with_flows(candidates).map(|(out, _)| out)
    }

    /// A round, with the matter moves its ticks made, tick by tick.
    pub fn round_with_flows(
        &mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
    ) -> Result<(Vec<Happening>, Vec<crate::flows::Flow>)> {
        self.round_with_flows_using(candidates, &mut |session| {
            session.advance_tick_with_flows().map(|out| out.flows)
        })
    }

    /// The game may resolve native-declared geometry through the same tick door.
    pub fn round_with_flows_using(
        &mut self,
        candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
        advance: &mut dyn FnMut(&mut Session) -> Result<Vec<crate::flows::Flow>>,
    ) -> Result<(Vec<Happening>, Vec<crate::flows::Flow>)> {
        self.round_with_flows_and_score(candidates, advance, &mut boundary::score)
    }

    pub fn round_with_flows_and_score(
        &mut self, candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
        advance: &mut dyn FnMut(&mut Session) -> Result<Vec<crate::flows::Flow>>,
        score: &mut dyn FnMut(&Session, &str, &Candidate, Tick) -> Result<Score>,
    ) -> Result<(Vec<Happening>, Vec<crate::flows::Flow>)> {
        BorrowedInterim::new(&mut self.session, self.participant, self.mode,
            self.pace, self.on_collapse)?.round(candidates, advance, score)
    }

}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod diag;

#[cfg(test)]
mod reproduction;
