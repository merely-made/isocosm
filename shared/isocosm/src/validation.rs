// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, meaning::mass, rules::*, schema::Key};
use std::collections::BTreeMap;

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
        Query::Mood { .. } | Query::MoodBelow { .. } if rules.mind.is_none() => {
            return Err("mood is read only in a world with a mind".into());
        },
        _ => (),
    }
    Ok(())
}

fn traits(rules: &Rules, keys: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    for k in keys {
        if !rules.traits.contains(k.as_ref()) {
            return Err(format!("unknown trait {}", k.as_ref()));
        }
    }
    Ok(())
}

/// A mind keeps strain in a strain account, and reads its needs only from
/// the member itself and its site's conditions, so a mood can never read a
/// mood and every mood reads alike across a cohort.
fn mind(rules: &Rules) -> Result<()> {
    let Some(m) = &rules.mind else {
        return Ok(());
    };
    if rules.accounts.get(&m.strain) != Some(&AccountKind::Strain) {
        return Err(format!("{} is not a strain account", m.strain));
    }
    for need in &m.needs {
        traits(rules, &need.traits)?;
        query(rules, &need.query)?;
        let own = matches!(
            need.query,
            Query::Alive(Binding::Actor)
                | Query::Trait {
                    who: Binding::Actor,
                    ..
                }
                | Query::Account {
                    who: Binding::Actor,
                    ..
                }
                | Query::Below {
                    who: Binding::Actor,
                    ..
                }
                | Query::Age { .. }
                | Query::Condition { .. }
                | Query::Part {
                    who: Binding::Actor,
                    ..
                }
        );
        if !own {
            return Err("a need reads only its member and its site's conditions".into());
        }
    }
    traits(rules, m.bearing_traits.keys().chain(m.rise_traits.keys()))
}

fn matter(rules: &Rules, value: &str) -> Result<()> {
    match rules.accounts.get(value) {
        Some(AccountKind::Matter { .. }) => Ok(()),
        _ => Err(format!("{value} is not a matter account")),
    }
}

/// Ruling 218: a competition refers only to what the rules declare, and a
/// bound never exceeds certainty. It is keyed by the matter it contests
/// (ruling 236). Its fights strain minds (ruling 221), so it needs the
/// world's mind, and every lost exchange costs reserve, so every fight
/// ends. A lineage fights with one reserve and one way of spending it in
/// every competition it enters, so what a tick's fights cost together
/// settles the same whatever order the competitions are read in (ruling
/// 240).
fn competitions(rules: &Rules) -> Result<()> {
    let mut fighting: BTreeMap<&Key, (&Key, &Key)> = BTreeMap::new();
    for (id, c) in &rules.competitions {
        key(id)?;
        matter(rules, id)?;
        for kind in &c.kinds {
            let reserve = (&kind.body, &kind.spend);
            if *fighting.entry(&kind.identity).or_insert(reserve) != reserve {
                return Err(format!(
                    "{} fights differently across competitions",
                    kind.identity
                ));
            }
        }
        if c.ration == 0
            || c.cost == 0
            || c.upset > 1000
            || c.kinds.is_empty()
            || !rules.traits.contains(&c.contest)
        {
            return Err(format!("invalid competition {id}"));
        }
        if rules.mind.is_none() {
            return Err(format!("{id} fights, and fights need the world's mind"));
        }
        let process = |p: &Key| match rules.processes.contains_key(p) {
            true => Ok(()),
            false => Err(format!("{id} names an unknown process {p}")),
        };
        process(&c.round)?;
        for kind in &c.kinds {
            traits(rules, [&kind.identity])?;
            matter(rules, &kind.body)?;
            query(rules, &kind.hungry)?;
            for p in [&kind.eat, &kind.share, &kind.spend] {
                process(p)?;
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
                // Easing a level destroys what it takes, so it never takes
                // matter, and only bodies keep levels.
                Effect::Ease { who, key, .. } => {
                    account(rules, key)?;
                    if matter(rules, key).is_ok() || *who == Binding::Place {
                        return Err(format!("{id} eases what cannot be eased"));
                    }
                },
                _ => (),
            }
        }
    }
    mind(rules)?;
    competitions(rules)
}
