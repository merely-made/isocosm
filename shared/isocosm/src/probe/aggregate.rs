// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A process applied to every member of a state at once, the aggregate form
//! of the sim plan's §3.1. What queries read and effects do is `meaning`'s,
//! shared with the individual runner. What is the crowd's own: it refuses
//! anything that depends on identity (risk, targets, relations, noted
//! events), a site ledger carries every member of the state, and a site
//! debit that would cover only some of them is refused, since a crowd has no
//! member order to ration by.

use crate::{
    Result,
    meaning::{self, Named, Parties, Scene, credit, debit, value},
    rules::{AccountKind, Binding, Effect, Process, Query, Rules},
    schema::*,
};

/// Zero entries dropped: no query and no inspection tells absent from zero.
pub(super) fn normalize(mut e: Entity) -> Entity {
    e.accounts.retain(|_, v| *v != 0);
    e
}

fn no_relations(_: &Key) -> Result<bool> {
    Err("the crowd keeps no relations".into())
}

/// What a query reads of one member's state at a site, or of the site alone.
pub(super) fn holds(q: &Query, e: Option<&Entity>, site: &Site, tick: Tick) -> Result<bool> {
    let scene = Scene {
        actor: e,
        target: Named::Unnamed,
        site: Some(site),
        tick,
        related: &no_relations,
    };
    Ok(meaning::read(q, &scene)?.0)
}

/// The crowd's parties: one state's members and their site.
struct Bin<'a> {
    member: &'a mut Entity,
    site: &'a mut Site,
    count: u64,
    contended: bool,
}

impl Parties for Bin<'_> {
    fn reach(&mut self, who: Binding) -> Result<()> {
        match who {
            Binding::Target => Err("the crowd has no targets".into()),
            _ => Ok(()),
        }
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => debit(&mut self.member.accounts, key, amount),
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                let have = value(&self.site.accounts, key);
                // Enough for all, or not enough for one: every member fares
                // alike. Anything between would feed some in identity order.
                if have >= total || have < amount {
                    debit(&mut self.site.accounts, key, total)
                } else {
                    self.contended = true;
                    Err(format!("contended site debit of {key}"))
                }
            },
            Binding::Target => Err("the crowd has no targets".into()),
        }
    }
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => credit(&mut self.member.accounts, key, amount),
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                credit(&mut self.site.accounts, key, total)
            },
            Binding::Target => Err("the crowd has no targets".into()),
        }
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        match who {
            Binding::Actor => Ok(self.member),
            _ => Err("the crowd binds only the actor's body".into()),
        }
    }
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site.conditions, key, delta, self.count)
    }
}

fn identity_bound(p: &Process) -> bool {
    let target = |b: &Binding| *b == Binding::Target;
    p.risk.is_some()
        || p.target.is_some()
        || p.note
        || p.requires.iter().any(|q| match q {
            Query::Related { .. } => true,
            Query::Alive(b) => target(b),
            Query::Trait { who, .. }
            | Query::Account { who, .. }
            | Query::Below { who, .. }
            | Query::Part { who, .. } => target(who),
            Query::Age { .. } | Query::Condition { .. } => false,
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

fn writes_site(e: &Effect) -> bool {
    matches!(
        e,
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
    if identity_bound(p) {
        return Err(format!(
            "{} depends on identity; it runs individually",
            p.id
        ));
    }
    let effects: Vec<&Effect> = p.commitments.iter().chain(&p.effects).collect();
    if count > 1 && effects.iter().any(|e| writes_site(e)) && p.requires.iter().any(reads_site) {
        // Each member would read the site after the last one wrote it.
        return Err(format!("{} reads a site it writes", p.id));
    }
    for q in &p.requires {
        // A query that errs blocks, as it does in the core.
        if !holds(q, Some(e), site, tick).unwrap_or(false) {
            return Ok(None);
        }
    }
    let (mut member, mut place) = (e.clone(), site.clone());
    let mut bin = Bin {
        member: &mut member,
        site: &mut place,
        count,
        contended: false,
    };
    for effect in effects {
        match meaning::effect(&mut bin, effect) {
            None => return Err(format!("the crowd cannot apply {effect:?}")),
            Some(Ok(())) => {},
            Some(Err(why)) if bin.contended => return Err(format!("{}: {why}", p.id)),
            Some(Err(_)) => return Ok(None),
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
