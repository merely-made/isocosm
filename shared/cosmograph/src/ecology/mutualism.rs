// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mutualism (working name `Mutualism`), an optional founding rule of the
//! bodied A1 and stock ecologies, which makes owner model 5 true
//! (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md` D39: "mutualism";
//! the catalog's mechanic `nutrient-sugar-trade`: "two bodies each give what
//! the other lacks (soil nutrient for sugar) and each pays only while the
//! other does"; `rules::hosting`'s `leak` gives one way only, which is no
//! trade).
//!
//! **The two goods.** The world has one soil (`world:soil`) and a tissue
//! account for each line, but the two are not alike to a producer and a
//! decomposer. A producer is the only body that fixes light into tissue, and
//! it fixes with soil it draws from its site's shared pool (`ecology:produce`:
//! one light and two soil an act, crowd-limited where it senses). A decomposer
//! cannot fix: its tissue comes from the dead, and in a bodied world nothing
//! eats it, so what it holds is locked from the producers. *Sugar* here is
//! the producer's tissue, which only a fixer makes (the decomposer digests it
//! into its own on its next turn, as a meal is); *nutrient* is the soil a
//! decomposer draws from the pool and carries to a partner by its own reach.
//! The nutrient does the producer no good unless it can fix, and the sugar
//! feeds the decomposer, which has no other living source; so each gives what
//! the other lacks, and no new matter key is coined.
//!
//! **Contact.** Partners meet where they live: both reside at one site (the
//! compartment the ecology already declares), and each has a living organ whose
//! port trades ([`crate::process::Trade`], carried by the catalog's lowered
//! exchange parts). A body with no organ, or no partner at its site, has no
//! turn.
//!
//! **One exchange, conditional on the last.** Each (producer line, decomposer
//! line) pair keeps two small counts on each site (site conditions, so nothing
//! is stored on a body and nothing is hashed where the rule is unset):
//! `trust`, which starts at [`Mutualism::credit`], and `owed`, which starts at
//! nought, and which together always sum to the credit. The decomposer's turn
//! needs trust: it draws [`Mutualism::nutrient`] of soil from the site, hands
//! it to a partner, pays [`Mutualism::fee`] of its own tissue to the soil, and
//! trust falls as the debt rises. The producer's turn needs the debt: it pays
//! [`Mutualism::sugar`] of its own tissue into the partner's, and the debt is
//! settled. A line that stops giving (it lacks a giving organ, or has died
//! out) leaves the debt unsettled, so the other stops giving once its credit
//! is used: a cheater takes at most the credit and no more. The producer
//! turns the nutrient into tissue with light (`mutualism:grow-<i>`), as
//! `ecology:produce` does, from soil it holds rather than the pool's.
//!
//! **Cost.** Every gift moves real matter by ordinary effects, so the flow
//! record has its legs and the budget stays closed: pool to decomposer to
//! producer (the nutrient), the decomposer's tissue to the pool (the fee), and
//! the producer's tissue to the decomposer's (the sugar). Nothing is paid
//! without a partner at the site, and nothing is received without paying.
//!
//! The turns have targets, so neither execution batches them; the grouped
//! execution runs them as the individual one does. Unset, no process, count or
//! declared condition is added and the world founds, hashes and runs as before.

use super::{Rates, Role};
use crate::{
    Founding, Result,
    generate::process,
    process::{Trade, Trade::{Gives, Takes}},
    rules::*,
    schema::{Entity, Tick},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const SOIL: &str = "world:soil";
const SUN: &str = "sim:energy";

/// How the trade runs, set on a founding. Working names; integers only; every
/// number is a placeholder nothing has measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mutualism {
    /// Minutes between a body's turns at the trade.
    pub period: u64,
    /// The soil a decomposer carries to its partner in a turn: two make one
    /// act of fixing.
    pub nutrient: u64,
    /// The decomposer's tissue it pays, to the soil, for carrying it.
    pub fee: u64,
    /// The producer's tissue it pays the partner's for a turn's nutrient.
    pub sugar: u64,
    /// The gifts a decomposer makes to a line before that line has to answer.
    pub credit: u32,
}

impl Default for Mutualism {
    /// A turn every four minutes; four soil carried at a cost of one, paid
    /// for with three; one gift of credit. The producer nets one tissue, the
    /// decomposer two, and the pool is drawn down by three.
    fn default() -> Self {
        Self { period: 4, nutrient: 4, fee: 1, sugar: 3, credit: 1 }
    }
}

impl Mutualism {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.period == 0 {
            return Err("mutualism's period must be at least a minute".into());
        }
        if self.nutrient == 0 || self.fee == 0 || self.sugar == 0 {
            return Err("mutualism's nutrient, fee and sugar must each be above nought".into());
        }
        if self.credit == 0 {
            return Err("mutualism's credit must be at least one gift".into());
        }
        Ok(())
    }
}

/// Whether `e` has a living organ, its tissue held, whose port does `what`.
#[must_use]
pub fn trades(e: &Entity, what: Trade) -> bool {
    e.living().any(|(_, p)| crate::mosaic::ports::active(p).is_some_and(|port| port.trades(what)))
}

fn counts(decomposer: u32, producer: u32) -> (String, String) {
    (format!("mutualism:trust-{decomposer}-{producer}"), format!("mutualism:owed-{decomposer}-{producer}"))
}

fn matter(i: u32) -> String {
    format!("matter:{i}-0")
}

