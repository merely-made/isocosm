// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The population model's bands (the execution plan's E4; D24, D27 to D29).
//! Outside the near tier, a site and lineage's members fold into bands by
//! age band and the needs they feel: one representative body with a count,
//! and a pool of what its members hold beyond count times the representative.
//! Nothing the game points at folds: attention's roots, relations, nudged
//! or released members, travellers, deliberative members, authored or
//! placed characters, members holding a pose.

use crate::{
    anatomy::books,
    meaning::{Named, Scene},
    population::Pool,
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A world's banding rule (D29): the width of an age band, in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Banding {
    pub age_band: Tick,
}

/// What puts members in one band.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Band<'a> {
    site: Id,
    lineage: &'a str,
    age: Tick,
    born: Option<Tick>,
    felt: Vec<bool>,
    form: Option<Form<'a>>,
    exact: Option<&'a Entity>,
}

/// A view of one anatomical and trait combination, excluding pooled matter.
struct Form<'a>(&'a Entity);

fn part_form(p: &Part) -> impl Ord + '_ {
    (&p.traits, &p.functions, &p.cells, &p.tracts, &p.lost, &p.losses, &p.port, &p.graft_origin)
}

impl Eq for Form<'_> {}

impl PartialEq for Form<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for Form<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Form<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let (a, b) = (self.0, other.0);
        (&a.soma, &a.absent, &a.traits, &a.body, &a.varied, &a.systems, a.drawn, a.disposition)
            .cmp(&(&b.soma, &b.absent, &b.traits, &b.body, &b.varied, &b.systems, b.drawn, b.disposition))
            .then_with(|| {
                a.parts.iter().map(|(id, p)| (id, part_form(p)))
                    .cmp(b.parts.iter().map(|(id, p)| (id, part_form(p))))
            })
    }
}

/// What the population model's bands hold in their pools (E4, D28): each
/// pool's matter, signed, summed over the world. A pool is below nought
/// where its representative holds more than its members' mean.
pub(crate) fn pooled(state: &crate::simulation::State, rules: &crate::rules::Rules) -> i128 {
    let mut gained = Ledger::new();
    let mut lost = Ledger::new();
    for g in state.population.groups.values() {
        for (k, v) in &g.pool {
            let side = if *v > 0 { &mut gained } else { &mut lost };
            *side.entry(k.clone()).or_default() += v.unsigned_abs() as u64;
        }
    }
    let mass = |a: &Ledger| crate::spread::ledger_mass(a, rules, &state.spreads);
    mass(&gained) as i128 - mass(&lost) as i128
}

impl Simulation {
    /// Folds the far members of each band into one cohort, conserving every
    /// account through its pool.
    pub(crate) fn fold_far(&mut self, counted_births: bool) {
        if let Some(banding) = self.genesis.rules.banding
            && (!self.genesis.rules.cohort_births || counted_births)
        {
            self.fold_bands(banding);
        }
    }

    /// Identities that cannot be reassigned by population compaction.
    pub(crate) fn band_identities(&self, preserve_history: bool) -> BTreeSet<Id> {
        let s = &self.state;
        let mut pointed = if preserve_history { self.kept() } else { s.roots.clone() };
        pointed.extend(s.relations.iter().flat_map(|r| [r.subject, r.object]));
        pointed.extend(s.released.keys());
        pointed.extend(s.nudges.iter().map(|n| n.critter));
        if preserve_history {
            for axis in s.record.axes() {
                if let Some(mark) = s.record.standing(axis) {
                    pointed.extend(&mark.holders);
                }
            }
            for event in s.events.values().filter(|event| event.legend || event.authored.is_some()) {
                pointed.insert(event.subject);
                pointed.extend(event.object);
            }
        }
        pointed
    }

    /// The same under `banding`.
    pub(crate) fn fold_bands(&mut self, banding: Banding) {
        let width = banding.age_band.max(1);
        let s = &self.state;
        let exact = self.genesis.rules.cohort_births;
        let varied = self.genesis.rules.inherited_variation != 0;
        let pointed = self.band_identities(varied || exact);
        let mut bands: BTreeMap<Band<'_>, Vec<Id>> = BTreeMap::new();
        for (&first, g) in &s.population.groups {
            let e = &g.entity;
            let held = e.journey.is_some() || e.authored.is_some() || e.patch.is_some();
            if !e.alive || held || e.method == Method::Deliberative
                || ((varied || exact) && e.method == Method::Inert)
            {
                continue;
            }
            let posed = s.spatial_poses.range(first..first + g.count).next().is_some();
            if posed || s.attention.foreground(first, g.count) || pointed.range(first..first + g.count).next().is_some() {
                continue;
            }
            let Some(site) = e.resident_site() else {
                continue;
            };
            let age = s.tick.saturating_sub(e.born) / width;
            let felt = if exact { vec![] } else { self.felt(e, site) };
            let form = (varied && !exact).then_some(Form(e));
            bands.entry(Band {
                site, lineage: &e.lineage, age, born: varied.then_some(e.born), felt, form,
                exact: exact.then_some(e),
            }).or_default().push(first);
        }
        let batches: Vec<_> = bands.into_values().filter(|f| f.len() > 1).collect();
        for firsts in batches {
            self.fold(&firsts);
        }
    }

