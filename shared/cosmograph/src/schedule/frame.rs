// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a pass reads (ruling 454): the world as the pass began. Each site
//! and body an act of the pass changes is kept as it began, so the acts after
//! it read it there, and every act's writes land as what they changed. A
//! pass whose acts take from ground they share is planned first; where they
//! would take more of a site's or a prey's account than it holds, each taker
//! gets the same fraction of its take, floored, and the remainder stays
//! where it was. A prey's meals take one share of it between them.

use crate::{
    flows::Holder,
    meaning::{self, share, value},
    rules::Rules,
    schema::*,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

mod meals;
pub(crate) use meals::{MealShare, MealSpec};

/// Derivative pass-local view of a body's holdings. It is deliberately reset
/// when a frame is cloned, since a clone may become a candidate world.
#[derive(Debug, Default)]
struct BooksCache(Mutex<BTreeMap<Id, Arc<Ledger>>>);

impl Clone for BooksCache {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl BooksCache {
    fn get_or_insert(&self, id: Id, make: impl FnOnce() -> Ledger) -> Arc<Ledger> {
        if let Some(ledger) = self
            .0
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&id)
            .cloned()
        {
            return ledger;
        }
        // Build outside the lock; callers never read a ledger while holding it.
        let built = Arc::new(make());
        let mut cache = self.0.lock().unwrap_or_else(|p| p.into_inner());
        cache.entry(id).or_insert_with(|| built.clone()).clone()
    }
}

/// Derivative pass-local plant readings (light, soil, crowding) by site,
/// and the plant lineages they count. Every act of a pass reads the world as
/// the pass began, so a site's readings are one set of numbers for the whole
/// pass. Reset when a frame is cloned, since a clone may become a candidate
/// world.
#[derive(Debug, Default)]
struct PlantCache(Mutex<PlantReadings>);

#[derive(Debug, Default)]
struct PlantReadings {
    lineages: Option<Arc<std::collections::BTreeSet<Key>>>,
    sites: BTreeMap<Id, [u64; 3]>,
}

impl Clone for PlantCache {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Derivative pass-local edible matter (`Simulation::edible_matter`) summed
/// over every admitted holder at a site, by site, feed and the one thing it
/// reads of the eater, its active intake port. Reset when a frame is cloned.
#[derive(Debug, Default)]
struct EdibleCache(Mutex<BTreeMap<EdibleKey, u128>>);

pub(crate) type EdibleKey = (Id, Key, Option<crate::process::IntakePort>);

impl Clone for EdibleCache {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Frame {
    plants: PlantCache,
    edible: EdibleCache,
    pub(crate) spatial_poses: BTreeMap<Id, Option<crate::spatial::foraging::Resident>>,
    books: BooksCache,
    notes_bound: Option<usize>,
    sites: BTreeMap<Id, Site>,
    polities: BTreeMap<Id, Option<Polity>>,
    institutions: Option<crate::institutions::Records>,
    projects: BTreeMap<crate::projects::ProjectId, Option<crate::projects::Project>>,
    project_bindings: BTreeMap<Id, Option<crate::projects::ProjectId>>,
    ownership: BTreeMap<Key, Option<crate::projects::OwnedLot>>,
    produced: BTreeMap<Key, Option<crate::projects::ProducedClaim>>,
    /// Bodies as the pass began, by their span then: a member lifted out of
    /// its group alone, a cohort written whole with its count.
    bodies: BTreeMap<Id, (u64, std::sync::Arc<Entity>)>,
    /// Changed bodies filed where they began, so an indexed target search
    /// can still find those the live index has moved elsewhere.
    pub(crate) targets: crate::targets::Targets,
    /// The acts the pass planned, by actor; none for a pass that takes
    /// nothing it shares.
    pub(crate) planned: Option<BTreeMap<Id, Planned>>,
}

/// One planned act: its target, the number it acts as, and its shares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Planned {
    pub(crate) target: Option<Id>,
    pub(crate) act: u64,
    pub(crate) shares: Shares,
}

/// What one act may take of shared ground.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Shares {
    /// Up to this much of each account a site or a target holds.
    pub(crate) accounts: BTreeMap<(Holder, Key), u64>,
    /// Ordered quotas, including empty grants, for exact Eat declarations.
    pub(crate) meals: std::collections::VecDeque<MealShare>,
}

/// What one act would take of shared ground, as planned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Demands {
    pub(crate) accounts: Vec<(Holder, Key, u64)>,
    /// Only explicit carrying joins generic part takes with bitten meals.
    pub(crate) graft_parts: std::collections::BTreeSet<Holder>,
    /// Every executed Eat, with its structural address and selected part.
    pub(crate) meals: Vec<MealSpec>,
}

