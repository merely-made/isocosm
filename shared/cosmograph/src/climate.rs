// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Temperature and what it costs a body outside its comfort (case 1215;
//! lineage-rounds D13 to D15). Temperature is a pure function of the rules,
//! a site and the tick, as `light::at` is: nothing stored, nothing hashed.
//! It is whole degrees throughout, with no floating point. Unset, a world
//! has no temperature and every function here answers none or nought.

use crate::{
    Result, Simulation, anatomy, draw,
    light::LATITUDE,
    rules::{
        Amount, Binding, Climate, Effect, Expr, Query, Reading, Rules, year_ticks,
    },
    schema::{Entity, Id, Key, Lineage, Site, Tick},
};
use std::collections::BTreeMap;

/// A pole's latitude, in hundredths of a degree.
const POLE: i128 = 9_000;
/// The triangle wave's peak.
const PEAK: i128 = 1_000;

/// The temperature at `site` (the map key `site_id`) on `tick`, in degrees:
/// the equator's, less the pole's share by latitude and the sea's by height,
/// plus the year's swing and the weather's anomaly for the site's spell.
/// None where the world has no climate.
pub fn temperature(
    rules: &Rules,
    seed: u64,
    site_id: Id,
    site: &Site,
    tick: Tick,
) -> Option<i64> {
    let c = rules.climate.as_ref()?;
    let latitude = site
        .conditions
        .get(LATITUDE)
        .copied()
        .unwrap_or(0)
        .clamp(-9_000, 9_000);
    let latitude = i128::from(latitude);
    let cold = i128::from(c.poleward) * latitude.abs() / POLE;
    let aloft = i128::from(c.lapse) * i128::from(height(rules, site)) / i128::from(c.rise.max(1));
    let swing = i128::from(c.season) * latitude * wave(rules, tick) / (POLE * PEAK);
    let spell = tick / c.spell.max(1);
    let span = u64::try_from(2 * c.weather.max(0) + 1).unwrap_or(1);
    let weather = i128::from(draw(seed, "climate:weather", &[site_id, spell]) % span)
        - i128::from(c.weather.max(0));
    let degrees = i128::from(c.base) - cold - aloft + swing + weather;
    Some(i64::try_from(degrees).unwrap_or(if degrees < 0 { i64::MIN } else { i64::MAX }))
}

/// A site's height above the sea in base units: its elevation less its
/// water level, nought where the world lays no terrain.
fn height(rules: &Rules, site: &Site) -> i64 {
    let Some(skeleton) = &rules.skeleton else { return 0 };
    let read = |key: &Key| site.conditions.get(key).copied().unwrap_or(0);
    read(&skeleton.elevation).saturating_sub(read(&skeleton.water))
}

/// A triangle over the world's year, from -1,000 to 1,000: nought at tick 0,
/// rising a quarter year, falling a half, rising a quarter. None where the
/// sun is tidally locked and the year does not turn.
fn wave(rules: &Rules, tick: Tick) -> i128 {
    if rules.sun.as_ref().is_some_and(|s| s.tidal_lock) {
        return 0;
    }
    let year = u128::from(year_ticks(rules.tick_microseconds()));
    let at = i128::try_from(u128::from(tick) % year * 4 * PEAK as u128 / year).unwrap_or(0);
    if at < PEAK {
        at
    } else if at < 3 * PEAK {
        2 * PEAK - at
    } else {
        at - 4 * PEAK
    }
}

/// The whole doublings of mass from `reference` to `mass`, negative below it.
fn doublings(mass: u128, reference: u64) -> i64 {
    let (mass, reference) = (mass.max(1), u128::from(reference.max(1)));
    if mass >= reference {
        return i64::from((mass / reference).ilog2());
    }
    let (mut held, mut halvings) = (mass, 0);
    while held < reference {
        held *= 2;
        halvings += 1;
    }
    -halvings
}

/// The comfort band of a body of adult `mass` mg, as the lineage's `offset`
/// shifts it: the centre falls `doubling` degrees for each doubling of mass
/// beyond `reference`, since a bigger body holds heat (case 1215;
/// lineage-rounds D14). The band is the centre less and plus the width.
pub fn band(climate: &Climate, mass: u128, offset: i64) -> [i64; 2] {
    let fall = climate.doubling.saturating_mul(doublings(mass, climate.reference));
    let centre = climate.centre.saturating_sub(fall).saturating_add(offset);
    [centre.saturating_sub(climate.width), centre.saturating_add(climate.width)]
}

/// How far `degrees` lie outside `band`: nought within it and at its edges,
/// else the distance to the nearer edge (case 1215; lineage-rounds D13).
pub fn strain(band: [i64; 2], degrees: i64) -> u64 {
    let beyond = if degrees < band[0] {
        i128::from(band[0]) - i128::from(degrees)
    } else if degrees > band[1] {
        i128::from(degrees) - i128::from(band[1])
    } else {
        0
    };
    u64::try_from(beyond).unwrap_or(u64::MAX)
}