/// The decomposer's turn toward producer line `p`.
fn nutrient(rule: &Mutualism, d: u32, p: u32, every: Tick) -> Process {
    let (trust, owed) = counts(d, p);
    let n = Amount::Fixed(rule.nutrient);
    let transfer = |from, to| Effect::Transfer { from, to, account: SOIL.into(), amount: n.clone() };
    let mut turn = process(
        &format!("mutualism:nutrient-{d}-{p}"),
        Causation::Transition,
        vec![
            transfer(Binding::Place, Binding::Actor),
            transfer(Binding::Actor, Binding::Target),
            Effect::Spend { from: vec![matter(d)], to: Binding::Place, amount: Amount::Fixed(rule.fee), into: Some(SOIL.into()) },
            Effect::Condition { key: trust.clone(), delta: -1 },
            Effect::Condition { key: owed, delta: 1 },
        ],
    );
    turn.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{d}") },
        Query::Trades { who: Binding::Actor, what: Gives },
        Query::Trades { who: Binding::Target, what: Takes },
        Query::Condition { key: trust, at_least: 1 },
        Query::Account { who: Binding::Place, key: SOIL.into(), at_least: rule.nutrient },
        Query::Account { who: Binding::Actor, key: matter(d), at_least: rule.fee },
    ]);
    turn.target = Some(partner(p));
    turn.period = Some(every);
    turn.priority = 1;
    turn
}

/// The producer's turn toward decomposer line `d`.
fn sugar(rule: &Mutualism, p: u32, d: u32, every: Tick) -> Process {
    let (trust, owed) = counts(d, p);
    let mut turn = process(
        &format!("mutualism:sugar-{p}-{d}"),
        Causation::Transition,
        vec![
            Effect::Spend { from: vec![matter(p)], to: Binding::Target, amount: Amount::Fixed(rule.sugar), into: None },
            Effect::Condition { key: owed.clone(), delta: -1 },
            Effect::Condition { key: trust, delta: 1 },
        ],
    );
    turn.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{p}") },
        Query::Trades { who: Binding::Actor, what: Gives },
        Query::Trades { who: Binding::Target, what: Takes },
        Query::Condition { key: owed, at_least: 1 },
        Query::Account { who: Binding::Actor, key: matter(p), at_least: rule.sugar },
    ]);
    turn.target = Some(partner(d));
    turn.period = Some(every);
    turn.priority = 2;
    turn
}

/// The decomposer digesting the sugar a partner handed over, into its own tissue.
fn digest(rule: &Mutualism, d: u32, p: u32, every: Tick) -> Process {
    let mut turn = process(
        &format!("mutualism:digest-{d}-{p}"),
        Causation::Transition,
        vec![Effect::Convert {
            who: Binding::Actor,
            from: vec![matter(p)],
            to: matter(d),
            amount: Amount::Fixed(rule.sugar),
            conversion: Conversion::Digestion,
        }],
    );
    turn.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{d}") },
        Query::Account { who: Binding::Actor, key: matter(p), at_least: 1 },
    ]);
    turn.period = Some(every);
    turn.priority = 4;
    turn
}

fn partner(line: u32) -> Target {
    Target { same_place: true, alive: Some(true), lineage: None, among: BTreeSet::from([format!("lineage:{line}")]), weighted: true }
}

/// The producer fixing with the soil a partner gave it: `ecology:produce`'s
/// capture and synthesis, from its own soil and not the pool's.
fn grow(p: u32, sensing: bool, every: Tick) -> Process {
    let mut grow = process(
        &format!("mutualism:grow-{p}"),
        Causation::Transition,
        vec![
            Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: SUN.into(), amount: 1.into() },
            Effect::Transform {
                who: Binding::Actor,
                take: [(SOIL.into(), 2)].into(),
                give: [(matter(p), 2)].into(),
                conversion: None,
            },
            Effect::Transform { who: Binding::Actor, take: [(SUN.into(), 1)].into(), give: Default::default(), conversion: None },
        ],
    );
    grow.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{p}") },
        Query::Condition { key: "world:habitable".into(), at_least: 1 },
        Query::Account { who: Binding::Actor, key: SOIL.into(), at_least: 2 },
    ]);
    if sensing {
        grow.requires.push(Query::Sensed { reading: "sense:light-share".into(), at_least: 1 });
    }
    grow.period = Some(every);
    grow.priority = 3;
    grow
}

/// Lays the mutualism rule on the founded world: the rule on its rules, the
/// two counts of each partnered pair on every site, and the turns of each
/// producer line and decomposer line. Does nothing where the founding sets none.
///
/// # Errors
/// Fails where the founding is no bodied ecology, runs in world time, a value
/// is out of range, or the turns pass the world's limit on processes.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.mutualism else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("mutualism is built for the bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("mutualism layers on the stock and A1 rates, not world time".into());
    }
    rule.validate()?;
    let lines = |role| (0..founding.lineages).filter(move |&i| Role::of(founding.web, i) == role).collect::<Vec<_>>();
    let (producers, decomposers) = (lines(Role::Producer), lines(Role::Decomposer));
    let every = crate::rules::minutes_ticks(rule.period, g.rules.tick_microseconds());
    let fixing = crate::rules::minutes_ticks(2, g.rules.tick_microseconds());
    let sensing = founding.bodies.as_ref().is_some_and(|b| b.sensing);
    let mut made: Vec<Process> = producers.iter().map(|&p| grow(p, sensing, fixing)).collect();
    for &d in &decomposers {
        for &p in &producers {
            made.extend([nutrient(&rule, d, p, every), sugar(&rule, p, d, every), digest(&rule, d, p, every)]);
            let (trust, owed) = counts(d, p);
            for site in g.sites.values_mut() {
                site.conditions.insert(trust.clone(), i64::from(rule.credit));
            }
            g.rules.conditions.extend([trust, owed]);
        }
    }
    if g.rules.processes.len() + made.len() > g.rules.limits.processes {
        return Err("mutualism's turns exceed the world's process limit".into());
    }
    g.rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    g.rules.mutualism = Some(rule);
    Ok(())
}
