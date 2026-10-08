// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's matter in its parts (the anatomy brief, rulings 459, 460, 463,
//! 464, 493, 494 and 504). A part with extents keeps its own ledger, its
//! cells and their mass read from its box; a body's own lineage's matter
//! lives in those parts, its totals being readings of them. Takes spread over
//! the parts by what each holds, gives by the room each has below what it
//! may hold, the reserve only in the cells that store, units left over going
//! to the largest remainders in part order. A body with no such part keeps
//! its entity ledger, as bodies did before parts held matter.

use crate::{
    Result,
    meaning::{credit, debit},
    rules::{AccountKind, BodyRules, Measure, Rules},
    schema::*,
};

/// What each part of a body may still take in an act whose carriage bound
/// it (ruling 581): a part not named takes nothing.
pub type Caps = std::collections::BTreeMap<Id, u64>;

pub(crate) const STORE: &str = "function:store";
pub(crate) const REPRODUCE: &str = "function:reproduce";

/// Each axis's extent in voxels, `2|h| + 1`.
fn axes(p: &Part) -> [u128; 3] {
    p.half_extent.map(|h| 2 * u128::from(h.unsigned_abs()) + 1)
}

/// The axes longest first.
fn sorted(p: &Part) -> [u128; 3] {
    let mut a = axes(p);
    a.sort_unstable_by(|x, y| y.cmp(x));
    a
}

pub fn voxels(p: &Part) -> u128 {
    axes(p).iter().product()
}

/// Mesocosm's lattice (ruling 460): along each axis one cell per two voxels
/// of half-extent, plus one, at most four an axis.
pub fn capacity(p: &Part) -> u32 {
    let axis = |h: i32| (h.unsigned_abs().max(1) / 2 + 1).clamp(1, 4);
    p.half_extent.iter().map(|h| axis(*h)).product()
}

/// The part's adult mass: its voxels priced at the reference mass a
/// segment, at least a milligram.
pub fn ceiling(p: &Part, b: BodyRules) -> u64 {
    let priced =
        voxels(p) * u128::from(b.reference_mass_mg) / u128::from(b.reference_segment_voxels.max(1));
    u64::try_from(priced).unwrap_or(u64::MAX).max(1)
}

/// What one cell weighs: the adult mass over the cells.
pub fn cell_mass(p: &Part, b: BodyRules) -> u64 {
    (ceiling(p, b) / u64::from(capacity(p))).max(1)
}

pub fn measure(p: &Part, m: Measure) -> u128 {
    let [a, b, c] = sorted(p);
    match m {
        Measure::Length => a,
        Measure::Area => a * b,
        Measure::CrossSection => b * c,
        Measure::Volume => a * b * c,
    }
}

/// What `function` takes of a measurement: the part's in proportion to the
/// cells it holds there, floored.
pub fn share_of(p: &Part, function: &str, m: Measure) -> u128 {
    let cells = u128::from(p.cells.get(function).copied().unwrap_or(0));
    measure(p, m) * cells / u128::from(capacity(p))
}

/// A part's name (ruling 494): a declared tube or shell, a hollow a box
/// cannot show; otherwise its box read as isometer reads one, refined by
/// the tree, a rod with two or more children a branch and a point between a
/// parent and a child a joint.
pub fn name(e: &Entity, id: Id) -> Option<&'static str> {
    let p = e.parts.get(&id)?;
    match p.shape.as_str() {
        "part-shape:tube" => return Some("part-shape:tube"),
        "part-shape:shell" => return Some("part-shape:shell"),
        _ => {},
    }
    let boxed = boxed(p.half_extent);
    let children = e
        .parts
        .values()
        .filter(|c| !c.severed && c.parent == Some(id))
        .count();
    Some(match boxed {
        "part-shape:rod" if children >= 2 => "part-shape:branch",
        "part-shape:point" if p.parent.is_some() && children >= 1 => "part-shape:joint",
        other => other,
    })
}

/// The name a box alone reads, as isometer's classifier reads one.
pub fn boxed(half_extent: [i32; 3]) -> &'static str {
    let h = half_extent.map(|h| h.unsigned_abs().max(1));
    let (max, min) = (h.iter().max().copied(), h.iter().min().copied());
    let (max, min) = (max.unwrap_or(1), min.unwrap_or(1));
    let long = h.iter().filter(|d| **d * 2 >= max).count();
    match long {
        _ if max <= 1 => "part-shape:point",
        1 => "part-shape:rod",
        2 if min * 2 <= max => "part-shape:sheet",
        _ => "part-shape:lump",
    }
}

/// A lineage's own matter accounts: its tissue, reserve and provision.
#[derive(Clone, Debug, Default)]
pub struct Own {
    pub tissue: Option<Key>,
    pub reserve: Option<Key>,
    pub provision: Option<Key>,
}

