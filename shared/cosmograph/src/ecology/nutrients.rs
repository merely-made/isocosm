// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Nutrient axes (working name `Nutrients`), an optional founding rule of the
//! bodied A1 and stock ecologies, which makes the catalog's mechanic
//! `nutrient-axes` true ("Soil holds more than one nutrient and one can be
//! scarce, so a draw is for a named nutrient and a deposit can supply it") and
//! owner model 1 of D39 with it
//! (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md` D39: "competition
//! for limiting resources"; case 38: "you should be able to die of not getting
//! an essential nutrient").
//!
//! **The nutrients.** A site holds `axes` nutrients (two to four, default
//! two), each an account of the ground. The first is `world:soil`, which keeps
//! its place and its founded amount, so every reading of the soil keeps its
//! meaning; the others are `world:soil-1`, `world:soil-2` and `world:soil-3`.
//! Every key is a working word: the world names no element of the Earth's, and
//! nothing says which nutrient is which. A site's other nutrients are set at
//! founding by a draw of the seed and the site, between `spread[0]` and
//! `spread[1]` percent of its soil, so some sites are short of one and some of
//! another.
//!
//! **Need in proportion (the limiting law).** A producer's act takes light
//! and fixes it with soil. Under the rule it takes `base` of every nutrient,
//! and `tilt` more of its line's lead nutrient (line `i` leads on nutrient
//! `i % axes`), and makes as much tissue as it took matter: its proportions
//! are fixed by its line, and lines differ. An act is whole: it needs every
//! nutrient it draws at its site (a requirement per nutrient), so a site
//! short of one stops the line's fixing however rich it is in the rest, and
//! the number of acts a site allows a line is that of its scarcest nutrient
//! (Liebig's law). A nutrient a line needs nought of does not limit it.
//!
//! **Essential, and no new death (case 38).** A producer that cannot fix for
//! want of a nutrient still pays its upkeep, so its tissue falls and the
//! existing death below one unit takes it (`ecology:death-<i>`); no process
//! of dying is added. Consumers and decomposers draw no nutrient: they eat
//! tissue, which carries what its fixers took, so they decline when their
//! food does.
//!
//! **Return in proportion.** Every way matter returns to the ground (upkeep,
//! rot, dung, a corpse, a decomposer's fee, the guard's rent) puts it into
//! `world:soil`, and none of those paths is touched, so the budget cannot be
//! opened by them. What the rule adds is a sorting of that soil: every act
//! counts on its site (`nutrients:owed-<a>`) the units of each other
//! nutrient it took, and a site turns soil into a nutrient while the count
//! stands, in chunks of 64, 8 and 1 (`nutrients:sort-<a>-<chunk>`, agentless,
//! after rot), the count falling as it does. So the ground gets back, in
//! the proportion they were drawn, what producers drew. Tissue holds no
//! nutrient shares of its own: the choice is the simplest honest one, the
//! return in the proportion drawn rather than the proportion held, which
//! differs only for matter that passes through an eater first.
//!
//! **A tuned organ.** An organ whose port draws ([`Trade::Draws`], carried by
//! the catalog's scarce-uptake parts lowered with the repertoire) lets its
//! body take `uptake` of the nutrient its site holds least of into its own
//! tissue every `period` minutes, a turn for each line that is a producer or
//! a decomposer and each nutrient, firing for the one that is scarcest
//! (`nutrients:tuned-<line>-<a>`). It makes no nutrient and needs no light:
//! it moves matter from the ground to the body, ahead of the plants whose acts
//! need it. The body's draw is counted as owed like a producer's, so the
//! ground gets it back.
//!
//! **Mutualism.** Where the mutualism rule is also set, the decomposer
//! carries its partner's need and not soil alone: `mutualism.nutrient / 2`
//! acts' worth of each nutrient the partner's line needs, drawn from the site
//! pool, and the plant fixes it with its own act as before. It has the same
//! pool to draw on, so it moves nutrient to a partner ahead of the other
//! plants; it makes none.
//!
//! **Both executions.** Every turn is an ordinary effect over the site's
//! shared ground, the counts are site conditions that shift by the count of
//! members (as `Effect::Condition` always does in a crowd), and the sorting
//! is agentless, so the individual and the grouped execution agree. Unset, no
//! account, process, count or declared condition is added and the world
//! founds, hashes and runs as before.

