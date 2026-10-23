// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which stored groups each due process could run for. A process's gates
//! are what it requires of its actor's own state: traits, being alive,
//! account and holdings thresholds, a live part expressing a function, an
//! age, and for a timed move, residing where one of its routes starts. A
//! group passes or fails them
//! until an act changes it, except its age, which comes due at a tick known
//! in advance (ruling 286). So each group is filed as ready for the
//! processes whose gates it passes, again whenever an act commits to it, and
//! a group still too young is kept on a timer for the tick it comes of age.
//! Filing reads each of a group's holdings once, whatever the gates asking.

use crate::{population::Population, rules::*, schema::*};
use std::collections::{BTreeMap, BTreeSet};

/// What a process requires of its actor's own state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Gates {
    traits: BTreeSet<Key>,
    alive: bool,
    at_least: Vec<(Key, u64)>,
    below: Vec<(Key, u64)>,
    /// Matter held over every matter account, and the accounts that count.
    holds: u64,
    matter: BTreeSet<Key>,
    /// Functions a live part must express (ruling 338).
    expresses: BTreeSet<Key>,
    age: Tick,
    /// For a timed move, the sites whose routes reach every place it goes.
    resides: Option<BTreeSet<Id>>,
    /// A chosen process is read by every deliberative member wherever it
    /// lives: its choice for the tick is made at the first choice pass that
    /// visits it (683, 813), so passing it by would move that choice.
    chosen: bool,
}

/// One body as its gates read it, each holding worked out once.
struct Reading<'a> {
    e: &'a Entity,
    rules: &'a Rules,
    held: BTreeMap<&'a str, u64>,
    books: Option<Ledger>,
}

impl<'a> Reading<'a> {
    fn new(e: &'a Entity, rules: &'a Rules) -> Self {
        Self { e, rules, held: BTreeMap::new(), books: None }
    }
    fn held(&mut self, key: &'a str) -> u64 {
        let (e, rules) = (self.e, self.rules);
        *self.held.entry(key).or_insert_with(|| crate::anatomy::held(e, rules, key))
    }
    fn books(&mut self) -> &Ledger {
        let e = self.e;
        self.books.get_or_insert_with(|| crate::anatomy::books(e))
    }
}

/// The matter a ledger holds in `matter`'s accounts, as `mass` reads it.
fn held(ledger: &Ledger, matter: &BTreeSet<Key>) -> u128 {
    let v = ledger.iter().filter(|(k, _)| matter.contains(*k));
    v.map(|(_, v)| u128::from(*v)).sum()
}

impl Gates {
    pub(crate) fn of(p: &Process, rules: &Rules) -> Self {
        let mut g = Self::default();
        for q in &p.requires {
            match q {
                Query::Alive(Binding::Actor) => g.alive = true,
                Query::Trait {
                    who: Binding::Actor,
                    key,
                } => {
                    g.traits.insert(key.clone());
                },
                Query::Account {
                    who: Binding::Actor,
                    key,
                    at_least,
                } => g.at_least.push((key.clone(), *at_least)),
                Query::Below {
                    who: Binding::Actor,
                    key,
                    amount,
                } => g.below.push((key.clone(), *amount)),
                Query::Age { at_least } => g.age = g.age.max(*at_least),
                Query::Holds {
                    who: Binding::Actor,
                    at_least,
                } => g.holds = g.holds.max(*at_least),
                Query::Expresses { function } => {
                    g.expresses.insert(function.clone());
                },
                _ => {},
            }
        }
        if g.holds > 0 {
            let matter = rules.accounts.iter().filter_map(|(k, kind)| match kind {
                AccountKind::Matter { .. } => Some(k.clone()),
                _ => None,
            });
            g.matter = matter.collect();
        }
        g
    }

    /// Where a timed move may start, as its route check reads it: a
    /// resident site whose routes reach each of its destinations. Routes
    /// change only when asserted, never during an advance.
    pub(crate) fn residing(mut self, p: &Process, sites: &BTreeMap<Id, Site>, immediate: bool) -> Self {
        let to: Vec<Id> = p.commitments.iter().chain(&p.effects).filter_map(|e| match e {
            Effect::Move { destination } | Effect::MovePaced { destination, .. } => Some(*destination),
            _ => None,
        }).collect();
        if !immediate && !to.is_empty() {
            let reaches = |s: &Site| to.iter().all(|t| s.routes.iter().any(|r| r.to == *t));
            self.resides = Some(sites.iter().filter(|(_, s)| reaches(s)).map(|(id, _)| *id).collect());
            self.chosen = p.causation == Causation::Choice;
        }
        self
    }

