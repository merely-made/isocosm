// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each query reads and what each effect does, written once (ruling
//! 219). The individual runner applies them to one member at a time; the
//! crowd applies them to every member of one state at once. Both call this
//! code, so the two can only differ in who is chosen, never in meaning.

use crate::{
    Result,
    rules::{AccountKind, Binding, Effect, Need, Query, Rules},
    schema::*,
};
use std::collections::BTreeMap;

/// The needs a world's minds read their mood from; none without a mind.
pub(crate) fn needs(rules: &Rules) -> &[Need] {
    rules.mind.as_ref().map_or(&[], |m| &m.needs)
}

pub(crate) fn value(ledger: &Ledger, key: &str) -> u64 {
    ledger.get(key).copied().unwrap_or(0)
}

/// The matter a ledger holds: its entries in accounts the rules declare
/// matter, whatever lineage they belong to.
pub(crate) fn mass(ledger: &Ledger, rules: &Rules) -> u128 {
    ledger
        .iter()
        .filter(|(k, _)| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. })))
        .map(|(_, v)| u128::from(*v))
        .sum()
}

pub(crate) fn debit(ledger: &mut Ledger, key: &str, amount: u64) -> Result<()> {
    let slot = ledger.entry(key.into()).or_default();
    *slot = slot
        .checked_sub(amount)
        .ok_or_else(|| format!("insufficient {key}"))?;
    Ok(())
}

pub(crate) fn credit(ledger: &mut Ledger, key: &str, amount: u64) -> Result<()> {
    let slot = ledger.entry(key.into()).or_default();
    *slot = slot
        .checked_add(amount)
        .ok_or_else(|| format!("account overflow: {key}"))?;
    Ok(())
}

/// A binding to the target, which a process may leave unnamed, or name and
/// then find missing.
pub(crate) enum Named<'a> {
    Unnamed,
    Missing,
    Found(&'a Entity),
}

/// What a query can see. Relations are answered by whoever keeps them.
pub(crate) struct Scene<'a> {
    pub actor: Option<&'a Entity>,
    pub target: Named<'a>,
    pub site: Option<&'a Site>,
    pub tick: Tick,
    pub related: &'a dyn Fn(&Key) -> Result<bool>,
    pub needs: &'a [Need],
}

impl<'a> Scene<'a> {
    fn body(&self, b: Binding) -> Result<&'a Entity> {
        match b {
            Binding::Actor => self.actor.ok_or_else(|| "body missing".into()),
            Binding::Target => match self.target {
                Named::Unnamed => Err("target is required".into()),
                Named::Missing => Err("body missing".into()),
                Named::Found(e) => Ok(e),
            },
            Binding::Place => Err("place is not a body".into()),
        }
    }
    fn ledger(&self, b: Binding) -> Result<&'a Ledger> {
        match b {
            Binding::Place => Ok(&self.site.ok_or("site missing")?.accounts),
            other => Ok(&self.body(other)?.accounts),
        }
    }
}

/// Whether a query holds, and the reading a receipt records for it.
pub(crate) fn read(q: &Query, s: &Scene) -> Result<(bool, String)> {
    Ok(match q {
        Query::Alive(b) => {
            let v = s.body(*b)?.alive;
            (v, v.to_string())
        },
        Query::Trait { who, key } => {
            let v = s.body(*who)?.traits.contains(key);
            (v, v.to_string())
        },
        Query::Account { who, key, at_least } => {
            let v = value(s.ledger(*who)?, key);
            (v >= *at_least, v.to_string())
        },
        Query::Below { who, key, amount } => {
            let v = value(s.ledger(*who)?, key);
            (v < *amount, v.to_string())
        },
        Query::Age { at_least } => {
            let v = s.tick.saturating_sub(s.body(Binding::Actor)?.born);
            (v >= *at_least, v.to_string())
        },
        Query::Condition { key, at_least } => {
            let v = s.site.and_then(|site| site.conditions.get(key));
            (v.is_some_and(|v| v >= at_least), format!("{v:?}"))
        },
        Query::Part {
            who,
            revision,
            part,
        } => {
            let b = s.body(*who)?;
            let p = b.parts.get(part);
            (
                b.body_revision == *revision && p.is_some_and(|p| !p.severed),
                format!("revision {}: {p:?}", b.body_revision),
            )
        },
        Query::Related { kind } => {
            let v = (s.related)(kind)?;
            (v, v.to_string())
        },
        Query::Mood { at_least } => {
            let v = mood(s)?;
            (v >= *at_least, v.to_string())
        },
        Query::MoodBelow { amount } => {
            let v = mood(s)?;
            (v < *amount, v.to_string())
        },
    })
}

