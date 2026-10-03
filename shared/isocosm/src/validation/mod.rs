// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, meaning::mass, rules::*, schema::Key};
use std::collections::BTreeMap;

mod body;
mod conversion;
mod recipe;

pub(crate) use body::part;
pub(crate) use conversion::kinds as conversions;
pub(crate) use recipe::development;

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
        Query::Expresses { function } if !rules.functions.contains_key(function) => {
            return Err(format!("unknown function {function}"));
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
                | Query::Holds {
                    who: Binding::Actor,
                    ..
                }
                | Query::Expresses { .. }
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
            if body::reads_part(&kind.hungry)? {
                return Err(format!("{id} reads a part no process binds"));
            }
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
        || rules.processes.len() > rules.limits.processes
    {
        return Err("invalid rules limits".into());
    }
    if rules.tick_microseconds == Some(0) {
        return Err("the clock's unit must be some time".into());
    }
    rules.deep_time_ceiling()?;
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
    body::catalogue(rules)?;
    recipe::kinds(rules)?;
    for (id, p) in &rules.processes {
        key(id)?;
        if id != &p.id {
            return Err("process key does not match identity".into());
        }
        body::process(p)?;
        // A part's ledger holds only matter (ruling 504).
        for q in &p.requires {
            if let Query::Account {
                who: Binding::Part,
                key,
                ..
            }
            | Query::Below {
                who: Binding::Part,
                key,
                ..
            } = q
                && !crate::meaning::matter(rules, key)
            {
                return Err(format!("{id}: a part keeps only matter, not {key}"));
            }
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
            if let Query::Computed(x) = q {
                x.validate().map_err(|why| format!("{id}: {why}"))?;
                reads(rules, p, id, x)?;
                let kept = x
                    .reads()
                    .iter()
                    .any(|u| matches!(u.reading, Reading::Kept { .. }));
                if x.draws() || kept {
                    return Err(format!("{id} requires what only an act computes"));
                }
            }
        }
        for e in effects {
            effect(rules, p, id, e)?;
        }
        kept_first(p, id)?;
    }
    mind(rules)?;
    competitions(rules)
}

/// A kept value is read only after its act keeps it, and kept only at the
/// top of the act, never within a guard's branch.
fn kept_first(p: &Process, id: &str) -> Result<()> {
    let risky = p.risk.iter().flat_map(|r| &r.effects);
    for outcomes in [p.effects.iter().collect::<Vec<_>>(), risky.collect()] {
        let mut kept = std::collections::BTreeSet::new();
        for e in p.commitments.iter().chain(outcomes) {
            let mut exprs: Vec<&Expr> = e.computed().into_iter().collect();
            for a in e.amounts() {
                if let Amount::Computed(x) = a {
                    exprs.push(x);
                }
            }
            for inner in e.branches() {
                if matches!(inner, Effect::Keep { .. }) {
                    return Err(format!("{id} keeps a value within a guard"));
                }
                exprs.extend(inner.computed());
            }
            for u in exprs.iter().flat_map(|x| x.reads()) {
                if let Reading::Kept { name } = u.reading
                    && !kept.contains(name)
                {
                    return Err(format!("{id} reads {name} before keeping it"));
                }
            }
            if let Effect::Keep { name, .. } = e {
                kept.insert(name.clone());
            }
        }
    }
    Ok(())
}

