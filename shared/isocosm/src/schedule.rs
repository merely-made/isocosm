// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The clock's due processes (ruling 256), run in order of due tick,
//! priority and identity, each over the stored groups as they stood when its
//! pass began. A process visits only the groups carrying every trait it
//! requires of its actor, filed as acts commit (ruling 258): any other group
//! would be blocked, so passing it by changes nothing but the count, and the
//! operation budget counts only the evaluations that run (ruling 259).

use crate::{
    Result,
    population::Population,
    rules::*,
    schema::*,
    simulation::{Execution, Outcome, Simulation, Work},
};
use std::collections::{BTreeMap, BTreeSet};

/// The traits a process requires of its actor.
pub(crate) fn required(p: &Process) -> BTreeSet<Key> {
    p.requires
        .iter()
        .filter_map(|q| match q {
            Query::Trait {
                who: Binding::Actor,
                key,
            } => Some(key.clone()),
            _ => None,
        })
        .collect()
}

pub(crate) fn carries(e: &Entity, traits: &BTreeSet<Key>) -> bool {
    traits.iter().all(|t| e.traits.contains(t))
}

/// Stored groups by first identity, filed under each trait a scheduled
/// process requires, kept only during an advance.
#[derive(Clone, Debug, Default)]
pub(crate) struct Filed {
    wanted: BTreeSet<Key>,
    by_trait: BTreeMap<Key, BTreeSet<Id>>,
    at: BTreeMap<Id, Vec<Key>>,
}

impl Filed {
    fn new(population: &Population, wanted: BTreeSet<Key>) -> Self {
        let mut filed = Self {
            wanted,
            ..Self::default()
        };
        let firsts: Vec<Id> = population.groups.keys().copied().collect();
        filed.touch(population, firsts);
        filed
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there.
    pub(crate) fn touch(&mut self, population: &Population, firsts: impl IntoIterator<Item = Id>) {
        for first in firsts {
            for key in self.at.remove(&first).unwrap_or_default() {
                if let Some(set) = self.by_trait.get_mut(&key) {
                    set.remove(&first);
                    if set.is_empty() {
                        self.by_trait.remove(&key);
                    }
                }
            }
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            let keys: Vec<Key> = group
                .entity
                .traits
                .intersection(&self.wanted)
                .cloned()
                .collect();
            for key in &keys {
                self.by_trait.entry(key.clone()).or_default().insert(first);
            }
            if !keys.is_empty() {
                self.at.insert(first, keys);
            }
        }
    }

    /// The required trait fewest groups carry now. Any one of them would do,
    /// since a group carrying them all is filed under each.
    fn rarest<'a>(&self, traits: &'a BTreeSet<Key>) -> Option<&'a Key> {
        traits
            .iter()
            .min_by_key(|t| self.by_trait.get(*t).map_or(0, BTreeSet::len))
    }

    /// The least stored group from `from` on that carries every one of
    /// `traits`, looked for among those filed under `under`.
    fn next(
        &self,
        population: &Population,
        under: &Key,
        traits: &BTreeSet<Key>,
        from: Id,
    ) -> Option<Id> {
        self.by_trait
            .get(under)?
            .range(from..)
            .copied()
            .find(|first| carries(&population.groups[first].entity, traits))
    }
}

/// A pass visits the groups stored when it began. An act that splits one
/// leaves pieces, so the pass keeps each split group's span and still
/// visits it whole, as a scan over the groups it began with would.
#[derive(Clone, Debug)]
pub(crate) struct Pass {
    /// Identities from here on were born during the pass.
    bound: Id,
    split: BTreeMap<Id, u64>,
    /// Debug builds keep every span, to check each one the pass reads.
    #[cfg(debug_assertions)]
    began: BTreeMap<Id, u64>,
}

impl Pass {
    fn new(population: &Population) -> Self {
        Self {
            bound: population.next_id,
            split: BTreeMap::new(),
            #[cfg(debug_assertions)]
            began: population
                .groups
                .iter()
                .map(|(&first, g)| (first, g.count))
                .collect(),
        }
    }

    fn span(&self, id: Id) -> Option<(Id, u64)> {
        let (&first, &count) = self.split.range(..=id).next_back()?;
        (id < first + count).then_some((first, count))
    }

    /// Keeps the span of the group holding `id`, before an act lifts `id`
    /// out of it.
    pub(crate) fn lifting(&mut self, population: &Population, id: Id) {
        if id >= self.bound || self.span(id).is_some() {
            return;
        }
        if let Some((&first, group)) = population.groups.range(..=id).next_back()
            && id < first + group.count
            && group.count > 1
        {
            self.split.insert(first, group.count);
        }
    }