    /// Which of the mind's needs a body feels, as mood reads them.
    fn felt(&self, e: &Entity, site: Id) -> Vec<bool> {
        let rules = &self.genesis.rules;
        let Some(mind) = &rules.mind else {
            return vec![];
        };
        let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
        let scene = Scene {
            actor: Some(e),
            target: Named::Unnamed,
            part: None,
            site: self.state.sites.get(&site),
            tick: self.state.tick,
            related: &related,
            rules,
            lineages: Some(&self.state.lineages),
        };
        mind.needs
            .iter()
            .map(|n| n.traits.is_subset(&e.traits) && crate::meaning::read(&n.query, &scene).is_ok_and(|(yes, _)| yes))
            .collect()
    }

    /// Folds `firsts` into one band under the first's body.
    fn fold(&mut self, firsts: &[Id]) {
        let population = &self.state.population;
        let representative = population.groups[&firsts[0]].entity.clone();
        let own = books(&representative);
        let mut pool = Pool::new();
        let mut offspring = None;
        let mut count = 0;
        for first in firsts {
            let g = &population.groups[first];
            count += g.count;
            crate::lineage::offspring::merge(&mut offspring, g.offspring.as_ref());
            let theirs = books(&g.entity);
            let keys: BTreeSet<&Key> = theirs.keys().chain(own.keys()).collect();
            for key in keys {
                let gap = i128::from(*theirs.get(key).unwrap_or(&0)) - i128::from(*own.get(key).unwrap_or(&0));
                *pool.entry(key.clone()).or_default() += gap * i128::from(g.count);
            }
            for (key, v) in &g.pool {
                *pool.entry(key.clone()).or_default() += v;
            }
        }
        pool.retain(|_, v| *v != 0);
        let next = population.next_id;
        if next.checked_add(count).is_none() {
            return;
        }
        if let Some(j) = &mut self.journal {
            for first in firsts {
                j.group(*first, self.state.population.groups.get(first));
            }
            j.group(next, None);
        }
        let population = &mut self.state.population;
        for first in firsts {
            population.groups.remove(first);
        }
        let band = population.insert(representative, count).expect("the band's members existed");
        let group = population.groups.get_mut(&band).expect("inserted above");
        group.pool = pool;
        group.offspring = offspring;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Execution, Founding, Session};

    #[test]
    fn far_members_fold_into_bands_conserving_every_account() {
        let g = Founding {
            seed: 3,
            sites: 2,
            population: 60,
            cohort_size: 1,
            lineages: 3,
            ecology: true,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        s.advance(30).unwrap();
        let state = |s: &Session| {
            let p = &s.sim.state().population;
            (s.sim.matter(), p.totals().unwrap(), p.count())
        };
        let before = state(&s);
        let groups = s.sim.state().population.groups.len();
        s.sim.fold_bands(Banding { age_band: 20 });
        assert_eq!(state(&s), before, "folding conserves matter, every account and every member");
        let p = &s.sim.state().population;
        assert!(p.groups.len() < groups, "some members folded");
        let (&first, band) = p.groups.iter().find(|(_, g)| !g.pool.is_empty()).expect("a band whose members differed");
        assert!(band.count > 1);
        s.sim.state.population.lift_shared(first + 1).unwrap();
        assert_eq!(state(&s), before, "lifting a band's member keeps the totals");
    }

    #[test]
    fn a_far_fold_counts_each_form_and_retains_notable_identity() {
        use std::sync::Arc;

        let mut g = Founding {
            seed: 5, ecology: true, inherit: true, sites: 3, population: 36,
            inherited_variation: 50, bodies: Some(crate::bodied::Bodies::default()),
            ..Default::default()
        }.generate().unwrap();
        let e = g.population.groups.values().find(|g| !g.entity.soma.is_empty()).unwrap().entity.clone();
        let mut variant = Arc::clone(&e);
        let anatomical = Arc::make_mut(&mut variant);
        anatomical.soma[0] = anatomical.soma[0].saturating_add(1);
        let expected = anatomical.soma.clone();
        g.population = Default::default();
        let kept = g.population.insert(e.clone(), 1).unwrap();
        g.population.insert(e.clone(), 3000).unwrap();
        g.population.insert(e, 5000).unwrap();
        g.population.insert(variant.clone(), 1000).unwrap();
        g.population.insert(variant, 2000).unwrap();
        let mut s = Session::new(g, Execution::Grouped).unwrap();
        s.sim.state.record.note("mark:example".into(), 1, kept);
        let before = s.sim.state.population.totals().unwrap();
        s.sim.fold_bands(Banding { age_band: 1 });
        let p = &s.sim.state.population;
        assert_eq!(p.groups.len(), 3);
        assert!(p.get(kept).is_some());
        assert_eq!(p.groups.values().find(|g| g.count == 3000).unwrap().entity.soma, expected);
        assert!(p.groups.values().any(|g| g.count == 8000));
        assert_eq!(p.totals().unwrap(), before);
    }
}