/// One act the plan found, in pass order.
pub(crate) struct Plan {
    pub(crate) actor: Id,
    pub(crate) target: Option<Id>,
    pub(crate) act: u64,
    pub(crate) demands: Demands,
}

/// What a taker asking `take` gets when `wanted` in all is asked of what
/// holds `held`: all of it, or the same fraction as every taker, floored.
fn fraction(take: u64, held: u64, wanted: u128) -> u64 {
    if wanted <= u128::from(held) {
        return take;
    }
    (u128::from(take) * u128::from(held) / wanted) as u64
}

/// The matter a meal may take: the named accounts, or every matter one.
pub(crate) fn edible(ledger: &Ledger, of: &[Key]) -> Ledger {
    match of {
        [] => ledger.clone(),
        _ => of
            .iter()
            .filter_map(|k| ledger.get(k).map(|v| (k.clone(), *v)))
            .collect(),
    }
}

impl Frame {
    /// The plant lineages, made once for the pass.
    pub(crate) fn plant_lineages(
        &self,
        make: impl FnOnce() -> std::collections::BTreeSet<Key>,
    ) -> Arc<std::collections::BTreeSet<Key>> {
        let mut cache = self.plants.0.lock().unwrap_or_else(|p| p.into_inner());
        cache.lineages.get_or_insert_with(|| Arc::new(make())).clone()
    }
    /// A site's `[light, soil, crowding]` as the pass began, made once.
    pub(crate) fn plant_readings(&self, site: Id, make: impl FnOnce() -> Option<[u64; 3]>) -> Option<[u64; 3]> {
        if let Some(r) = self.plants.0.lock().unwrap_or_else(|p| p.into_inner()).sites.get(&site) {
            return Some(*r);
        }
        // Made outside the lock; nothing it reads holds this cache.
        let made = make()?;
        let mut cache = self.plants.0.lock().unwrap_or_else(|p| p.into_inner());
        Some(*cache.sites.entry(site).or_insert(made))
    }
    /// A site's edible matter for one feed and intake, made once.
    pub(crate) fn edible(&self, key: EdibleKey, make: impl FnOnce() -> u128) -> u128 {
        if let Some(v) = self.edible.0.lock().unwrap_or_else(|p| p.into_inner()).get(&key) {
            return *v;
        }
        // Made outside the lock; nothing it reads holds this cache.
        let made = make();
        *self.edible.0.lock().unwrap_or_else(|p| p.into_inner()).entry(key).or_insert(made)
    }
    #[cfg(test)]
    pub(crate) fn edible_len(&self) -> usize {
        self.edible.0.lock().unwrap_or_else(|p| p.into_inner()).len()
    }
    pub(crate) fn books(&self, id: Id, body: &Entity) -> Arc<Ledger> {
        self.books.get_or_insert(id, || crate::anatomy::books(body))
    }
    pub(crate) fn institutions(&self) -> Option<&crate::institutions::Records> {
        self.institutions.as_ref()
    }
    pub(crate) fn keep_institutions(&mut self, was: &crate::institutions::Records) {
        if self.institutions.is_none() {
            self.institutions = Some(was.clone());
        }
    }
    pub(crate) fn polity(&self, id: Id) -> Option<&Option<Polity>> {
        self.polities.get(&id)
    }
    pub(crate) fn keep_polity(&mut self, id: Id, was: Option<&Polity>) {
        self.polities.entry(id).or_insert_with(|| was.cloned());
    }
    pub(crate) fn starting(notes_bound: usize) -> Self {
        Self {
            notes_bound: Some(notes_bound),
            ..Self::default()
        }
    }
    pub(crate) fn notes_bound(&self) -> Option<usize> {
        self.notes_bound
    }
    pub(crate) fn produced_claims(&self) -> &BTreeMap<Key, Option<crate::projects::ProducedClaim>> {
        &self.produced
    }
    pub(crate) fn keep_produced(
        &mut self,
        key: &str,
        was: Option<&crate::projects::ProducedClaim>,
    ) {
        self.produced
            .entry(key.into())
            .or_insert_with(|| was.cloned());
    }
    pub(crate) fn stock_lot(&self, key: &str) -> Option<&Option<crate::projects::OwnedLot>> {
        self.ownership.get(key)
    }
    pub(crate) fn keep_stock_lot(&mut self, key: &str, was: Option<&crate::projects::OwnedLot>) {
        self.ownership
            .entry(key.into())
            .or_insert_with(|| was.cloned());
    }
    pub(crate) fn project(
        &self,
        id: crate::projects::ProjectId,
    ) -> Option<&Option<crate::projects::Project>> {
        self.projects.get(&id)
    }
    pub(crate) fn project_binding(&self, actor: Id) -> Option<Option<crate::projects::ProjectId>> {
        self.project_bindings.get(&actor).copied()
    }
    pub(crate) fn keep_project(
        &mut self,
        id: crate::projects::ProjectId,
        was: Option<&crate::projects::Project>,
    ) {
        self.projects.entry(id).or_insert_with(|| was.cloned());
    }
    pub(crate) fn keep_project_binding(
        &mut self,
        actor: Id,
        was: Option<crate::projects::ProjectId>,
    ) {
        self.project_bindings.entry(actor).or_insert(was);
    }
    pub(crate) fn site(&self, id: Id) -> Option<&Site> {
        self.sites.get(&id)
    }

