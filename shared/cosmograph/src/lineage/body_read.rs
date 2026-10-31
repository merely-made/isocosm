// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Borrowed native geometry and physiology, without portable origin conversion.

use crate::{Result, rules::{AccountKind, Rules}, schema::{Entity, Part}};
use isometer_core::{BodyDocument, PartId};

#[derive(Clone, Copy, Debug)]
pub struct BodyRead<'a> {
    entity: &'a Entity,
    document: &'a BodyDocument,
    rules: &'a Rules,
}

#[derive(Clone, Copy, Debug)]
pub struct PartRead<'a> {
    pub geometry: &'a isometer_core::Part,
    /// Tombstones may retain geometry without physiology.
    pub physiology: Option<&'a Part>,
    rules: &'a Rules,
}

#[derive(Clone, Copy, Debug)]
pub struct StockRead<'a> {
    pub account: &'a str,
    pub amount: u64,
    /// An absent declaration stays unresolved, never recipient tissue.
    pub kind: Option<&'a AccountKind>,
}

impl<'a> BodyRead<'a> {
    pub fn new(entity: &'a Entity, rules: &'a Rules) -> Result<Self> {
        let document = entity.body.as_ref().ok_or("native body has no document")?;
        for (index, geometry) in document.parts.iter().enumerate() {
            if usize::try_from(geometry.id.0).ok() != Some(index) {
                return Err("native body ids disagree with indexed geometry".into());
            }
            if geometry.attachment.is_some_and(|at| document.part(at.parent).is_none()) {
                return Err("native body attachment has an absent geometry address".into());
            }
        }
        if document.part(document.root).is_none()
            || entity.parts.keys().any(|id| document.part(*id).is_none()) {
            return Err("native body has an absent geometry address".into());
        }
        Ok(Self { entity, document, rules })
    }

    pub fn entity(&self) -> &'a Entity { self.entity }
    pub fn document(&self) -> &'a BodyDocument { self.document }
    pub fn rules(&self) -> &'a Rules { self.rules }
    pub fn lineage(&self) -> &'a str { &self.entity.lineage }
    pub fn revision(&self) -> u64 { self.entity.body_revision }

    pub fn part(&self, id: PartId) -> Option<PartRead<'a>> {
        Some(PartRead { geometry: self.document.part(id)?,
            physiology: self.entity.parts.get(&id).map(std::sync::Arc::as_ref), rules: self.rules })
    }

    pub fn parts(&self) -> impl Iterator<Item = PartRead<'a>> + '_ {
        self.document.parts.iter().map(|geometry| PartRead { geometry,
            physiology: self.entity.parts.get(&geometry.id).map(std::sync::Arc::as_ref), rules: self.rules })
    }
}

impl<'a> PartRead<'a> {
    /// Arrangement provenance is independent of the stock source lineages.
    pub fn graft_origin(&self) -> Option<&'a crate::graft::Origin> {
        self.physiology?.graft_origin.as_ref()
    }

    pub fn stocks(&self) -> impl Iterator<Item = StockRead<'a>> + '_ {
        self.physiology.into_iter().flat_map(|part| part.matter.iter())
            .map(|(account, amount)| StockRead { account, amount: *amount,
                kind: self.rules.accounts.get(account) })
    }

    pub fn tissue_mass_mg(&self) -> Result<u64> {
        self.stocks().try_fold(0u64, |mass, stock| match stock.kind {
            Some(AccountKind::Matter { reserve: false, provision: false, .. }) =>
                mass.checked_add(stock.amount).ok_or_else(|| "native tissue mass overflows".into()),
            Some(_) => Ok(mass),
            None => Err(format!("native part has undeclared account {}", stock.account)),
        })
    }
}