/// The actor's mood, read and never kept: the weights of the needs that
/// hold for it (ruling 227).
pub(crate) fn mood(s: &Scene) -> Result<i64> {
    let actor = s.body(Binding::Actor)?;
    let mut total = 0i64;
    for need in s.needs {
        if need.traits.iter().all(|t| actor.traits.contains(t)) && read(&need.query, s)?.0 {
            total = total.checked_add(need.weight).ok_or("mood overflow")?;
        }
    }
    Ok(total)
}

/// The states an effect writes. A crowd's parties stand for every member of
/// one state at once: the members' shared state changes once for all, and
/// a ledger they share with others, the site's, carries each of them.
pub(crate) trait Parties {
    /// Checks a binding resolves to a ledger, as a transform does before
    /// it takes or gives anything.
    fn reach(&mut self, who: Binding) -> Result<()>;
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()>;
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()>;
    fn body(&mut self, who: Binding) -> Result<&mut Entity>;
    /// Moves a condition at the site.
    fn shift(&mut self, key: &str, delta: i64) -> Result<()>;
}

/// The effects both runners apply. The others write the world's records,
/// relations, notes, births, polities and legend, which only the individual
/// runner keeps; `None` hands those back to it.
pub(crate) fn effect(p: &mut impl Parties, e: &Effect) -> Option<Result<()>> {
    Some(match e {
        Effect::Transfer {
            from,
            to,
            account,
            amount,
        } => p
            .take(*from, account, *amount)
            .and_then(|()| p.give(*to, account, *amount)),
        Effect::Transform { who, take, give } => transform(p, *who, take, give),
        Effect::Condition { key, delta } => p.shift(key, *delta),
        Effect::Trait { who, key, present } => p.body(*who).and_then(|b| mark(b, key, *present)),
        Effect::Practice { key, amount } => p
            .body(Binding::Actor)
            .and_then(|b| practice(b, key, *amount)),
        Effect::Death => p.body(Binding::Actor).map(|b| b.alive = false),
        Effect::Ease { who, key, amount } => p.body(*who).map(|b| ease(b, key, *amount)),
        _ => return None,
    })
}

fn ease(e: &mut Entity, key: &str, amount: u64) {
    if let Some(v) = e.accounts.get_mut(key) {
        *v -= (*v).min(amount);
    }
}

fn transform(p: &mut impl Parties, who: Binding, take: &Ledger, give: &Ledger) -> Result<()> {
    p.reach(who)?;
    for (key, amount) in take {
        p.take(who, key, *amount)?;
    }
    for (key, amount) in give {
        p.give(who, key, *amount)?;
    }
    Ok(())
}

fn mark(e: &mut Entity, key: &str, present: bool) -> Result<()> {
    if present {
        e.traits.insert(key.into());
    } else {
        e.traits.remove(key);
    }
    e.body_revision = e
        .body_revision
        .checked_add(1)
        .ok_or("body revision overflow")?;
    Ok(())
}

fn practice(e: &mut Entity, key: &str, amount: u64) -> Result<()> {
    let slot = e.skills.entry(key.into()).or_default();
    *slot = slot.checked_add(amount).ok_or("skill overflow")?;
    Ok(())
}

/// A condition moved by `delta`, `times` over.
pub(crate) fn shift(
    conditions: &mut BTreeMap<Key, i64>,
    key: &str,
    delta: i64,
    times: u64,
) -> Result<()> {
    let times = i64::try_from(times).map_err(|e| e.to_string())?;
    let total = delta.checked_mul(times).ok_or("condition overflow")?;
    let slot = conditions.entry(key.into()).or_default();
    *slot = slot.checked_add(total).ok_or("condition overflow")?;
    Ok(())
}
