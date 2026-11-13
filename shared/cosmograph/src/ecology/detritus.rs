// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The detritus web (working name `Detritus`), an optional founding rule of
//! the bodied A1 and stock ecologies, which makes the catalog's mechanic
//! `dung-stock` true ("A meal leaves an unkept remainder as dung, a dead
//! stock of its own that dung mouths take; the stock and A1 ecologies keep a
//! meal whole and deposit none") and owner model 4 of D39 with it
//! (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md` D39: "the
//! detritus web"; D37's dead matter in categories).
//!
//! **What dung is, and where it is held.** Dead matter today is a dead body
//! whose parts still hold their matter; its class (litter, wood, carrion) is
//! derived at the bite from the body's kingdom and the part (`categories`).
//! Dung is not a body: an animal's meal leaves it at its site, so it is held
//! where the site holds its other dead-matter accounts, as the account
//! `world:dung`, a matter account of the ground (as `world:soil` and world
//! time's `world:detritus` are). It is the fourth [`DeadClass`], `Dung`,
//! which no body's dead matter can be: a mouth takes it only if its
//! dead-stock port is not limited to other classes (the decomposer's root,
//! limited to none, takes every class; a mouth limited to litter does not),
//! read at the turn by [`takes`]. Hash-neutral: the class is a bit of a port
//! that no founded port sets, and the account exists only under the rule.
//!
//! **Who deposits.** An animal (a body of the fauna kingdom), at every meal it
//! makes of a body, living or dead, as the catalog's mouths that emit dung
//! are the grazer's lips and molars (living plant), the carnassial jaw (living
//! prey and fresh dead), the carrion mouth and the litter mouth (dead
//! matter). A decomposer deposits none: its meal is the dead, kept whole, and
//! what it does not keep returns to the soil by rot as before. A producer
//! eats nothing. A meal's `share` of what it took (rounded by a draw, so a
//! one-unit bite deposits with the share's chance) is not given to the eater: it
//! goes to the site's dung.
//! The deposit is of matter taken, never made: the eater's gain is smaller by
//! what the site's dung gains, and the flow record has a leg from the prey
//! to the site for it.
//!
//! **Who eats it, and rot.** A line whose body has a living dead-stock mouth
//! that takes dung (the decomposer's root, or a lowered dung mouth) gets a
//! turn at the dung of its site (`detritus:eat-<i>`): it takes a `bite` and
//! keeps `kept` of it as tissue, the rest going to the soil (a dung meal is
//! digested; its remainder is not dung again). Whatever nothing eats, rot
//! returns to the soil at `rot` units a tick a site (`detritus:rot`, an
//! agentless process), so the loop closes without a dung feeder: producer
//! tissue, to a grazer, to dung, to soil (and to a decomposer or dung feeder,
//! to its tissue), to the producers.
//!
//! **Both executions.** The deposit is `taken * n / d`, rounded by a draw of
//! the seed, tick, site and bite ([`left_by`]), never a per-act draw, so the
//! individual and the grouped (crowd) meal agree on it; the
//! turns are ordinary effects over the site's shared ground, as world time's
//! detritus turn is. Unset, no account, process or share is added and the
//! world founds, hashes and runs as before. The rule leans on the categories
//! rule, which gives dead stock its classes.

use super::{Rates, Role};
use crate::{
    Founding, Result,
    generate::process,
    process::DeadClass,
    rules::*,
    schema::{Entity, Tick},
    simulation::Genesis,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The account a site's dung is held in.
pub const DUNG: &str = "world:dung";
const SOIL: &str = "world:soil";
const FAUNA: &str = "kingdom:fauna";

/// How dung is left and taken, set on a founding. Working names; integers
/// only; every number is a placeholder nothing has measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detritus {
    /// The share `[n, d]` of the matter an animal's meal takes that it leaves
    /// as dung, floored.
    pub share: [u64; 2],
    /// The dung a dung mouth takes in a turn.
    pub bite: u64,
    /// Of a bite, what the eater keeps as tissue; the rest goes to the soil.
    pub kept: u64,
    /// Minutes between a dung mouth's turns.
    pub period: u64,
    /// The units of a site's dung that rot to its soil each tick.
    pub rot: u64,
}

impl Default for Detritus {
    /// A third of a meal is left (a meal of three leaves one); a turn takes
    /// three and keeps two, every three minutes; a site's dung rots one unit
    /// a tick.
    fn default() -> Self {
        Self { share: [1, 3], bite: 3, kept: 2, period: 3, rot: 1 }
    }
}

impl Detritus {
    pub(crate) fn validate(&self) -> Result<()> {
        let [n, d] = self.share;
        if n == 0 || d == 0 || n >= d {
            return Err("detritus's share must be a ratio of two numbers, above nought and below one".into());
        }
        if self.bite == 0 || self.kept > self.bite {
            return Err("detritus's bite must be above nought and keep no more than it takes".into());
        }
        if self.period == 0 || self.rot == 0 {
            return Err("detritus's period and rot must each be above nought".into());
        }
        Ok(())
    }

    /// The dung a meal that took `taken` leaves: the share, its fraction
    /// rounded up where `roll` (a draw) falls below it, so small bites (a part
    /// holds a unit or two) leave their share on average.
    #[must_use]
    pub fn dung(&self, taken: u64, roll: u64) -> u64 {
        let [n, d] = self.share;
        let (d, whole) = (u128::from(d.max(1)), u128::from(taken) * u128::from(n));
        let extra = u128::from(u128::from(roll) % d < whole % d);
        u64::try_from(whole / d + extra).unwrap_or(u64::MAX)
    }
}

