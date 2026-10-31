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
//! The index is kept across advances (`kept`).

mod kept;

use crate::{population::Population, rules::*, schema::*, simulation::State};
use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, Weak}};

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

    /// Whether what an act changed of a body can move this gate's outcome.
    fn moved_by(&self, c: &Changed) -> bool {
        c.whole
            || (c.residence && (self.resides.is_some() || !self.reads.is_empty()))
            || self.at_least.iter().chain(&self.below).any(|(k, _)| c.keys.contains(k))
            || (self.holds > 0 && c.keys.iter().any(|k| self.matter.contains(k)))
    }

    /// Whether `e` passes every gate at the world's tick, as the
    /// requirements read it.
    pub(crate) fn open(&self, e: &Entity, world: &World) -> bool {
        self.met(&mut Reading::new(e, world)) && world.tick.saturating_sub(e.born) >= self.age
    }
}

/// A set of gated processes by their number in the index.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Bits(Vec<u64>);

impl Bits {
    fn with(n: usize) -> Self {
        Self(vec![0; n.div_ceil(64)])
    }
    fn insert(&mut self, i: usize) {
        self.0[i / 64] |= 1 << (i % 64);
    }
    fn remove(&mut self, i: usize) {
        self.0[i / 64] &= !(1 << (i % 64));
    }
    fn contains(&self, i: usize) -> bool {
        self.0[i / 64] & (1 << (i % 64)) != 0
    }
    fn union(&mut self, other: &Self) {
        for (a, b) in self.0.iter_mut().zip(&other.0) {
            *a |= b;
        }
    }
    fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.0.iter().enumerate().flat_map(|(w, bits)| {
            let mut left = *bits;
            std::iter::from_fn(move || {
                if left == 0 {
                    return None;
                }
                let at = left.trailing_zeros() as usize;
                left &= left - 1;
                Some(w * 64 + at)
            })
        })
    }
}

/// Stored groups by first identity, filed as ready for each gated process
/// on a period, kept across advances. Processes are numbered, so that a
/// group filed again as it was changes nothing in the index.
#[derive(Clone, Debug, Default)]
pub(crate) struct Filed {
    names: BTreeMap<Key, usize>,
    gates: Vec<Gates>,
    by_trait: BTreeMap<Key, Bits>,
    untraited: Bits,
    ready: Vec<BTreeSet<Id>>,
    at: BTreeMap<Id, Bits>,
    /// Groups passing every gate of a process but their age, by the tick
    /// they come of age.
    timers: BTreeSet<(Tick, usize, Id)>,
    /// Groups by the site they reside at, filed again when its conditions
    /// change.
    residents: BTreeMap<Id, BTreeSet<Id>>,
    resides_at: BTreeMap<Id, Id>,
    /// Each group's filing: the gates its body passed but for age, and the
    /// body it was read from.
    filings: BTreeMap<Id, Filing>,
    basis: kept::Basis,
}

/// The gates a body passed but for age, and that body. The weak handle
/// keeps its allocation from reuse, and any write to the body after it
/// moves the body to another (`Arc::make_mut`), so the same address is the
/// same body.
#[derive(Clone, Debug)]
struct Filing {
    body: Weak<Entity>,
    passed: Bits,
}

/// What one act changed of a body, as filing reads it: the accounts it
/// wrote, its own or its parts' matter; whether its residence moved; and
/// whether anything else a gate reads did (traits, life, method, lineage,
/// age, its body's document or a part's shape), which files it afresh.
#[derive(Clone, Debug, Default)]
pub(crate) struct Changed {
    keys: BTreeSet<Key>,
    residence: bool,
    whole: bool,
}

impl Changed {
    /// What `now` changed of `was`.
    pub(crate) fn between(was: &Entity, now: &Entity) -> Self {
        let mut keys = BTreeSet::new();
        let mut ledger = |a: &Ledger, b: &Ledger| {
            for k in a.keys().chain(b.keys()) {
                if a.get(k) != b.get(k) {
                    keys.insert(k.clone());
                }
            }
        };
        ledger(&was.accounts, &now.accounts);
        let mut whole = was.parts.len() != now.parts.len();
        for (id, part) in &now.parts {
            match was.parts.get(id) {
                Some(old) if Arc::ptr_eq(old, part) => {},
                Some(old) => {
                    ledger(&old.matter, &part.matter);
                    let mut shape = Part::clone(old);
                    shape.matter = part.matter.clone();
                    whole |= shape != **part;
                },
                None => whole = true,
            }
        }
        whole |= was.traits != now.traits
            || was.alive != now.alive
            || was.method != now.method
            || was.lineage != now.lineage
            || was.born != now.born
            || was.body != now.body;
        Self { keys, residence: was.resident_site() != now.resident_site(), whole }
    }
}

/// The candidates whose gates the body read passes, but for age.
fn passing<'a>(gates: &'a [Gates], candidates: &Bits, read: &mut Reading<'a>) -> Bits {
    let mut passed = Bits::with(gates.len());
    for i in candidates.iter() {
        if gates[i].met(read) {
            passed.insert(i);
        }
    }
    passed
}

impl Filed {
    pub(crate) fn new(
        population: &Population,
        gates: BTreeMap<Key, Gates>,
        world: &World,
    ) -> Self {
        let n = gates.len();
        let mut filed = Self { untraited: Bits::with(n), ..Self::default() };
        for (i, (id, g)) in gates.into_iter().enumerate() {
            if g.traits.is_empty() {
                filed.untraited.insert(i);
            }
            for t in &g.traits {
                filed.by_trait.entry(t.clone()).or_insert_with(|| Bits::with(n)).insert(i);
            }
            filed.names.insert(id, i);
            filed.gates.push(g);
        }
        filed.ready = vec![BTreeSet::new(); n];
        let firsts: Vec<Id> = population.groups.keys().copied().collect();
        filed.touch(population, firsts, world);
        filed
    }

