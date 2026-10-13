// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{anatomy, rules::{AccountKind, Rules}, schema::Lineage};
use std::collections::BTreeMap;

pub(crate) fn prepare(
    actor: &Entity, donor: &Entity, carry: &Carry, reserve: &Key, soil: &Key,
    rules: &Rules, lineages: &BTreeMap<Key, Lineage>,
) -> Result<Preview> {
    if !actor.alive || actor.resident_site().is_none()
        || actor.resident_site() != donor.resident_site() {
        return Err("graft bodies must have the same actual resident site".into());
    }
    if actor.body_revision != carry.recipient_revision || donor.body_revision != carry.donor_revision {
        return Err("graft body revision is stale".into());
    }
    let recipient = actor.body.as_ref().ok_or("graft recipient has no geometry")?;
    let source = donor.body.as_ref().ok_or("graft donor has no geometry")?;
    let development = |e: &Entity| lineages.get(&e.lineage).and_then(|l| l.development.as_ref())
        .ok_or_else(|| "graft body has no declared development".to_string());
    let (mine, theirs) = (development(actor)?, development(donor)?);
    let affinity = rules.affinity.clone().unwrap_or_default();
    if mine.domain >= affinity.domains || theirs.domain >= affinity.domains {
        return Err("graft names an invalid tissue domain".into());
    }
    let branch = source.descendants(carry.donor_root);
    if branch.is_empty() || branch.len() > rules.limits.operations {
        return Err("graft branch is absent or exceeds the native operation budget".into());
    }
    let mut holdings = Ledger::new();
    let mut origins = BTreeMap::new();
    let mut verdicts = BTreeMap::new();
    let mut refused_tissue_mg = 0u64;
    for id in &branch {
        let part = donor.parts.get(id).ok_or("graft branch lacks its actual physiology")?;
        crate::validation::part(rules, donor, *id)?;
        let origin = super::provenance::origin(donor, *id, rules, lineages)?;
        refused_tissue_mg = refused_tissue_mg.checked_add(super::provenance::refused(
            part, &origin, rules, lineages, &affinity, mine.domain)?).ok_or("incoming graft tissue mass overflows")?;
        verdicts.insert(*id, affinity.verdict(origin.domain, mine.domain));
        origins.insert(*id, origin);
        for (key, amount) in &part.matter {
            let held = holdings.entry(key.clone()).or_insert(0u64);
            *held = held.checked_add(*amount).ok_or("graft branch holdings overflow")?;
        }
    }
    let incoming_tissue_mg = super::provenance::tissue(rules, &holdings)?;
    let retained_tissue_mg = super::provenance::retained(actor, rules, lineages, &affinity, mine.domain)?;
    let verdict = if verdicts.values().any(|v| *v == Verdict::Refused) || refused_tissue_mg > 0 {
        Verdict::Refused
    } else if verdicts.values().any(|v| *v == Verdict::Adapter) { Verdict::Adapter } else { Verdict::Native };
    let compatibility = if verdict == Verdict::Refused {
        Some(super::provenance::allowance(actor, rules, refused_tissue_mg, retained_tissue_mg)?)
    } else { None };
    if !matches!(rules.accounts.get(reserve), Some(AccountKind::Matter {
        lineage, reserve: true, provision: false
    }) if *lineage == actor.lineage) || !matches!(rules.accounts.get(soil), Some(AccountKind::Matter {
        reserve: false, provision: false, ..
    })) { return Err("graft cost accounts do not name the actor's reserve and declared soil".into()); }
    let expression_cost_mg = branch.iter().try_fold(0u64, |sum, id| {
            if verdicts[id] == Verdict::Adapter { return Ok(sum); }
            let cells = donor.parts[id].cells.values().try_fold(0u64, |n, cells|
                n.checked_add(u64::from(*cells)).ok_or("graft expression cells overflow"))?;
            let price = cells.checked_mul(anatomy::cell_mass(donor.extent(*id), rules.body()))
                .ok_or("graft expression price overflows")?;
            sum.checked_add(price).ok_or("graft expression total overflows")
        })?;
    let reserve_cost_mg = expression_cost_mg.checked_add(compatibility.as_ref().map_or(0, |r| r.penalty_mg))
        .ok_or("graft total reserve cost overflows")?;
    if anatomy::held(actor, rules, reserve) < reserve_cost_mg {
        return Err("graft reserve cannot pay its full configured cost".into());
    }
    let mut funds = actor.clone();
    match anatomy::take(&mut funds, rules, reserve, reserve_cost_mg) {
        Some(paid) => { paid?; },
        None => crate::meaning::debit(&mut funds.accounts, reserve, reserve_cost_mg)?,
    }
    let mapping: Vec<_> = branch.iter().enumerate().map(|(n, old)| {
        let id = recipient.len().checked_add(n).and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| "graft part identity space exhausted".to_string())?;
        Ok((*old, PartId(id)))
    }).collect::<Result<_>>()?;
    super::placement::check(actor, donor, carry, &mapping)?;
    Ok(Preview { carry: carry.clone(), verdict, branch, mapping, origins, verdicts, holdings,
        incoming_tissue_mg, refused_tissue_mg, retained_tissue_mg, compatibility,
        reserve: reserve.clone(), soil: soil.clone(), expression_cost_mg, reserve_cost_mg })
}

/// Original documents remain in history; the donor keeps stable tombstones.
/// Only grafted recipe sites are cleared, so they do not masquerade as growth.
pub(crate) fn transplant_part(actor: &mut Entity, donor: &mut Entity, preview: &Preview,
    old: PartId, new: PartId) -> Result<()> {
    let ids: BTreeMap<_, _> = preview.mapping.iter().copied().collect();
    if ids.get(&old) != Some(&new) { return Err("graft part is outside the admitted mapping".into()); }
    let mut geometry = donor.geo(old).ok_or("graft source geometry disappeared")?.clone();
    let target = actor.body.as_mut().ok_or("graft recipient geometry disappeared")?;
    if target.len() != new.0 as usize { return Err("graft part mapping changed".into()); }
    geometry.id = new;
    geometry.situs = None;
    geometry.attachment = if old == preview.carry.donor_root {
        Some(preview.carry.attachment)
    } else {
        let mut at = geometry.attachment.ok_or("graft descendant has no parent")?;
        at.parent = *ids.get(&at.parent).ok_or("graft descendant parent is outside its branch")?;
        Some(at)
    };
    let mut physiology = donor.parts.remove(&old).ok_or("graft source physiology disappeared")?;
    physiology.graft_origin = Some(preview.origins.get(&old).ok_or("graft arrangement origin disappeared")?.clone());
    if preview.verdicts.get(&old) == Some(&Verdict::Adapter) {
        physiology.cells.clear(); physiology.functions.clear(); physiology.tracts.clear();
        for allocation in physiology.losses.values_mut() { *allocation = None; }
    }
    target.parts.push(geometry);
    actor.parts.insert(new, physiology);
    let source = donor.body.as_mut().expect("source checked above");
    let part = source.parts.get_mut(old.0 as usize).ok_or("graft donor tombstone is absent")?;
    part.severed = true;
    part.situs = None;
    Ok(())
}

pub(crate) fn finish(actor: &mut Entity, donor: &mut Entity) -> Result<()> {
    if donor.living().next().is_none() { donor.alive = false; }
    actor.body_revision = actor.body_revision.checked_add(1).ok_or("graft recipient revision overflows")?;
    donor.body_revision = donor.body_revision.checked_add(1).ok_or("graft donor revision overflows")?;
    Ok(())
}