    /// The span, when the pass began, of the group now starting at `first`.
    fn origin(&self, population: &Population, first: Id) -> (Id, u64) {
        let span = self
            .span(first)
            .unwrap_or_else(|| (first, population.groups[&first].count));
        #[cfg(debug_assertions)]
        {
            let began = self.began.range(..=first).next_back();
            let began = began.map(|(&f, &c)| (f, c));
            debug_assert_eq!(Some(span), began, "the pass lost a group's span");
        }
        span
    }
}

impl Simulation {
    pub(crate) fn advance_to(&mut self, end: Tick) -> Result<Work> {
        let mut queue = BTreeSet::new();
        let mut wanted = BTreeSet::new();
        for p in self.genesis.rules.processes.values() {
            if let Some(period) = p.period
                && let Some(due) = self
                    .state
                    .tick
                    .checked_add(period - self.state.tick % period)
                && due <= end
            {
                queue.insert((due, p.priority, p.id.clone()));
                wanted.extend(required(p));
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
        if !wanted.is_empty() {
            self.filed = Some(Filed::new(&self.state.population, wanted));
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
            let genesis = std::sync::Arc::clone(&self.genesis);
            let process = &genesis.rules.processes[&id];
            let traits = required(process);
            let under = match &self.filed {
                Some(filed) => filed.rarest(&traits).cloned(),
                None => None,
            };
            let pass = Pass::new(&self.state.population);
            let bound = pass.bound;
            self.pass = Some(pass);
            let mut from = 0;
            while let Some(found) = self
                .next_group(under.as_ref(), &traits, from)
                .filter(|f| *f < bound)
            {
                let pass = self.pass.as_ref().expect("a pass under way");
                let (first, count) = pass.origin(&self.state.population, found);
                self.passed_by(&traits, from, first);
                from = first + count;
                self.visit(process, &id, &traits, (first, count), work)?;
            }
            self.passed_by(&traits, from, bound);
            self.pass = None;
            let next = due.checked_add(process.period.unwrap());
            if let Some(next) = next.filter(|next| *next <= end) {
                queue.insert((next, priority, id));
            }
        }
        Ok(())
    }

    /// The least stored group from `from` on that could match: any group,
    /// for a process requiring no trait, or else one filed `under` a trait
    /// it requires that carries them all.
    fn next_group(&self, under: Option<&Key>, traits: &BTreeSet<Key>, from: Id) -> Option<Id> {
        let population = &self.state.population;
        match under {
            None => population.groups.range(from..).next().map(|(&f, _)| f),
            Some(key) => {
                let filed = self.filed.as_ref().expect("groups filed for the advance");
                filed.next(population, key, traits, from)
            },
        }
    }

    /// Debug builds check the filing against a scan: none of the groups a
    /// pass went past, from `from` up to `until`, carried the traits.
    fn passed_by(&self, traits: &BTreeSet<Key>, from: Id, until: Id) {
        #[cfg(debug_assertions)]
        for (first, g) in self.state.population.groups.range(from..until.max(from)) {
            debug_assert!(
                !carries(&g.entity, traits),
                "the filed pass went past group {first}"
            );
        }
        #[cfg(not(debug_assertions))]
        let _ = (traits, from, until);
    }

    /// One group of the pass, as a scan visits it: its gates read at its
    /// first member, then each member, or the whole cohort at once, and a
    /// member without a required trait is not evaluated.
    fn visit(
        &mut self,
        process: &Process,
        id: &str,
        traits: &BTreeSet<Key>,
        (first, count): (Id, u64),
        work: &mut Work,
    ) -> Result<()> {
        let Some(entity) = self.state.population.get(first) else {
            return Ok(());
        };
        if !entity.alive {
            return Ok(());
        }
        if process.shape == Shape::Agentless {
            if entity.kingdom != "kingdom:world" {
                return Ok(());
            }
        } else if entity.method == Method::Inert {
            return Ok(());
        }
        if process
            .need_account
            .as_ref()
            .is_some_and(|a| entity.accounts.get(a).copied().unwrap_or(0) >= process.need_below)
        {
            return Ok(());
        }
        let bulk = self.mode == Execution::Grouped && process.bulk_safe();
        let (calls, multiplicity) = if bulk { (1, count) } else { (count, 1) };
        for offset in 0..calls {
            let actor = first + offset;
            let entity = self.state.population.get(actor);
            if !entity.is_some_and(|e| carries(e, traits)) {
                continue;
            }
            if work.evaluations >= self.genesis.rules.limits.events_per_advance as u64 {
                return Err(
                    "advance exceeds configured operation budget; use shorter advances".into(),
                );
            }
            let target = self.choose_target(actor, process);
            let r = self.apply(actor, target, id, None, multiplicity);
            work.evaluations += 1;
            work.represented += multiplicity;
            if matches!(r.outcome, Outcome::Accepted | Outcome::RiskOutcome) {
                work.accepted += multiplicity;
            } else {
                work.blocked += multiplicity;
            }
        }
        Ok(())
    }
}
