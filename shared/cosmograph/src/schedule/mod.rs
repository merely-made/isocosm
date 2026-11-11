// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The clock's due processes (ruling 256), run in order of due tick,
//! priority and identity, each over the stored groups as they stood when its
//! pass began. A process visits only the groups ready for it: carrying the
//! traits it requires of its actor (ruling 258), and passing its thresholds
//! on the actor's own accounts and age, a group's age coming due on a timer
//! (ruling 286), a live part expressing any function it requires (ruling
//! 338), and for a timed move, residing where one of its routes starts,
//! unless it is a deliberative member's to choose. Any other group would be
//! blocked, so passing it by changes nothing but the count. The operation budget counts only the evaluations
//! that run (ruling 259), by the members each stands for (ruling 285).
//! Every act of a pass reads the world as the pass began, and a pass whose
//! acts take from ground they share is planned first, so that ground running
//! short is shared out among them (ruling 454).

mod filed;
mod frame;
mod pass;
mod turn;

pub(crate) use filed::{Changed, Filed, Gates, World};
pub(crate) use frame::{Demands, Frame, MealSpec, Planned, Shares, edible, fits, merge, merge_site};
pub(crate) use pass::Pass;
use turn::Turns;

use crate::{
    Result,
    directing::Deliberated,
    rules::*,
    schema::*,
    simulation::{Execution, Simulation, Work},
};
use std::collections::BTreeSet;

/// Whether an act of `p`'s pass could read what another act of it writes:
/// a target, a take of shared ground, or a site it both reads and writes.
/// A Choice implicitly scores every due option, including its targets.
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
        ) || matches!(q, Query::Computed(x) if at_site(x))
    }) || effects.iter().any(|e| computes(e));
    let writes_site = effects.iter().any(|e| {
        matches!(
            e,
            Effect::Transfer { .. }
                | Effect::Transform {
                    who: Binding::Place,
                    ..
                }
                | Effect::Sunlight {
                    who: Binding::Place,
                    ..
                }
                | Effect::Condition { .. }
                | Effect::Spend { .. }
                | Effect::MovePaced { .. }
                | Effect::Grow { .. }
                | Effect::Heal { .. }
                | Effect::Convert {
                    who: Binding::Place,
                    ..
                }
                | Effect::When { .. }
        )
    });
    // A count of members reads the bodies its pass may bear, move or kill.
    // So does a sense reading (1071): its crowding, gradient or activity.
    let counts = p.requires.iter()
        .any(|q| matches!(q, Query::Members { .. } | Query::MembersBelow { .. } | Query::MembersPer { .. }
            | Query::MembersPerHead { .. } | Query::FoodPerHead { .. } | Query::EdiblePerHead { .. } | Query::Sensed { .. }));
    p.target.is_some()
        || counts
        || p.risk.is_some()
        || p.causation == Causation::Choice
        || !p.goal_directives.is_empty()
        || p.takes_shared()
        || (reads_site && writes_site)
}

