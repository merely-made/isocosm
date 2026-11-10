// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Temperature and what it costs a body outside its comfort (case 1215;
//! lineage-rounds D13 to D15). Temperature is a pure function of the rules,
//! a site and the tick, as `light::at` is: nothing stored, nothing hashed.
//! It is whole degrees throughout, with no floating point. Water and light
//! add two more strains to the same process: store cells buffer drought,
//! and fixing cells set the light a body needs (D14). Unset, a world has no
//! temperature and every function here answers none or nought.

use crate::{
    Result, Simulation, anatomy, draw,
    ecology::{LIGHT, world_time::WATER},
    light::{self, LATITUDE},
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
pub(crate) fn wave(rules: &Rules, tick: Tick) -> i128 {
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

/// The function key of a cell that fixes light.
const FIX: &str = "function:fix";

/// The store, fixing and all cells of a body, summed over its parts' cells by
/// function as the sheet sums them.
pub(crate) fn tally<'a>(cells: impl IntoIterator<Item = (&'a Key, &'a u32)>) -> [u64; 3] {
    let mut sums = [0; 3];
    for (function, n) in cells {
        let n = u64::from(*n);
        sums[0] += n * u64::from(*function == anatomy::STORE);
        sums[1] += n * u64::from(function == FIX);
        sums[2] += n;
    }
    sums
}

/// The least `world:water` a body of `store` cells among `cells` is
/// comfortable with: `thirst` less `buffer` for a body all of store cells,
/// in proportion, and never below nought (case 1215; lineage-rounds D14).
pub fn thirst(climate: &Climate, store: u64, cells: u64) -> i64 {
    let held = if cells == 0 {
        0
    } else {
        i128::from(climate.buffer) * i128::from(store.min(cells)) / i128::from(cells)
    };
    i64::try_from((i128::from(climate.thirst) - held).max(0)).unwrap_or(0)
}

/// The light a body of `fix` cells among `cells` needs: `shade` for a body
/// all of fixing cells, in proportion, nought for one with none (case 1215;
/// lineage-rounds D14).
pub fn light(climate: &Climate, fix: u64, cells: u64) -> i64 {
    if cells == 0 {
        return 0;
    }
    let need = i128::from(climate.shade) * i128::from(fix.min(cells)) / i128::from(cells);
    i64::try_from(need).unwrap_or(i64::MAX)
}

/// What `e` needs of water and of light, as `[thirst, light]`, from its
/// build: the cells of its parts. None where the world has no climate.
/// A lineage's development shifts only the temperature band, so `lineages`
/// is not read yet.
pub fn needs(
    rules: &Rules,
    e: &Entity,
    _lineages: &BTreeMap<Key, Lineage>,
) -> Option<[i64; 2]> {
    let c = rules.climate.as_ref()?;
    let [store, fix, cells] = tally(e.parts.values().flat_map(|p| p.cells.iter()));
    let cells = if e.body.is_none() { 0 } else { cells };
    let [store, fix] = if cells == 0 { [0, 0] } else { [store, fix] };
    Some([thirst(c, store, cells), light(c, fix, cells)])
}

/// The light at `site` on `tick` in the units the ecology stages sunlight
/// in: its full credit scaled by the sun's intensity, rounded as the staging
/// of `Effect::Sunlight` rounds it, only with the site's latitude.
fn lit(rules: &Rules, site: &Site, tick: Tick) -> i64 {
    let sky = light::at(rules, tick, Some(site));
    let intensity = sky.sun.map_or(0.0, |sun| f64::from(sun.intensity));
    i64::try_from((LIGHT as f64 * intensity).round() as u64).unwrap_or(i64::MAX)
}

/// The matter `e`, resident at site `place`, owes on `tick` for being
/// outside its comfort: the sum of three strains, each its points beyond
/// the body's need (degrees outside the band, water below its thirst, light
/// below what it needs) times that part's rate. Each part keeps its fraction
/// by paying the whole matter due by the end of the tick less that due by
/// its start, so that a small strain is not lost to rounding, and the parts
/// add exactly. Nought where the world has no climate.
pub(crate) fn owed(
    rules: &Rules,
    seed: u64,
    lineages: &BTreeMap<Key, Lineage>,
    e: &Entity,
    (place, site): (Id, &Site),
    tick: Tick,
) -> u64 {
    let Some(c) = rules.climate.as_ref() else { return 0 };
    let Some([want, shade]) = needs(rules, e, lineages) else { return 0 };
    let heat = match (comfort(rules, e, lineages), temperature(rules, seed, place, site, tick)) {
        (Some(band), Some(degrees)) => strain(band, degrees),
        _ => 0,
    };
    let water = site.conditions.get(WATER).copied().unwrap_or(100);
    let dry = u64::try_from(want.saturating_sub(water)).unwrap_or(0);
    let dark = if shade > 0 { u64::try_from(shade - lit(rules, site, tick)).unwrap_or(0) } else { 0 };
    let t = u128::from(tick);
    let due = |points: u64, [n, d]: [u32; 2]| {
        let rate = u128::from(points) * u128::from(n);
        let by = |t: u128| t * rate / u128::from(d.max(1));
        by(t + 1) - by(t)
    };
    let total = due(heat, c.strain) + due(dry, c.drought) + due(dark, c.dim);
    u64::try_from(total).unwrap_or(u64::MAX)
}

/// The suffix a lineage's upkeep process id carries, the lineage's number.
pub(crate) fn upkeep_of(id: &str) -> Option<&str> {
    id.strip_prefix("body:upkeep-").or_else(|| id.strip_prefix("ecology:upkeep-"))
}

/// Each lineage's priced upkeep, by its number: the process, and its rank (a
/// body-priced upkeep outranks the flat one a lineage may keep).
pub(crate) fn upkeeps(rules: &Rules) -> BTreeMap<String, (&str, usize)> {
    let mut lines = BTreeMap::<String, (&str, usize)>::new();
    for (id, p) in &rules.processes {
        let Some(i) = upkeep_of(id) else { continue };
        let paid = p.effects.iter().find_map(paying);
        let rank = usize::from(id.starts_with("body:"));
        if paid.is_some() && lines.get(i).is_none_or(|(_, held)| *held < rank) {
            lines.insert(i.into(), (id.as_str(), rank));
        }
    }
    lines
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
    let lines = upkeeps(rules);
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
pub(crate) fn paying(effect: &Effect) -> Option<(Vec<Key>, Key)> {
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