use super::{Rates, Role};
use crate::{
    Founding, Result,
    generate::process,
    rules::*,
    process::Trade,
    schema::{Id, Tick},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SOIL: &str = "world:soil";
const SUN: &str = "sim:energy";
/// The most nutrients a site holds.
pub const MAX_AXES: u32 = 4;
/// The chunks soil is sorted in, largest first.
const CHUNKS: [u64; 3] = [64, 8, 1];

/// How the ground's nutrients are held and needed, set on a founding. Working
/// names; integers only; every number is a placeholder nothing has measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nutrients {
    /// How many nutrients a site holds, the soil among them.
    pub axes: u32,
    /// What a producer's act takes of every nutrient.
    pub base: u64,
    /// What it takes of its line's lead nutrient besides.
    pub tilt: u64,
    /// A site's other nutrients hold between these percents of its soil.
    pub spread: [u64; 2],
    /// What a body with an organ tuned to the scarce nutrient draws of the
    /// nutrient its site holds least of, in a turn, into its own tissue.
    pub uptake: u64,
    /// Minutes between such turns.
    pub period: u64,
}

impl Default for Nutrients {
    /// Two nutrients; an act takes one of each and one more of its line's
    /// own (so lines need two to one and one to two, three matter an act);
    /// a site's second holds between three tenths and seventeen tenths of
    /// its soil; a tuned organ draws one every four minutes.
    fn default() -> Self {
        Self { axes: 2, base: 1, tilt: 1, spread: [30, 170], uptake: 1, period: 4 }
    }
}

/// The account holding a site's nutrient `a`: the soil for the first.
#[must_use]
pub fn axis(a: u32) -> String {
    if a == 0 { SOIL.into() } else { format!("{SOIL}-{a}") }
}

fn owed(a: u32) -> String {
    format!("nutrients:owed-{a}")
}

impl Nutrients {
    pub(crate) fn validate(&self) -> Result<()> {
        if !(2..=MAX_AXES).contains(&self.axes) {
            return Err(format!("nutrients' axes must be from two to {MAX_AXES}"));
        }
        if self.base.saturating_add(self.tilt) == 0 {
            return Err("nutrients' base and tilt must not both be nought: a producer would take nothing".into());
        }
        let [lo, hi] = self.spread;
        if lo == 0 || lo > hi || hi > 1000 {
            return Err("nutrients' spread must be two percents, the first above nought and no more than the second, at most 1000".into());
        }
        if self.uptake == 0 || self.period == 0 {
            return Err("nutrients' uptake and period must each be above nought".into());
        }
        Ok(())
    }

    /// What an act of line `line` takes of nutrient `a`.
    #[must_use]
    pub fn need(&self, line: u32, a: u32) -> u64 {
        self.base + if a == line % self.axes { self.tilt } else { 0 }
    }

    /// The matter an act of `line` makes: all it took.
    #[must_use]
    pub fn yields(&self, line: u32) -> u64 {
        (0..self.axes).map(|a| self.need(line, a)).sum()
    }

    /// A site's nutrient `a` where its soil holds `soil`, by `roll`.
    #[must_use]
    pub fn held(&self, soil: u64, roll: u64) -> u64 {
        let [lo, hi] = self.spread;
        let percent = u128::from(lo) + u128::from(roll) % u128::from(hi - lo + 1);
        u64::try_from(u128::from(soil) * percent / 100).unwrap_or(u64::MAX)
    }
}