    /// Whether the process requires nothing of its actor's own state.
    pub(crate) fn none(&self) -> bool {
        *self == Self::default()
    }

    /// Whether the body read passes every gate but its age.
    fn met<'a>(&'a self, r: &mut Reading<'a>) -> bool {
        let e = r.e;
        let resident = |sites: &BTreeSet<Id>| {
            (self.chosen && e.method == Method::Deliberative)
                || e.resident_site().is_some_and(|s| sites.contains(&s))
        };
        (!self.alive || e.alive)
            && self.traits.iter().all(|t| e.traits.contains(t))
            && self.resides.as_ref().is_none_or(resident)
            && self.at_least.iter().all(|(k, v)| r.held(k) >= *v)
            && self.below.iter().all(|(k, v)| r.held(k) < *v)
            && (self.holds == 0 || held(r.books(), &self.matter) >= u128::from(self.holds))
            && self.expresses.iter().all(|f| expressing(e, f).is_some())
    }

    /// Whether `e` passes every gate at `tick`, as the requirements read it.
    pub(crate) fn open(&self, e: &Entity, tick: Tick, rules: &Rules) -> bool {
        self.met(&mut Reading::new(e, rules)) && tick.saturating_sub(e.born) >= self.age
    }
}

/// Stored groups by first identity, filed as ready for each gated due
/// process, kept only during an advance.
#[derive(Clone, Debug, Default)]
pub(crate) struct Filed {
    gates: BTreeMap<Key, Gates>,
    by_trait: BTreeMap<Key, BTreeSet<Key>>,
    untraited: BTreeSet<Key>,
    ready: BTreeMap<Key, BTreeSet<Id>>,
    at: BTreeMap<Id, BTreeSet<Key>>,
    /// Groups passing every gate of a process but their age, by the tick
    /// they come of age.
    timers: BTreeSet<(Tick, Key, Id)>,
}

impl Filed {
    pub(crate) fn new(
        population: &Population,
        gates: BTreeMap<Key, Gates>,
        tick: Tick,
        rules: &Rules,
    ) -> Self {
        let mut filed = Self::default();
        for (id, g) in &gates {
            if g.traits.is_empty() {
                filed.untraited.insert(id.clone());
            }
            for t in &g.traits {
                let processes = filed.by_trait.entry(t.clone()).or_default();
                processes.insert(id.clone());
            }
        }
        filed.gates = gates;
        let firsts: Vec<Id> = population.groups.keys().copied().collect();
        filed.touch(population, firsts, tick, rules);
        filed
    }

    pub(crate) fn gates(&self, process: &str) -> Option<&Gates> {
        self.gates.get(process)
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there, as it stands at `tick`.
    pub(crate) fn touch(
        &mut self,
        population: &Population,
        firsts: impl IntoIterator<Item = Id>,
        tick: Tick,
        rules: &Rules,
    ) {
        for first in firsts {
            for id in self.at.remove(&first).unwrap_or_default() {
                if let Some(set) = self.ready.get_mut(&id) {
                    set.remove(&first);
                }
            }
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            let e = &group.entity;
            let mut candidates: BTreeSet<Key> = self.untraited.clone();
            for t in &e.traits {
                if let Some(ids) = self.by_trait.get(t) {
                    candidates.extend(ids.iter().cloned());
                }
            }
            let mut read = Reading::new(e, rules);
            let (mut due, mut later) = (vec![], vec![]);
            for id in candidates {
                let g = &self.gates[&id];
                if !g.met(&mut read) {
                    continue;
                }
                let ripe = e.born.saturating_add(g.age);
                if ripe <= tick {
                    due.push(id);
                } else {
                    later.push((ripe, id));
                }
            }
            for id in due {
                self.file(id, first);
            }
            for (ripe, id) in later {
                self.timers.insert((ripe, id, first));
            }
        }
    }

    fn file(&mut self, id: Key, first: Id) {
        self.ready.entry(id.clone()).or_default().insert(first);
        self.at.entry(first).or_default().insert(id);
    }

    /// Files the groups come of age by `tick` that still pass every other
    /// gate. A timer whose group has since changed is read afresh.
    pub(crate) fn ripen(&mut self, population: &Population, tick: Tick, rules: &Rules) {
        while self
            .timers
            .first()
            .is_some_and(|(when, _, _)| *when <= tick)
        {
            let (_, id, first) = self.timers.pop_first().expect("a timer is due");
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            if self.gates[&id].open(&group.entity, tick, rules) {
                self.file(id, first);
            }
        }
    }

    /// The least stored group from `from` on that is ready for `process`.
    pub(crate) fn next(&self, process: &str, from: Id) -> Option<Id> {
        self.ready.get(process)?.range(from..).next().copied()
    }
}
