// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ecology in world time (cases 1077, 1078): each role's life history,
//! metabolism, feeding and growth stated in world units and read at the
//! world's own unit. Animals feed only while hungry; most of a bite is shed
//! as detritus (1091), which decomposers turn to the soil producers grow
//! from; producers are many bites large, so grazing wounds rather than
//! kills, and a wounded producer regrows faster; births come of stored
//! surplus, producers seeding once a season; founders are born before the
//! world starts, their ages staggered; the founding is a biomass pyramid.
//! Hazard rises with age past maturity (Gompertz).
//!
//! The numbers are candidates within the measured search space (1077's
//! ranges), tuned against a year-long web; they are data, not rulings.

use super::Role;
use crate::{Founding, Result, generate::process, rules::*, schema::{Key, Tick}, simulation::Genesis};
use std::collections::{BTreeMap, BTreeSet};

mod dormant;
mod plants;
pub use plants::{Plants, reserve};

const DAY: u64 = 1440;

/// The media seeds and spores ride (1092): a lineage draws one.
const AIR: &str = "world:air";
pub(crate) const WATER: &str = "world:water";
const MEDIA: [&str; 2] = [AIR, WATER];
/// A medium's strength where neither the site's region or world (1103) nor
/// a laid terrain gives one: a candidate pending its ruling.
const UNLAID_STRENGTH: i64 = 100;

/// The founded soil, as a multiple of the stock world's (1074: enough
/// founded food).
const SOIL_FACTOR: u64 = 3;

/// The light one act of a producer's growth spends.
const LIGHT_PER_GROWTH: u64 = 3_000;

/// What bodies shed and bites lose, until decomposers turn it to soil.
pub const DETRITUS: &str = "world:detritus";
/// Foreign, conserved food on a real intake part; never retained tissue.
pub const INGESTA: &str = "world:ingesta";

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
    /// One matter of upkeep per this many minutes, shed as detritus.
    pub upkeep: u64,
    /// The body an adult keeps, and what a birth gives the young.
    pub adult: u64,
    pub child: u64,
    /// How often a hungry animal eats, a bite's size, and how much of it
    /// builds body; the rest is shed (an animal's as dung, a decomposer's
    /// straight to soil).
    pub feed: u64,
    pub bite: u64,
    pub kept: u64,
    /// How often a producer grows two matter from the site's light and soil,
    /// and how often again while it is below adult size.
    pub grow: u64,
    pub regrow: u64,
    /// How often a ready body breeds, if its store affords it: a producer
    /// seeds once a season, an animal at its own spacing.
    pub breed: u64,
    /// Its share of a founding, which with `adult` makes the pyramid.
    pub founders: u64,
    /// Founders' ages are drawn from zero to this many days.
    pub founding_age_days: u64,
    /// Whether its young ride a medium to a neighbouring site (1092), and
    /// the range its lineage draws a scatter propensity from, in millionths
    /// of births (1093). A walker's young wait on walking speed (1085).
    pub spreads: bool,
    pub scatter: (u32, u32),
    /// A walker's crowding (1111): where at least `crowd` of its lineage
    /// live at a site, each adult leaves along each route with this chance
    /// a day, in millionths; and it breeds only where at least `prey` of
    /// what it eats live for each member of its lineage there (1145).
    pub crowd: u64,
    pub leave: u32,
    pub prey: u64,
}

