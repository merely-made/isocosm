// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A process applied to every member of a state at once, the aggregate form
//! of the sim plan's §3.1. What queries read and effects do is `meaning`'s,
//! shared with the individual runner. What is the crowd's own: it refuses
//! anything that depends on identity (risk, relations, noted events, and a
//! target outside a meal), and a site ledger carries every member of the
//! state. Every member reads its site as the pass began and as its own act
//! has written it (ruling 454), binds its part as the core does (ruling
//! 338), keeps its act's values, and reads the draws the crowd fixed for
//! the members its act stands for. A pass's takes of a site are planned
//! first and shared out where they would take more than it holds.

use crate::{
    Result,
    meaning::{self, Named, Parties, Scene, credit, debit},
    rules::{
        Amount, Binding, BodyRules, Draw, Effect, Expr, PartsOf, Process, Query, Read, Reading,
        Rules, expressing,
    },
    schema::*,
};
use std::collections::BTreeMap;

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
    fn scene(&self, part: Option<Id>) -> Scene<'_> {
        Scene {
            actor: self.member,
            target: Named::Unnamed,
            part,
            site: Some(self.site),
            tick: self.tick,
            related: &no_relations,
            rules: self.rules,
        }
    }
    pub(super) fn holds(&self, q: &Query) -> Result<bool> {
        Ok(meaning::read(q, &self.scene(None))?.0)
    }
    /// As `holds`, with the part `p` binds in the member (ruling 338).
    pub(super) fn holds_for(&self, p: &Process, q: &Query) -> Result<bool> {
        let part = p.expresses().and_then(|f| expressing(self.member?, f));
        Ok(meaning::read(q, &self.scene(part))?.0)
    }
    pub(super) fn mood(&self) -> Result<i64> {
        meaning::mood(&self.scene(None))
    }
}

/// One state's act: whom it stands for and what it reads beyond them.
pub(super) struct Act<'a> {
    /// The site as the pass began.
    pub start: &'a Site,
    pub count: u64,
    pub tick: Tick,
    pub rules: &'a Rules,
    /// Each slot's draw, fixed for the members the act stands for.
    pub draws: &'a BTreeMap<u8, u64>,
    /// A meal's prey as the pass began, the matter the meal's share takes
    /// of it (ruling 454), and the part the bite lands on where the prey
    /// keeps its matter in parts (ruling 459).
    pub meal: Option<(&'a Entity, &'a Ledger, Option<Id>)>,
}

/// One state's members as their act leaves them, and what the act reads.
struct Doing<'a> {
    rules: &'a Rules,
    member: Entity,
    part: Option<Id>,
    /// The site as every member's writes leave it.
    site: Site,
    /// The site as one member's act sees it: the pass's start and its own
    /// writes.
    seen: Site,
    count: u64,
    /// The prey as one hunter's act sees it, the share it takes, and the
    /// part its bite lands on.
    prey: Option<Entity>,
    portion: Option<Ledger>,
    bitten: Option<Id>,
    kept: BTreeMap<Key, i64>,
    draws: &'a BTreeMap<u8, u64>,
}

impl Parties for Doing<'_> {
    fn held(&mut self, who: Binding, key: &str) -> Result<u64> {
        match who {
            Binding::Actor => Ok(crate::anatomy::held(&self.member, self.rules, key)),
            // One member's share of a shared site is the whole of it only
            // where the act stands for one member, the world's own.
            Binding::Place if self.count == 1 => Ok(meaning::value(&self.seen.accounts, key)),
            Binding::Target => match &self.prey {
                Some(prey) => Ok(crate::anatomy::held(prey, self.rules, key)),
                None => Err("the crowd has no target".into()),
            },
            _ => Err("the crowd reads no one member's share of shared ground".into()),
        }
    }
    fn reach(&mut self, who: Binding) -> Result<()> {
        match who {
            Binding::Target if self.prey.is_none() => Err("the crowd has no target".into()),
            _ => Ok(()),
        }
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => match crate::anatomy::take(&mut self.member, self.rules, key, amount)
            {
                Some(done) => done.map(|_| ()),
                None => debit(&mut self.member.accounts, key, amount),
            },
            // Shares make every take of the site whole (ruling 454).
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                debit(&mut self.site.accounts, key, total)?;
                debit(&mut self.seen.accounts, key, amount)
            },
            Binding::Target => Err("the crowd takes from a prey only by its meal".into()),
            Binding::Part => Err("a part keeps no ledger".into()),
        }
    }
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        match who {
            Binding::Actor => match crate::anatomy::give(&mut self.member, self.rules, key, amount)
            {
                Some(done) => done.map(|_| ()),
                None => credit(&mut self.member.accounts, key, amount),
            },
            Binding::Place => {
                let total = amount.checked_mul(self.count).ok_or("amount overflow")?;
                credit(&mut self.site.accounts, key, total)?;
                credit(&mut self.seen.accounts, key, amount)
            },
            Binding::Target => Err("the crowd gives a prey nothing".into()),
            Binding::Part => Err("a part keeps no ledger".into()),
        }
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        match who {
            Binding::Actor => Ok(&mut self.member),
            _ => Err("the crowd writes only the actor's body".into()),
        }
    }
    fn part(&mut self) -> Result<(&mut Entity, Id)> {
        let part = self.part.ok_or("no part is bound")?;
        Ok((&mut self.member, part))
    }
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site.conditions, key, delta, self.count)?;
        meaning::shift(&mut self.seen.conditions, key, delta, 1)
    }
}

