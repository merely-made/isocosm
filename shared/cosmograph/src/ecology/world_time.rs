// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ecology in world time (cases 1077, 1078): each role's life history,
//! metabolism, feeding and growth stated in world units and read at the
//! world's own unit. Animals feed only while hungry; most of a bite returns
//! to the site as soil; producers are many bites large, so grazing wounds
//! rather than kills; births come of stored surplus; the founding is a
//! biomass pyramid. Hazard rises with age past maturity (Gompertz).
//!
//! The numbers are candidates within the measured search space (1077's
//! ranges), tuned against a year-long web; they are data, not rulings.

use super::Role;
use crate::{Founding, Result, generate::process, rules::*, simulation::Genesis};
use std::collections::BTreeMap;

const DAY: u64 = 1440;

/// One role's life in world time. Minutes unless named otherwise; matter in
/// the body's own account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleTime {
    /// Age at first breeding, in days.
    pub maturity_days: u64,
    /// The yearly chance of dying just past maturity, in millionths, and
    /// the years it takes to double.
    pub hazard_per_million: u32,
    pub doubling_years: u32,
    /// One matter of upkeep per this many minutes.
    pub upkeep: u64,
    /// The body an adult keeps, and what a birth gives the young.
    pub adult: u64,
    pub child: u64,
    /// How often a hungry animal eats, a bite's size, and how much of it
    /// builds body; the rest returns to the site as soil.
    pub feed: u64,
    pub bite: u64,
    pub kept: u64,
    /// How often a producer grows two matter from the site's light and soil.
    pub grow: u64,
    /// How often a ready body breeds, if its store affords it: a producer
    /// seeds once a season.
    pub breed: u64,
    /// Its share of a founding, which with `adult` makes the pyramid.
    pub founders: u64,
}

/// The candidate values by role.
pub fn role_time(role: Role) -> RoleTime {
    match role {
        Role::Producer => RoleTime { maturity_days: 30, hazard_per_million: 100_000, doubling_years: 3,
            upkeep: DAY, adult: 60, child: 10, feed: 0, bite: 0, kept: 0, grow: 720, breed: 90 * DAY, founders: 40 },
        Role::Grazer => RoleTime { maturity_days: 120, hazard_per_million: 200_000, doubling_years: 2,
            upkeep: 1_680, adult: 12, child: 6, feed: 720, bite: 6, kept: 2, grow: 0, breed: DAY, founders: 8 },
        Role::Omnivore => RoleTime { maturity_days: 180, hazard_per_million: 150_000, doubling_years: 3,
            upkeep: 2_400, adult: 12, child: 6, feed: 720, bite: 6, kept: 2, grow: 0, breed: DAY, founders: 2 },
        Role::Predator => RoleTime { maturity_days: 365, hazard_per_million: 120_000, doubling_years: 3,
            upkeep: 2_160, adult: 15, child: 8, feed: DAY, bite: 9, kept: 3, grow: 0, breed: DAY, founders: 1 },
        Role::Decomposer => RoleTime { maturity_days: 14, hazard_per_million: 500_000, doubling_years: 1,
            upkeep: 10 * DAY, adult: 8, child: 3, feed: 240, bite: 3, kept: 1, grow: 0, breed: DAY, founders: 6 },
    }
}

/// The processes and bodies of a world-time ecology; the stock world's
/// sunlight, soil and kingdoms are already laid by `super::configure`.
pub(super) fn configure(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let unit = g.rules.tick_microseconds();
    let minutes = |m: u64| minutes_ticks(m, unit);
    let year = year_ticks(unit);
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        let Some(i) = e.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<u32>().ok()) else { continue };
        let time = role_time(Role::of(founding.web, i));
        e.accounts.insert(format!("matter:{i}-0"), time.adult);
    }
    for i in 0..founding.lineages {
        let role = Role::of(founding.web, i);
        let time = role_time(role);
        let body = format!("matter:{i}-0");
        let own = format!("ability:cycle-{i}");
        let trait_of = || Query::Trait { who: Binding::Actor, key: own.clone() };
        // No ceiling: lifespan is the tail of the rising hazard (1077).
        g.rules.processes.remove(&format!("ecology:age-{i}"));
        if let Some(upkeep) = g.rules.processes.get_mut(&format!("ecology:upkeep-{i}")) {
            upkeep.period = Some(minutes(time.upkeep));
        }
        let maturity = minutes(time.maturity_days * DAY);
        let mut hazard = process(&format!("ecology:hazard-{i}"), Causation::Transition, vec![]);
        hazard.requires.extend([trait_of(), Query::Age { at_least: maturity }]);
        hazard.risk = Some(Risk { per_million: 0, effects: vec![Effect::Death], aging: Some(Aging {
            maturity, year, per_million_per_year: time.hazard_per_million, doubling_years: time.doubling_years }) });
        hazard.period = Some(1);
        hazard.priority = 32;
        hazard.note = true;
        g.rules.processes.insert(hazard.id.clone(), hazard);
        // A birth is what the store affords: an adult's body and the young's.
        let mut birth = process(&format!("ecology:birth-{i}"), Causation::Transition, vec![Effect::Birth {
            provision: BTreeMap::from([(body.clone(), time.child)]), scatter: None }]);
        birth.requires.extend([trait_of(), Query::Age { at_least: maturity },
            Query::Account { who: Binding::Actor, key: body.clone(), at_least: time.adult + time.child }]);
        birth.period = Some(minutes(time.breed));
        birth.priority = 40;
        birth.note = true;
        g.rules.processes.insert(birth.id.clone(), birth);
        if role == Role::Producer {
            if let Some(grow) = g.rules.processes.get_mut(&format!("ecology:produce-{i}")) {
                grow.period = Some(minutes(time.grow));
                // A producer grows to what a seeding needs, then holds.
                grow.requires.push(Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult + time.child });
            }
            continue;
        }
        let Some(feed) = g.rules.processes.get_mut(&format!("ecology:feed-{i}")) else { continue };
        // Eat only while hungry, below what a birth needs; most of the bite
        // goes back to the site as soil (1078).
        feed.effects = vec![
            Effect::Eat { from: Binding::Target, amount: Amount::Fixed(time.bite), into: body.clone(), of: vec![],
                whole: false },
            Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(body.clone(), time.bite - time.kept)]),
                give: BTreeMap::from([("world:soil".into(), time.bite - time.kept)]), conversion: None },
            Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: "world:soil".into(),
                amount: (time.bite - time.kept).into() },
        ];
        feed.requires = vec![trait_of(),
            Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult + time.child },
            Query::Holds { who: Binding::Target, at_least: time.bite }];
        feed.period = Some(minutes(time.feed));
    }
    Ok(())
}

/// A world-time founding's lineage shares: the role's founders.
pub(crate) fn founders(role: Role) -> u64 {
    role_time(role).founders
}
