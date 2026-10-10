// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The effects both runners apply to bodies and ledgers: transfers, spends,
//! conversions, marks, allocation, practice and conditions.

use super::*;

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
            let (b, id) = p.part()?;
            let before = (!rules.systems.is_empty()).then(|| b.clone());
            allocate(rules, b, id, from.as_deref(), to, cells)?;
            if let Some(before) = before {
                crate::systems::take_up(b, rules, &before);
            }
            Ok(())
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
        Effect::Carry {
            who,
            function,
            role,
            ask,
            lands,
        } => ask
            .resolved()
            .and_then(|ask| carriage::carry(p, rules, *who, (function, *role), (ask, lands))),
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
fn mark_part(e: &mut Entity, part: PartId, key: &str, present: bool) -> Result<()> {
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
    part: PartId,
    from: Option<&str>,
    to: &str,
    cells: u64,
) -> Result<()> {
    let half = e.extent(part);
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
            let free = u64::from(crate::anatomy::capacity(half)).saturating_sub(used);
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
