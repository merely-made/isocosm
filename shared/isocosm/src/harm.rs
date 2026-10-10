// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Harm: a wound takes cells from one part (checkpoint 10's brief, wing
//! rulings 704 to 719, built here as far as Eponym's world move needs it,
//! 767). A wound is its own act, never a bite (705): it takes cells across
//! every function and the free pool together (709), keeps them lost for
//! good (706), and spills whatever each account holds over its lowered
//! bound to the part's site in the part's own matter (708). A part with no
//! cells left is severed by isometer's semantics, its subtree tombstoned
//! with ids kept and its matter moved out, and a root with none dies (707,
//! 710, 711). Severing bumps the body's revision; a wound alone does not
//! (717). A game that resolves a blow hands it back as a wound (669).
//! Detached matter stays in a fragment; parent tombstones hold none (470,
//! 707, 715). The fragment keeps its wounded cells, with no free restoration.

use crate::{
    Result,
    anatomy::{self, Half},
    meaning::{credit, debit},
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod fragments;
pub(crate) mod rot;
pub(crate) use fragments::{Fragment, detach};

pub const HEAL: &str = "growth:heal";
pub const FRAGMENT: &str = "growth:fragment";

/// The free pool's place among a part's functions when a wound shares out.
const FREE: &str = "";

/// What one wound did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wounded {
    pub part: PartId,
    /// Cells taken, by function, the free pool under the empty key.
    pub lost: BTreeMap<Key, u32>,
    /// Matter spilled to the site, by account.
    pub spilled: Ledger,
    /// Parts tombstoned, the wounded part first.
    pub severed: Vec<PartId>,
    pub died: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<Id>,
}

pub(crate) struct Hit {
    pub wounded: Wounded,
    pub fragment: Option<Fragment>,
}

impl Simulation {
    /// Wounds `entity`: `cells` cells from `part`, or from a part drawn by
    /// its living cells where none is named (716).
    pub(crate) fn wound(
        &mut self,
        entity: Id,
        part: Option<PartId>,
        cells: u32,
    ) -> Result<Wounded> {
        let e = self.state.population.get(entity).ok_or("unknown entity")?;
        if !e.alive {
            return Err("a wound to the dead".into());
        }
        let part = match part {
            Some(p) if e.lives(p) => p,
            Some(_) => return Err("a wound to a part the body lacks".into()),
            None => self.drawn_part(entity, e)?,
        };
        let site = e.place;
        if !self.state.sites.contains_key(&site) {
            return Err("a wound outside any site".into());
        }
        let mut e = self
            .state
            .population
            .get(entity)
            .cloned()
            .ok_or("unknown entity")?;
        let regrows = self
            .state
            .lineages
            .get(&e.lineage)
            .is_some_and(|l| l.traits.contains(FRAGMENT));
        let mut hit = hurt(
            &self.genesis.rules,
            &mut e,
            part,
            cells,
            self.state.tick,
            regrows,
        )?;
        let mut accounts = self.state.sites[&site].accounts.clone();
        for (key, amount) in &hit.wounded.spilled {
            credit(&mut accounts, key, *amount)?;
        }
        if hit.fragment.is_some() {
            if self.state.population.count() >= self.genesis.rules.limits.entities {
                return Err("population limit".into());
            }
            self.state
                .population
                .next_id
                .checked_add(1)
                .ok_or("identity exhausted")?;
        }
        *self.state.population.lift(entity)? = e;
        self.state.sites.get_mut(&site).unwrap().accounts = accounts;
        let mut legs = spilled_legs(entity, site, &hit.wounded);
        if let Some(fragment) = hit.fragment {
            let living = fragment.child.alive;
            let id = self.state.population.insert(fragment.child.clone(), 1)?;
            legs.extend(fragment.legs(entity, id));
            hit.wounded.fragment = Some(id);
            if living {
                for relation in parentage(entity, id) {
                    self.state.relations.insert(relation);
                }
            }
        }
        self.flowed(crate::flows::MadeBy::Command("Wound".into()), legs, 1);
        Ok(hit.wounded)
    }

    /// The part a wound lands on, drawn by living cells, so bigger parts
    /// are hit more often (716).
    fn drawn_part(&self, entity: Id, e: &Entity) -> Result<PartId> {
        let total = cells(e);
        if total == 0 {
            return Err("a body with no cells to wound".into());
        }
        let seed = self.genesis.dynamics_seed();
        let at = crate::draw(seed, "harm:part", &[entity, self.state.tick]) % total;
        part_at(e, at).ok_or("a body with no cells to wound".into())
    }
}

pub(crate) fn cells(e: &Entity) -> u64 {
    e.living()
        .map(|(id, p)| u64::from(anatomy::living_cells(e.extent(id), p)))
        .sum()
}

pub(crate) fn part_at(e: &Entity, mut at: u64) -> Option<PartId> {
    for (id, p) in e.living() {
        let n = u64::from(anatomy::living_cells(e.extent(id), p));
        if at < n {
            return Some(id);
        }
        at -= n;
    }
    None
}

