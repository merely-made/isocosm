// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A process applied to every member of a state at once, the aggregate form
//! of the sim plan's §3.1. What queries read and effects do is `meaning`'s,
//! shared with the individual runner. What is the crowd's own: it refuses
//! anything that depends on identity (risk, targets, relations, noted
//! events), and a site ledger carries every member of the state. Every
//! member reads its site as the pass began, and a pass's takes of a site
//! are planned first and shared out where they would take more than it
//! holds (ruling 454).

use crate::{
    Result,
    meaning::{self, Named, Parties, Scene, credit, debit},
    rules::{Amount, Binding, Effect, Process, Query, Rules},
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

/// What one member, or a site alone, shows a query.
pub(super) struct Seen<'a> {
    pub member: Option<&'a Entity>,
    pub site: &'a Site,
    pub tick: Tick,
    pub rules: &'a Rules,
}

impl Seen<'_> {
    fn scene(&self) -> Scene<'_> {
        Scene {
            actor: self.member,
            target: Named::Unnamed,
            part: None,
            site: Some(self.site),
            tick: self.tick,
            related: &no_relations,
            rules: self.rules,
        }
    }
    pub(super) fn holds(&self, q: &Query) -> Result<bool> {
        Ok(meaning::read(q, &self.scene())?.0)
    }
    pub(super) fn mood(&self) -> Result<i64> {
        meaning::mood(&self.scene())
    }
}

/// The crowd's parties: one state's members and their site.
struct Bin<'a> {
    member: &'a mut Entity,
    site: &'a mut Site,
    count: u64,
}

impl Parties for Bin<'_> {
    fn held(&mut self, who: Binding, key: &str) -> Result<u64> {
        match who {
            Binding::Actor => Ok(meaning::value(&self.member.accounts, key)),
            // A site's ledger is shared, so no one member holds a share of it.
            _ => Err("the crowd reads no one member's share of shared ground".into()),
        }
    }
    fn reach(&mut self, who: Binding) -> Result<()> {
        match who {
            Binding::Target => Err("the crowd has no targets".into()),
            _ => Ok(()),
        }
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => debit(&mut self.member.accounts, key, amount),
            // Shares make every take of the site whole (ruling 454).
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                debit(&mut self.site.accounts, key, total)
            },
            Binding::Target => Err("the crowd has no targets".into()),
            Binding::Part => Err("a part keeps no ledger".into()),
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
            Binding::Part => Err("a part keeps no ledger".into()),
        }
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        match who {
            Binding::Actor => Ok(self.member),
            _ => Err("the crowd binds only the actor's body".into()),
        }
    }
    fn part(&mut self) -> Result<(&mut Entity, Id)> {
        Err("the crowd binds no parts".into())
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
            | Query::Part { who, .. }
            | Query::Holds { who, .. } => target(who),
            Query::Age { .. }
            | Query::Condition { .. }
            | Query::Mood { .. }
            | Query::MoodBelow { .. }
            | Query::Expresses { .. } => false,
        })
}

/// A mood's needs may read the site's conditions, so it counts as a read.
fn reads_site(q: &Query) -> bool {
    matches!(
        q,
        Query::Account {
            who: Binding::Place,
            ..
        } | Query::Below {
            who: Binding::Place,
            ..
        } | Query::Holds {
            who: Binding::Place,
            ..
        } | Query::Condition { .. }
            | Query::Mood { .. }
            | Query::MoodBelow { .. }
    )
}

/// Whether `p` reads nothing of the site and takes nothing from it, so what
/// it makes of a member does not depend on the site.
pub(super) fn site_free(p: &Process) -> bool {
    let takes = |e: &Effect| {
        matches!(
            e,
            Effect::Transfer {
                from: Binding::Place,
                ..
            } | Effect::Transform {
                who: Binding::Place,
                ..
            }
        )
    };
    !p.requires.iter().any(reads_site) && !p.commitments.iter().chain(&p.effects).any(takes)
}