/// The comfort band of `e`: its adult mass, the sum of its parts'
/// ceilings (its build, not what it holds), or the reference where it has
/// no geometry, shifted by its lineage's development. None where the world
/// has no climate.
pub fn comfort(
    rules: &Rules,
    e: &Entity,
    lineages: &BTreeMap<Key, Lineage>,
) -> Option<[i64; 2]> {
    let c = rules.climate.as_ref()?;
    let b = rules.body();
    let mass: u128 = e
        .living()
        .map(|(id, _)| u128::from(anatomy::ceiling(e.extent(id), b)))
        .sum();
    let mass = if e.body.is_none() || mass == 0 { u128::from(c.reference) } else { mass };
    let offset = lineages
        .get(&e.lineage)
        .and_then(|l| l.development.as_ref())
        .map_or(0, |d| i64::from(d.comfort));
    Some(band(c, mass, offset))
}

/// The matter `e`, resident at site `place`, owes on `tick` for being
/// outside its comfort: its strain times the world's rate, the fraction kept
/// by paying the whole matter due by the end of the tick less that due by
/// its start, so that a small strain is not lost to rounding. Nought where
/// the world has no climate.
pub(crate) fn owed(
    rules: &Rules,
    seed: u64,
    lineages: &BTreeMap<Key, Lineage>,
    e: &Entity,
    (place, site): (Id, &Site),
    tick: Tick,
) -> u64 {
    let Some(c) = rules.climate.as_ref() else { return 0 };
    let (Some(band), Some(degrees)) =
        (comfort(rules, e, lineages), temperature(rules, seed, place, site, tick))
    else {
        return 0;
    };
    let rate = u128::from(strain(band, degrees)) * u128::from(c.strain[0]);
    let by = |t: u128| t * rate / u128::from(c.strain[1].max(1));
    let tick = u128::from(tick);
    u64::try_from(by(tick + 1) - by(tick)).unwrap_or(u64::MAX)
}

/// Gives each lineage whose upkeep the world prices a process beside it,
/// `climate:strain-<i>`, that spends the strain's matter from the same
/// accounts into the same soil as its upkeep does (case 1215; lineage-rounds
/// D13). A body that cannot pay gives what it has, as upkeep's spend does.
/// A no-op where the world has no climate; running it twice adds nothing.
///
/// # Errors
/// Fails if the new processes pass the world's limit on them.
pub(crate) fn processes(rules: &mut Rules) -> Result<()> {
    if rules.climate.is_none() {
        return Ok(());
    }
    let mut lines = BTreeMap::<String, (&str, usize)>::new();
    for (id, p) in &rules.processes {
        let Some(i) = id
            .strip_prefix("body:upkeep-")
            .or_else(|| id.strip_prefix("ecology:upkeep-"))
        else {
            continue;
        };
        let paid = p.effects.iter().find_map(paying);
        // A body-priced upkeep outranks the flat one a lineage may keep.
        let rank = usize::from(id.starts_with("body:"));
        if paid.is_some() && lines.get(i).is_none_or(|(_, held)| *held < rank) {
            lines.insert(i.into(), (id.as_str(), rank));
        }
    }
    let mut made = Vec::new();
    for (i, (id, _)) in lines {
        let beside = &rules.processes[id];
        let Some((from, into)) = beside.effects.iter().find_map(paying) else { continue };
        let own = format!("climate:strain-{i}");
        if rules.processes.contains_key(&own) {
            continue;
        }
        let owed = Expr::Read(Reading::Strain { who: Binding::Actor });
        let spend = Effect::Spend {
            from,
            to: Binding::Place,
            amount: Amount::Computed(owed),
            into: Some(into),
        };
        let mut strain = crate::generate::process(&own, crate::rules::Causation::Transition, vec![spend]);
        strain.requires = beside
            .requires
            .iter()
            .filter(|q| matches!(q, Query::Trait { .. }))
            .cloned()
            .collect();
        strain.period = Some(1);
        strain.priority = beside.priority;
        made.push(strain);
    }
    if rules.processes.len() + made.len() > rules.limits.processes {
        return Err("a climate's strain per lineage exceeds the world's process limit".into());
    }
    rules.processes.extend(made.into_iter().map(|p| (p.id.clone(), p)));
    Ok(())
}

/// What an upkeep effect pays from and into: the accounts it takes, and
/// where they go.
fn paying(effect: &Effect) -> Option<(Vec<Key>, Key)> {
    match effect {
        Effect::Spend { from, into: Some(into), .. } => Some((from.clone(), into.clone())),
        Effect::Transform { take, give, .. } if !take.is_empty() => {
            Some((take.keys().cloned().collect(), give.keys().next()?.clone()))
        },
        _ => None,
    }
}

impl Simulation {
    /// The temperature at `site` now, in degrees, where the world has a
    /// climate and the site is there (case 1215; lineage-rounds D15).
    pub fn temperature(&self, site: Id) -> Option<i64> {
        let found = self.state.sites.get(&site)?;
        temperature(&self.genesis.rules, self.genesis.seed, site, found, self.state.tick)
    }
}
