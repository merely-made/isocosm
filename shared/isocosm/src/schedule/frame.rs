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
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(crate) struct Frame {
    sites: BTreeMap<Id, Site>,
    /// Bodies as the pass began, by their span then: a member lifted out of
    /// its group alone, a cohort written whole with its count.
    bodies: BTreeMap<Id, (u64, Entity)>,
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
    /// The matter a meal takes of its prey.
    pub(crate) meal: Option<Ledger>,
}

/// What one act would take of shared ground, as planned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Demands {
    pub(crate) accounts: Vec<(Holder, Key, u64)>,
    /// A meal: of whom, how much, and of which accounts, none naming every
    /// matter account.
    pub(crate) meal: Option<(Holder, u64, Vec<Key>)>,
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
    pub(crate) fn site(&self, id: Id) -> Option<&Site> {
        self.sites.get(&id)
    }

    pub(crate) fn body(&self, id: Id) -> Option<&Entity> {
        let (&first, (count, e)) = self.bodies.range(..=id).next_back()?;
        (id < first + count).then_some(e)
    }

    /// Keeps a site as the pass began, before an act first changes it.
    pub(crate) fn keep_site(&mut self, id: Id, site: &Site) {
        self.sites.entry(id).or_insert_with(|| site.clone());
    }

    /// Keeps a body as the pass began, over the span the act writes.
    pub(crate) fn keep_body(&mut self, id: Id, span: u64, e: &Entity) {
        if self.body(id).is_none() {
            self.bodies.insert(id, (span, e.clone()));
        }
    }

    /// Whether the pass has changed this body or site already.
    pub(crate) fn changed(&self, holder: Holder) -> bool {
        match holder {
            Holder::Site(id) => self.sites.contains_key(&id),
            Holder::Entity(id) => self.body(id).is_some(),
            Holder::Dev => false,
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
        let mut eaten: BTreeMap<Holder, (u128, Vec<Key>)> = BTreeMap::new();
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
        for (p, takes) in plans.iter().zip(&takes) {
            for (k, amount) in takes {
                *wanted.entry(k.clone()).or_default() += u128::from(*amount);
            }
            if let Some((holder, amount, of)) = &p.demands.meal {
                let e = eaten.entry(*holder).or_insert((0, of.clone()));
                e.0 += u128::from(*amount);
            }
        }
        let held = |(holder, key): &(Holder, Key)| ground(*holder).map_or(0, |l| value(&l, key));
        // A prey's meals take one share of what it may give, split in pass
        // order between its eaters.
        let mut meals: BTreeMap<Holder, (u64, u128, Ledger)> = BTreeMap::new();
        for (holder, (asked, of)) in &eaten {
            let mut offered = ground(*holder).map_or_else(Ledger::new, |l| edible(&l, of));
            offered.retain(|k, _| meaning::matter(rules, k));
            let total: u128 = offered.values().map(|v| u128::from(*v)).sum();
            let total = u64::try_from(total).unwrap_or(u64::MAX);
            meals.insert(*holder, (total, *asked, offered));
        }
        let mut granted: BTreeMap<Holder, Vec<u64>> = BTreeMap::new();
        for p in &plans {
            if let Some((holder, amount, _)) = &p.demands.meal {
                let (total, asked, _) = &meals[holder];
                let g = fraction(*amount, *total, *asked);
                granted.entry(*holder).or_default().push(g);
            }
        }
        let mut portions: BTreeMap<Holder, std::vec::IntoIter<Ledger>> = BTreeMap::new();
        for (holder, grants) in granted {
            let (_, _, offered) = &meals[&holder];
            let mut left = share(offered, rules, grants.iter().sum());
            let split: Vec<Ledger> = grants
                .into_iter()
                .map(|g| {
                    let part = share(&left, rules, g);
                    for (k, v) in &part {
                        *left.get_mut(k).expect("a portion of what is left") -= v;
                    }
                    part
                })
                .collect();
            portions.insert(holder, split.into_iter());
        }
        let mut planned = BTreeMap::new();
        for (p, takes) in plans.into_iter().zip(takes) {
            let accounts = takes
                .into_iter()
                .map(|(k, take)| {
                    let g = fraction(take, held(&k), wanted[&k]);
                    (k, g)
                })
                .collect();
            let meal = p.demands.meal.as_ref().map(|(holder, ..)| {
                let next = portions.get_mut(holder).and_then(Iterator::next);
                next.expect("a portion for every meal")
            });
            let shares = Shares { accounts, meal };
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
        born,
        alive,
        parts,
        tenets,
        disposition
    );
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
