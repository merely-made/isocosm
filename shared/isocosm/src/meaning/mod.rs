// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each query reads and what each effect does, written once (ruling
//! 219). The individual runner applies them to one member at a time; the
//! crowd applies them to every member of one state at once. Both call this
//! code, so the two can only differ in who is chosen, never in meaning.

use crate::{
    Result,
    rules::{
        AccountKind, Binding, BodyRules, Conversion, Development, Effect, Need, Query, Reading,
        Rules, Seeding, expressing,
    },
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod births;
mod grow;
pub(crate) use grow::grow;

/// The needs a world's minds read their mood from; none without a mind.
pub(crate) fn needs(rules: &Rules) -> &[Need] {
    rules.mind.as_ref().map_or(&[], |m| &m.needs)
}

pub(crate) fn value(ledger: &Ledger, key: &str) -> u64 {
    ledger.get(key).copied().unwrap_or(0)
}

/// X2's native readings of a body (ruling 453): each the sum, over its
/// living parts, of what one part shows.
pub(crate) fn body_reading(
    body: &Entity,
    r: &Reading,
    rules: &Rules,
    lineages: Option<&BTreeMap<Key, Lineage>>,
) -> i64 {
    if let Reading::Lacking { .. } = r {
        let d = lineages.and_then(|l| l.get(&body.lineage)?.development.as_ref());
        let lacking = d.map_or(0, |d| crate::growth::lacking_mass(rules, d, body));
        return i64::try_from(lacking).unwrap_or(i64::MAX);
    }
    let living = body.parts.values().filter(|p| !p.severed);
    let total: u128 = living.map(|p| r.of_part(p, rules.body())).sum();
    i64::try_from(total).unwrap_or(i64::MAX)
}

/// Whether the rules declare `key` matter, of whatever lineage.
pub(crate) fn matter(rules: &Rules, key: &str) -> bool {
    matches!(rules.accounts.get(key), Some(AccountKind::Matter { .. }))
}

/// The matter a ledger holds: its entries in accounts the rules declare
/// matter, whatever lineage they belong to.
pub(crate) fn mass(ledger: &Ledger, rules: &Rules) -> u128 {
    ledger
        .iter()
        .filter(|(k, _)| matter(rules, k))
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
    /// The actor's part the process binds, if it binds one.
    pub part: Option<Id>,
    pub site: Option<&'a Site>,
    pub tick: Tick,
    pub related: &'a dyn Fn(&Key) -> Result<bool>,
    pub rules: &'a Rules,
    /// The lineages a body's recipe is read from, where the runner keeps
    /// them.
    pub lineages: Option<&'a BTreeMap<Key, Lineage>>,
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
            Binding::Part => Err("a part is not a body".into()),
        }
    }
    /// The bound part, as the actor holds it.
    fn part(&self) -> Result<&'a Part> {
        let id = self.part.ok_or("no part is bound")?;
        let actor = self.body(Binding::Actor)?;
        Ok(actor.parts.get(&id).ok_or("bound part missing")?)
    }
    /// What a binding holds of `key`: a body's own matter through its parts
    /// (ruling 504), the bound part's own ledger, or the site's.
    fn held(&self, b: Binding, key: &str) -> Result<u64> {
        match b {
            Binding::Place => Ok(value(&self.site.ok_or("site missing")?.accounts, key)),
            Binding::Part => Ok(value(&self.part()?.matter, key)),
            other => Ok(crate::anatomy::held(self.body(other)?, self.rules, key)),
        }
    }
    /// The matter a binding holds in all, a body's parts' included.
    fn matter(&self, b: Binding) -> Result<u128> {
        match b {
            Binding::Place => Ok(mass(&self.site.ok_or("site missing")?.accounts, self.rules)),
            Binding::Part => Ok(mass(&self.part()?.matter, self.rules)),
            other => Ok(mass(&crate::anatomy::books(self.body(other)?), self.rules)),
        }
    }
}

