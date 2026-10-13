// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit paid carrying (TG3a/755), distinct from whole-meal incorporation
//! (516). A process declares the operation; its logged act supplies the
//! current branch address and complete attachment. Nothing chooses a mount.

use crate::{Result, schema::*, rules::Verdict};
use serde::{Deserialize, Serialize};

mod branch;
mod placement;
mod admission;
mod provenance;

pub const KIND: &str = "graft:carry";

/// The native lineage and domain of the carried arrangement, independent
/// of the source stocks that later regrowth fills it with.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub lineage: Key,
    pub domain: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Carry {
    pub donor_root: PartId,
    pub donor_revision: u64,
    pub recipient_revision: u64,
    pub attachment: isometer_core::Attachment,
}

/// Unsaved input scoped to one controlled native act, never to a tick.
#[derive(Clone, Debug)]
pub(crate) struct Input {
    pub actor: Id,
    pub target: Id,
    pub process: Key,
    pub carry: Carry,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preview {
    pub carry: Carry,
    pub verdict: Verdict,
    pub branch: Vec<PartId>,
    pub mapping: Vec<(PartId, PartId)>,
    pub origins: std::collections::BTreeMap<PartId, Origin>,
    pub verdicts: std::collections::BTreeMap<PartId, Verdict>,
    pub holdings: Ledger,
    pub incoming_tissue_mg: u64,
    /// The incoming portion charged to the cumulative refused-origin cap.
    pub refused_tissue_mg: u64,
    pub retained_tissue_mg: u64,
    pub compatibility: Option<crate::rules::GraftReceipt>,
    pub reserve: Key,
    pub soil: Key,
    pub expression_cost_mg: u64,
    pub reserve_cost_mg: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub actor: Id,
    pub donor: Id,
    pub action: Key,
    pub process: Key,
    pub preview: Preview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prepared {
    pub graft: Preview,
    pub debits: crate::projects::DebitPreview,
}

pub(crate) use branch::{prepare, transplant_part, finish};
pub(crate) use provenance::validate_origin;

pub(crate) fn process_costs(process: &crate::rules::Process) -> Result<(&Key, &Key)> {
    let all = process.commitments.iter().chain(&process.effects)
        .chain(process.risk.iter().flat_map(|risk| &risk.effects));
    let mut found = Vec::new();
    for effect in all {
        collect_costs(effect, &mut found);
    }
    match found.as_slice() {
        [costs] => Ok(*costs),
        _ => Err("a controlled graft requires exactly one declared carrying operation".into()),
    }
}

fn collect_costs<'a>(effect: &'a crate::rules::Effect, out: &mut Vec<(&'a Key, &'a Key)>) {
    if let crate::rules::Effect::Graft { reserve, soil } = effect { out.push((reserve, soil)); }
    for child in effect.branches() { collect_costs(child, out); }
}

pub(crate) fn validate(process: &crate::rules::Process, rules: &crate::rules::Rules) -> Result<()> {
    let mut found = Vec::new();
    for effect in process.commitments.iter().chain(&process.effects)
        .chain(process.risk.iter().flat_map(|risk| &risk.effects)) {
        collect_costs(effect, &mut found);
    }
    if found.is_empty() { return Ok(()); }
    let (reserve, soil) = process_costs(process)?;
    if !process.note || !rules.note_kinds.contains(KIND) || process.target.is_none() {
        return Err("graft carrying requires its target, genuine actor event and declared graft:carry kind".into());
    }
    if !matches!(rules.accounts.get(reserve), Some(crate::rules::AccountKind::Matter {
        reserve: true, provision: false, ..
    })) || !matches!(rules.accounts.get(soil), Some(crate::rules::AccountKind::Matter {
        reserve: false, provision: false, ..
    })) {
        return Err("graft carrying requires explicit reserve and soil matter accounts".into());
    }
    if let Some(compatibility) = &rules.compatibility {
        compatibility.evaluate(0, 0, 0, |_| false)?;
        for raise in &compatibility.raised_by {
            if !rules.traits.contains(&raise.condition) {
                return Err(format!("graft allowance names unknown trait condition {}", raise.condition));
            }
        }
    }
    Ok(())
}