/// How a state's act runs: planned on a copy of its site, each member's
/// takes of it recorded; applied with each member's shares; or, outside a
/// pass, as it asks, as the core runs a host's act.
pub(super) enum Run<'a> {
    Plan(&'a mut Ledger),
    Act(&'a Ledger),
    Free,
}

/// One member's take of its site, recorded while planning and held to its
/// share when acting; `None` leaves the effect as it was.
fn share_out(e: &Effect, run: &mut Run, left: &mut Ledger, site: &Site) -> Result<Option<Effect>> {
    if matches!(run, Run::Free) {
        return Ok(None);
    }
    match e {
        Effect::Transfer {
            from: Binding::Place,
            account,
            amount,
            ..
        } => {
            let asked = amount.resolved()?;
            let given = match run {
                // A plan asks for the whole take and stages what is there.
                Run::Plan(takes) => {
                    let t = takes.entry(account.clone()).or_default();
                    *t = t.saturating_add(asked);
                    asked.min(meaning::value(&site.accounts, account))
                },
                Run::Act(_) | Run::Free => {
                    let share = left.entry(account.clone()).or_default();
                    let given = asked.min(*share);
                    *share -= given;
                    given
                },
            };
            let mut e = e.clone();
            if let Effect::Transfer { amount, .. } = &mut e {
                *amount = Amount::Fixed(given);
            }
            Ok(Some(e))
        },
        Effect::Transform {
            who: Binding::Place,
            take,
            ..
        } => {
            for (k, v) in take {
                match run {
                    Run::Plan(takes) => {
                        let t = takes.entry(k.clone()).or_default();
                        *t = t.saturating_add(*v);
                    },
                    Run::Act(_) | Run::Free => {
                        let share = left.entry(k.clone()).or_default();
                        if *share < *v {
                            return Err("a transform of shared ground cannot be shared out".into());
                        }
                        *share -= v;
                    },
                }
            }
            Ok(None)
        },
        _ => Ok(None),
    }
}

/// Applies `p` to `count` members sharing state `e` at `site`, reading the
/// site as the pass began, `start`. `Ok(None)` is the core's blocked
/// outcome and changes nothing; on success the site is updated and the
/// members' new state returned.
#[allow(clippy::too_many_arguments)]
pub(super) fn apply(
    p: &Process,
    e: &Entity,
    start: &Site,
    site: &mut Site,
    count: u64,
    tick: Tick,
    rules: &Rules,
    mut run: Run,
) -> Result<Option<Entity>> {
    if identity_bound(p) {
        return Err(format!(
            "{} depends on identity; it runs individually",
            p.id
        ));
    }
    // Until the vertical probe certifies it, the crowd binds no parts.
    if p.expresses().is_some() {
        return Err(format!("{} binds a part; it runs individually", p.id));
    }
    let seen = Seen {
        member: Some(e),
        site: start,
        tick,
        rules,
    };
    for q in &p.requires {
        // A query that errs blocks, as it does in the core.
        if !seen.holds(q).unwrap_or(false) {
            return Ok(None);
        }
    }
    let mut left = match &run {
        Run::Act(shares) => (*shares).clone(),
        Run::Plan(_) | Run::Free => Ledger::new(),
    };
    let (mut member, mut place) = (e.clone(), site.clone());
    let mut bin = Bin {
        member: &mut member,
        site: &mut place,
        count,
    };
    for effect in p.commitments.iter().chain(&p.effects) {
        let shared = share_out(effect, &mut run, &mut left, bin.site)?;
        let effect = shared.as_ref().unwrap_or(effect);
        match meaning::effect(&mut bin, rules, effect) {
            None => return Err(format!("the crowd cannot apply {effect:?}")),
            Some(Ok(())) => {},
            Some(Err(_)) => return Ok(None),
        }
    }
    *site = place;
    Ok(Some(normalize(member)))
}
