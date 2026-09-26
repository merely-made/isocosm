// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, meaning::mass, rules::*};

pub(crate) fn key(value: &str) -> Result<()> {
    let valid = value.len() <= 256
        && value.split_once(':').is_some_and(|(a, b)| {
            !a.is_empty()
                && !b.is_empty()
                && value
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b":_-/ .".contains(&c) && c != b' ')
        });
    if valid {
        Ok(())
    } else {
        Err(format!("invalid namespaced key {value:?}"))
    }
}

fn account(rules: &Rules, value: &str) -> Result<()> {
    if rules.accounts.contains_key(value) {
        Ok(())
    } else {
        Err(format!("unknown account {value}"))
    }
}

fn query(rules: &Rules, q: &Query) -> Result<()> {
    match q {
        Query::Account { key, .. } => account(rules, key)?,
        Query::Below { key, .. } => account(rules, key)?,
        Query::Condition { key, .. } if !rules.conditions.contains(key) => {
            return Err(format!("unknown condition {key}"));
        },
        Query::Trait { key, .. } if !rules.traits.contains(key) => {
            return Err(format!("unknown trait {key}"));
        },
        Query::Related { kind } if !rules.relations.contains(kind) => {
            return Err(format!("unknown relation {kind}"));
        },
        _ => (),
    }
    Ok(())
}

fn matter(rules: &Rules, value: &str) -> Result<()> {
    match rules.accounts.get(value) {
        Some(AccountKind::Matter { .. }) => Ok(()),
        _ => Err(format!("{value} is not a matter account")),
    }
}

/// Ruling 218: a competition refers only to what the rules declare, and a
/// bound never exceeds certainty.
fn competitions(rules: &Rules) -> Result<()> {
    for (id, c) in &rules.competitions {
        key(id)?;
        matter(rules, &c.food)?;
        if c.ration == 0 || c.kinds.is_empty() || !rules.traits.contains(&c.contest) {
            return Err(format!("invalid competition {id}"));
        }
        for kind in &c.kinds {
            if !rules.traits.contains(&kind.identity) {
                return Err(format!("unknown trait {}", kind.identity));
            }
            matter(rules, &kind.body)?;
            query(rules, &kind.hungry)?;
            for process in [&kind.eat, &kind.share, &kind.strain] {
                if !rules.processes.contains_key(process) {
                    return Err(format!("{id} names an unknown process {process}"));
                }
            }
        }
    }
    if let Some(s) = &rules.similitude
        && (s.default_bound > 1000 || s.bounds.values().any(|b| *b > 1000))
    {
        return Err("a similitude bound exceeds certainty".into());
    }
    Ok(())
}

pub(crate) fn rules(rules: &Rules) -> Result<()> {
    if rules.version != crate::VERSION {
        return Err("unsupported rules version".into());
    }
    if rules.epoch_ticks == 0
        || rules.limits.events_per_advance == 0
        || rules.limits.operations == 0
        || rules.limits.entities == 0
        || rules.limits.advance_ticks == 0
        || rules.processes.len() > rules.limits.processes
    {
        return Err("invalid rules limits".into());
    }
    if rules.field.strength > 1_000_000
        || rules.field.legend_floor > 1_000_000
        || rules.field.decay_per_tick == 0
    {
        return Err("invalid reach policy".into());
    }
    for id in rules
        .accounts
        .keys()
        .chain(&rules.conditions)
        .chain(&rules.traits)
        .chain(&rules.relations)
        .chain(&rules.note_kinds)
    {
        key(id)?;
    }
    for (id, p) in &rules.processes {
        key(id)?;
        if id != &p.id {
            return Err("process key does not match identity".into());
        }
        if p.period == Some(0) {
            return Err(format!("zero period: {id}"));
        }
        let effects = p
            .commitments
            .iter()
            .chain(&p.effects)
            .chain(p.risk.iter().flat_map(|r| &r.effects));
        if effects.clone().count() > rules.limits.operations
            || p.requires.len() > rules.limits.operations
        {
            return Err(format!("operation budget: {id}"));
        }
        if p.risk.as_ref().is_some_and(|r| r.per_million > 1_000_000) {
            return Err("invalid risk probability".into());
        }
        if let Some(a) = &p.need_account {
            account(rules, a)?;
        }
        for invariant in &p.invariants {
            if invariant != "sim:matter" && invariant != "sim:nonnegative" {
                return Err(format!("unsupported invariant {invariant}"));
            }
        }
        for q in &p.requires {
            query(rules, q)?;
        }
        for e in effects {
            match e {
                Effect::Transfer { account: a, .. } => account(rules, a)?,
                Effect::Transform { take, give, .. } => {
                    for a in take.keys().chain(give.keys()) {
                        account(rules, a)?;
                    }
                    // Matter remains matter even for transforms between provenance kinds.
                    if mass(take, rules) != mass(give, rules) {
                        return Err(format!("unbalanced matter transform: {id}"));
                    }
                },
                Effect::Condition { key, .. } if !rules.conditions.contains(key) => {
                    return Err(format!("unknown condition {key}"));
                },
                Effect::Trait { key, .. } if !rules.traits.contains(key) => {
                    return Err(format!("unknown trait {key}"));
                },
                Effect::Relate { kind, .. } if !rules.relations.contains(kind) => {
                    return Err(format!("unknown relation {kind}"));
                },
                Effect::Note { kind, .. } if !rules.note_kinds.contains(kind) => {
                    return Err(format!("unknown note kind {kind}"));
                },
                Effect::Birth { provision } => {
                    for a in provision.keys() {
                        account(rules, a)?;
                    }
                },
                Effect::FoundPolity { support, .. } => account(rules, support)?,
                Effect::Record { axis, account: a } => {
                    key(axis)?;
                    account(rules, a)?;
                    if !p.note {
                        return Err("recorded feats require a causal event".into());
                    }
                },
                _ => (),
            }
        }
    }
    competitions(rules)
}