/// The dung a meal of `taken` by `eater` at `site` on `tick` leaves: none
/// where the world sets no rule, or the eater is not an animal. The roll is
/// a draw of the seed, the tick, the site and the bite, so the individual
/// and the grouped meal, which share no per-act draw, round alike; with no
/// seed to give (a crowd outside a world) the share is floored.
#[must_use]
pub fn left_by(rule: Option<Detritus>, eater: &Entity, taken: u64, seed: Option<u64>, tick: Tick, site: crate::schema::Id) -> u64 {
    match rule {
        Some(rule) if eater.kingdom == FAUNA => {
            rule.dung(taken, seed.map_or(u64::MAX, |seed| crate::draw(seed, "dung", &[tick, site, taken])))
        },
        _ => 0,
    }
}

/// Whether `e` has a living dead-stock mouth, its tissue held, that takes
/// dead matter of `class`: with no limit it takes every class. A body of the
/// fungus kingdom that declares no port at all (the founded decomposer, whose
/// feed is the dead's, every class) is taken to have the founding root.
#[must_use]
pub fn takes(e: &Entity, class: DeadClass) -> bool {
    if e.kingdom == "kingdom:myco" && e.parts.values().all(|p| p.port.is_none()) {
        return true;
    }
    e.living().any(|(_, p)| {
        crate::mosaic::ports::active(p).is_some_and(|port| port.admits_deadstock() && port.takes(class))
    })
}

fn matter(i: u32) -> String {
    format!("matter:{i}-0")
}

/// A dung mouth's turn at its site's dung: takes a bite, keeps some as
/// tissue and returns the rest to the soil.
fn eat(rule: &Detritus, i: u32, cap: u64, every: Tick) -> Process {
    let rest = rule.bite - rule.kept;
    let mut turn = process(
        &format!("detritus:eat-{i}"),
        Causation::Transition,
        vec![
            Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: DUNG.into(), amount: rule.bite.into() },
            Effect::Transform {
                who: Binding::Actor,
                take: BTreeMap::from([(DUNG.into(), rule.bite)]),
                give: BTreeMap::from([(matter(i), rule.kept), (SOIL.into(), rest)]),
                conversion: None,
            },
            Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: SOIL.into(), amount: rest.into() },
        ],
    );
    turn.requires.extend([
        Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{i}") },
        Query::Takes { who: Binding::Actor, class: DeadClass::Dung },
        Query::Below { who: Binding::Actor, key: matter(i), amount: cap },
        Query::Account { who: Binding::Place, key: DUNG.into(), at_least: rule.bite },
    ]);
    turn.period = Some(every);
    // After the meals (priority 10 at most), as world time's detritus turn.
    turn.priority = 11;
    turn
}

/// What no mouth eats rots to the soil, a site at a time.
fn rot(rule: &Detritus) -> Process {
    let mut rot = process(
        "detritus:rot",
        Causation::Agentless,
        vec![Effect::Transform {
            who: Binding::Place,
            take: BTreeMap::from([(DUNG.into(), rule.rot)]),
            give: BTreeMap::from([(SOIL.into(), rule.rot)]),
            conversion: None,
        }],
    );
    rot.requires.push(Query::Account { who: Binding::Place, key: DUNG.into(), at_least: rule.rot });
    rot.period = Some(1);
    rot.priority = 50;
    rot
}

/// Lays the detritus rule on the founded world: the dung account, a site's
/// empty stock, the rot, and a dung turn for each line that is not a producer
/// (whether it has a mouth for it is read at the turn). Does nothing where
/// the founding sets none.
///
/// # Errors
/// Fails where the founding is no bodied ecology, sets no categories rule,
/// runs in world time, a value is out of range, or the turns pass the world's
/// limit on processes.
pub(crate) fn apply(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let Some(rule) = founding.detritus else { return Ok(()) };
    if !founding.ecology || founding.bodies.is_none() {
        return Err("detritus is built for the bodied ecology".into());
    }
    if founding.ecology_rates == Rates::WorldTime {
        return Err("detritus layers on the stock and A1 rates, not world time".into());
    }
    if founding.categories.is_none() {
        return Err("detritus leans on the categories rule, which classes dead stock".into());
    }
    rule.validate()?;
    let unit = g.rules.tick_microseconds();
    let every = crate::rules::minutes_ticks(rule.period, unit);
    let child = founding.stock_min.max(2);
    let mut made = vec![self::rot(&rule)];
    for i in (0..founding.lineages).filter(|&i| Role::of(founding.web, i) != Role::Producer) {
        let consumer = Role::of(founding.web, i).consumer();
        made.push(eat(&rule, i, founding.ecology_rates.birth_gate(founding.stock_min, consumer) + child, every));
    }
    if g.rules.processes.len() + made.len() > g.rules.limits.processes {
        return Err("detritus's turns exceed the world's process limit".into());
    }
    g.rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    g.rules.accounts.insert(DUNG.into(), AccountKind::Matter { lineage: "world:ground".into(), reserve: false, provision: false });
    for site in g.sites.values_mut() {
        site.accounts.insert(DUNG.into(), 0);
    }
    g.rules.detritus = Some(rule);
    Ok(())
}