/// Whether a query holds, and the reading a receipt records for it.
pub(crate) fn read(q: &Query, s: &Scene) -> Result<(bool, String)> {
    Ok(match q {
        Query::Alive(Binding::Part) => {
            let v = !s.part()?.severed;
            (v, v.to_string())
        },
        Query::Alive(b) => {
            let v = s.body(*b)?.alive;
            (v, v.to_string())
        },
        Query::Trait {
            who: Binding::Part,
            key,
        } => {
            let v = s.part()?.traits.contains(key);
            (v, v.to_string())
        },
        Query::Trait { who, key } => {
            let v = s.body(*who)?.traits.contains(key);
            (v, v.to_string())
        },
        Query::Account { who, key, at_least } => {
            let v = s.held(*who, key)?;
            (v >= *at_least, v.to_string())
        },
        Query::Below { who, key, amount } => {
            let v = s.held(*who, key)?;
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
        Query::Holds { who, at_least } => {
            let v = s.matter(*who)?;
            (v >= u128::from(*at_least), v.to_string())
        },
        Query::Computed(x) => {
            let v = computed(x, s)?;
            (v != 0, v.to_string())
        },
        // The address a receipt carries: which part, at which revision.
        Query::Expresses { function } => {
            let actor = s.body(Binding::Actor)?;
            match expressing(actor, function) {
                Some(id) => (
                    true,
                    format!("part {id} at revision {}", actor.body_revision),
                ),
                None => (false, "no live part".into()),
            }
        },
    })
}

/// An expression's value as a scene shows it, a sum over parts reading a
/// body's living parts.
fn computed(x: &crate::rules::Expr, s: &Scene) -> Result<i64> {
    let mut read = |r: &Reading| -> Result<i64> {
        match (r, r.who()) {
            (Reading::Kept { .. }, _) => Err("a requirement keeps no values".into()),
            (Reading::Account { key, .. }, who) => {
                i64::try_from(s.held(who, key)?).map_err(|e| e.to_string())
            },
            (r, Binding::Part) => {
                Ok(i64::try_from(r.of_part(s.part()?, s.rules.body())).unwrap_or(i64::MAX))
            },
            (r, who) => Ok(body_reading(s.body(who)?, r, s.rules, s.lineages)),
        }
    };
    let mut parts = |who: Binding| -> Result<(Vec<Part>, BodyRules)> {
        let living = s.body(who)?.parts.values().filter(|p| !p.severed);
        Ok((living.cloned().collect(), s.rules.body()))
    };
    let mut draw = |_: u64, _: u8| -> Result<u64> { Err("a requirement draws nothing".into()) };
    x.eval_in(&mut read, &mut draw, &mut parts)
}

/// The actor's mood, read and never kept: the weights of the needs that
/// hold for it (ruling 227).
pub(crate) fn mood(s: &Scene) -> Result<i64> {
    let actor = s.body(Binding::Actor)?;
    let mut total = 0i64;
    for need in needs(s.rules) {
        if need.traits.iter().all(|t| actor.traits.contains(t)) && read(&need.query, s)?.0 {
            total = total.checked_add(need.weight).ok_or("mood overflow")?;
        }
    }
    Ok(total)
}

/// What eating `amount` takes of a ledger's matter (ruling 287): each
/// matter account's exact share floored, the units left over going to the
/// largest remainders, ties in key order. Takes it all when it holds less.
pub(crate) fn share(ledger: &Ledger, rules: &Rules, amount: u64) -> Ledger {
    let matter: Vec<(&Key, u64)> = ledger
        .iter()
        .filter(|(k, v)| {
            **v > 0 && matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. }))
        })
        .map(|(k, v)| (k, *v))
        .collect();
    let total: u128 = matter.iter().map(|(_, v)| u128::from(*v)).sum();
    let wanted = u128::from(amount).min(total);
    if wanted == total {
        return matter.into_iter().map(|(k, v)| (k.clone(), v)).collect();
    }
    let mut taken: Vec<(Key, u64, u128)> = matter
        .iter()
        .map(|(k, v)| {
            let product = u128::from(*v) * wanted;
            ((*k).clone(), (product / total) as u64, product % total)
        })
        .collect();
    let assigned: u128 = taken.iter().map(|t| u128::from(t.1)).sum();
    let mut order: Vec<usize> = (0..taken.len()).collect();
    // A stable sort keeps key order among equal remainders.
    order.sort_by(|a, b| taken[*b].2.cmp(&taken[*a].2));
    for &i in order.iter().take((wanted - assigned) as usize) {
        taken[i].1 += 1;
    }
    taken
        .into_iter()
        .filter(|t| t.1 > 0)
        .map(|(k, v, _)| (k, v))
        .collect()
}

