// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which stored groups each due process could run for. A process's gates
//! are what it requires of its actor's own state: traits, being alive,
//! account and holdings thresholds, a live part expressing a function, and
//! an age. A group passes or fails them
//! until an act changes it, except its age, which comes due at a tick known
//! in advance (ruling 286). So each group is filed as ready for the
//! processes whose gates it passes, again whenever an act commits to it, and
//! a group still too young is kept on a timer for the tick it comes of age.

use crate::{meaning::value, population::Population, rules::*, schema::*};
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

    /// Whether the process requires nothing of its actor's own state.
    pub(crate) fn none(&self) -> bool {
        *self == Self::default()
    }

    /// Whether `e` passes every gate but its age.
    fn met(&self, e: &Entity) -> bool {
        (!self.alive || e.alive)
            && self.traits.iter().all(|t| e.traits.contains(t))
            && self
                .at_least
                .iter()
                .all(|(k, v)| value(&e.accounts, k) >= *v)
            && self.below.iter().all(|(k, v)| value(&e.accounts, k) < *v)
            && (self.holds == 0 || held(&e.accounts, &self.matter) >= u128::from(self.holds))
            && self.expresses.iter().all(|f| expressing(e, f).is_some())
    }

    /// Whether `e` passes every gate at `tick`, as the requirements read it.
    pub(crate) fn open(&self, e: &Entity, tick: Tick) -> bool {
        self.met(e) && tick.saturating_sub(e.born) >= self.age
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
    pub(crate) fn new(population: &Population, gates: BTreeMap<Key, Gates>, tick: Tick) -> Self {
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
        filed.touch(population, firsts, tick);
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
            for id in candidates {
                let g = &self.gates[&id];
                if !g.met(e) {
                    continue;
                }
                let ripe = e.born.saturating_add(g.age);
                if ripe <= tick {
                    self.file(id, first);
                } else {
                    self.timers.insert((ripe, id, first));
                }
            }
        }
    }

    fn file(&mut self, id: Key, first: Id) {
        self.ready.entry(id.clone()).or_default().insert(first);
        self.at.entry(first).or_default().insert(id);
    }

    /// Files the groups come of age by `tick` that still pass every other
    /// gate. A timer whose group has since changed is read afresh.
    pub(crate) fn ripen(&mut self, population: &Population, tick: Tick) {
        while self
            .timers
            .first()
            .is_some_and(|(when, _, _)| *when <= tick)
        {
            let (_, id, first) = self.timers.pop_first().expect("a timer is due");
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            if self.gates[&id].open(&group.entity, tick) {
                self.file(id, first);
            }
        }
    }

    /// The least stored group from `from` on that is ready for `process`.
    pub(crate) fn next(&self, process: &str, from: Id) -> Option<Id> {
        self.ready.get(process)?.range(from..).next().copied()
    }
}
