// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The clock's due processes (ruling 256), run in order of due tick,
//! priority and identity, each over the stored groups as they stood when its
//! pass began. A process visits only the groups ready for it: carrying the
//! traits it requires of its actor (ruling 258), and passing its thresholds
//! on the actor's own accounts and age, a group's age coming due on a timer
//! (ruling 286), and a live part expressing any function it requires (ruling
//! 338). Any other group would be blocked, so passing it by changes
//! nothing but the count. The operation budget counts only the evaluations
//! that run (ruling 259), by the members each stands for (ruling 285).
//! Every act of a pass reads the world as the pass began, and a pass whose
//! acts take from ground they share is planned first, so that ground running
//! short is shared out among them (ruling 454).

mod filed;
mod frame;
mod pass;

pub(crate) use filed::{Filed, Gates};
pub(crate) use frame::{Demands, Frame, Planned, Shares, edible, fits, merge, merge_site};
pub(crate) use pass::Pass;

use crate::{
    Result,
    rules::*,
    schema::*,
    simulation::{Execution, Simulation, Work},
};
use std::collections::{BTreeMap, BTreeSet};

/// Whether an act of `p`'s pass could read what another act of it writes:
/// a target, a take of shared ground, or a site it both reads and writes.
fn framed(p: &Process) -> bool {
    let at_site = |x: &Expr| x.reads().iter().any(|u| u.body() == Binding::Place);
    let computes = |e: &Effect| {
        let guard = e.computed().is_some_and(at_site);
        guard
            || e.amounts().iter().any(|a| match a {
                Amount::Computed(x) => at_site(x),
                Amount::Fixed(_) => false,
            })
    };
    let effects: Vec<&Effect> = p.commitments.iter().chain(&p.effects).collect();
    let reads_site = p.requires.iter().any(|q| {
        matches!(
            q,
            Query::Account {
                who: Binding::Place,
                ..
            } | Query::Below {
                who: Binding::Place,
                ..
            } | Query::Holds {
                who: Binding::Place,
                ..
            } | Query::Condition { .. }
                | Query::Mood { .. }
                | Query::MoodBelow { .. }
        )
    }) || effects.iter().any(|e| computes(e));
    let writes_site = effects.iter().any(|e| {
        matches!(
            e,
            Effect::Transfer { .. }
                | Effect::Transform {
                    who: Binding::Place,
                    ..
                }
                | Effect::Condition { .. }
                | Effect::Spend { .. }
                | Effect::Convert {
                    who: Binding::Place,
                    ..
                }
                | Effect::When { .. }
        )
    });
    p.target.is_some() || p.risk.is_some() || p.takes_shared() || (reads_site && writes_site)
}

impl Simulation {
    pub(crate) fn advance_to(&mut self, end: Tick) -> Result<Work> {
        let mut queue = BTreeSet::new();
        let mut gated = BTreeMap::new();
        for p in self.genesis.rules.processes.values() {
            if let Some(period) = p.period
                && let Some(due) = self
                    .state
                    .tick
                    .checked_add(period - self.state.tick % period)
                && due <= end
            {
                queue.insert((due, p.priority, p.id.clone()));
                let gates = Gates::of(p, &self.genesis.rules);
                if !gates.none() {
                    gated.insert(p.id.clone(), gates);
                }
            }
        }
        let targeted = self
            .genesis
            .rules
            .processes
            .values()
            .any(|p| p.period.is_some() && p.target.is_some());
        if targeted {
            self.targets = Some(crate::targets::Targets::new(&self.state.population));
        }
        if !gated.is_empty() {
            let population = &self.state.population;
            self.filed = Some(Filed::new(population, gated, self.state.tick));
        }
        let mut work = Work::default();
        let done = self.scheduled(queue, end, &mut work);
        self.targets = None;
        self.filed = None;
        self.pass = None;
        done?;
        self.state.tick = end;
        if self.mode == Execution::Grouped {
            let roots = self.kept();
            let population = &mut self.state.population;
            match &mut self.journal {
                Some(j) => {
                    population.restrict_logged(&roots, &mut |first, was| j.group(first, was))
                },
                None => population.restrict(&roots),
            }
        }
        Ok(work)
    }

    /// Runs every due process in order of due tick, priority and identity.
    fn scheduled(
        &mut self,
        mut queue: BTreeSet<(Tick, i32, Key)>,
        end: Tick,
        work: &mut Work,
    ) -> Result<()> {
        while let Some((due, priority, id)) = queue.pop_first() {
            self.state.tick = due;
            if let Some(filed) = &mut self.filed {
                filed.ripen(&self.state.population, due);
            }
            let genesis = std::sync::Arc::clone(&self.genesis);
            let process = &genesis.rules.processes[&id];
            let gates = self.filed.as_ref().and_then(|f| f.gates(&id)).cloned();
            let pass = Pass::new(&self.state.population);
            let bound = pass.bound;
            self.pass = Some(pass);
            // A pass keeps its start only where one act could read what
            // another writes (ruling 454).
            self.frame = framed(process).then(Frame::default);
            if process.takes_shared() {
                self.plan(process, &id, gates.as_ref(), bound);
            }
            let mut from = 0;
            while let Some(found) = self
                .next_group(&id, gates.is_some(), from)
                .filter(|f| *f < bound)
            {
                let pass = self.pass.as_ref().expect("a pass under way");
                let (first, count) = pass.origin(&self.state.population, found);
                self.passed_by(gates.as_ref(), from, first);
                from = first + count;
                self.visit(process, &id, gates.as_ref(), (first, count), work)?;
            }
            self.passed_by(gates.as_ref(), from, bound);
            self.pass = None;
            self.frame = None;
            let next = due.checked_add(process.period.unwrap());
            if let Some(next) = next.filter(|next| *next <= end) {
                queue.insert((next, priority, id));
            }
        }
        Ok(())
    }

