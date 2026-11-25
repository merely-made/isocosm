// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A regional outlook from an unattended fine sample. The source world stays
//! playable at its original tick. Measured transactions run on regional stocks;
//! a shortage limits all legs of a transaction together. This local outlook
//! does not resolve new anatomy, individual lives or unobserved strategies.
use crate::{
    Result, Session, Simulation,
    flows::{Flow, Holder, Kind, MadeBy},
    schema::{Id, Key, Tick},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Region {
    /// Bodies are counted at their last residence until arrival.
    pub living: BTreeMap<Key, u64>,
    /// Microbial surface matter across ground, finer places and hosts, by lineage.
    pub surfaces: BTreeMap<Key, u128>,
    /// Matter in living organisms, including dormant ones, by lineage.
    pub biomass: BTreeMap<Key, u128>,
    pub carcasses: BTreeMap<Key, u128>,
    /// Local matter outside bodies, by native account.
    pub resources: BTreeMap<Key, u128>,
}

impl Region {
    /// Read the current regional stocks and population directly from a fine world.
    pub fn read(sim: &Simulation) -> Result<BTreeMap<Id, Region>> {
        regions(&census(sim)?, sim.state().sites.keys().copied())
    }

    /// A bodied lineage persists through living members, a microbial surface
    /// through measurable stock. World-class geography has no population.
    pub fn extant_lineages(&self) -> impl Iterator<Item = &str> {
        self.living
            .keys()
            .chain(self.surfaces.keys())
            .map(String::as_str)
    }
}

#[derive(Clone, Debug)]
pub struct RegionalForecast {
    pub start_tick: Tick,
    pub sample_ticks: Tick,
    pub end_tick: Tick,
    pub sites: BTreeMap<Id, Region>,
    observed: ObservedFlows,
}

/// Largest net gain and loss by process in the fine sample, per site/lineage.
type ObservedFlows = BTreeMap<(Id, Key, bool), (Key, u128)>;

impl RegionalForecast {
    /// Native process and matter gained (or lost) in the observation interval.
    /// These are historical measurements, independent of the forecast horizon.
    pub fn strongest_flow(&self, site: Id, lineage: &str, gains: bool) -> Option<(&str, u128)> {
        self.observed
            .get(&(site, lineage.into(), gains))
            .map(|(k, n)| (k.as_str(), *n))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reservoir {
    Members(Key),
    Living(Key),
    Surface(Key),
    Dead(Key),
    Ground(Key),
}
type Address = (Id, Reservoir);
type Balances = BTreeMap<Address, i128>;

/// A closed material transaction. Each unit carries its coupled inputs and
/// outputs. Demographic transactions can also change counts.
#[derive(Clone, Debug)]
struct Rate {
    changes: Balances,
    lots: u64,
    needs: BTreeMap<Address, i128>,
}

impl Rate {
    fn new(mut changes: Balances, initial: &Balances, sampled: &Balances) -> Result<Option<Self>> {
        changes.retain(|_, change| *change != 0);
        if changes.is_empty() {
            return Ok(None);
        }
        if changes
            .iter()
            .filter(|((_, k), _)| !matches!(k, Reservoir::Members(_)))
            .map(|(_, n)| n)
            .sum::<i128>()
            != 0
        {
            return Err("a regional transaction includes unrepresented matter".into());
        }
        let mut units = 0u128;
        for v in changes.values() {
            let mut n = v.unsigned_abs();
            while n != 0 {
                (units, n) = (n, units % n);
            }
        }
        let lots = u64::try_from(units).map_err(|_| "regional rate exceeds integer range")?;
        let mut needs = BTreeMap::new();
        for ((site, reservoir), change) in &mut changes {
            *change /= i128::from(lots);
            if let Some(a) = match reservoir {
                Reservoir::Living(line) | Reservoir::Members(line) => {
                    Some((*site, Reservoir::Members(line.clone())))
                },
                Reservoir::Surface(_) => Some((*site, reservoir.clone())),
                _ => None,
            } {
                // A destination first colonized during the sample can be
                // entered from its measured source. Local production needs stock.
                let start = initial.get(&a).copied().unwrap_or(0);
                let end = sampled.get(&a).copied().unwrap_or(0);
                if start > 0 {
                    needs.insert(a, ((start + end) / 2).max(1));
                }
            }
        }
        Ok(Some(Self {
            changes,
            lots,
            needs,
        }))
    }

    fn apply(
        &self,
        stocks: &mut Balances,
        width: Tick,
        interval: Tick,
        fraction: &mut u128,
        mass_per_member: &BTreeMap<Address, i128>,
    ) {
        // Keep fractional events between intervals. One observed loss still
        // reaches a small surviving cohort as its density falls.
        const SCALE: u128 = 1_000_000_000;
        let demand = u128::from(self.lots) * u128::from(width);
        let duration = u128::from(interval);
        let demand = demand / duration * SCALE + (demand % duration) * SCALE / duration;
        let mut density = SCALE;
        for (address, observed) in &self.needs {
            let living = stocks.get(address).copied().unwrap_or(0);
            if living < *observed {
                density = density.min(living as u128 * SCALE / *observed as u128);
            }
        }
        if density == 0 {
            *fraction = 0;
            return;
        }
        let demand = demand * density / SCALE + *fraction;
        *fraction = demand % SCALE;
        let mut n = (demand / SCALE) as i128;
        if n == 0 {
            return;
        }
        for (a, delta) in &self.changes {
            if *delta < 0 {
                n = n.min(stocks.get(a).copied().unwrap_or(0) / -*delta);
            }
        }
        // New members need the observed stock per viable member. A site
        // that ran out of food can no longer extrapolate the sample's births.
        for ((site, key), delta) in &self.changes {
            let Reservoir::Members(line) = key else {
                continue;
            };
            if *delta <= 0 {
                continue;
            }
            let members = (*site, key.clone());
            let mass = (*site, Reservoir::Living(line.clone()));
            let Some(&each) = mass_per_member.get(&members) else {
                continue;
            };
            let net_cost = *delta * each - self.changes.get(&mass).copied().unwrap_or(0);
            if net_cost > 0 {
                let required = stocks.get(&members).copied().unwrap_or(0) * each;
                let available = (stocks.get(&mass).copied().unwrap_or(0) - required).max(0);
                n = n.min(available / net_cost);
            }
        }
        for (a, delta) in &self.changes {
            *stocks.entry(a.clone()).or_default() += *delta * n;
        }
    }
}

pub struct RegionalModel {
    from: Tick,
    sample: Tick,
    initial: BTreeMap<Id, Region>,
    observed: ObservedFlows,
    sampled: Balances,
    rates: Vec<Rate>,
    mass_per_member: BTreeMap<Address, i128>,
}

fn address(
    sim: &Simulation,
    endpoint: &(Holder, Key),
    kind: Option<&Kind>,
    location: Option<(Id, bool)>,
) -> Option<Address> {
    let account = &endpoint.1;
    let site = match endpoint.0 {
        Holder::Site(site) => site,
        _ => location?.0,
    };
    if sim
        .state()
        .spreads
        .identities
        .contains(&crate::spread::Identity::Surface(account.clone()))
    {
        return Some((site, Reservoir::Surface(account.clone())));
    }
    if matches!(endpoint.0, Holder::Site(_)) {
        return Some((site, Reservoir::Ground(account.clone())));
    }
    let (site, alive) = location?;
    let lineage = kind?.lineage.clone();
    Some((
        site,
        if alive {
            Reservoir::Living(lineage)
        } else {
            Reservoir::Dead(lineage)
        },
    ))
}

fn census(sim: &Simulation) -> Result<Balances> {
    let mut b = Balances::new();
    let s = sim.state();
    let rules = &sim.genesis().rules;
    let is_matter = |key: &str| crate::meaning::matter(rules, key) || s.spreads.owns(key);
    let surfaces: BTreeSet<&str> = s
        .spreads
        .identities
        .iter()
        .filter_map(|identity| match identity {
            crate::spread::Identity::Surface(line) => Some(line.as_str()),
            crate::spread::Identity::Territory(_) => None,
        })
        .collect();
    let regional_stock = |account: &str| {
        if surfaces.contains(account) {
            Reservoir::Surface(account.into())
        } else {
            Reservoir::Ground(account.into())
        }
    };
    let places = s
        .sites
        .iter()
        .map(|(site, place)| (*site, &place.accounts))
        .chain(s.spreads.places.iter().map(|p| (p.place.site, &p.accounts)));
    for (site, ledger) in places {
        for (k, n) in ledger {
            if is_matter(k) && *n > 0 {
                *b.entry((site, regional_stock(k))).or_default() += i128::from(*n);
            }
        }
    }
    // In-flight spread stock stays with the departure region until arrival.
    for t in &s.spreads.transit {
        *b.entry((t.from, regional_stock(&t.identity.key())))
            .or_default() += i128::from(t.amount);
    }
    for g in s.population.groups.values() {
        let e = &g.entity;
        let mut mass = 0i128;
        let books = crate::anatomy::books(e);
        for (account, amount) in books
            .iter()
            .filter(|(k, _)| is_matter(k))
            .map(|(k, n)| (k, i128::from(*n) * i128::from(g.count)))
            .chain(
                g.pool
                    .iter()
                    .filter(|(k, _)| is_matter(k))
                    .map(|(k, n)| (k, *n)),
            )
        {
            if surfaces.contains(account.as_str()) {
                *b.entry((e.place, Reservoir::Surface(account.clone())))
                    .or_default() += amount;
            } else {
                mass = mass.checked_add(amount).ok_or("cohort mass overflow")?;
            }
        }
        if mass < 0 {
            return Err("negative cohort reserve".into());
        }
        if e.alive
            && s.lineages.contains_key(&e.lineage)
            && !crate::directing::is_participant(e)
            && e.lineage != "world:ground"
        {
            *b.entry((e.place, Reservoir::Members(e.lineage.clone())))
                .or_default() += i128::from(g.count);
        }
        if mass > 0 {
            let reservoir = if e.alive {
                Reservoir::Living(e.lineage.clone())
            } else {
                Reservoir::Dead(e.lineage.clone())
            };
            *b.entry((e.place, reservoir)).or_default() += mass;
        }
    }
    let captured: i128 = b
        .iter()
        .filter(|((_, k), _)| !matches!(k, Reservoir::Members(_)))
        .map(|(_, n)| n)
        .sum();
    if captured as u128 != sim.matter() {
        return Err("regional outlook requires all matter in bodies and site stocks".into());
    }
    Ok(b)
}

fn regions(b: &Balances, sites: impl Iterator<Item = Id>) -> Result<BTreeMap<Id, Region>> {
    let mut out: BTreeMap<_, _> = sites.map(|s| (s, Region::default())).collect();
    for ((site, r), &n) in b.iter().filter(|(_, n)| **n != 0) {
        let g = out.get_mut(site).ok_or("a regional balance has no site")?;
        match r {
            Reservoir::Members(k) => {
                g.living.insert(
                    k.clone(),
                    u64::try_from(n).map_err(|_| "regional population overflow")?,
                );
            },
            Reservoir::Living(k) => {
                g.biomass.insert(k.clone(), n as u128);
            },
            Reservoir::Surface(k) => {
                g.surfaces.insert(k.clone(), n as u128);
            },
            Reservoir::Dead(k) => {
                g.carcasses.insert(k.clone(), n as u128);
            },
            Reservoir::Ground(k) => {
                g.resources.insert(k.clone(), n as u128);
            },
        }
    }
    Ok(out)
}

fn record(sim: &Simulation, f: &Flow, turns: &mut BTreeMap<(Key, Id), Balances>) -> Result<()> {
    let MadeBy::Process(key) = &f.made_by else {
        return Ok(());
    };
    let is_matter = |key: &str| {
        crate::meaning::matter(&sim.genesis().rules, key) || sim.state().spreads.owns(key)
    };
    match (is_matter(&f.from.1), is_matter(&f.to.1)) {
        (false, false) => return Ok(()),
        (true, true) => {},
        _ => return Err("regional flow crosses the material ledger boundary".into()),
    }
    let from = address(sim, &f.from, f.from_kind.as_ref(), f.body_locations[0])
        .ok_or("unsupported source in regional flow")?;
    let to = address(sim, &f.to, f.to_kind.as_ref(), f.body_locations[1])
        .ok_or("unsupported destination in regional flow")?;
    if from != to {
        let amount = i128::from(f.amount)
            .checked_mul(i128::from(f.count))
            .ok_or("regional flow overflow")?;
        let book = turns.entry((key.clone(), from.0)).or_default();
        *book.entry(from).or_default() -= amount;
        *book.entry(to).or_default() += amount;
    }
    Ok(())
}

impl RegionalModel {
    /// Measure actual native feeding, growth, exchange, reproduction and loss
    /// on a fine copy; the caller controls when this work runs.
    pub fn observe(source: &Session, sample_ticks: Tick) -> Result<Self> {
        if sample_ticks == 0 {
            return Err("regional outlook needs a nonzero observation interval".into());
        }
        let state = source.sim.state();
        let nudged: BTreeSet<_> = state.nudges.iter().map(|n| n.critter).collect();
        if !state.attention.participants().is_empty()
            || nudged
                .into_iter()
                .any(|id| !source.sim.live_nudges(id, state.tick).is_empty())
        {
            return Err(
                "regional outlook needs an unattended source without pending directions".into(),
            );
        }
        let mut copy = source.clone();
        let initial = census(&copy.sim)?;
        let mut turns = BTreeMap::new();
        for _ in 0..sample_ticks {
            for f in copy.advance_tick_with_flows()?.flows {
                record(&copy.sim, &f, &mut turns)?;
            }
        }
        let sampled = census(&copy.sim)?;
        let mut rest = sampled.clone();
        for (a, n) in &initial {
            *rest.entry(a.clone()).or_default() -= n;
        }
        let mut rates = Vec::new();
        let mut observed = ObservedFlows::new();
        for ((process, _), changes) in turns {
            for (a, n) in &changes {
                if let (site, Reservoir::Living(line) | Reservoir::Surface(line)) = a
                    && *n != 0
                {
                    let entry = observed.entry((*site, line.clone(), *n > 0)).or_default();
                    if n.unsigned_abs() > entry.1 {
                        *entry = (process.clone(), n.unsigned_abs());
                    }
                }
                *rest.entry(a.clone()).or_default() -= n;
            }
            if let Some(rate) = Rate::new(changes, &initial, &sampled)? {
                rates.push(rate);
            }
        }
        // Birth, death and moving to a new site change the owner of matter
        // besides the native transfer legs. Retain those changes together
        // with the counts that actually occurred. Sites with net migration
        // exchange their balances in a single conservative operation.
        let mut by_lin: BTreeMap<(Id, Option<Key>), Balances> = BTreeMap::new();
        for (a, n) in rest.into_iter().filter(|(_, v)| *v != 0) {
            let lineage = match &a.1 {
                Reservoir::Members(k)
                | Reservoir::Living(k)
                | Reservoir::Surface(k)
                | Reservoir::Dead(k) => Some(k.clone()),
                Reservoir::Ground(_) => None,
            };
            by_lin.entry((a.0, lineage)).or_default().insert(a, n);
        }
        let mut by_site: BTreeMap<Id, Balances> = BTreeMap::new();
        // Keep local birth/death changes independent when their mass books
        // close. A migration can require a coupled source and destination.
        let material = |changes: &Balances| -> i128 {
            changes
                .iter()
                .filter(|((_, r), _)| !matches!(r, Reservoir::Members(_)))
                .map(|(_, n)| n)
                .sum()
        };
        for ((site, _), changes) in by_lin {
            if material(&changes) != 0 {
                by_site.entry(site).or_default().extend(changes);
            } else if let Some(rate) = Rate::new(changes, &initial, &sampled)? {
                rates.push(rate);
            }
        }
        let mut migration = Balances::new();
        for changes in by_site.into_values() {
            if material(&changes) != 0 {
                migration.extend(changes);
            } else if let Some(rate) = Rate::new(changes, &initial, &sampled)? {
                rates.push(rate);
            }
        }
        if let Some(rate) = Rate::new(migration, &initial, &sampled)? {
            rates.push(rate);
        }
        let mut mass_per_member = BTreeMap::new();
        for census in [&initial, &sampled] {
            for ((site, kind), count) in census {
                if let Reservoir::Members(line) = kind {
                    let a = (*site, Reservoir::Living(line.clone()));
                    let each = census.get(&a).copied().unwrap_or(0) / count;
                    mass_per_member
                        .entry((*site, kind.clone()))
                        .and_modify(|min: &mut i128| *min = (*min).min(each))
                        .or_insert(each);
                }
            }
        }
        Ok(Self {
            observed,
            mass_per_member,
            from: source.sim.state().tick,
            sample: sample_ticks,
            initial: regions(&initial, source.sim.state().sites.keys().copied())?,
            sampled,
            rates,
        })
    }

    pub fn start(&self) -> &BTreeMap<Id, Region> {
        &self.initial
    }

    /// Span from the original tick, including the fine sample. Reapply
    /// observed rates once per sample interval with substrate exhaustion
    /// and local loss limiting later turnover.
    pub fn project(&self, span_ticks: Tick) -> Result<RegionalForecast> {
        if span_ticks < self.sample {
            return Err("forecast span ends before the observation interval".into());
        }
        let end_tick = self
            .from
            .checked_add(span_ticks)
            .ok_or("forecast clock overflow")?;
        let mut b = self.sampled.clone();
        let mut fractions = vec![0; self.rates.len()];
        let mut left = span_ticks - self.sample;
        while left > 0 {
            let width = left.min(self.sample);
            for (rate, fraction) in self.rates.iter().zip(&mut fractions) {
                rate.apply(&mut b, width, self.sample, fraction, &self.mass_per_member);
            }
            // A lost population leaves its remaining matter as carrion. If
            // all its stores ran out, a population that needs biomass collapses.
            for ((site, key), &each) in &self.mass_per_member {
                let Reservoir::Members(line) = key else {
                    continue;
                };
                let members = (*site, key.clone());
                let living = (*site, Reservoir::Living(line.clone()));
                if each > 0 && b.get(&living).copied().unwrap_or(0) == 0 {
                    b.remove(&members);
                }
                if b.get(&members).copied().unwrap_or(0) == 0 {
                    let carcass = b.remove(&living).unwrap_or(0);
                    *b.entry((*site, Reservoir::Dead(line.clone()))).or_default() += carcass;
                }
            }
            left -= width;
        }
        Ok(RegionalForecast {
            observed: self.observed.clone(),
            start_tick: self.from,
            sample_ticks: self.sample,
            end_tick,
            sites: regions(&b, self.initial.keys().copied())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_supply_binds_growth_and_broods_on_later_steps() {
        let members = (1, Reservoir::Members("sprout".into()));
        let biomass = (1, Reservoir::Living("sprout".into()));
        let soil = (1, Reservoir::Ground("soil".into()));
        let initial = Balances::from([
            (members.clone(), 2),
            (biomass.clone(), 10),
            (soil.clone(), 6),
        ]);
        let changed = Balances::from([
            (members.clone(), 1),
            (biomass.clone(), 5),
            (soil.clone(), -5),
        ]);
        let growth = Rate::new(changed, &initial, &initial).unwrap().unwrap();
        let birth = Rate::new(Balances::from([(members.clone(), 1)]), &initial, &initial)
            .unwrap()
            .unwrap();
        let minimum = BTreeMap::from([(members.clone(), 5)]);
        let mut after = initial;
        for _ in 0..40 {
            growth.apply(&mut after, 1, 1, &mut 0, &minimum);
            birth.apply(&mut after, 1, 1, &mut 0, &minimum);
        }
        assert_eq!((after[&members], after[&biomass], after[&soil]), (3, 15, 1));
    }
}