/// The candidate values by role.
pub(crate) fn role_time(role: Role) -> RoleTime {
    match role {
        Role::Producer => RoleTime { maturity_days: 30, hazard_per_million: 100_000, doubling_years: 3,
            upkeep: DAY, adult: 60, child: 10, feed: 0, bite: 0, kept: 0, grow: 720, regrow: 240,
            breed: 90 * DAY, founders: 40, founding_age_days: 730,
            spreads: true, scatter: (50_000, 150_000), crowd: 0, leave: 0, prey: 0 },
        Role::Grazer => RoleTime { maturity_days: 120, hazard_per_million: 200_000, doubling_years: 2,
            upkeep: 1_680, adult: 12, child: 6, feed: 360, bite: 3, kept: 1, grow: 0, regrow: 0,
            breed: 90 * DAY, founders: 8, founding_age_days: 1_095,
            spreads: false, scatter: (0, 0), crowd: 12, leave: 20_000, prey: 3 },
        Role::Omnivore => RoleTime { maturity_days: 180, hazard_per_million: 150_000, doubling_years: 3,
            upkeep: 2_400, adult: 12, child: 6, feed: 360, bite: 3, kept: 1, grow: 0, regrow: 0,
            breed: 180 * DAY, founders: 2, founding_age_days: 1_460,
            spreads: false, scatter: (0, 0), crowd: 6, leave: 20_000, prey: 6 },
        Role::Predator => RoleTime { maturity_days: 365, hazard_per_million: 120_000, doubling_years: 3,
            upkeep: 2_160, adult: 15, child: 8, feed: DAY, bite: 9, kept: 3, grow: 0, regrow: 0,
            breed: 180 * DAY, founders: 1, founding_age_days: 1_825,
            spreads: false, scatter: (0, 0), crowd: 4, leave: 20_000, prey: 5 },
        Role::Decomposer => RoleTime { maturity_days: 21, hazard_per_million: 500_000, doubling_years: 1,
            upkeep: 720, adult: 8, child: 3, feed: 240, bite: 3, kept: 1, grow: 0, regrow: 0,
            breed: 45 * DAY, founders: 12, founding_age_days: 180,
            spreads: true, scatter: (50_000, 150_000), crowd: 0, leave: 0, prey: 0 },
    }
}

/// `amount` of `body` put onto the site's `account`.
fn shed(body: &str, account: &str, amount: u64) -> Vec<Effect> {
    vec![Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(body.into(), amount)]),
            give: BTreeMap::from([(account.into(), amount)]), conversion: None },
        Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: account.into(), amount: amount.into() }]
}