/// A line's fixing as one act does it: the light, each nutrient it needs
/// (from the pool where `pool`, else the ones the body already holds), made
/// into tissue, and each other nutrient counted as drawn.
fn fixing(rule: &Nutrients, line: u32) -> Vec<Effect> {
    let need: Vec<(u32, u64)> = (0..rule.axes).map(|a| (a, rule.need(line, a))).filter(|(_, n)| *n > 0).collect();
    let mut out = vec![Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: SUN.into(), amount: 1.into() }];
    out.extend(need.iter().map(|(a, n)| Effect::Transfer {
        from: Binding::Place,
        to: Binding::Actor,
        account: axis(*a),
        amount: (*n).into(),
    }));
    out.push(Effect::Transform {
        who: Binding::Actor,
        take: need.iter().map(|(a, n)| (axis(*a), *n)).collect(),
        give: BTreeMap::from([(format!("matter:{line}-0"), rule.yields(line))]),
        conversion: None,
    });
    out.push(Effect::Transform { who: Binding::Actor, take: [(SUN.into(), 1)].into(), give: BTreeMap::new(), conversion: None });
    out.extend(need.iter().filter(|(a, _)| *a > 0).map(|(a, n)| Effect::Condition { key: owed(*a), delta: i64::try_from(*n).unwrap_or(i64::MAX) }));
    out
}

/// Retunes a producer's fixing process to the rule: its effects are `fixing`'s
/// (without the pool's transfers where the body holds what it fixes with),
/// it needs each nutrient it takes, and a sensed soil share need only cover
/// the first.
fn retune(p: &mut Process, rule: &Nutrients, line: u32, pool: bool) {
    let mut effects = fixing(rule, line);
    if !pool {
        effects.retain(|e| !matches!(e, Effect::Transfer { account, .. } if account != SUN));
    }
    p.effects = effects;
    p.requires.retain(|q| !matches!(q, Query::Account { key, .. } if key == SOIL));
    let who = if pool { Binding::Place } else { Binding::Actor };
    for a in (0..rule.axes).filter(|&a| rule.need(line, a) > 0) {
        p.requires.push(Query::Account { who, key: axis(a), at_least: rule.need(line, a) });
    }
    p.requires.retain(|q| !matches!(q, Query::Sensed { reading, .. } if reading == "sense:soil-share"));
    if pool && p.requires.iter().any(|q| matches!(q, Query::Sensed { .. })) && rule.need(line, 0) > 0 {
        p.requires.push(Query::Sensed { reading: "sense:soil-share".into(), at_least: rule.need(line, 0) });
    }
}

/// A decomposer's carrying to plant line `plant`: `acts` acts' worth of each
/// nutrient the plant needs, in place of the soil the mutualism rule carried.
fn carry(p: &mut Process, rule: &Nutrients, plant: u32, acts: u64) {
    let moves = (0..rule.axes).filter(|&a| rule.need(plant, a) > 0).flat_map(|a| {
        let amount = Amount::Fixed(rule.need(plant, a) * acts);
        let transfer = |from, to| Effect::Transfer { from, to, account: axis(a), amount: amount.clone() };
        [transfer(Binding::Place, Binding::Actor), transfer(Binding::Actor, Binding::Target)]
    });
    p.effects.splice(0..2, moves.collect::<Vec<_>>());
    p.requires.retain(|q| !matches!(q, Query::Account { who: Binding::Place, key, .. } if key == SOIL));
    for a in (0..rule.axes).filter(|&a| rule.need(plant, a) > 0) {
        p.requires.push(Query::Account { who: Binding::Place, key: axis(a), at_least: rule.need(plant, a) * acts });
    }
}

/// Turns `chunk` units of a site's soil into nutrient `a` while the site
/// owes that many.
fn sort(a: u32, chunk: u64) -> Process {
    let mut turn = process(
        &format!("nutrients:sort-{a}-{chunk}"),
        Causation::Agentless,
        vec![
            Effect::Transform {
                who: Binding::Place,
                take: BTreeMap::from([(SOIL.into(), chunk)]),
                give: BTreeMap::from([(axis(a), chunk)]),
                conversion: None,
            },
            Effect::Condition { key: owed(a), delta: -i64::try_from(chunk).unwrap_or(i64::MAX) },
        ],
    );
    turn.requires.extend([
        Query::Account { who: Binding::Place, key: SOIL.into(), at_least: chunk },
        Query::Condition { key: owed(a), at_least: i64::try_from(chunk).unwrap_or(i64::MAX) },
    ]);
    turn.period = Some(1);
    // After rot (50), so what the dead gave back this tick is sorted this tick.
    turn.priority = 51;
    turn
}