impl Simulation {
    pub(crate) fn advance_to(&mut self, end: Tick) -> Result<Work> {
        // A paced cohort chooses by its first identity (E8) and cohorts
        // regroup at an advance's end, so a Grouped span with a paced lineage
        // runs a tick at a time: the same ticks group and choose alike
        // however an advance cuts them (D41).
        if self.mode == Execution::Grouped && end > self.state.tick.saturating_add(1) && self.paced() {
            let mut work = Work::default();
            while self.state.tick < end {
                let next = self.advance_to(self.state.tick + 1)?;
                work.evaluations += next.evaluations;
                work.represented += next.represented;
                work.accepted += next.accepted;
                work.blocked += next.blocked;
                work.arrivals += next.arrivals;
            }
            return Ok(work);
        }
        let _span = crate::profile::Span::new("schedule.total");
        self.refresh_attention();
        self.protect_attention();
        let prepare = crate::profile::Span::new("schedule.prepare");
        let turns = Turns::of(self.genesis.rules.processes.values());
        let mut queue = BTreeSet::new();
        for p in self.genesis.rules.processes.values() {
            if let Some(due) = self.next_due(p, self.state.tick)
                && due <= end
            {
                queue.insert((due, p.priority, turns.rank(p, due), p.id.clone()));
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
        self.far_food = self.far_food_table();
        // The index is kept across advances (E1c): reconciled with the world
        // as this advance finds it, or built afresh where it no longer answers.
        match &mut self.filed {
            Some(f) if f.answers(&self.genesis, &self.state) => f.reconcile(&self.state, &self.genesis.rules),
            _ => self.filed = Filed::build(&self.genesis, &self.state),
        }
        self.journeys = (!self.state.movement.immediate())
            .then(|| crate::journey::Journeys::new(&self.state.population));
        let mut work = Work::default();
        self.chosen.clear();
        drop(prepare);
        let done = self.scheduled(queue, &turns, end, &mut work);
        let settle = crate::profile::Span::new("schedule.settle");
        self.targets = None;
        self.far_food = None;
        self.journeys = None;
        match (&done, &mut self.filed) {
            (Ok(()), Some(f)) => f.settle(&self.state),
            _ => self.filed = None,
        }
        self.pass = None;
        self.frame = None;
        self.chosen.clear();
        done?;
        self.state.tick = end;
        self.refresh_attention();
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
        drop(settle);
        // Outside the near tier, members leave the physical path (E5) and
        // fold into bands (E4); posed members whose matter moved them off
        // balance re-settle (D38).
        { let _s = crate::profile::Span::new("spatial.arrive"); self.arrive_travellers(); }
        self.unplace_far();
        { let _s = crate::profile::Span::new("spatial.draw"); self.draw_into_frames(); }
        { let _s = crate::profile::Span::new("spatial.settle"); self.settle_drifted(); }
        self.fold_far();
        Ok(work)
    }

    /// Runs every due process in order of due tick, priority, turn and identity.
    fn scheduled(
        &mut self,
        mut queue: BTreeSet<(Tick, i32, u64, Key)>,
        turns: &Turns,
        end: Tick,
        work: &mut Work,
    ) -> Result<()> {
        loop {
            let next = queue.first().map_or(end, |(tick, ..)| *tick);
            {
                let _s = crate::profile::Span::new("pass.arrivals");
                self.arrive_all_until(next, work)?;
            }
            let Some((due, priority, _, id)) = queue.pop_first() else { break; };
            self.state.tick = due;
            let ripen = crate::profile::Span::new("pass.ripen");
            self.refresh_attention();
            if let Some(filed) = &mut self.filed {
                filed.ripen(&self.state.population, &World::of(&self.state, &self.genesis.rules));
            }
            drop(ripen);
            let tallied = crate::profile::summing().then_some((work.evaluations, work.accepted));
            let genesis = std::sync::Arc::clone(&self.genesis);
            let process = &genesis.rules.processes[&id];
            let _process_span = crate::profile::Span::new(&process.id);
            let gates = self.filed.as_ref().and_then(|f| f.gates(&id)).cloned();
            let pass = Pass::new(&self.state.population);
            let bound = pass.bound;
            self.pass = Some(pass);
            // A pass keeps its start only where one act could read what
            // another writes (ruling 454).
            self.frame = framed(process).then(|| Frame::starting(self.state.notes.len()));
            self.chosen.begin_pass();
            self.diagnostic_birth_opportunities(process, |born| self.due_for(process, born));
            #[cfg(test)]
            self.record_food_pass(&id);
            if process.takes_shared() {
                let label = crate::profile::summing().then(|| format!("{id} [plan]"));
                let _s = label.as_deref().map(crate::profile::Span::new);
                self.plan(process, &id, gates.as_ref(), bound);
                self.geometry_ready()?;
            }
            let mut from = 0;
            while let Some(found) = self
                .next_group(process, gates.is_some(), from)
                .filter(|f| *f < bound)
            {
                let pass = self.pass.as_ref().expect("a pass under way");
                let (first, count) = pass.origin(&self.state.population, found);
                self.passed_by(process, gates.as_ref(), from, first);
                from = first + count;
                self.visit(process, &id, gates.as_ref(), (first, count), work)?;
            }
            self.passed_by(process, gates.as_ref(), from, bound);
            if let Some((evaluations, accepted)) = tallied {
                crate::profile::tally(&format!("{id} #evaluated"), work.evaluations - evaluations);
                crate::profile::tally(&format!("{id} #accepted"), work.accepted - accepted);
            }
            self.pass = None;
            self.frame = None;
            if let Some(next) = self.next_due(process, due).filter(|next| *next <= end) {
                queue.insert((next, priority, turns.rank(process, next), id));
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
            .next_group(process, gates.is_some(), from)
            .filter(|f| *f < bound)
        {
            let count = self.state.population.groups[&first].count;
            from = first + count;
            let Some((calls, multiplicity)) = self.calls(process, first, count) else {
                continue;
            };
            let (start, step, calls, multiplicity) = self.due(process, first, calls, multiplicity, count);
            for offset in (start..calls).step_by(step) {
                let actor = first + offset;
                let entity = self.state.population.get(actor);
                if entity.is_none_or(|e| {
                    gates.is_some_and(|g| !g.open(e, &World::of(&self.state, &self.genesis.rules)))
                }) {
                    continue;
                }
                let deliberated = self.deliberate(actor, process);
                if matches!(deliberated, Deliberated::Foregone) {
                    continue;
                }
                let chosen = deliberated.target();
                let target = chosen.or_else(|| self.choose_target(actor, process));
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
            crate::flows::Holder::Entity(id) => population.get(id).map(crate::anatomy::books),
            crate::flows::Holder::Part(id, part) => population
                .get(id)
                .and_then(|e| e.parts.get(&part).map(|p| p.matter.clone())),
            crate::flows::Holder::Dev => None,
            crate::flows::Holder::ProjectReservation(id) => self.state.projects.get(id).map(|p| p.reserved.clone()),
            crate::flows::Holder::ProjectConstruction(id) => self.state.projects.get(id).map(|p| p.construction.clone()),
        };
        let mut frame = self.frame.take().expect("a pass under way");
        frame.share(plans, ground, rules);
        self.frame = Some(frame);
    }

    /// Whether any lineage decides less often than every tick (D15).
    fn paced(&self) -> bool {
        self.state.lineages.values().any(|l| l.decides.is_some_and(|p| p > 1))
    }

    /// Which of a group's members decide this tick (D15, D16, case 1076):
    /// a choice or a move runs for a member only every `decides` ticks of its
    /// lineage, offset by the member's own identity, so a kind's decisions
    /// spread across the ticks and the execution mode never changes who
    /// chooses. A paced group is evaluated member by member, only those due:
    /// the first offset, the stride, the calls and what each stands for.
    fn due(&self, process: &Process, first: Id, calls: u64, multiplicity: u64, count: u64)
        -> (u64, usize, u64, u64) {
        if process.causation != Causation::Choice {
            return (0, 1, calls, multiplicity);
        }
        let lineage = self.state.population.groups.get(&first).map(|g| &g.entity.lineage);
        let period = lineage.and_then(|l| self.state.lineages.get(l)).and_then(|l| l.decides).unwrap_or(1);
        if period <= 1 {
            return (0, 1, calls, multiplicity);
        }
        let start = (period - (self.state.tick + first) % period) % period;
        (start, usize::try_from(period).unwrap_or(usize::MAX), count, 1)
    }

    /// Whether `p` falls due at this tick for a body born at `born` (1104):
    /// a body's periodic processes run on its own clock, offset by its birth,
    /// so members do not pulse together. The world's own processes keep the
    /// world's clock.
    pub(crate) fn due_for(&self, p: &Process, born: Tick) -> bool {
        let tick = self.state.tick;
        match p.period {
            None => true,
            Some(period) if period <= 1 || p.causation == Causation::Agentless => tick % period.max(1) == 0,
            Some(period) => tick > born && (tick - born) % period == 0,
        }
    }

    /// The first tick after `after` at which `p` falls due for some living
    /// body, by each body's own clock (1104).
    fn next_due(&self, p: &Process, after: Tick) -> Option<Tick> {
        let period = p.period?.max(1);
        if period == 1 || p.causation == Causation::Agentless {
            return after.checked_add(period - after % period);
        }
        let next = (after + 1) % period;
        let wait = self.state.population.groups.values()
            .filter(|g| g.entity.alive)
            .map(|g| (g.entity.born % period + period - next) % period)
            .min()?;
        after.checked_add(1 + wait)
    }

    /// How a group is evaluated: the calls, each standing for some members,
    /// or none when the process passes it by whole.
    fn calls(&self, process: &Process, first: Id, count: u64) -> Option<(u64, u64)> {
        let entity = self.body_at_start(first)?;
        if !entity.alive || !self.due_for(process, entity.born) {
            return None;
        }
        if self.genesis.rules.passes_dormant(process).is_some_and(|key| entity.traits.contains(key)) {
            return None;
        }
        if process.causation == Causation::Agentless {
            if entity.kingdom != "kingdom:world" {
                return None;
            }
        } else if entity.method == Method::Inert {
            return None;
        }
        if process.need_account.as_ref().is_some_and(|a| {
            crate::anatomy::held(entity, &self.genesis.rules, a) >= process.need_below
        }) {
            return None;
        }
        let bulk = self.mode == Execution::Grouped && process.bulk_safe()
            && !self.state.attention.foreground(first, count);
        Some(if bulk { (1, count) } else { (count, 1) })
    }

    /// The least stored group from `from` on that could match: any group,
    /// for a process without gates, or else one filed as ready for it. An
    /// agentless process acts only for world bodies (`calls`), whose kingdom
    /// no act changes, so it goes past every other group (EX4).
    fn next_group(&self, process: &Process, gated: bool, from: Id) -> Option<Id> {
        if gated {
            let filed = self.filed.as_ref().expect("groups filed for the advance");
            return filed.next(&process.id, from);
        }
        let mut groups = self.state.population.groups.range(from..);
        if process.causation == Causation::Agentless {
            return groups.find(|(_, g)| g.entity.kingdom == "kingdom:world").map(|(&f, _)| f);
        }
        groups.next().map(|(&f, _)| f)
    }

    /// Debug builds check the filing against a scan: none of the groups a
    /// pass went past, from `from` up to `until`, was due for it, nor for
    /// an agentless pass a world body.
    fn passed_by(&self, process: &Process, gates: Option<&Gates>, from: Id, until: Id) {
        #[cfg(debug_assertions)]
        for (first, g) in self.state.population.groups.range(from..until.max(from)) {
            let world = World::of(&self.state, &self.genesis.rules);
            let closed = match gates {
                Some(gates) => !gates.due(*first, g.count, &g.entity, &world),
                None => process.causation == Causation::Agentless && g.entity.kingdom != "kingdom:world",
            };
            debug_assert!(closed, "the filed pass went past group {first}");
        }
        #[cfg(not(debug_assertions))]
        let _ = (process, gates, from, until);
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
        let (start, step, calls, multiplicity) = self.due(process, first, calls, multiplicity, count);
        for offset in (start..calls).step_by(step) {
            let actor = first + offset;
            let Some(entity) = self.body_at_start(actor) else {
                continue;
            };
            if gates.is_some_and(|g| !g.open(entity, &World::of(&self.state, &self.genesis.rules))) {
                continue;
            }
            // A deliberative critter takes only the act it chose (683).
            let deliberated = self.deliberate(actor, process);
            if matches!(deliberated, Deliberated::Foregone) {
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
            let answering = match &deliberated {
                Deliberated::Chosen { answers, .. } if !answers.is_empty() => Some(answers),
                _ => None,
            };
            let learning = matches!(deliberated, Deliberated::Chosen { .. }) && self.learns(actor);
            let before = (answering.is_some() || learning).then(|| self.wellbeing(actor));
            let planned = self.frame.as_ref().and_then(|f| f.planned.as_ref());
            let (target, accepted) = match planned.map(|p| p.get(&actor).cloned()) {
                None if self.risk_misses(actor, process) => (None, false),
                None => {
                    let chosen = deliberated.target();
                    let target = chosen.or_else(|| self.choose_target(actor, process));
                    (
                        target,
                        self.scheduled_apply(actor, target, id, multiplicity).accepted(),
                    )
                },
                Some(None) => (None, false),
                Some(Some(plan)) => (
                    plan.target,
                    self.act(actor, id, multiplicity, plan).accepted(),
                ),
            };
            self.geometry_ready()?;
            #[cfg(test)]
            self.record_food_outcome(actor, id, accepted);
            if let (Some(answers), Some(before)) = (answering, before) {
                self.answered(actor, id, answers, before, accepted);
            }
            // A member's own delta learns from every act its policy chose (M3.6).
            if let (true, Some(before)) = (learning, before) {
                let served = self.served_by(actor, before, accepted);
                let bettered = self.bettered_by(actor, before, accepted);
                self.learn_from(actor, multiplicity, id, served, bettered);
            }
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