    pub(crate) fn body(&self, id: Id) -> Option<&Entity> {
        self.body_shared(id).map(|e| e.as_ref())
    }

    pub(crate) fn body_shared(&self, id: Id) -> Option<&std::sync::Arc<Entity>> {
        let (&first, (count, e)) = self.bodies.range(..=id).next_back()?;
        (id < first + count).then_some(e)
    }

    /// Keeps a site as the pass began, before an act first changes it.
    pub(crate) fn keep_site(&mut self, id: Id, site: &Site) {
        self.sites.entry(id).or_insert_with(|| site.clone());
    }

    /// Keeps a body as the pass began, over the span the act writes.
    pub(crate) fn keep_body(&mut self, id: Id, span: u64, e: std::sync::Arc<Entity>) {
        if self.body(id).is_none() {
            self.targets.keep(id, &e);
            self.bodies.insert(id, (span, e));
        }
    }

    /// Whether the pass has changed this body or site already.
    pub(crate) fn changed(&self, holder: Holder) -> bool {
        match holder {
            Holder::Site(id) => self.sites.contains_key(&id),
            Holder::Entity(id) | Holder::Part(id, _) => self.body(id).is_some(),
            Holder::Dev => false,
            Holder::ProjectReservation(id) | Holder::ProjectConstruction(id) => {
                self.projects.contains_key(&id)
            },
        }
    }

    /// Shares each planned act out of what the pass began with, `ground`
    /// reading a holder's ledger then.
    pub(crate) fn share(
        &mut self,
        plans: Vec<Plan>,
        ground: impl Fn(Holder) -> Option<Ledger>,
        rules: &Rules,
    ) {
        let mut wanted: BTreeMap<(Holder, Key), u128> = BTreeMap::new();
        // Each taker's take of an account, all its demands of it together.
        let takes: Vec<BTreeMap<(Holder, Key), u64>> = plans
            .iter()
            .map(|p| {
                let mut takes = BTreeMap::new();
                for (holder, key, amount) in &p.demands.accounts {
                    let t: &mut u64 = takes.entry((*holder, key.clone())).or_default();
                    *t = t.saturating_add(*amount);
                }
                takes
            })
            .collect();
        for takes in &takes {
            for (k, amount) in takes {
                *wanted.entry(k.clone()).or_default() += u128::from(*amount);
            }
        }
        let held = |(holder, key): &(Holder, Key)| ground(*holder).map_or(0, |l| value(&l, key));
        let meals::Allocated {
            meals,
            bitten,
            graft_parts,
        } = meals::allocate(&plans, &wanted, &ground, rules);
        let mut planned = BTreeMap::new();
        for ((p, takes), meal) in plans.into_iter().zip(takes).zip(meals) {
            let accounts = takes
                .into_iter()
                .map(|(k, take)| {
                    // Explicit carrying asks exact part stocks. Bites and
                    // grafts share that one pass-start part/account budget.
                    let meals = if graft_parts.contains(&k.0) {
                        bitten.get(&k).copied().unwrap_or(0)
                    } else {
                        0
                    };
                    let shared = wanted[&k] + meals;
                    let g = fraction(take, held(&k), shared);
                    (k, g)
                })
                .collect();
            let shares = Shares {
                accounts,
                meals: meal,
            };
            planned.insert(
                p.actor,
                Planned {
                    target: p.target,
                    act: p.act,
                    shares,
                },
            );
        }
        self.planned = Some(planned);
    }
}

