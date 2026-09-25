// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A process definition applied to every member of a bin at once: the
//! aggregate form of the sim plan's §3.1. Queries mean what the core's
//! interpreter means by them. Anything identity-dependent (risk, targets,
//! noted events) is refused, as is a site debit that would cover only some
//! of the bin, since a crowd has no member order to ration by.

use crate::{
    Result,
    rules::{AccountKind, Binding, Effect, Process, Query, Rules},
    schema::*,
};

/// Zero entries dropped: no query or inspection tells absent from zero.
pub(super) fn normalize(mut e: Entity) -> Entity {
    e.accounts.retain(|_, v| *v != 0);
    e
}

pub(super) fn value(ledger: &Ledger, key: &str) -> u64 {
    ledger.get(key).copied().unwrap_or(0)
}

fn ledger<'a>(who: &Binding, e: &'a Entity, site: &'a Site) -> Result<&'a Ledger> {
    match who {
        Binding::Actor => Ok(&e.accounts),
        Binding::Place => Ok(&site.accounts),
        Binding::Target => Err("the crowd has no targets".into()),
    }
}

pub(super) fn holds(q: &Query, e: &Entity, site: &Site, tick: Tick) -> Result<bool> {
    let actor = |b: &Binding| match b {
        Binding::Actor => Ok(()),
        other => Err(format!("the crowd reads no {other:?} body")),
    };
    Ok(match q {
        Query::Alive(b) => {
            actor(b)?;
            e.alive
        },
        Query::Trait { who, key } => {
            actor(who)?;
            e.traits.contains(key)
        },
        Query::Account { who, key, at_least } => value(ledger(who, e, site)?, key) >= *at_least,
        Query::Below { who, key, amount } => value(ledger(who, e, site)?, key) < *amount,
        Query::Age { at_least } => tick.saturating_sub(e.born) >= *at_least,
        Query::Condition { key, at_least } => {
            site.conditions.get(key).is_some_and(|v| v >= at_least)
        },
        other => return Err(format!("the crowd cannot read {other:?}")),
    })
}

/// A query that reads only the site, answered for the site alone.
pub(super) fn site_holds(q: &Query, site: &Site) -> Result<bool> {
    Ok(match q {
        Query::Account {
            who: Binding::Place,
            key,
            at_least,
        } => value(&site.accounts, key) >= *at_least,
        Query::Below {
            who: Binding::Place,
            key,
            amount,
        } => value(&site.accounts, key) < *amount,
        Query::Condition { key, at_least } => {
            site.conditions.get(key).is_some_and(|v| v >= at_least)
        },
        other => return Err(format!("{other:?} reads more than a site")),
    })
}

fn reads_site(q: &Query) -> bool {
    matches!(
        q,
        Query::Account {
            who: Binding::Place,
            ..
        } | Query::Below {
            who: Binding::Place,
            ..
        } | Query::Condition { .. }
    )
}

/// One member's worth from the member, or every member's from the site.
/// False is the core's blocked outcome: the member is short, or the site
/// cannot cover even one member, so every member of the bin fails alike.
fn take(
    who: &Binding,
    e: &mut Entity,
    site: &mut Site,
    key: &str,
    amount: u64,
    count: u64,
) -> Result<bool> {
    let (ledger, total) = match who {
        Binding::Actor => (&mut e.accounts, amount),
        Binding::Place => (
            &mut site.accounts,
            amount.checked_mul(count).ok_or("amount overflow")?,
        ),
        Binding::Target => return Err("the crowd has no targets".into()),
    };
    let have = value(ledger, key);
    if have >= total {
        ledger.insert(key.into(), have - total);
        Ok(true)
    } else if have < amount {
        Ok(false)
    } else {
        Err(format!("contended site debit of {key}"))
    }
}

fn give(
    who: &Binding,
    e: &mut Entity,
    site: &mut Site,
    key: &str,
    amount: u64,
    count: u64,
) -> Result<()> {
    let (ledger, total) = match who {
        Binding::Actor => (&mut e.accounts, amount),
        Binding::Place => (
            &mut site.accounts,
            amount.checked_mul(count).ok_or("amount overflow")?,
        ),
        Binding::Target => return Err("the crowd has no targets".into()),
    };
    let slot = ledger.entry(key.into()).or_default();
    *slot = slot.checked_add(total).ok_or("account overflow")?;
    Ok(())
}

/// Applies `p` to `count` members sharing state `e` at `site`. `Ok(None)` is
/// the core's blocked outcome and changes nothing; on success the site is
/// updated and the members' new state returned.
pub(super) fn apply(
    p: &Process,
    e: &Entity,
    site: &mut Site,
    count: u64,
    tick: Tick,
) -> Result<Option<Entity>> {
    if p.risk.is_some() || p.target.is_some() || p.note {
        return Err(format!(
            "{} depends on identity; it runs individually",
            p.id
        ));
    }
    let writes_site = p.commitments.iter().chain(&p.effects).any(|effect| {
        matches!(
            effect,
            Effect::Condition { .. }
                | Effect::Transfer {
                    to: Binding::Place,
                    ..
                }
                | Effect::Transfer {
                    from: Binding::Place,
                    ..
                }
                | Effect::Transform {
                    who: Binding::Place,
                    ..
                }
        )
    });
    if count > 1 && writes_site && p.requires.iter().any(reads_site) {
        // Each member would read the site after the last one wrote it.
        return Err(format!("{} reads a site it writes", p.id));
    }
    for q in &p.requires {
        if !holds(q, e, site, tick)? {
            return Ok(None);
        }
    }
    let (mut member, mut place) = (e.clone(), site.clone());
    for effect in p.commitments.iter().chain(&p.effects) {
        match effect {
            Effect::Transfer {
                from,
                to,
                account,
                amount,
            } => {
                if !take(from, &mut member, &mut place, account, *amount, count)? {
                    return Ok(None);
                }
                give(to, &mut member, &mut place, account, *amount, count)?;
            },
            Effect::Transform {
                who,
                take: t,
                give: g,
            } => {
                for (key, amount) in t {
                    if !take(who, &mut member, &mut place, key, *amount, count)? {
                        return Ok(None);
                    }
                }
                for (key, amount) in g {
                    give(who, &mut member, &mut place, key, *amount, count)?;
                }
            },
            Effect::Condition { key, delta } => {
                let times = i64::try_from(count).map_err(|e| e.to_string())?;
                let total = delta.checked_mul(times).ok_or("condition overflow")?;
                let slot = place.conditions.entry(key.clone()).or_default();
                *slot = slot.checked_add(total).ok_or("condition overflow")?;
            },
            Effect::Death => member.alive = false,
            other => return Err(format!("the crowd cannot apply {other:?}")),
        }
    }
    *site = place;
    Ok(Some(normalize(member)))
}

pub(super) fn matter(ledger: &Ledger, rules: &Rules) -> u128 {
    ledger
        .iter()
        .filter(|(k, _)| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. })))
        .map(|(_, v)| u128::from(*v))
        .sum()
}