    /// Plans every act of the pass against the world as it began, nothing
    /// written, and shares out the ground they would take (ruling 454).
    fn plan(&mut self, process: &Process, id: &str, gates: Option<&Gates>, bound: Id) {
        let mut plans = Vec::new();
        let mut act = self.state.next_action;
        let mut from = 0;
        while let Some(first) = self
            .next_group(id, gates.is_some(), from)
            .filter(|f| *f < bound)
        {
            let count = self.state.population.groups[&first].count;
            from = first + count;
            let Some((calls, multiplicity)) = self.calls(process, first, count) else {
                continue;
            };
            for offset in 0..calls {
                let actor = first + offset;
                let entity = self.state.population.get(actor);
                if entity.is_none_or(|e| gates.is_some_and(|g| !g.open(e, self.state.tick))) {
                    continue;
                }
                let target = self.choose_target(actor, process);
                if let Some(demands) = self.demands(actor, target, id, multiplicity, act) {
                    plans.push(frame::Plan {
                        actor,
                        target,
                        act,
                        demands,
                    });
                    act += multiplicity;
                }
            }
        }
        let rules = &self.genesis.rules;
        let (sites, population) = (&self.state.sites, &self.state.population);
        let ground = |h: crate::flows::Holder| match h {
            crate::flows::Holder::Site(id) => sites.get(&id).map(|s| s.accounts.clone()),
            crate::flows::Holder::Entity(id) => population.get(id).map(|e| e.accounts.clone()),
            crate::flows::Holder::Dev => None,
        };
        let mut frame = self.frame.take().expect("a pass under way");
        frame.share(plans, ground, rules);
        self.frame = Some(frame);
    }

    /// How a group is evaluated: the calls, each standing for some members,
    /// or none when the process passes it by whole.
    fn calls(&self, process: &Process, first: Id, count: u64) -> Option<(u64, u64)> {
        let entity = self.body_at_start(first)?;
        if !entity.alive {
            return None;
        }
        if process.causation == Causation::Agentless {
            if entity.kingdom != "kingdom:world" {
                return None;
            }
        } else if entity.method == Method::Inert {
            return None;
        }
        if process
            .need_account
            .as_ref()
            .is_some_and(|a| entity.accounts.get(a).copied().unwrap_or(0) >= process.need_below)
        {
            return None;
        }
        let bulk = self.mode == Execution::Grouped && process.bulk_safe();
        Some(if bulk { (1, count) } else { (count, 1) })
    }

    /// The least stored group from `from` on that could match: any group,
    /// for a process without gates, or else one filed as ready for it.
    fn next_group(&self, process: &str, gated: bool, from: Id) -> Option<Id> {
        if gated {
            let filed = self.filed.as_ref().expect("groups filed for the advance");
            return filed.next(process, from);
        }
        let population = &self.state.population;
        population.groups.range(from..).next().map(|(&f, _)| f)
    }

    /// Debug builds check the filing against a scan: none of the groups a
    /// pass went past, from `from` up to `until`, was open for it.
    fn passed_by(&self, gates: Option<&Gates>, from: Id, until: Id) {
        #[cfg(debug_assertions)]
        for (first, g) in self.state.population.groups.range(from..until.max(from)) {
            debug_assert!(
                gates.is_some_and(|gates| !gates.open(&g.entity, self.state.tick)),
                "the filed pass went past group {first}"
            );
        }
        #[cfg(not(debug_assertions))]
        let _ = (gates, from, until);
    }

    /// One group of the pass, as a scan visits it: its gates read at its
    /// first member, then each member, or the whole cohort at once, and a
    /// member its process's gates keep out is not evaluated.
    fn visit(
        &mut self,
        process: &Process,
        id: &str,
        gates: Option<&Gates>,
        (first, count): (Id, u64),
        work: &mut Work,
    ) -> Result<()> {
        let Some((calls, multiplicity)) = self.calls(process, first, count) else {
            return Ok(());
        };
        for offset in 0..calls {
            let actor = first + offset;
            let Some(entity) = self.body_at_start(actor) else {
                continue;
            };
            if gates.is_some_and(|g| !g.open(entity, self.state.tick)) {
                continue;
            }
            // The budget counts the members an evaluation stands for (ruling
            // 285), so a budget means the same grouped and individually.
            let limit = self.genesis.rules.limits.events_per_advance as u64;
            if work.represented.saturating_add(multiplicity) > limit {
                return Err(
                    "advance exceeds configured operation budget; use shorter advances".into(),
                );
            }
            // A planned pass acts as it planned; an act it found blocked
            // stays blocked, the world it reads being the same.
            let planned = self.frame.as_ref().and_then(|f| f.planned.as_ref());
            let (target, accepted) = match planned.map(|p| p.get(&actor).cloned()) {
                None => {
                    let target = self.choose_target(actor, process);
                    (
                        target,
                        self.apply(actor, target, id, None, multiplicity).accepted(),
                    )
                },
                Some(None) => (None, false),
                Some(Some(plan)) => (
                    plan.target,
                    self.act(actor, id, multiplicity, plan).accepted(),
                ),
            };
            work.evaluations += 1;
            work.represented += multiplicity;
            if accepted {
                work.accepted += multiplicity;
                // The matter the target held as the pass began.
                if let Some(target_matter) = self.watching(id, target) {
                    self.watched(crate::watch::Watched {
                        tick: self.state.tick,
                        process: id.into(),
                        actor,
                        target,
                        count: multiplicity,
                        target_matter,
                    });
                }
            } else {
                work.blocked += multiplicity;
            }
        }
        Ok(())
    }
}
