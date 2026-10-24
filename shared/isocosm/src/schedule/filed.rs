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

use crate::{population::Population, rules::*, schema::*, simulation::State};
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
    /// A site's conditions, read as the act reads them.
    reads: Vec<Query>,
    /// A healing reading of the actor at least one, which needs a lost cell
    /// it recorded the loss of (`harm::healing::next`); filing asks only for
    /// that cell, the act reads the whole reading.
    healable: bool,
    /// A chosen process is read by every deliberative member wherever it
    /// lives and whatever its body or site reads: its choice for the tick is
    /// made at the first choice pass that visits it (683, 813), so passing it
    /// by would move that choice.
    chosen: bool,
}

/// What gates read beyond a body: the tick, the rules, the sites bodies
/// reside at and the lineages a reading consults.
pub(crate) struct World<'a> {
    pub(crate) tick: Tick,
    pub(crate) rules: &'a Rules,
    pub(crate) sites: &'a BTreeMap<Id, Site>,
    pub(crate) lineages: &'a BTreeMap<Key, Lineage>,
}

impl<'a> World<'a> {
    pub(crate) fn of(state: &'a State, rules: &'a Rules) -> Self {
        Self::at(state, rules, state.tick)
    }
    pub(crate) fn at(state: &'a State, rules: &'a Rules, tick: Tick) -> Self {
        Self { tick, rules, sites: &state.sites, lineages: &state.lineages }
    }
}

/// One body as its gates read it, each holding worked out once.
struct Reading<'a> {
    e: &'a Entity,
    world: &'a World<'a>,
    held: BTreeMap<&'a str, u64>,
    books: Option<Ledger>,
}

impl<'a> Reading<'a> {
    fn new(e: &'a Entity, world: &'a World<'a>) -> Self {
        Self { e, world, held: BTreeMap::new(), books: None }
    }
    fn held(&mut self, key: &'a str) -> u64 {
        let (e, rules) = (self.e, self.world.rules);
        *self.held.entry(key).or_insert_with(|| crate::anatomy::held(e, rules, key))
    }
    fn books(&mut self) -> &Ledger {
        let e = self.e;
        self.books.get_or_insert_with(|| crate::anatomy::books(e))
    }
    /// Whether a requirement holds as the act's query reads it: the body,
    /// the site it resides at, the tick and the lineages; no part, target
    /// or relation, which a gated requirement never reads.
    fn holds(&self, q: &Query) -> bool {
        let w = self.world;
        let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
        let scene = crate::meaning::Scene {
            actor: Some(self.e),
            target: crate::meaning::Named::Unnamed,
            part: None,
            site: self.e.resident_site().and_then(|s| w.sites.get(&s)),
            tick: w.tick,
            related: &related,
            rules: w.rules,
            lineages: Some(w.lineages),
        };
        crate::meaning::read(q, &scene).is_ok_and(|(yes, _)| yes)
    }
}

/// Whether a requirement asks that the actor's healing reading come to at
/// least one, as rest's does.
fn heals(q: &Query) -> bool {
    use crate::rules::Reading as R;
    match q {
        Query::Computed(Expr::AtLeast(read, at_least)) => {
            matches!(read.as_ref(), Expr::Read(R::Healing { who: Binding::Actor, .. }))
                && matches!(at_least.as_ref(), Expr::Const(k) if *k >= 1)
        },
        _ => false,
    }
}