/// Lands an act's change of `base` into `new` on `live`, which other acts
/// of the pass may have changed since: accounts and skills by what the act
/// added or took, anything else the act changed by its value.
pub(crate) fn merge(live: &mut Entity, base: &Entity, new: Entity) {
    if *live == *base {
        *live = new;
        return;
    }
    delta(&mut live.accounts, &base.accounts, &new.accounts);
    delta(&mut live.skills, &base.skills, &new.skills);
    merge_parts(&mut live.parts, &base.parts, new.parts.clone());
    if new.body != base.body {
        live.body = new.body.clone();
    }
    for t in new.traits.difference(&base.traits) {
        live.traits.insert(t.clone());
    }
    for t in base.traits.difference(&new.traits) {
        live.traits.remove(t);
    }
    let raised = new.body_revision.saturating_sub(base.body_revision);
    live.body_revision = live.body_revision.saturating_add(raised);
    macro_rules! changed {
        ($($field:ident),*) => {$(
            if new.$field != base.$field {
                live.$field = new.$field.clone();
            }
        )*};
    }
    changed!(
        lineage,
        kingdom,
        scale,
        provenance,
        method,
        place,
        arrived,
        visits,
        journey,
        patch,
        born,
        alive,
        tenets,
        disposition,
        spent
    );
}

/// Parts landed the same way (ruling 504): each part's matter by what the
/// act added or took, anything else the act changed of it by its value. A
/// part another act of the pass removed stays removed: the act left it as
/// it found it, or it would not have fitted.
fn merge_parts(
    live: &mut BTreeMap<PartId, Arc<Part>>,
    base: &BTreeMap<PartId, Arc<Part>>,
    new: BTreeMap<PartId, Arc<Part>>,
) {
    for id in base.keys().filter(|id| !new.contains_key(*id)) {
        live.remove(id);
    }
    for (id, part) in new {
        let (Some(was), Some(now)) = (base.get(&id), live.get_mut(&id)) else {
            if !base.contains_key(&id) {
                live.insert(id, part);
            }
            continue;
        };
        // A part the act left as it found it lands as nothing.
        if Arc::ptr_eq(&part, was) {
            continue;
        }
        let (mut part, now) = (Arc::unwrap_or_clone(part), Arc::make_mut(now));
        delta(&mut now.matter, &was.matter, &part.matter);
        part.matter = std::mem::take(&mut now.matter);
        let mut kept = Part::clone(was);
        kept.matter = part.matter.clone();
        if part != kept {
            *now = part;
        } else {
            now.matter = part.matter;
        }
    }
}

/// A site's change landed the same way: accounts and conditions by what
/// the act added or took.
pub(crate) fn merge_site(live: &mut Site, base: &Site, new: Site) {
    if *live == *base {
        *live = new;
        return;
    }
    delta(&mut live.accounts, &base.accounts, &new.accounts);
    for (k, v) in &new.conditions {
        let was = base.conditions.get(k).copied().unwrap_or(0);
        if *v != was {
            *live.conditions.entry(k.clone()).or_default() += v - was;
        }
    }
    for (k, was) in &base.conditions {
        if !new.conditions.contains_key(k) {
            *live.conditions.entry(k.clone()).or_default() -= was;
        }
    }
}

/// Every key either ledger holds, once.
fn keys<'a>(base: &'a Ledger, new: &'a Ledger) -> impl Iterator<Item = &'a Key> {
    base.keys()
        .chain(new.keys().filter(|k| !base.contains_key(*k)))
}

fn delta(live: &mut Ledger, base: &Ledger, new: &Ledger) {
    for key in keys(base, new) {
        let (was, now) = (value(base, key), value(new, key));
        if was != now {
            let slot = live.entry(key.clone()).or_default();
            let landed = (u128::from(*slot) + u128::from(now)).checked_sub(u128::from(was));
            let landed = landed.expect("a pass shares out no more than its ground held");
            *slot = u64::try_from(landed).expect("checked before the act was accepted");
        }
    }
}

/// Whether landing the change of `base` into `new` on `live` keeps every
/// account whole.
pub(crate) fn fits(live: &Ledger, base: &Ledger, new: &Ledger) -> bool {
    keys(base, new).all(|key| {
        let (was, now) = (value(base, key), value(new, key));
        let landed = u128::from(value(live, key)) + u128::from(now);
        landed >= u128::from(was) && landed - u128::from(was) <= u128::from(u64::MAX)
    })
}
