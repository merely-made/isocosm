// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! World-time plants with bodies (cases 1114, 1115): an adult of 60 keeps a
//! root store of 20 below ground, which no bite reaches (1110), and the rest
//! as tissue over its parts. Growth fills the tissue, then the store;
//! regrowth moves the store into the tissue, the parts with most room taking
//! most; upkeep and seeding draw on the store first; a plant dies only when
//! its tissue and its store are both spent.

use super::{DETRITUS, RoleTime};
use crate::{Result, anatomy, rules::*, schema::{Key, Tick}, simulation::Genesis};
use std::collections::BTreeMap;

/// What a plant's root store holds at most, and the store cells its root is
/// given to hold it.
pub const STORE: u64 = 20;
const STORE_CELLS: u32 = 4;
/// What a seeding leaves in the store beside the young's share.
const STORE_KEPT: u64 = 5;

/// How a world-time founding bodies its plants (1115): one or two stretches
/// of four to eight segments, and a root that stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plants {
    pub span: ([u64; 2], [u64; 2]),
}

impl Default for Plants {
    fn default() -> Self {
        Self { span: ([1, 2], [4, 8]) }
    }
}

impl Plants {
    /// Gives the recipe's root store cells, taken from its fullest function.
    pub(crate) fn storing(&self, kinds: &mut BTreeMap<Key, Template>, recipe: &mut Recipe) -> Result<()> {
        let root = recipe.tagmata[0].segment.clone();
        let name = format!("{root}-storing");
        if !kinds.contains_key(&name) {
            let mut t = kinds.get(&root).ok_or(format!("an unknown root kind {root}"))?.clone();
            let mut moved = 0;
            while moved < STORE_CELLS {
                let Some(f) = t.cells.iter().filter(|(f, _)| f.as_str() != anatomy::REPRODUCE)
                    .max_by_key(|(_, n)| **n).map(|(f, _)| f.clone()) else { break };
                let n = t.cells.get_mut(&f).expect("found above");
                *n -= 1;
                if *n == 0 { t.cells.remove(&f); }
                moved += 1;
            }
            *t.cells.entry(anatomy::STORE.into()).or_default() += moved;
            kinds.insert(name.clone(), t);
        }
        recipe.tagmata[0].segment = name;
        Ok(())
    }
}

/// A plant lineage's root store account.
pub fn reserve(i: u32) -> Key {
    format!("reserve:{i}")
}

/// Declares lineage `i`'s store, splits each founder's body between tissue
/// and store, and rewrites its growth, regrowth, upkeep, seeding and death
/// to the store's account, `period` reading minutes at the world's unit.
pub(super) fn bodied(g: &mut Genesis, i: u32, time: &RoleTime, period: &dyn Fn(u64) -> Tick) {
    let (tissue, store, lineage) = (format!("matter:{i}-0"), reserve(i), format!("lineage:{i}"));
    let grown = time.adult - STORE;
    g.rules.accounts.insert(store.clone(), AccountKind::Matter { lineage: lineage.clone(), reserve: true, provision: false });
    for group in g.population.groups.values_mut() {
        if group.entity.lineage == lineage {
            let e = group.entity_mut();
            e.accounts.insert(tissue.clone(), grown);
            e.accounts.insert(store.clone(), STORE);
        }
    }
    let below = |key: &Key, amount| Query::Below { who: Binding::Actor, key: key.clone(), amount };
    let holds = |key: &Key, at_least| Query::Account { who: Binding::Actor, key: key.clone(), at_least };
    let processes = &mut g.rules.processes;
    // Growth fills the tissue, then the store.
    if let Some(grow) = processes.get_mut(&format!("ecology:produce-{i}")) {
        grow.requires.retain(|q| !matches!(q, Query::Below { .. }));
        let mut stocking = grow.clone();
        grow.requires.push(below(&tissue, grown));
        stocking.id = format!("ecology:produce-{i}-reserve");
        for e in &mut stocking.effects {
            if let Effect::Transform { give, .. } = e && let Some(n) = give.remove(&tissue) {
                give.insert(store.clone(), n);
            }
        }
        stocking.requires.extend([holds(&tissue, grown), below(&store, STORE)]);
        processes.insert(stocking.id.clone(), stocking);
    }
    // Regrowth moves the store into the tissue.
    if let Some(regrow) = processes.get_mut(&format!("ecology:regrow-{i}")) {
        regrow.effects = vec![Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(store.clone(), 2)]),
            give: BTreeMap::from([(tissue.clone(), 2)]), conversion: None }];
        regrow.requires.retain(|q| matches!(q, Query::Alive(_) | Query::Trait { .. }));
        regrow.requires.extend([holds(&store, 2), below(&tissue, grown)]);
        regrow.period = Some(period(time.regrow));
    }
    // Upkeep is paid from the store before the tissue.
    if let Some(upkeep) = processes.get_mut(&format!("ecology:upkeep-{i}")) {
        upkeep.effects = vec![Effect::Spend { from: vec![store.clone(), tissue.clone()], to: Binding::Place,
            amount: 1.into(), into: Some(DETRITUS.into()) }];
    }
    // A seed is filled from the store.
    if let Some(birth) = processes.get_mut(&format!("ecology:birth-{i}")) {
        let fill = Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(store.clone(), time.child)]),
            give: BTreeMap::from([(tissue.clone(), time.child)]), conversion: None };
        birth.effects.insert(0, fill.clone());
        if let Some(risk) = &mut birth.risk { risk.effects.insert(0, fill); }
        birth.requires.retain(|q| !matches!(q, Query::Account { .. }));
        birth.requires.push(holds(&store, time.child + STORE_KEPT));
    }
    // A plant dies only when every part is empty.
    if let Some(death) = processes.get_mut(&format!("ecology:death-{i}")) {
        death.requires.push(below(&store, 1));
    }
}
