// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Arrangement provenance is native saved data. Opaque geometry origins
//! are never parsed into lineage identities. Old unknown imported parts
//! remain readable, but cannot authorize a new marked carry.

use super::*;
use crate::rules::{AccountKind, Affinity, Rules};
use std::collections::BTreeMap;

pub(crate) fn validate_origin(part: &Part, rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>) -> Result<()> {
    let Some(origin) = &part.graft_origin else { return Ok(()); };
    crate::validation::key(&origin.lineage)?;
    let declared = lineages.get(&origin.lineage).and_then(|line| line.development.as_ref())
        .ok_or("graft arrangement origin has no declared source development")?;
    let affinity = rules.affinity.clone().unwrap_or_default();
    if declared.domain != origin.domain || origin.domain >= affinity.domains {
        return Err("graft arrangement origin disagrees with its declared source domain".into());
    }
    Ok(())
}

pub(super) fn origin(e: &Entity, id: PartId, rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>) -> Result<Origin> {
    let part = e.parts.get(&id).ok_or("graft origin has no actual part")?;
    validate_origin(part, rules, lineages)?;
    if let Some(origin) = &part.graft_origin { return Ok(origin.clone()); }
    let development = lineages.get(&e.lineage).and_then(|line| line.development.as_ref())
        .ok_or("graft origin has no declared native development")?;
    let [tagma, _, slot] = e.situs(id).ok_or("graft arrangement has unknown imported origin")?;
    let tagma = development.recipe.tagmata.get(usize::from(tagma))
        .ok_or("graft native arrangement has an absent recipe address")?;
    if slot > 0 && tagma.bears.is_none() {
        return Err("graft native arrangement has no declared borne kind".into());
    }
    let origin = Origin { lineage: e.lineage.clone(), domain: development.domain };
    if origin.domain >= rules.affinity.clone().unwrap_or_default().domains {
        return Err("graft origin names an invalid native domain".into());
    }
    Ok(origin)
}

pub(super) fn tissue(rules: &Rules, holdings: &Ledger) -> Result<u64> {
    holdings.iter().try_fold(0u64, |sum, (key, amount)| {
        match rules.accounts.get(key) {
            Some(AccountKind::Matter { reserve: false, provision: false, .. }) =>
                sum.checked_add(*amount).ok_or_else(|| "graft tissue mass overflows".into()),
            Some(AccountKind::Matter { .. }) => Ok(sum),
            _ => Err(format!("graft branch holds undeclared or nonmatter account {key}")),
        }
    })
}

pub(super) fn refused(part: &Part, origin: &Origin, rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>, affinity: &Affinity, into: u16) -> Result<u64> {
    if affinity.verdict(origin.domain, into) == Verdict::Refused {
        // Count the arrangement's living tissue even when it was regrown
        // from recipient stocks, and never count the same stock twice.
        return tissue(rules, &part.matter);
    }
    crate::matter::scruple::of(part, rules)?.into_iter().try_fold(0u64, |sum, (line, mg)| {
        let development = lineages.get(&line).and_then(|line| line.development.as_ref());
        if let Some(d) = development {
            if d.domain >= affinity.domains { return Err("graft stock source has an invalid domain".into()); }
            if affinity.verdict(d.domain, into) == Verdict::Refused {
                return sum.checked_add(mg).ok_or_else(|| "retained graft tissue mass overflows".into());
            }
        }
        Ok(sum)
    })
}

pub(super) fn retained(actor: &Entity, rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>, affinity: &Affinity, into: u16) -> Result<u64> {
    actor.living().try_fold(0u64, |sum, (id, part)| {
        let origin = origin(actor, id, rules, lineages)?;
        sum.checked_add(refused(part, &origin, rules, lineages, affinity, into)?)
            .ok_or_else(|| "retained graft tissue mass overflows".into())
    })
}