    /// Files again every group residing at `site`, after its conditions
    /// changed, each read afresh.
    pub(crate) fn touch_site(&mut self, population: &Population, site: Id, world: &World) {
        let firsts: Vec<Id> = self.residents.get(&site).into_iter().flatten().copied().collect();
        self.refile(population, firsts, &BTreeMap::new(), world, true);
    }

    /// Files again every group that may start at one of `firsts`, after the
    /// population changed there, as it stands at the world's tick.
    pub(crate) fn touch(&mut self, population: &Population, firsts: impl IntoIterator<Item = Id>, world: &World) {
        self.refile(population, firsts, &BTreeMap::new(), world, false);
    }

    /// The same after an act, knowing what it changed of each body it
    /// wrote: a group still on the body it was filed from keeps its filing;
    /// one whose body the act changed reads again only the gates the change
    /// can move; any other is read whole.
    pub(crate) fn touch_changed(
        &mut self,
        population: &Population,
        firsts: impl IntoIterator<Item = Id>,
        changes: &BTreeMap<Id, Changed>,
        world: &World,
    ) {
        self.refile(population, firsts, changes, world, false);
    }

    pub(crate) fn gates(&self, process: &str) -> Option<&Gates> {
        self.names.get(process).map(|&i| &self.gates[i])
    }

    fn refile(
        &mut self,
        population: &Population,
        firsts: impl IntoIterator<Item = Id>,
        changes: &BTreeMap<Id, Changed>,
        world: &World,
        afresh: bool,
    ) {
        let (tick, n) = (world.tick, self.gates.len());
        let mut firsts: Vec<Id> = firsts.into_iter().collect();
        firsts.sort_unstable();
        firsts.dedup();
        // The filings the touched groups had: a group an act lifted a member
        // out of began, with its pieces, from the filing of the group it was.
        let was: Vec<(Id, Filing)> = firsts.iter().filter_map(|f| Some((*f, self.filings.get(f)?.clone()))).collect();
        for first in firsts {
            let filed = self.at.remove(&first).unwrap_or_else(|| Bits::with(n));
            if let Some(site) = self.resides_at.remove(&first)
                && let Some(set) = self.residents.get_mut(&site)
            {
                set.remove(&first);
            }
            let Some(group) = population.groups.get(&first) else {
                for i in filed.iter() {
                    self.ready[i].remove(&first);
                }
                self.filings.remove(&first);
                continue;
            };
            let e = &group.entity;
            if let Some(site) = e.resident_site() {
                self.residents.entry(site).or_default().insert(first);
                self.resides_at.insert(first, site);
            }
            let mut candidates = self.untraited.clone();
            for t in &e.traits {
                if let Some(bits) = self.by_trait.get(t) {
                    candidates.union(bits);
                }
            }
            let gates = &self.gates;
            let mut read = Reading::new(e, world);
            let began = was.iter().rev().find(|(f, _)| *f <= first).map(|(_, filing)| filing);
            let same = |b: &Filing| std::ptr::eq(b.body.as_ptr(), Arc::as_ptr(e));
            let passed = match (began, changes.get(&first)) {
                (Some(b), _) if !afresh && same(b) => b.passed.clone(),
                (Some(b), Some(c)) if !afresh && !c.whole => {
                    let mut passed = b.passed.clone();
                    for i in candidates.iter().filter(|i| gates[*i].moved_by(c)) {
                        if gates[i].met(&mut read) {
                            passed.insert(i);
                        } else {
                            passed.remove(i);
                        }
                    }
                    passed
                },
                _ => passing(gates, &candidates, &mut read),
            };
            // Debug builds check a kept or narrowed filing against a whole one.
            #[cfg(debug_assertions)]
            debug_assert_eq!(passed, passing(gates, &candidates, &mut Reading::new(e, world)), "group {first}'s filing differs from a whole reading");
            let mut due = Bits::with(n);
            for i in passed.iter() {
                let ripe = e.born.saturating_add(gates[i].age);
                if ripe <= tick {
                    due.insert(i);
                } else {
                    self.timers.insert((ripe, i, first));
                }
            }
            if due != filed {
                for i in filed.iter().filter(|i| !due.contains(*i)) {
                    self.ready[i].remove(&first);
                }
                for i in due.iter().filter(|i| !filed.contains(*i)) {
                    self.ready[i].insert(first);
                }
            }
            self.at.insert(first, due);
            self.filings.insert(first, Filing { body: Arc::downgrade(e), passed });
        }
    }

    /// Files the groups come of age by `tick` that still pass every other
    /// gate. A timer whose group has since changed is read afresh.
    pub(crate) fn ripen(&mut self, population: &Population, world: &World) {
        let n = self.gates.len();
        while self
            .timers
            .first()
            .is_some_and(|(when, _, _)| *when <= world.tick)
        {
            let (_, i, first) = self.timers.pop_first().expect("a timer is due");
            let Some(group) = population.groups.get(&first) else {
                continue;
            };
            if self.gates[i].open(&group.entity, world) {
                self.ready[i].insert(first);
                self.at.entry(first).or_insert_with(|| Bits::with(n)).insert(i);
            }
        }
    }

    /// The least stored group from `from` on that is ready for `process`.
    pub(crate) fn next(&self, process: &str, from: Id) -> Option<Id> {
        let i = *self.names.get(process)?;
        self.ready[i].range(from..).next().copied()
    }
}
