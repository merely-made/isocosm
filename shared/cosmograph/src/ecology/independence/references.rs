// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::AccountKind, simulation::Genesis};
use std::collections::BTreeSet;
use std::cell::RefCell;

pub(super) struct References<'a> {
    pairs: &'a [ReferencePair], pub genesis: &'a Genesis,
    used: RefCell<BTreeSet<(bool, usize)>>,
}

impl<'a> References<'a> {
    pub fn new(genesis: &'a Genesis, pairs: &'a [ReferencePair]) -> Result<Self> {
        let inventory = inventory::inventory(genesis)?;
        let (mut candidate, mut compared) = (BTreeSet::new(), BTreeSet::new());
        for pair in pairs {
            if std::mem::discriminant(&pair.candidate) != std::mem::discriminant(&pair.compared)
                || !candidate.insert(&pair.candidate) || !compared.insert(&pair.compared)
                || !inventory.contains_key(&pair.candidate) || !inventory.contains_key(&pair.compared) {
                return Err("mechanical reference correspondence must be real, typed and one-to-one".into());
            }
        }
        Ok(Self { pairs, genesis, used: RefCell::new(BTreeSet::new()) })
    }

    pub fn token(&self, reference: NativeReference, candidate: bool) -> Result<u32> {
        let index = self.pairs.iter().position(|pair|
            if candidate { pair.candidate == reference } else { pair.compared == reference })
            .ok_or("consumed mechanical reference lacks explicit typed correspondence")?;
        self.used.borrow_mut().insert((candidate,index));
        u32::try_from(index).map_err(|_| "too many mechanical reference correspondences".into())
    }

    pub fn complete(&self, extra: &NativeReference) -> Result<()> {
        if !self.pairs.iter().any(|pair| &pair.candidate == extra)
            || (0..self.pairs.len()).any(|index|
                !self.used.borrow().contains(&(true,index)) || !self.used.borrow().contains(&(false,index))) {
            return Err("every actual correspondence, including the extra noun, must be consumed on both sides".into());
        }
        Ok(())
    }

    pub fn stock(&self, key: &str, candidate: bool) -> Result<(u32, StockType)> {
        let account = self.token(NativeReference::Account(key.into()), candidate)?;
        let kind = match self.genesis.rules.accounts.get(key).ok_or("mechanical account is absent")? {
            AccountKind::Matter { lineage, reserve, provision } => StockType::Matter {
                lineage: self.token(NativeReference::Lineage(lineage.clone()), candidate)?,
                world: self.genesis.lineages.get(lineage).ok_or("stock provenance lineage is absent")?.kingdom == "kingdom:world",
                reserve: *reserve, provision: *provision,
            },
            AccountKind::Energy => StockType::Energy, AccountKind::Attention => StockType::Attention,
            AccountKind::Time => StockType::Time, AccountKind::Obligation => StockType::Obligation,
            AccountKind::Strain => StockType::Strain,
        };
        Ok((account, kind))
    }
}