pub fn own(rules: &Rules, lineage: &str) -> Own {
    let mut own = Own::default();
    for (key, kind) in &rules.accounts {
        let AccountKind::Matter {
            lineage: l,
            reserve,
            provision,
        } = kind
        else {
            continue;
        };
        let slot = match (l == lineage, reserve, provision) {
            (false, ..) => continue,
            (true, true, _) => &mut own.reserve,
            (true, _, true) => &mut own.provision,
            _ => &mut own.tissue,
        };
        slot.get_or_insert_with(|| key.clone());
    }
    own
}

/// Whether `key` is a reserve account.
fn reserve(rules: &Rules, key: &str) -> bool {
    matches!(
        rules.accounts.get(key),
        Some(AccountKind::Matter { reserve: true, .. })
    )
}

/// Whether `key` is a provision account (ruling 518).
fn provision(rules: &Rules, key: &str) -> bool {
    matches!(
        rules.accounts.get(key),
        Some(AccountKind::Matter {
            provision: true,
            ..
        })
    )
}

/// Whether `key` is the body's own lineage's matter and the body has parts
/// to hold it, so it lives in them.
pub fn anatomical(e: &Entity, rules: &Rules, key: &str) -> bool {
    let own = matches!(
        rules.accounts.get(key),
        Some(AccountKind::Matter { lineage, .. }) if *lineage == e.lineage
    );
    own && e.parts.values().any(|p| !p.severed && p.bodied())
}

/// What a part may hold of `key` (rulings 463 and 518): its adult mass in
/// tissue, in reserve its store cells' mass, and in provision its
/// reproduce cells'.
pub fn bound(p: &Part, rules: &Rules, key: &str) -> u64 {
    let b = rules.body();
    let cells = |function: &str| u64::from(p.cells.get(function).copied().unwrap_or(0));
    if reserve(rules, key) {
        cells(STORE).saturating_mul(cell_mass(p, b))
    } else if provision(rules, key) {
        cells(REPRODUCE).saturating_mul(cell_mass(p, b))
    } else {
        ceiling(p, b)
    }
}

/// The parts that can hold matter, living and bodied, in part order.
fn bodies(e: &Entity) -> impl Iterator<Item = (Id, &Part)> {
    e.parts
        .iter()
        .filter(|(_, p)| !p.severed && p.bodied())
        .map(|(id, p)| (*id, p))
}

/// What the body holds of `key`: its ledger's and its parts'.
pub fn held(e: &Entity, rules: &Rules, key: &str) -> u64 {
    let own = e.accounts.get(key).copied().unwrap_or(0);
    if !anatomical(e, rules, key) {
        return own;
    }
    bodies(e)
        .map(|(_, p)| p.matter.get(key).copied().unwrap_or(0))
        .fold(own, u64::saturating_add)
}

/// How much the body has room for of `key` in its parts.
pub fn room(e: &Entity, rules: &Rules, key: &str) -> u64 {
    room_within(e, rules, key, None)
}

/// The same within `caps`, where a carriage bound the act.
pub fn room_within(e: &Entity, rules: &Rules, key: &str, caps: Option<&Caps>) -> u64 {
    bodies(e)
        .map(|(id, p)| capped(space(p, rules, key), id, caps))
        .fold(0, u64::saturating_add)
}

fn capped(room: u64, id: Id, caps: Option<&Caps>) -> u64 {
    match caps {
        Some(c) => room.min(c.get(&id).copied().unwrap_or(0)),
        None => room,
    }
}

/// What one part has room for of `key`: below what it may hold, its room
/// for tissue shrunk by all the tissue it keeps, of other accounts and
/// other lineages, as an incorporated part keeps its donor's (ruling 544).
pub(crate) fn space(p: &Part, rules: &Rules, key: &str) -> u64 {
    let held = p.matter.get(key).copied().unwrap_or(0);
    let room = bound(p, rules, key).saturating_sub(held);
    if reserve(rules, key) || provision(rules, key) {
        return room;
    }
    let tissue = p
        .matter
        .iter()
        .filter(|(k, _)| k.as_str() != key && !reserve(rules, k) && !provision(rules, k));
    room.saturating_sub(tissue.map(|(_, v)| *v).fold(0, u64::saturating_add))
}