impl Doing<'_> {
    /// Computes what an amount or a guard reads, as the core's stage does.
    fn compute<T>(
        &mut self,
        f: impl FnOnce(&mut Read, &mut Draw, &mut PartsOf) -> Result<T>,
    ) -> Result<T> {
        let living = |e: &Entity| -> Vec<Part> {
            e.parts.values().filter(|p| !p.severed).cloned().collect()
        };
        let (mine, theirs) = (living(&self.member), self.prey.as_ref().map(living));
        let (rules, b) = (self.rules, self.rules.body());
        let mut parts = |who: Binding| -> Result<(Vec<Part>, BodyRules)> {
            match who {
                Binding::Actor => Ok((mine.clone(), b)),
                Binding::Target => match theirs.clone() {
                    Some(t) => Ok((t, b)),
                    None => Err("no target is bound".into()),
                },
                _ => Err(format!("{who:?} has no parts")),
            }
        };
        let (member, seen, prey, kept) = (&self.member, &self.seen, &self.prey, &self.kept);
        let part = self.part.and_then(|id| member.parts.get(&id));
        let mut read = |r: &Reading| -> Result<i64> {
            let body = |who: Binding| match who {
                Binding::Actor => Ok(member),
                Binding::Target => prey.as_ref().ok_or("no target is bound"),
                _ => Err("not a body"),
            };
            match (r, r.who()) {
                (Reading::Kept { name }, _) => kept
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("no value {name} was kept")),
                (Reading::Account { key, .. }, Binding::Place) => {
                    i64::try_from(meaning::value(&seen.accounts, key)).map_err(|e| e.to_string())
                },
                (Reading::Account { key, .. }, Binding::Part) => {
                    let part = part.ok_or("no part is bound")?;
                    i64::try_from(meaning::value(&part.matter, key)).map_err(|e| e.to_string())
                },
                (Reading::Account { key, .. }, who) => {
                    let held = crate::anatomy::held(body(who)?, rules, key);
                    i64::try_from(held).map_err(|e| e.to_string())
                },
                (r, Binding::Part) => {
                    let part = part.ok_or("no part is bound")?;
                    Ok(i64::try_from(r.of_part(part, b)).unwrap_or(i64::MAX))
                },
                (r, who) => Ok(meaning::body_reading(body(who)?, r, rules)),
            }
        };
        let draws = self.draws;
        let mut draw = |below: u64, slot: u8| -> Result<u64> {
            let fixed = draws.get(&slot).ok_or("a draw the crowd did not fix")?;
            Ok(fixed % below)
        };
        f(&mut read, &mut draw, &mut parts)
    }

    /// Applies `effects` in order; `Ok(false)` is the core's blocked
    /// outcome, an error what the crowd cannot apply at all.
    fn run(
        &mut self,
        effects: &[&Effect],
        rules: &Rules,
        how: &mut Run,
        left: &mut Ledger,
    ) -> Result<bool> {
        for &effect in effects {
            if !self.one(effect, rules, how, left)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn one(&mut self, e: &Effect, rules: &Rules, how: &mut Run, left: &mut Ledger) -> Result<bool> {
        match e {
            Effect::Keep { name, value } => {
                let Ok(v) = self.compute(|r, d, p| value.eval_in(r, d, p)) else {
                    return Ok(false);
                };
                self.kept.insert(name.clone(), v);
                Ok(true)
            },
            Effect::When { .. } => {
                let Ok(branch) = self.compute(|r, d, p| e.branch(r, d, p)) else {
                    return Ok(false);
                };
                let branch = branch.expect("a guard chooses a branch");
                self.run(&branch.iter().collect::<Vec<_>>(), rules, how, left)
            },
            // The meal's share of its prey, taken as its pass shared it out.
            Effect::Eat { into, .. } => {
                let (Some(prey), Some(portion)) = (&mut self.prey, self.portion.take()) else {
                    return Err("the crowd eats only in a hunt".into());
                };
                let mut total = 0u64;
                for (key, value) in &portion {
                    match self.bitten.and_then(|id| prey.parts.get_mut(&id)) {
                        Some(part) => debit(&mut part.matter, key, *value)?,
                        None => debit(&mut prey.accounts, key, *value)?,
                    }
                    total += value;
                }
                credit(&mut self.member.accounts, into, total)?;
                Ok(true)
            },
            _ => {
                let resolved = match e.computes() {
                    true => match self.compute(|r, d, p| e.resolve(r, d, p)) {
                        Ok(resolved) => resolved,
                        Err(_) => return Ok(false),
                    },
                    false => e.clone(),
                };
                let shared = share_out(&resolved, how, left, &self.seen)?;
                let effect = shared.unwrap_or(resolved);
                match meaning::effect(self, rules, &effect) {
                    None => Err(format!("the crowd cannot apply {effect:?}")),
                    Some(Ok(())) => Ok(true),
                    Some(Err(_)) => Ok(false),
                }
            },
        }
    }
}

fn identity_bound(p: &Process, meal: bool) -> bool {
    let target = |b: &Binding| *b == Binding::Target;
    p.risk.is_some()
        || (p.target.is_some() && !meal)
        || p.note
        || p.requires.iter().any(|q| {
            let reads = match q {
                Query::Related { .. } => return true,
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
                Query::Computed(x) => x.reads().iter().any(|u| target(&u.body())),
            };
            reads && !meal
        })
}

/// Whether a query reads a target, which a hunt decides for its prey.
pub(super) fn binds_target(q: &Query) -> bool {
    match q {
        Query::Related { .. } => true,
        Query::Alive(who) => *who == Binding::Target,
        Query::Trait { who, .. }
        | Query::Account { who, .. }
        | Query::Below { who, .. }
        | Query::Part { who, .. }
        | Query::Holds { who, .. } => *who == Binding::Target,
        Query::Computed(x) => x.reads().iter().any(|u| u.body() == Binding::Target),
        _ => false,
    }
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
    run: Run,
) -> Result<Option<Entity>> {
    let draws = BTreeMap::new();
    let a = Act {
        start,
        count,
        tick,
        rules,
        draws: &draws,
        meal: None,
    };
    act(p, e, site, run, a)
}

/// Applies `p` as `apply` does, with the draws and the meal of `a`.
pub(super) fn act(
    p: &Process,
    e: &Entity,
    site: &mut Site,
    mut run: Run,
    a: Act,
) -> Result<Option<Entity>> {
    if identity_bound(p, a.meal.is_some()) {
        return Err(format!(
            "{} depends on identity; it runs individually",
            p.id
        ));
    }
    let seen = Seen {
        member: Some(e),
        site: a.start,
        tick: a.tick,
        rules: a.rules,
    };
    // A query of the prey is the hunt's, which chose it; a query that errs
    // blocks, as it does in the core.
    for q in p.requires.iter().filter(|q| !binds_target(q)) {
        if !seen.holds_for(p, q).unwrap_or(false) {
            return Ok(None);
        }
    }
    let mut left = match &run {
        Run::Act(shares) => (*shares).clone(),
        Run::Plan(_) | Run::Free => Ledger::new(),
    };
    let mut doing = Doing {
        rules: a.rules,
        member: e.clone(),
        part: p.expresses().and_then(|f| expressing(e, f)),
        site: site.clone(),
        seen: a.start.clone(),
        count: a.count,
        prey: a.meal.map(|(prey, _, _)| prey.clone()),
        portion: a.meal.map(|(_, portion, _)| portion.clone()),
        bitten: a.meal.and_then(|(_, _, part)| part),
        kept: BTreeMap::new(),
        draws: a.draws,
    };
    let effects: Vec<&Effect> = p.commitments.iter().chain(&p.effects).collect();
    if !doing.run(&effects, a.rules, &mut run, &mut left)? {
        return Ok(None);
    }
    *site = doing.site;
    Ok(Some(normalize(doing.member)))
}

/// Each draw slot `p` reads and its bound; a slot read under two bounds is
/// one the crowd cannot fix one draw for.
pub(super) fn slots(p: &Process) -> Result<BTreeMap<u8, u64>> {
    fn walk(x: &Expr, slots: &mut BTreeMap<u8, u64>) -> Result<()> {
        if let Expr::Draw { below, slot } = x
            && *slots.entry(*slot).or_insert(*below) != *below
        {
            return Err(format!("slot {slot} is drawn under two bounds"));
        }
        x.children().into_iter().try_for_each(|c| walk(c, slots))
    }
    let mut slots = BTreeMap::new();
    let top = || p.commitments.iter().chain(&p.effects);
    for e in top().chain(top().flat_map(Effect::branches)) {
        if let Some(x) = e.computed() {
            walk(x, &mut slots)?;
        }
        for a in e.amounts() {
            if let Amount::Computed(x) = a {
                walk(x, &mut slots)?;
            }
        }
    }
    Ok(slots)
}

/// What a hunt's meal, its first effect, asks of its prey for a hunter in
/// state `e`, read of the hunter and its site as the pass began.
pub(super) fn mouthful(p: &Process, e: &Entity, start: &Site, rules: &Rules) -> Result<u64> {
    let Some(Effect::Eat { amount, .. }) = p.effects.first() else {
        return Err(format!("{} feeds by eating its target first", p.id));
    };
    let draws = BTreeMap::new();
    let mut doing = Doing {
        rules,
        member: e.clone(),
        part: p.expresses().and_then(|f| expressing(e, f)),
        site: start.clone(),
        seen: start.clone(),
        count: 1,
        prey: None,
        portion: None,
        bitten: None,
        kept: BTreeMap::new(),
        draws: &draws,
    };
    doing
        .compute(|r, d, parts| amount.resolve(r, d, parts))?
        .resolved()
}