/// Whether a body has a lost cell it recorded the loss of, without which
/// no healing reading of it is more than nought.
fn healable(e: &Entity) -> bool {
    e.alive && e.parts.values().any(|p| p.lost.iter().any(|c| p.losses.contains_key(c)))
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

    /// The requirements a gate can read as the act does, where filing stays
    /// true through the pass: a process without a target, whose acts change
    /// only their own actor; a site's condition only where the process
    /// shifts none and every commit lands as staged (no spread or custody).
    pub(crate) fn reading(mut self, p: &Process, rules: &Rules) -> Self {
        if p.target.is_some() {
            return self;
        }
        let effects = || p.commitments.iter().chain(&p.effects).chain(p.risk.iter().flat_map(|r| &r.effects));
        let shifts = effects().any(|e| {
            matches!(e, Effect::Condition { .. }) || e.branches().any(|b| matches!(b, Effect::Condition { .. }))
        });
        let conditions = !shifts && rules.spread_capacity.is_none() && rules.item_custody.is_none();
        let condition = |q: &&Query| conditions && matches!(q, Query::Condition { .. });
        self.reads = p.requires.iter().filter(condition).cloned().collect();
        self.healable = p.requires.iter().any(heals);
        if !self.reads.is_empty() || self.healable {
            self.chosen |= p.causation == Causation::Choice;
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
        let chooses = self.chosen && e.method == Method::Deliberative;
        let resident = |sites: &BTreeSet<Id>| e.resident_site().is_some_and(|s| sites.contains(&s));
        (!self.alive || e.alive)
            && self.traits.iter().all(|t| e.traits.contains(t))
            && (chooses || self.resides.as_ref().is_none_or(resident))
            && self.at_least.iter().all(|(k, v)| r.held(k) >= *v)
            && self.below.iter().all(|(k, v)| r.held(k) < *v)
            && (self.holds == 0 || held(r.books(), &self.matter) >= u128::from(self.holds))
            && self.expresses.iter().all(|f| expressing(e, f).is_some())
            && (chooses || !self.healable || healable(e))
            && (chooses || self.reads.iter().all(|q| r.holds(q)))
    }

    /// Whether `e` passes every gate at the world's tick, as the
    /// requirements read it.
    pub(crate) fn open(&self, e: &Entity, world: &World) -> bool {
        self.met(&mut Reading::new(e, world)) && world.tick.saturating_sub(e.born) >= self.age
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
    /// Groups by the site they reside at, filed again when its conditions
    /// change.
    residents: BTreeMap<Id, BTreeSet<Id>>,
    resides_at: BTreeMap<Id, Id>,
}

impl Filed {
    pub(crate) fn new(
        population: &Population,
        gates: BTreeMap<Key, Gates>,
        world: &World,
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
        filed.touch(population, firsts, world);
        filed
    }

    /// Files again every group residing at `site`, after its conditions
    /// changed.
    pub(crate) fn touch_site(&mut self, population: &Population, site: Id, world: &World) {
        let firsts: Vec<Id> = self.residents.get(&site).into_iter().flatten().copied().collect();
        self.touch(population, firsts, world);
    }

    pub(crate) fn gates(&self, process: &str) -> Option<&Gates> {
        self.gates.get(process)
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there, as it stands at the world's tick.
    pub(crate) fn touch(
        &mut self,
        population: &Population,
        firsts: impl IntoIterator<Item = Id>,
        world: &World,
    ) {
        let tick = world.tick;
        for first in firsts {
            for id in self.at.remove(&first).unwrap_or_default() {
                if let Some(set) = self.ready.get_mut(&id) {
                    set.remove(&first);
                }
            }
            if let Some(site) = self.resides_at.remove(&first)
                && let Some(set) = self.residents.get_mut(&site)
            {
                set.remove(&first);
            }
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            let e = &group.entity;
            if let Some(site) = e.resident_site() {
                self.residents.entry(site).or_default().insert(first);
                self.resides_at.insert(first, site);
            }
            let mut candidates: BTreeSet<Key> = self.untraited.clone();
            for t in &e.traits {
                if let Some(ids) = self.by_trait.get(t) {
                    candidates.extend(ids.iter().cloned());
                }
            }
            let mut read = Reading::new(e, world);
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
    pub(crate) fn ripen(&mut self, population: &Population, world: &World) {
        while self
            .timers
            .first()
            .is_some_and(|(when, _, _)| *when <= world.tick)
        {
            let (_, id, first) = self.timers.pop_first().expect("a timer is due");
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            if self.gates[&id].open(&group.entity, world) {
                self.file(id, first);
            }
        }
    }

    /// The least stored group from `from` on that is ready for `process`.
    pub(crate) fn next(&self, process: &str, from: Id) -> Option<Id> {
        self.ready.get(process)?.range(from..).next().copied()
    }
}
