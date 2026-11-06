// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Observer state for the world-time experiment. Fixed geographic components
//! permit month-to-month comparisons without changing the sim's biomass regions.

use cosmograph::{
    Session,
    rules::AccountKind,
    schema::{Id, Key},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const LEVELS: [&str; 3] = ["life:producer", "life:consumer", "life:decomposer"];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub members: u64,
    pub biomass: u128,
    pub bodied_members: u64,
}

impl Totals {
    fn add(&mut self, other: &Self) {
        self.members += other.members;
        self.biomass += other.biomass;
        self.bodied_members += other.bodied_members;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Region {
    pub sites: BTreeSet<Id>,
    pub lineages: BTreeMap<Key, Totals>,
    pub levels: BTreeMap<Key, Totals>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Sample {
    pub elapsed_ticks: u64,
    pub lineages: BTreeMap<Key, Totals>,
    pub rooted_tissue: BTreeMap<Key, BTreeMap<Key, super::tissue::Tissue>>,
    pub resident_sites: BTreeMap<Id, BTreeMap<Key, Totals>>,
    pub travelling: BTreeMap<Key, Totals>,
    /// Members in transit remain associated with their recorded source site.
    pub fixed_geographic_regions: Vec<Region>,
    pub births: usize,
    pub starvation_deaths: usize,
    pub hazard_deaths: usize,
    pub soil: BTreeMap<Id, u64>,
    pub detritus: BTreeMap<Id, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OccupancyChange {
    pub elapsed_ticks: u64,
    pub site: Id,
    pub lineage: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RegionalLoss {
    pub elapsed_ticks: u64,
    pub sites: BTreeSet<Id>,
    pub level: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LineageVerdict {
    pub lineage: Key,
    pub alive: bool,
    pub within_fourfold: bool,
    pub no_third_loss: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeedingDefinition {
    pub target: Option<cosmograph::rules::Target>,
    pub port: Option<cosmograph::rules::PortFeeding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub version: u32,
    pub seed: u64,
    pub founding: Option<cosmograph::Founding>,
    pub genesis_digest: Key,
    pub rules_digest: Key,
    pub history_digest: Key,
    pub initial_state_hash: u64,
    pub native_version: u32,
    pub tick_microseconds: u64,
    pub execution: cosmograph::Execution,
    pub started_tick: u64,
    pub month_ticks: u64,
    pub body_fixture: &'static str,
    pub root_store_qualified: bool,
    pub root_store_admission: super::admission::Admission,
    pub feeding_definitions: BTreeMap<Key, FeedingDefinition>,
    pub region_basis: &'static str,
    pub samples: Vec<Sample>,
    pub observed_local_losses: Vec<OccupancyChange>,
    pub observed_recolonizations: Vec<OccupancyChange>,
    pub observed_regional_losses: Vec<RegionalLoss>,
    pub first_year: Option<Vec<LineageVerdict>>,
    pub second_year_extinctions: BTreeSet<Key>,
    pub outcome: Outcome,
    #[serde(skip)]
    footprints: Vec<BTreeSet<Id>>,
    #[serde(skip)]
    expected: BTreeSet<Key>,
    #[serde(skip)]
    ever_resident: BTreeSet<(Id, Key)>,
}

impl Report {
    pub fn new(s: &Session) -> Self {
        let genesis = s.sim.genesis();
        let month_ticks = cosmograph::rules::year_ticks(genesis.rules.tick_microseconds()) / 12;
        assert!(
            month_ticks > 0,
            "world-time observation needs at least twelve ticks per world year"
        );
        let root_store_admission = super::admission::Admission::inspect(genesis);
        let mut out = Self {
            version: 2,
            seed: genesis.seed,
            founding: genesis.founding.clone(),
            genesis_digest: cosmograph::digest(genesis),
            rules_digest: cosmograph::digest(&genesis.rules),
            history_digest: cosmograph::digest(&s.entries),
            initial_state_hash: s.sim.state_hash(),
            native_version: genesis.version,
            tick_microseconds: genesis.rules.tick_microseconds(),
            execution: s.sim.mode(),
            started_tick: s.sim.state().tick,
            month_ticks,
            body_fixture: "native generated bodies; root-store structural admission and ecological trajectory are separate verdicts",
            root_store_qualified: root_store_admission.qualified,
            root_store_admission,
            feeding_definitions: genesis
                .rules
                .processes
                .iter()
                .filter(|(id, _)| id.starts_with("ecology:feed-"))
                .map(|(id, p)| {
                    (
                        id.clone(),
                        FeedingDefinition {
                            target: p.target.clone(),
                            port: p.port_feeding.clone(),
                        },
                    )
                })
                .collect(),
            region_basis: "fixed weakly connected founding-site components; levels are configured traits, not measured diets",
            samples: vec![],
            observed_local_losses: vec![],
            observed_recolonizations: vec![],
            observed_regional_losses: vec![],
            first_year: None,
            second_year_extinctions: BTreeSet::new(),
            outcome: Outcome::Inconclusive,
            footprints: footprints(s),
            // The original year bar compares the lineages living at founding.
            expected: s
                .sim
                .state()
                .population
                .groups
                .values()
                .filter(|g| g.entity.alive && g.entity.kingdom != "kingdom:world")
                .map(|g| g.entity.lineage.clone())
                .collect(),
            ever_resident: BTreeSet::new(),
        };
        out.observe(s);
        out
    }

    pub fn observe(&mut self, s: &Session) {
        let sample = sample(s, self.started_tick, &self.footprints);
        let count = |values: Option<&BTreeMap<Key, Totals>>, lineage: &Key| {
            values.and_then(|v| v.get(lineage)).map_or(0, |t| t.members)
        };
        if let Some(before) = self.samples.last() {
            for (was, now) in before
                .fixed_geographic_regions
                .iter()
                .zip(&sample.fixed_geographic_regions)
            {
                for (level, totals) in &now.levels {
                    if was.levels[level].members > 0 && totals.members == 0 {
                        self.observed_regional_losses.push(RegionalLoss {
                            elapsed_ticks: sample.elapsed_ticks,
                            sites: now.sites.clone(),
                            level: level.clone(),
                        });
                    }
                }
            }
            for (&site, now) in &sample.resident_sites {
                let was = before.resident_sites.get(&site);
                let names: BTreeSet<_> = self
                    .expected
                    .iter()
                    .chain(now.keys())
                    .chain(was.into_iter().flat_map(|by| by.keys()))
                    .collect();
                for lineage in names {
                    let (old, new) = (count(was, lineage), count(Some(now), lineage));
                    let change = OccupancyChange {
                        elapsed_ticks: sample.elapsed_ticks,
                        site,
                        lineage: lineage.clone(),
                    };
                    if old > 0 && new == 0 {
                        self.observed_local_losses.push(change);
                    } else if old == 0
                        && new > 0
                        && self.ever_resident.contains(&(site, lineage.clone()))
                    {
                        self.observed_recolonizations.push(change);
                    }
                }
            }
        }
        self.ever_resident
            .extend(sample.resident_sites.iter().flat_map(|(&site, by)| {
                by.iter()
                    .filter(|(_, t)| t.members > 0)
                    .map(move |(lineage, _)| (site, lineage.clone()))
            }));
        self.samples.push(sample);
        self.evaluate();
    }

    fn at_month(&self, month: u64) -> Option<&Sample> {
        self.samples
            .iter()
            .find(|s| s.elapsed_ticks == self.month_ticks * month)
    }

    fn evaluate(&mut self) {
        let monthly: Option<Vec<_>> = (0..=12).map(|m| self.at_month(m)).collect();
        self.first_year = monthly.map(|months| {
            self.expected
                .iter()
                .map(|name| {
                    let series: Vec<u64> = months
                        .iter()
                        .map(|m| m.lineages.get(name).map_or(0, |t| t.members))
                        .collect();
                    let window = &series[3..=12];
                    let (lo, hi) = (*window.iter().min().unwrap(), *window.iter().max().unwrap());
                    LineageVerdict {
                        lineage: name.clone(),
                        alive: series[12] > 0,
                        within_fourfold: lo > 0 && u128::from(hi) <= u128::from(lo) * 4,
                        no_third_loss: series
                            .windows(2)
                            .skip(2)
                            .all(|w| u128::from(w[1]) * 3 >= u128::from(w[0]) * 2),
                    }
                })
                .collect()
        });
        self.second_year_extinctions = self
            .samples
            .iter()
            .filter(|s| {
                s.elapsed_ticks > 12 * self.month_ticks && s.elapsed_ticks <= 24 * self.month_ticks
            })
            .flat_map(|s| {
                self.expected
                    .iter()
                    .filter(|name| s.lineages.get(*name).is_none_or(|t| t.members == 0))
                    .cloned()
            })
            .collect();
        let complete = (0..=24).all(|m| self.at_month(m).is_some());
        self.outcome = if !complete {
            Outcome::Inconclusive
        } else if self.first_year.as_ref().is_some_and(|v| {
            !v.is_empty()
                && v.iter()
                    .all(|v| v.alive && v.within_fourfold && v.no_third_loss)
        }) && self.second_year_extinctions.is_empty()
        {
            Outcome::Passed
        } else {
            Outcome::Failed
        };
    }
}

fn sample(s: &Session, started: u64, footprints: &[BTreeSet<Id>]) -> Sample {
    let state = s.sim.state();
    let rules = &s.sim.genesis().rules;
    let mut out = Sample {
        elapsed_ticks: state.tick - started,
        lineages: BTreeMap::new(),
        rooted_tissue: super::tissue::read(s),
        resident_sites: state
            .sites
            .keys()
            .map(|&id| (id, BTreeMap::new()))
            .collect(),
        travelling: BTreeMap::new(),
        fixed_geographic_regions: footprints
            .iter()
            .map(|sites| Region {
                sites: sites.clone(),
                lineages: BTreeMap::new(),
                levels: LEVELS
                    .into_iter()
                    .map(|key| (key.into(), Totals::default()))
                    .collect(),
            })
            .collect(),
        births: events(s, "ecology:birth"),
        starvation_deaths: events(s, "ecology:death"),
        hazard_deaths: events(s, "ecology:hazard"),
        soil: state
            .sites
            .iter()
            .map(|(&id, site)| (id, site.accounts.get("world:soil").copied().unwrap_or(0)))
            .collect(),
        detritus: state
            .sites
            .iter()
            .map(|(&id, site)| {
                (
                    id,
                    site.accounts.get("world:detritus").copied().unwrap_or(0),
                )
            })
            .collect(),
    };
    for group in state.population.groups.values() {
        let e = &group.entity;
        if !e.alive || e.kingdom == "kingdom:world" {
            continue;
        }
        let matter =
            |key: &Key| matches!(rules.accounts.get(key), Some(AccountKind::Matter { .. }));
        let representative: u128 = cosmograph::anatomy::books(e)
            .iter()
            .filter(|(k, _)| matter(k))
            .map(|(_, amount)| u128::from(*amount))
            .sum();
        let pool: i128 = group
            .pool
            .iter()
            .filter(|(key, _)| matter(key))
            .map(|(_, n)| *n)
            .sum();
        let totals = Totals {
            members: group.count,
            biomass: (representative * u128::from(group.count)).saturating_add_signed(pool),
            bodied_members: if e.body.is_some() { group.count } else { 0 },
        };
        out.lineages
            .entry(e.lineage.clone())
            .or_default()
            .add(&totals);
        if let Some(site) = e.resident_site() {
            out.resident_sites
                .entry(site)
                .or_default()
                .entry(e.lineage.clone())
                .or_default()
                .add(&totals);
        } else {
            out.travelling
                .entry(e.lineage.clone())
                .or_default()
                .add(&totals);
        }
        if let Some(region) = out
            .fixed_geographic_regions
            .iter_mut()
            .find(|r| r.sites.contains(&e.place))
        {
            region
                .lineages
                .entry(e.lineage.clone())
                .or_default()
                .add(&totals);
            for (level, held) in &mut region.levels {
                if e.traits.contains(level) {
                    held.add(&totals);
                }
            }
        }
    }
    out
}

fn events(s: &Session, prefix: &str) -> usize {
    s.sim
        .state()
        .events
        .values()
        .filter(|e| e.process.starts_with(prefix))
        .count()
}

fn footprints(s: &Session) -> Vec<BTreeSet<Id>> {
    let sites = &s.sim.genesis().sites;
    let mut neighbours: BTreeMap<Id, BTreeSet<Id>> =
        sites.keys().map(|&id| (id, BTreeSet::new())).collect();
    for (&id, site) in sites {
        for route in &site.routes {
            neighbours.entry(id).or_default().insert(route.to);
            neighbours.entry(route.to).or_default().insert(id);
        }
    }
    let mut unseen: BTreeSet<Id> = sites.keys().copied().collect();
    let mut out = vec![];
    while let Some(&first) = unseen.first() {
        let mut region = BTreeSet::new();
        let mut queue = VecDeque::from([first]);
        while let Some(site) = queue.pop_front() {
            if !unseen.remove(&site) {
                continue;
            }
            region.insert(site);
            queue.extend(neighbours[&site].iter().copied());
        }
        out.push(region);
    }
    out
}