/// The processes and bodies of a world-time ecology; the stock world's
/// sunlight, soil and kingdoms are already laid by `super::configure`.
pub(super) fn configure(founding: &Founding, g: &mut Genesis) -> Result<()> {
    let experimental = founding.experimental_food_contract;
    let rooted = experimental && founding.bodies.is_some();
    if rooted && founding.web == crate::ecology::Web::Diets {
        g.rules.accounts.insert(INGESTA.into(), AccountKind::Matter { lineage: "world:ground".into(), reserve: false, provision: false });
    }
    let unit = g.rules.tick_microseconds();
    let minutes = |m: u64| minutes_ticks(m, unit);
    let year = year_ticks(unit);
    // Founders are born before the world starts, their ages staggered over
    // each role's span; the start is a whole number of years.
    let oldest = (0..founding.lineages).map(|i| role_time(Role::of(founding.web, i)).founding_age_days)
        .max().unwrap_or(0);
    let mut dormancy = dormant::declare(founding, g)?;
    let started = minutes(oldest * DAY).div_ceil(year) * year;
    g.started = started;
    for (&first, group) in g.population.groups.iter_mut() {
        let e = group.entity_mut();
        e.born = started;
        e.arrived = started;
        let Some(i) = e.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<u32>().ok()) else { continue };
        let time = role_time(Role::of(founding.web, i));
        e.accounts.insert(format!("matter:{i}-0"), time.adult);
        let span = minutes(time.founding_age_days * DAY).max(1);
        e.born = started - crate::draw(founding.seed, "founding-age", &[first]) % span;
    }
    g.rules.accounts.insert(DETRITUS.into(), AccountKind::Matter { lineage: "world:ground".into(),
        reserve: false, provision: false });
    media(g);
    for site in g.sites.values_mut() {
        site.accounts.insert(DETRITUS.into(), 0);
        if let Some(soil) = site.accounts.get_mut("world:soil") { *soil *= SOIL_FACTOR; }
    }
    for i in 0..founding.lineages {
        let role = Role::of(founding.web, i);
        let mut time = role_time(role);
        if experimental {
            // Keep the trial's measured inputs separate from the upstream retune.
            match role {
                Role::Grazer | Role::Omnivore => {
                    (time.feed, time.bite, time.kept, time.prey) = (720, 6, 2, 30);
                }
                Role::Predator => time.prey = 10,
                Role::Producer | Role::Decomposer => {}
            }
        }
        if rooted && role == Role::Grazer {
            // 1114 adopts a bite of three; retain the meal's one-third
            // extraction (1078, 1091), with the rest shed as dung.
            time.bite = 3;
            time.kept = 1;
        }
        let share = dormancy.as_ref().filter(|_| role == Role::Producer).map(|d| d.drawn(founding.seed, i));
        if let Some(share) = share { time.child = share; }
        let body = format!("matter:{i}-0");
        if rooted && role == Role::Producer {
            g.rules.root_stores.insert(body.clone(), RootStore {
                identity: format!("ability:cycle-{i}"), exposed: 40, stored: 20, seed: time.child, ground_y: 2,
            });
            if let Some(death) = g.rules.processes.get_mut(&format!("ecology:death-{i}")) {
                death.requires.retain(|q| !matches!(q, Query::Below { .. }));
                death.requires.push(Query::EmptyBody);
            }
        }
        let own = format!("ability:cycle-{i}");
        let trait_of = || Query::Trait { who: Binding::Actor, key: own.clone() };
        // No ceiling: lifespan is the tail of the rising hazard (1077).
        g.rules.processes.remove(&format!("ecology:age-{i}"));
        if let Some(upkeep) = g.rules.processes.get_mut(&format!("ecology:upkeep-{i}")) {
            upkeep.period = Some(minutes(time.upkeep));
            upkeep.effects = shed(&body, DETRITUS, 1);
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
        // After feeding and before upkeep, so a full body's birth tick is
        // not lost to the upkeep that falls on the same tick.
        birth.priority = 15;
        birth.note = true;
        // Some births land next door (1089): the lineage's drawn propensity
        // picks a birth to scatter, and the medium's strength at the parent's
        // site whether it goes.
        if time.spreads {
            let medium = MEDIA[(crate::draw(founding.seed, "medium", &[u64::from(i)]) % 2) as usize];
            let (low, high) = time.scatter;
            let propensity = low + (crate::draw(founding.seed, "scatter", &[u64::from(i)])
                % u64::from(high - low + 1)) as u32;
            birth.risk = Some(Risk { per_million: propensity, aging: None, effects: vec![Effect::Birth {
                provision: BTreeMap::from([(body.clone(), time.child)]), scatter: Some(medium.into()) }] });
        }
        if time.prey > 0 {
            // Breeding slows where food is scarce (1111), read per head (1145).
            let among = (0..founding.lineages).filter(|&j| role.eats(Role::of(founding.web, j)))
                .map(|j| format!("lineage:{j}")).collect();
            let food = if experimental {
                Query::MembersPerHead { among, per: None, at_least: time.prey, per_members: time.crowd }
            } else {
                Query::MembersPer { among, per: BTreeSet::from([format!("lineage:{i}")]), at_least: time.prey }
            };
            birth.requires.push(food);
        }
        g.rules.processes.insert(birth.id.clone(), birth);
        if time.leave > 0 {
            leaving(g, i, maturity, minutes(DAY), time)?;
        }
        if role == Role::Producer {
            if let Some(grow) = g.rules.processes.get_mut(&format!("ecology:produce-{i}")) {
                grow.period = Some(minutes(time.grow));
                // Light limits a crowded site: an act of growth spends what a
                // day's sun gives a site only a few hundred times.
                for effect in &mut grow.effects {
                    match effect {
                        Effect::Transfer { account, amount, .. } if account == "sim:energy" =>
                            *amount = LIGHT_PER_GROWTH.into(),
                        Effect::Transform { take, .. } if take.contains_key("sim:energy") =>
                            { take.insert("sim:energy".into(), LIGHT_PER_GROWTH); },
                        _ => {},
                    }
                }
                for q in &mut grow.requires {
                    if let Query::Sensed { reading, at_least } = q && reading == "sense:light-share" {
                        *at_least = LIGHT_PER_GROWTH;
                    }
                }
                if rooted {
                    for (reading, at_least) in [("sense:light-share", LIGHT_PER_GROWTH), ("sense:soil-share", 2)] {
                        if !grow.requires.iter().any(|q| matches!(q, Query::Sensed { reading: r, .. } if r == reading)) {
                            grow.requires.push(Query::Sensed { reading: reading.into(), at_least });
                        }
                    }
                }
                // A producer grows to what a seeding needs, then holds; a
                // wounded one, below adult size, grows again besides.
                let mut regrow = grow.clone();
                grow.requires.push(Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult + time.child });
                regrow.id = format!("ecology:regrow-{i}");
                regrow.period = Some(minutes(time.regrow));
                if rooted {
                    // Regrowth redistributes the store, independently of
                    // light and soil. The native effect reads the actual
                    // exposed deficit, even while surplus fills the root.
                    regrow.effects = vec![Effect::Regrow { account: body.clone(), amount: 2.into() }];
                    regrow.requires = vec![Query::Alive(Binding::Actor), trait_of()];
                    // It takes no soil, so it has no turn to take.
                    regrow.turn = None;
                } else {
                    regrow.requires.push(Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult });
                }
                g.rules.processes.insert(regrow.id.clone(), regrow);
            }
            if !experimental && founding.bodies.is_some() {
                plants::bodied(g, i, &time, &minutes);
            }
            if let (Some(rule), Some(share)) = (&mut dormancy, share) {
                dormant::record(rule, i, share);
                dormant::seeds(g, i, rule, share, minutes(rule.season));
            }
            continue;
        }
        let Some(feed) = g.rules.processes.get_mut(&format!("ecology:feed-{i}")) else { continue };
        // Eat only while hungry, below what a birth needs. An animal sheds the
        // lost share as dung; a decomposer turns it straight to soil. The
        // lost share is eaten in its own right and put down whole, so a bite
        // that finds less than it read (two eaters at one carcass in a tick)
        // fails as a whole rather than costing the eater its body.
        let lost = if role == Role::Decomposer { "world:soil" } else { DETRITUS };
        // An animal's bite takes tissue, never a plant's root store (1114);
        // a decomposer takes all a carcass holds.
        let of: Vec<Key> = if experimental || role == Role::Decomposer {
            vec![]
        } else {
            (0..founding.lineages).map(|j| format!("matter:{j}-0")).collect()
        };
        let eat = |into: &str, amount: u64| Effect::Eat { from: Binding::Target,
            amount: Amount::Fixed(amount), into: into.into(), of: of.clone(), whole: false };
        feed.effects = vec![eat(&body, time.kept), eat(lost, time.bite - time.kept),
            Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: lost.into(),
                amount: (time.bite - time.kept).into() }];
        feed.requires = vec![Query::Alive(Binding::Actor), trait_of(),
            Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult + time.child },
            Query::Holds { who: Binding::Target, at_least: time.bite }];
        // Each lineage feeds on a clock a minute apart from the next, so
        // when food is short no lineage always comes first by its index.
        let feed_every = minutes(time.feed + u64::from(i));
        feed.period = Some(feed_every);
        if role == Role::Decomposer {
            // Decomposers also eat the site's detritus, turning most
            // of it to soil for the producers (1078).
            // A metabolic transition, not a choice: the mind's needs stay
            // as they are (LineagePolicy), and it eats no body.
            let mut detritus = process(&format!("ecology:detritus-{i}"), Causation::Transition, vec![
                Effect::Transfer { from: Binding::Place, to: Binding::Actor, account: DETRITUS.into(),
                    amount: time.bite.into() },
                Effect::Transform { who: Binding::Actor, take: BTreeMap::from([(DETRITUS.into(), time.bite)]),
                    give: BTreeMap::from([(body.clone(), time.kept), ("world:soil".into(), time.bite - time.kept)]),
                    conversion: None },
                Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: "world:soil".into(),
                    amount: (time.bite - time.kept).into() },
            ]);
            detritus.requires.extend([trait_of(),
                Query::Below { who: Binding::Actor, key: body.clone(), amount: time.adult + time.child },
                Query::Account { who: Binding::Place, key: DETRITUS.into(), at_least: time.bite }]);
            detritus.period = Some(feed_every);
            detritus.priority = 11;
            g.rules.processes.insert(detritus.id.clone(), detritus);
        }
    }
    g.rules.dormancy = dormancy;
    Ok(())
}

