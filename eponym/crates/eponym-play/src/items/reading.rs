// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An on-demand reading of accepted native stocks and provenance. Neither
//! a nominal item weight nor an account name invents a material identity.

use super::{Item, ItemId};
use crate::GameState;
use cosmograph::{anatomy, rules::AccountKind, schema::{Id, Key, Lineage, Provenance}};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceLineage {
    Known { lineage: Key, kingdom: Key, revision: u64, parent: Option<Key> },
    Missing { lineage: Key },
    /// The native stock has no currently resolvable provenance reference.
    Unresolved,
}

impl SourceLineage {
    fn read(key: &str, lineages: &BTreeMap<Key, Lineage>) -> Self {
        match lineages.get(key) {
            Some(line) => Self::Known { lineage: key.into(), kingdom: line.kingdom.clone(),
                revision: line.revision, parent: line.parent.clone() },
            None => Self::Missing { lineage: key.into() },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemHoldingKind {
    Matter { source: SourceLineage, reserve: bool, provision: bool },
    /// An identity registered by native spreads, not inferred from its key.
    SpreadMatter { source: SourceLineage },
    NonMatter(AccountKind),
    /// The accepted ledger row lacks a rule or registered-stock definition.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemHolding {
    pub account: Key,
    /// Milligrams only for Matter/SpreadMatter; other account units are native.
    pub amount: u64,
    pub kind: ItemHoldingKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MakerProvenance {
    /// Born/Intrinsic declares no maker. This does not assert nobody made it.
    NotDeclared,
    Known { entity: Id, alive: bool, lineage: SourceLineage },
    /// The recorded maker ID has no currently resolvable native entity.
    Missing { entity: Id },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemReading {
    /// The existing game reading of kind, custody and in-site location.
    pub item: Item,
    pub native_tick: u64,
    pub body_revision: u64,
    pub lineage: SourceLineage,
    pub holdings: Vec<ItemHolding>,
    /// Native matter only, including registered spread stocks counted once.
    /// Unknown account rows remain explicit and are not assigned a mass.
    pub matter_mass_mg: u128,
    pub provenance: Provenance,
    /// Resolves the exact Born reference, which need not be today's lineage.
    /// None means the native origin is Made or Intrinsic instead.
    pub born_from: Option<SourceLineage>,
    pub maker: MakerProvenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemReadingError {
    MissingItem(ItemId),
    MissingNativeEntity(Id),
}

impl GameState {
    /// Reads one actual item at this accepted cut. This creates no saved
    /// fields, changes no stock, and imposes no new carry or crafting policy.
    pub fn item_reading(&self, id: ItemId) -> Result<ItemReading, ItemReadingError> {
        let item = *self.items().get(id).ok_or(ItemReadingError::MissingItem(id))?;
        let sim = self.world().sim();
        let state = sim.state();
        let entity = state.population.get(id.0)
            .ok_or(ItemReadingError::MissingNativeEntity(id.0))?;
        let rules = &sim.genesis().rules;
        let books = anatomy::books(entity);
        let holdings = books.iter().map(|(account, amount)| {
            let kind = match rules.accounts.get(account) {
                Some(AccountKind::Matter { lineage, reserve, provision }) => ItemHoldingKind::Matter {
                    source: SourceLineage::read(lineage, &state.lineages),
                    reserve: *reserve, provision: *provision,
                },
                Some(kind) => ItemHoldingKind::NonMatter(kind.clone()),
                None if state.spreads.owns(account) => ItemHoldingKind::SpreadMatter {
                    source: cosmograph::spread::provenance(state, account)
                        .map_or(SourceLineage::Unresolved, |line| SourceLineage::read(&line, &state.lineages)),
                },
                None => ItemHoldingKind::Unknown,
            };
            ItemHolding { account: account.clone(), amount: *amount, kind }
        }).collect();
        let maker = match &entity.provenance {
            Provenance::Made(id) => match state.population.get(*id) {
                Some(maker) => MakerProvenance::Known { entity: *id, alive: maker.alive,
                    lineage: SourceLineage::read(&maker.lineage, &state.lineages) },
                None => MakerProvenance::Missing { entity: *id },
            },
            Provenance::Born(_) | Provenance::Intrinsic(_) => MakerProvenance::NotDeclared,
        };
        let born_from = match &entity.provenance {
            Provenance::Born(lineage) => Some(SourceLineage::read(lineage, &state.lineages)),
            Provenance::Made(_) | Provenance::Intrinsic(_) => None,
        };
        Ok(ItemReading {
            item, native_tick: state.tick, body_revision: entity.body_revision,
            lineage: SourceLineage::read(&entity.lineage, &state.lineages), holdings,
            matter_mass_mg: cosmograph::spread::ledger_mass(&books, rules, &state.spreads),
            provenance: entity.provenance.clone(), born_from, maker,
        })
    }
}