/// The states an effect writes. A crowd's parties stand for every member of
/// one state at once: the members' shared state changes once for all, and
/// a ledger they share with others, the site's, carries each of them.
pub(crate) trait Parties {
    /// Checks a binding resolves to a ledger, as a transform does before
    /// it takes or gives anything.
    fn reach(&mut self, who: Binding) -> Result<()>;
    /// What one member's share of a binding's ledger holds of `key`.
    fn held(&mut self, who: Binding, key: &str) -> Result<u64>;
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()>;
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()>;
    fn body(&mut self, who: Binding) -> Result<&mut Entity>;
    /// The actor's body and the part the process binds in it.
    fn part(&mut self) -> Result<(&mut Entity, Id)>;
    /// Moves a condition at the site.
    fn shift(&mut self, key: &str, delta: i64) -> Result<()>;
    /// What a lineage's bodies develop from (ruling 478).
    fn development(&mut self, lineage: &str) -> Result<Development> {
        Err(format!("{lineage}'s bodies cannot grow here"))
    }
}

/// The effects both runners apply. The others write the world's records,
/// relations, notes, births, polities and legend, which only the individual
/// runner keeps; `None` hands those back to it.
pub(crate) fn effect(p: &mut impl Parties, rules: &Rules, e: &Effect) -> Option<Result<()>> {
    Some(match e {
        Effect::Transfer {
            from,
            to,
            account,
            amount,
        } => amount.resolved().and_then(|amount| {
            p.take(*from, account, amount)
                .and_then(|()| p.give(*to, account, amount))
        }),
        Effect::Transform {
            who,
            take,
            give,
            conversion,
        } => transform(p, rules, *who, (take, give), *conversion),
        Effect::Condition { key, delta } => p.shift(key, *delta),
        Effect::Trait {
            who: Binding::Part,
            key,
            present,
        } => p.part().and_then(|(b, id)| mark_part(b, id, key, *present)),
        Effect::Trait { who, key, present } => p.body(*who).and_then(|b| mark(b, key, *present)),
        Effect::Practice { key, amount } => amount.resolved().and_then(|amount| {
            p.body(Binding::Actor)
                .and_then(|b| practice(b, key, amount))
        }),
        Effect::Death => p.body(Binding::Actor).map(|b| b.alive = false),
        Effect::Ease { who, key, amount } => amount
            .resolved()
            .and_then(|amount| p.body(*who).map(|b| ease(b, key, amount))),
        Effect::Allocate { from, to, cells } => cells.resolved().and_then(|cells| {
            p.part()
                .and_then(|(b, id)| allocate(rules, b, id, from.as_deref(), to, cells))
        }),
        Effect::Spend {
            from,
            to,
            amount,
            into,
        } => amount
            .resolved()
            .and_then(|amount| spend(p, from, *to, into.as_deref(), amount).map(|_| ())),
        Effect::Convert {
            who,
            from,
            to,
            amount,
            conversion,
        } => amount
            .resolved()
            .and_then(|amount| convert(p, rules, *who, (from, to), amount, *conversion))
            .map(|_| ()),
        Effect::Grow {
            from,
            into,
            conversion,
        } => grow(p, rules, from, into, *conversion).map(|_| ()),
        _ => return None,
    })
}

/// An ordered take (ruling 446): each of the actor's accounts in turn gives
/// what it holds, up to what is still owed, arriving at `to` as itself or
/// as `into`. Returns what each account paid.
pub(crate) fn spend(
    p: &mut impl Parties,
    from: &[Key],
    to: Binding,
    into: Option<&str>,
    amount: u64,
) -> Result<Vec<(Key, u64)>> {
    let mut owed = amount;
    let mut paid = Vec::new();
    for key in from {
        let given = p.held(Binding::Actor, key)?.min(owed);
        if given == 0 {
            continue;
        }
        p.take(Binding::Actor, key, given)?;
        p.give(to, into.unwrap_or(key), given)?;
        owed -= given;
        paid.push((key.clone(), given));
    }
    Ok(paid)
}