/// A world-time founding's lineage shares: the role's founders.
pub(crate) fn founders(role: Role) -> u64 {
    role_time(role).founders
}

/// Each site's air and water strength for scattering (1103), out of a
/// hundred: what its region or world sets controls; otherwise it is read
/// from the laid terrain, water from how far the site stands above the sea
/// and air from its exposure, its height and relief within the world's.
/// The readings are candidates.
fn media(g: &mut Genesis) {
    g.rules.conditions.extend(MEDIA.map(Into::into));
    let Some(skeleton) = g.rules.skeleton.clone() else {
        for site in g.sites.values_mut() {
            for m in MEDIA { site.conditions.entry(m.into()).or_insert(UNLAID_STRENGTH); }
        }
        return;
    };
    let read = |key: &str| -> Vec<i64> { g.sites.values().filter_map(|s| s.conditions.get(key).copied()).collect() };
    let (heights, reliefs) = (read(&skeleton.elevation), read(&skeleton.relief));
    let (low, high) = (heights.iter().copied().min().unwrap_or(0), heights.iter().copied().max().unwrap_or(0));
    let roughest = reliefs.iter().copied().max().unwrap_or(0).max(1);
    let share = |part: i64, whole: i64| (i128::from(part.max(0)) * 100 / i128::from(whole.max(1))) as i64;
    for site in g.sites.values_mut() {
        let height = site.conditions.get(&skeleton.elevation).copied().unwrap_or(low);
        let relief = site.conditions.get(&skeleton.relief).copied().unwrap_or(0);
        let sea = site.conditions.get(&skeleton.water).copied().unwrap_or(low);
        let water = 100 - share(height - sea, high - sea);
        let air = (share(height - low, high - low) + share(relief, roughest)) / 2;
        site.conditions.entry(WATER.into()).or_insert(water.clamp(0, 100));
        site.conditions.entry(AIR.into()).or_insert(air.clamp(0, 100));
    }
}