/// `amount` split by `weights`: each its exact share floored, the units
/// left over to the largest remainders, ties in the order given.
pub(crate) fn apportion(weights: &[(Id, u64)], amount: u64) -> Vec<(Id, u64)> {
    let total: u128 = weights.iter().map(|(_, w)| u128::from(*w)).sum();
    if total == 0 {
        return vec![];
    }
    let wanted = u128::from(amount).min(total);
    let mut split: Vec<(Id, u64, u128)> = weights
        .iter()
        .map(|(id, w)| {
            let product = u128::from(*w) * wanted;
            (*id, (product / total) as u64, product % total)
        })
        .collect();
    let assigned: u128 = split.iter().map(|s| u128::from(s.1)).sum();
    let mut order: Vec<usize> = (0..split.len()).collect();
    order.sort_by(|a, b| split[*b].2.cmp(&split[*a].2));
    for &i in order.iter().take((wanted - assigned) as usize) {
        split[i].1 += 1;
    }
    split
        .into_iter()
        .filter(|s| s.1 > 0)
        .map(|s| (s.0, s.1))
        .collect()
}

/// Takes `amount` of the body's own `key` from its parts, by what each
/// holds (ruling 464). `None` where the key does not live in parts.
pub fn take(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
) -> Option<Result<Vec<(Id, u64)>>> {
    take_within(e, rules, key, amount, None)
}

/// The same where a carriage bound the act: what leaves a part frees as
/// much of what it may take in.
pub fn take_within(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
    caps: Option<&mut Caps>,
) -> Option<Result<Vec<(Id, u64)>>> {
    if !anatomical(e, rules, key) {
        return None;
    }
    let weights: Vec<(Id, u64)> = bodies(e)
        .map(|(id, p)| (id, p.matter.get(key).copied().unwrap_or(0)))
        .collect();
    let total: u128 = weights.iter().map(|(_, w)| u128::from(*w)).sum();
    if total < u128::from(amount) {
        return Some(Err(format!("insufficient {key}")));
    }
    let split = apportion(&weights, amount);
    if let Some(caps) = caps {
        for (id, n) in &split {
            let slot = caps.entry(*id).or_default();
            *slot = slot.saturating_add(*n);
        }
    }
    Some(apply(e, key, &split, debit).map(|()| split))
}

/// Gives `amount` of the body's own `key` to its parts, by the room each
/// has below what it may hold (rulings 463 and 464), none overfilling.
pub fn give(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
) -> Option<Result<Vec<(Id, u64)>>> {
    give_within(e, rules, key, amount, None)
}

/// The same within `caps`, where a carriage bound the act (581), each part
/// taking no more than what reached it.
pub fn give_within(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
    mut caps: Option<&mut Caps>,
) -> Option<Result<Vec<(Id, u64)>>> {
    if !anatomical(e, rules, key) {
        return None;
    }
    let weights: Vec<(Id, u64)> = bodies(e)
        .map(|(id, p)| (id, capped(space(p, rules, key), id, caps.as_deref())))
        .collect();
    let room: u128 = weights.iter().map(|(_, w)| u128::from(*w)).sum();
    if room < u128::from(amount) {
        return Some(Err(format!("no room in the body for {amount} of {key}")));
    }
    let split = apportion(&weights, amount);
    if let Some(caps) = caps.as_deref_mut() {
        for (id, n) in &split {
            let slot = caps.entry(*id).or_default();
            *slot = slot.saturating_sub(*n);
        }
    }
    Some(apply(e, key, &split, credit).map(|()| split))
}

fn apply(
    e: &mut Entity,
    key: &str,
    split: &[(Id, u64)],
    f: fn(&mut Ledger, &str, u64) -> Result<()>,
) -> Result<()> {
    for (id, amount) in split {
        let part = e.parts.get_mut(id).ok_or("routed part missing")?;
        f(&mut part.matter, key, *amount)?;
    }
    Ok(())
}

/// The part a bite lands on (ruling 459): drawn by what each holds of the
/// accounts the meal names, `draw` choosing among them. `None` where the
/// body keeps no matter in parts.
pub fn bitten(e: &Entity, of: &[Key], draw: u64) -> Option<Id> {
    let weights: Vec<(Id, u64)> = bodies(e)
        .map(|(id, p)| {
            let edible = of.iter().filter_map(|k| p.matter.get(k)).sum::<u64>();
            (id, edible)
        })
        .collect();
    let total: u128 = weights.iter().map(|(_, w)| u128::from(*w)).sum();
    if total == 0 {
        return None;
    }
    let mut point = u128::from(draw) % total;
    for (id, w) in weights {
        if point < u128::from(w) {
            return Some(id);
        }
        point -= u128::from(w);
    }
    None
}

/// Every account the body holds, its parts' matter summed in: what its
/// planning and its totals read.
pub fn books(e: &Entity) -> Ledger {
    let mut books = e.accounts.clone();
    for p in e.parts.values() {
        for (k, v) in &p.matter {
            let slot = books.entry(k.clone()).or_default();
            *slot = slot.saturating_add(*v);
        }
    }
    books
}