/// A conversion of up to `amount` of a ledger's `from` accounts, a share of
/// each in proportion as a meal takes (ruling 287), into `to`. A synthesis
/// or digestion gives only the body's own lineage's matter (ruling 357).
/// Returns what it took.
pub(crate) fn convert(
    p: &mut impl Parties,
    rules: &Rules,
    who: Binding,
    (from, to): (&[Key], &Key),
    amount: u64,
    conversion: Conversion,
) -> Result<Ledger> {
    p.reach(who)?;
    if let Conversion::Synthesis | Conversion::Digestion = conversion {
        let own = p.body(who)?.lineage.clone();
        let mine = matches!(rules.accounts.get(to), Some(AccountKind::Matter { lineage, .. }) if *lineage == own);
        if !mine {
            return Err(format!(
                "{conversion:?} gives {to}, which is not {own}'s own matter"
            ));
        }
    }
    let mut offered = Ledger::new();
    for key in from {
        offered.insert(key.clone(), p.held(who, key)?);
    }
    let taken = share(&offered, rules, amount);
    let mut total = 0u64;
    for (key, value) in &taken {
        p.take(who, key, *value)?;
        total = total.checked_add(*value).ok_or("conversion overflow")?;
    }
    p.give(who, to, total)?;
    Ok(taken)
}

fn ease(e: &mut Entity, key: &str, amount: u64) {
    if let Some(v) = e.accounts.get_mut(key) {
        *v -= (*v).min(amount);
    }
}

/// A transform takes and gives on one ledger. A declared synthesis or
/// digestion gives only the body's own lineage's matter (ruling 357);
/// admission has checked the rest of what each conversion takes and gives.
fn transform(
    p: &mut impl Parties,
    rules: &Rules,
    who: Binding,
    (take, give): (&Ledger, &Ledger),
    conversion: Option<Conversion>,
) -> Result<()> {
    p.reach(who)?;
    if let Some(kind @ (Conversion::Synthesis | Conversion::Digestion)) = conversion {
        let own = p.body(who)?.lineage.clone();
        let foreign = give.keys().find(|k| {
            !matches!(rules.accounts.get(*k), Some(AccountKind::Matter { lineage, .. }) if *lineage == own)
        });
        if let Some(key) = foreign {
            return Err(format!(
                "{kind:?} gives {key}, which is not {own}'s own matter"
            ));
        }
    }
    for (key, amount) in take {
        p.take(who, key, *amount)?;
    }
    for (key, amount) in give {
        p.give(who, key, *amount)?;
    }
    Ok(())
}

fn set(traits: &mut BTreeSet<Key>, key: &str, present: bool) {
    if present {
        traits.insert(key.into());
    } else {
        traits.remove(key);
    }
}

fn revise(e: &mut Entity) -> Result<()> {
    e.body_revision = e
        .body_revision
        .checked_add(1)
        .ok_or("body revision overflow")?;
    Ok(())
}

fn mark(e: &mut Entity, key: &str, present: bool) -> Result<()> {
    set(&mut e.traits, key, present);
    revise(e)
}

/// A part's trait changes its body, so the body's revision moves.
fn mark_part(e: &mut Entity, part: Id, key: &str, present: bool) -> Result<()> {
    let p = e.parts.get_mut(&part).ok_or("bound part missing")?;
    set(&mut p.traits, key, present);
    revise(e)
}

/// X6: `cells` of the bound part moved to a function it expresses, from
/// another's or from its free cells, the part's capacity never exceeded. A
/// development places an acquired function its shape admits (ruling 338).
fn allocate(
    rules: &Rules,
    e: &mut Entity,
    part: Id,
    from: Option<&str>,
    to: &str,
    cells: u64,
) -> Result<()> {
    let p = e.parts.get_mut(&part).ok_or("bound part missing")?;
    let cells = u32::try_from(cells).map_err(|_| "allocation overflow")?;
    if !p.functions.contains(to) {
        let acquired = rules
            .functions
            .get(to)
            .is_some_and(|f| f.seeding == Seeding::Acquired);
        if !acquired {
            return Err(format!("the bound part cannot come to express {to}"));
        }
        p.functions.insert(to.into());
    }
    match from {
        Some(f) => {
            let held = p.cells.get(f).copied().unwrap_or(0);
            if held < cells {
                return Err(format!("the bound part holds {held} cells for {f}"));
            }
            match held - cells {
                0 => p.cells.remove(f),
                left => p.cells.insert(f.into(), left),
            };
        },
        None => {
            let used: u64 = p.cells.values().map(|c| u64::from(*c)).sum();
            let free = u64::from(crate::anatomy::capacity(p)).saturating_sub(used);
            if free < u64::from(cells) {
                return Err(format!("the bound part has {free} free cells"));
            }
        },
    }
    if cells > 0 {
        let slot = p.cells.entry(to.into()).or_default();
        *slot = slot.checked_add(cells).ok_or("allocation overflow")?;
    }
    revise(e)
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

#[cfg(test)]
mod tests;