/// A crowded walker's adults leave along routes (1111): a move per route,
/// within the world's journeys (Processes 16), open to an adult of lineage
/// `i` at the route's site while at least `crowd` of its lineage live there,
/// each day taken with the lineage's chance. A site names itself by the
/// condition `site:<id>`, as migration's moves read it (753).
fn leaving(g: &mut Genesis, i: u32, maturity: Tick, day: Tick, time: RoleTime) -> Result<()> {
    let lineage = format!("lineage:{i}");
    let mut moves = Vec::new();
    for (&id, site) in &mut g.sites {
        let here: Key = format!("site:{id}");
        site.conditions.insert(here.clone(), 1);
        g.rules.conditions.insert(here.clone());
        for route in &site.routes {
            let parallel = site.routes.iter().filter(|r| r.to == route.to).count() > 1;
            let via = route.border.map(|b| b.side).filter(|_| parallel);
            let name = match via {
                Some(side) => format!("move:{id}-{}-{side}/{lineage}", route.to),
                None => format!("move:{id}-{}/{lineage}", route.to),
            };
            let mut p = process(&name, Causation::Transition, vec![]);
            p.requires.extend([
                Query::Trait { who: Binding::Actor, key: format!("ability:cycle-{i}") },
                Query::Condition { key: here.clone(), at_least: 1 },
                Query::Age { at_least: maturity },
                Query::Members { among: BTreeSet::from([lineage.clone()]), at_least: time.crowd },
            ]);
            p.risk = Some(Risk { per_million: time.leave, aging: None,
                effects: vec![Effect::Move { destination: route.to, via }] });
            p.period = Some(day);
            p.priority = 45;
            moves.push(p);
        }
    }
    if g.rules.processes.len() + moves.len() > g.rules.limits.processes {
        return Err("a walker's move per route exceeds the world's process limit".into());
    }
    g.rules.processes.extend(moves.into_iter().map(|p| (p.id.clone(), p)));
    Ok(())
}