/// A body with a tuned organ drawing nutrient `a` for its tissue while its
/// site holds no other nutrient in lesser store: it competes for the scarce
/// nutrient with the plants whose acts need it, and pays no light for it.
fn tuned(rule: &Nutrients, line: u32, a: u32, every: Tick) -> Process {
    let held = |b: u32| Expr::Read(Reading::Account { who: Binding::Place, key: axis(b) });
    let scarcest = Expr::Mul((0..rule.axes).filter(|&b| b != a).map(|b| Expr::AtLeast(Box::new(held(b)), Box::new(held(a)))).collect());
    let body = format!("matter:{line}-0");
    let mut effects = vec![
        Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: axis(a), amount: rule.uptake.into() },
        Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::from([(axis(a), rule.uptake)]),
            give: BTreeMap::from([(body, rule.uptake)]),
            conversion: None,
        },
    ];
    if a > 0 {
        effects.push(Effect::Condition { key: owed(a), delta: i64::try_from(rule.uptake).unwrap_or(i64::MAX) });
    }
    let mut turn = process(&format!("nutrients:tuned-{line}-{a}"), Causation::Transition, effects);
    turn.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{line}") },
        Query::Trades { who: Binding::Actor, what: Trade::Draws },
        Query::Account { who: Binding::Place, key: axis(a), at_least: rule.uptake },
        Query::Computed(scarcest),
    ]);
    turn.period = Some(every);
    // After the fixing (0), before rot and the sorting.
    turn.priority = 5;
    turn
}

/// Lays the nutrient rule on the founded world: the accounts and their stock
/// at each site, the sorting, the producers' retuned fixing and, where the
/// mutualism rule is set, its retuned turns. Does nothing where the founding
/// sets none.
///
/// # Errors
/// Fails where the founding is no bodied ecology, runs in world time, a value
/// is out of range, or the turns pass the world's limit on processes.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.nutrients else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("nutrients are built for the bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("nutrients layer on the stock and A1 rates, not world time".into());
    }
    rule.validate()?;
    let producers: Vec<u32> = (0..founding.lineages).filter(|&i| Role::of(founding.web, i) == Role::Producer).collect();
    let every = crate::rules::minutes_ticks(rule.period, g.rules.tick_microseconds());
    let drawers = (0..founding.lineages).filter(|&i| matches!(Role::of(founding.web, i), Role::Producer | Role::Decomposer));
    let mut made: Vec<Process> = (1..rule.axes).flat_map(|a| CHUNKS.map(|c| sort(a, c))).collect();
    made.extend(drawers.flat_map(|i| (0..rule.axes).map(move |a| (i, a))).map(|(i, a)| tuned(&rule, i, a, every)));
    if g.rules.processes.len() + made.len() > g.rules.limits.processes {
        return Err("nutrients' turns exceed the world's process limit".into());
    }
    for &i in &producers {
        if let Some(grow) = g.rules.processes.get_mut(&format!("ecology:produce-{i}")) {
            retune(grow, &rule, i, true);
        }
        if let Some(grow) = g.rules.processes.get_mut(&format!("mutualism:grow-{i}")) {
            retune(grow, &rule, i, false);
        }
    }
    if let Some(m) = founding.mutualism {
        let acts = (m.nutrient / 2).max(1);
        for (id, p) in g.rules.processes.iter_mut().filter(|(id, _)| id.starts_with("mutualism:nutrient-")) {
            if let Some(plant) = id.rsplit('-').next().and_then(|n| n.parse::<u32>().ok()) {
                carry(p, &rule, plant, acts);
            }
        }
    }
    g.rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    for a in 1..rule.axes {
        g.rules.accounts.insert(axis(a), AccountKind::Matter { lineage: "world:ground".into(), reserve: false, provision: false });
        g.rules.conditions.insert(owed(a));
    }
    for (&id, site) in &mut g.sites {
        let soil = site.accounts.get(SOIL).copied().unwrap_or(0);
        for a in 1..rule.axes {
            site.accounts.insert(axis(a), rule.held(soil, roll(founding.seed, id, a)));
        }
    }
    g.rules.nutrients = Some(rule);
    Ok(())
}

fn roll(seed: u64, site: Id, a: u32) -> u64 {
    crate::draw(seed, "nutrient", &[site, u64::from(a)])
}