pub(crate) fn hurt(
    rules: &crate::rules::Rules,
    e: &mut Entity,
    part: PartId,
    cells: u32,
    tick: Tick,
    regrows: bool,
) -> Result<Hit> {
    if !e.alive || !e.lives(part) {
        return Err("a wound to no living part".into());
    }
    let half = e.extent(part);
    let p = e.parts.get_mut(&part).ok_or("an unlaid part")?;
    let mut wounded = Wounded {
        part,
        lost: take(p, half, cells),
        spilled: Ledger::new(),
        severed: vec![],
        died: false,
        fragment: None,
    };
    spill(p, half, rules, &mut wounded.spilled)?;
    let mut fragment = None;
    if anatomy::living_cells(half, p) == 0 {
        if e.parent_of(part).is_none() {
            e.alive = false;
            wounded.died = true;
        } else {
            let detached = detach(e, part, tick, regrows)?;
            wounded.severed = detached.mapping.iter().map(|(old, _)| *old).collect();
            fragment = Some(detached);
            e.body_revision = e.body_revision.saturating_add(1);
        }
    }
    Ok(Hit { wounded, fragment })
}

pub(crate) fn spilled_legs(entity: Id, site: Id, w: &Wounded) -> Vec<crate::flows::Leg> {
    use crate::flows::{Holder, Leg};
    w.spilled
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(key, n)| Leg {
            from: (Holder::Part(entity, w.part), key.clone()),
            to: (Holder::Site(site), key.clone()),
            amount: *n,
        })
        .collect()
}

pub(crate) fn parentage(parent: Id, child: Id) -> [Relation; 2] {
    [(child, parent, "sim:parent"), (parent, child, "sim:child")].map(|(subject, object, kind)| {
        Relation {
            subject,
            object,
            kind: kind.into(),
            value: 0,
        }
    })
}

/// Takes up to `cells` living cells from `p` across its functions and free
/// pool in proportion, largest remainders first in key order (709), and
/// marks them lost.
fn take(p: &mut Part, half: Half, cells: u32) -> BTreeMap<Key, u32> {
    let living = anatomy::living_cells(half, p);
    let used: u32 = p.cells.values().sum();
    let mut pool: BTreeMap<Key, u32> = p.cells.clone();
    pool.insert(FREE.into(), living.saturating_sub(used));
    let n = cells.min(living);
    let shares = shares(&pool, n);
    let held: std::collections::BTreeSet<_> =
        p.tracts.iter().flat_map(|t| t.cells.clone()).collect();
    let order = crate::mosaic::path(crate::mosaic::dims(half));
    for (function, k) in &shares {
        let mut gone: Vec<crate::mosaic::CellId> = match function.as_str() {
            FREE => order
                .iter()
                .rev()
                .filter(|c| !held.contains(*c) && !p.lost.contains(*c))
                .take(*k as usize)
                .copied()
                .collect(),
            f => p
                .tracts
                .iter()
                .find(|t| t.function == f)
                .map(|t| t.cells.iter().rev().take(*k as usize).copied().collect())
                .unwrap_or_default(),
        };
        // Counts with no tract laid lose cells from the lattice's end.
        let short = *k as usize - gone.len();
        let spare = order
            .iter()
            .rev()
            .filter(|c| !held.contains(*c) && !p.lost.contains(*c) && !gone.contains(*c))
            .take(short)
            .copied()
            .collect::<Vec<_>>();
        gone.extend(spare);
        p.lost.extend(gone);
        if function != FREE {
            let left = p
                .cells
                .get(function)
                .copied()
                .unwrap_or(0)
                .saturating_sub(*k);
            match left {
                0 => p.cells.remove(function),
                left => p.cells.insert(function.clone(), left),
            };
        }
    }
    p.lost.sort();
    p.lost.dedup();
    crate::mosaic::sync(p, half);
    shares.into_iter().filter(|(_, k)| *k > 0).collect()
}

/// `n` shared over `pool` in proportion, largest remainders first in key
/// order, none past what each holds.
fn shares(pool: &BTreeMap<Key, u32>, n: u32) -> BTreeMap<Key, u32> {
    let total: u64 = pool.values().map(|v| u64::from(*v)).sum();
    if total == 0 {
        return BTreeMap::new();
    }
    let mut out: BTreeMap<Key, u32> = BTreeMap::new();
    let mut rest: Vec<(u64, Key)> = vec![];
    for (k, v) in pool {
        let exact = u64::from(*v) * u64::from(n);
        out.insert(k.clone(), (exact / total) as u32);
        rest.push((exact % total, k.clone()));
    }
    rest.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut left = n - out.values().sum::<u32>();
    for (_, k) in rest.iter().cycle().take(rest.len() * 2) {
        if left == 0 {
            break;
        }
        if out[k] < pool[k] {
            *out.get_mut(k).unwrap() += 1;
            left -= 1;
        }
    }
    out
}

/// Moves what each account of `p` holds over its bound out to `spilled`
/// (708).
fn spill(
    p: &mut Part,
    half: Half,
    rules: &crate::rules::Rules,
    spilled: &mut Ledger,
) -> Result<()> {
    for (key, held) in p.matter.clone() {
        let over = held.saturating_sub(anatomy::bound(half, p, rules, &key));
        if over > 0 {
            debit(&mut p.matter, &key, over)?;
            credit(spilled, &key, over)?;
        }
    }
    p.matter.retain(|_, v| *v > 0);
    Ok(())
}
