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
        Amount, Binding, BodyRules, Development, Draw, Effect, Expr, PartsOf, Process, Query, Read,
        Reading, Rules, expressing,
    },
    schema::*,
};
use std::collections::BTreeMap;

mod doing;
use doing::Doing;
pub(in crate::probe) use doing::Run;

/// Zero entries dropped: no query and no inspection tells absent from zero.
pub(super) fn normalize(mut e: Entity) -> Entity {
    e.accounts.retain(|_, v| *v != 0);
    e
}

fn no_relations(_: &Key) -> Result<bool> {
    Err("the crowd keeps no relations".into())
}

/// The lineages a crowd's bodies develop from, where they have bodies.
pub(super) type Lineages<'a> = Option<&'a BTreeMap<Key, Lineage>>;

/// What one member, or a site alone, shows a query.
pub(super) struct Seen<'a> {
    pub member: Option<&'a Entity>,
    pub site: &'a Site,
    pub tick: Tick,
    pub rules: &'a Rules,
    pub lineages: Lineages<'a>,
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
            lineages: self.lineages,
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
    pub lineages: Lineages<'a>,
    /// Each slot's draw, fixed for the members the act stands for.
    pub draws: &'a BTreeMap<u8, u64>,
    /// A meal's prey as the pass began, the matter the meal's share takes
    /// of it (ruling 454), and the part the bite lands on where the prey
    /// keeps its matter in parts (ruling 459).
    pub meal: Option<(&'a Entity, &'a Ledger, Option<Id>)>,
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
    (rules, lineages): (&Rules, Lineages),
    run: Run,
) -> Result<Option<Entity>> {
    let draws = BTreeMap::new();
    let a = Act {
        start,
        count,
        tick,
        rules,
        lineages,
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
        lineages: a.lineages,
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
        lineages: a.lineages,
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
pub(super) fn mouthful(
    p: &Process,
    e: &Entity,
    start: &Site,
    (rules, lineages): (&Rules, Lineages),
) -> Result<u64> {
    let Some(Effect::Eat { amount, .. }) = p.effects.first() else {
        return Err(format!("{} feeds by eating its target first", p.id));
    };
    let draws = BTreeMap::new();
    let mut doing = Doing {
        rules,
        lineages,
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