/// One effect's checks; a guarded effect's guard and inner effect are
/// checked in turn (X5).
fn effect(rules: &Rules, p: &Process, id: &str, e: &Effect) -> Result<()> {
    conversion::declared(rules, id, e)?;
    for a in e.amounts() {
        amount(rules, p, id, a)?;
    }
    if let Effect::Keep { value, .. } = e {
        value.validate().map_err(|why| format!("{id}: {why}"))?;
        reads(rules, p, id, value)?;
    }
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
        // The target is eaten into the eater's own matter; a site or
        // the eater itself is not eaten, a meal of nothing is no meal, and
        // a meal naming accounts names matter (ruling 456).
        Effect::Eat {
            from,
            amount,
            into,
            of,
        } => {
            matter(rules, into)?;
            for key in of {
                matter(rules, key)?;
            }
            if *from != Binding::Target || *amount == Amount::Fixed(0) {
                return Err(format!("{id} eats what cannot be eaten"));
            }
        },
        Effect::When { guard, .. } => {
            if e.branches().any(|b| matches!(b, Effect::When { .. })) {
                return Err(format!("{id} guards a guard"));
            }
            guard.validate().map_err(|why| format!("{id}: {why}"))?;
            reads(rules, p, id, guard)?;
            for inner in e.branches() {
                effect(rules, p, id, inner)?;
            }
        },
        // An ordered take drains the actor's own accounts into another
        // ledger (ruling 446).
        Effect::Spend { from, to, into, .. } => {
            if let Some(into) = into {
                matter(rules, into)?;
            }
            if from.is_empty() {
                return Err(format!("{id} spends from no account"));
            }
            for a in from {
                account(rules, a)?;
            }
            if matches!(to, Binding::Actor | Binding::Part)
                || (*to == Binding::Target && p.target.is_none())
            {
                return Err(format!("{id} spends to {to:?}, which it cannot"));
            }
        },
        // Growth turns matter in hand into the actor's new parts, their
        // price returned to the place (rulings 479 and 510).
        Effect::Grow {
            from,
            into,
            conversion,
        } => {
            matter(rules, from)?;
            matter(rules, into)?;
            if *conversion == Conversion::Mineralization {
                return Err(format!("{id} grows a body by mineralizing"));
            }
        },
        // A conversion takes matter of one ledger into another account of
        // it, as a declared conversion (rulings 342 and 357).
        Effect::Convert { who, from, to, .. } => {
            for a in from.iter().chain([to]) {
                matter(rules, a)?;
            }
            if from.is_empty() || from.contains(to) || *who == Binding::Part {
                return Err(format!("{id} converts what cannot be converted"));
            }
            if *who == Binding::Target && p.target.is_none() {
                return Err(format!("{id} converts a target it does not bind"));
            }
        },
        // Allocation moves the bound part's cells between catalogue
        // functions (X6).
        Effect::Allocate { from, to, .. } => {
            if p.expresses().is_none() {
                return Err(format!("{id} allocates without binding a part"));
            }
            for f in from.iter().chain([to]) {
                if !rules.functions.contains_key(f) {
                    return Err(format!("{id} allocates to an unknown function {f}"));
                }
            }
            if from.as_ref() == Some(to) {
                return Err(format!("{id} allocates a function to itself"));
            }
        },
        _ => (),
    }
    Ok(())
}

/// A computed amount keeps its bounds and reads only what its act binds,
/// a part's own ledger included (rulings 338 and 504).
fn amount(rules: &Rules, p: &Process, id: &str, a: &Amount) -> Result<()> {
    a.validate().map_err(|why| format!("{id}: {why}"))?;
    match a {
        Amount::Computed(e) => reads(rules, p, id, e),
        Amount::Fixed(_) => Ok(()),
    }
}

/// What an expression reads, each binding one its act binds.
fn reads(rules: &Rules, p: &Process, id: &str, e: &Expr) -> Result<()> {
    for u in e.reads() {
        let (r, who) = (u.reading, u.reading.who());
        // A part is read only where the act binds one or a sum over a
        // body's parts reads each in turn (ruling 455).
        let unbound = match who {
            Binding::Part => u.folded.is_none() && p.expresses().is_none(),
            Binding::Target => p.target.is_none(),
            Binding::Actor | Binding::Place => false,
        } || (u.folded == Some(Binding::Target) && p.target.is_none());
        if unbound {
            return Err(format!(
                "{id} reads an amount from {who:?}, which it does not bind"
            ));
        }
        match r {
            Reading::Account { key, .. } => account(rules, key)?,
            _ if who == Binding::Place => {
                return Err(format!("{id} reads a body from a site, which has none"));
            },
            Reading::CellWeight { .. } if who != Binding::Part => {
                return Err(format!("{id} reads a cell weight of a whole body"));
            },
            Reading::Span { function, .. }
            | Reading::Cells { function, .. }
            | Reading::CellMass { function, .. }
                if !rules.functions.contains_key(function) =>
            {
                return Err(format!("{id} reads an unknown function {function}"));
            },
            _ => {},
        }
    }
    Ok(())
}
