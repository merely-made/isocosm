// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Execution, Founding, Session, ecology::Rates, population::Population};

const PRODUCER: &str = "lineage:0";
const GRAZER: &str = "lineage:1";

/// Three world-time sites in a line, 0-1-2: producers everywhere, five
/// grazers resident at 2 and one travelling from 2 to 1.
fn line() -> Session {
    let has = |g: &crate::simulation::Genesis, a: Id, b: Id| g.sites[&a].routes.iter().any(|r| r.to == b);
    // The first seed whose drawn routes hold the line both ways.
    let mut g = (1..64)
        .map(|seed| {
            Founding {
                seed,
                sites: 3,
                population: 60,
                cohort_size: 1,
                lineages: 3,
                ecology: true,
                ecology_rates: Rates::WorldTime,
                ..Founding::default()
            }
            .generate()
            .unwrap()
        })
        .find(|g| [(0, 1), (1, 0), (1, 2), (2, 1)].iter().all(|&(a, b)| has(g, a, b)))
        .expect("a seed with the line");
    for (&id, site) in g.sites.iter_mut() {
        site.routes.retain(|r| r.to.abs_diff(id) == 1);
    }
    let proto = |lineage: &str| {
        let group = g.population.groups.values().find(|c| c.entity.lineage == lineage).unwrap();
        group.entity.as_ref().clone()
    };
    let (producer, grazer) = (proto(PRODUCER), proto(GRAZER));
    let mut pop = Population::default();
    for site in 0..3 {
        pop.insert(Entity { place: site, ..producer.clone() }, 4).unwrap();
    }
    pop.insert(Entity { place: 2, ..grazer.clone() }, 5).unwrap();
    let traveller = pop.insert(Entity { place: 2, ..grazer }, 1).unwrap();
    g.population = pop;
    let started = g.started;
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    // Founders arrive at the start, so the traveller sets out afterwards.
    let journey = Journey { from: 2, to: 1, departed: started, arrives: started + 5, border: None };
    s.sim.state.population.lift(traveller).unwrap().journey = Some(journey);
    s
}

#[test]
fn scopes_name_their_sites_and_count_residents_apart_from_travellers() {
    let s = line();
    let patch = reading(&s.sim, &Scope::Patch(1)).unwrap();
    assert_eq!(patch.sites, BTreeSet::from([1]));
    assert_eq!(patch.living, BTreeMap::from([(PRODUCER.into(), 4)]));
    assert_eq!(patch.arriving, BTreeMap::from([(GRAZER.into(), 1)]));
    assert_eq!(patch.levels["life:producer"], 4);
    assert_eq!(patch.levels["life:consumer"], 0);
    assert!(patch.biomass["life:producer"] > 0);
    let footprint = reading(&s.sim, &Scope::Footprint(BTreeSet::from([0, 1]))).unwrap();
    assert_eq!(footprint.sites, BTreeSet::from([0, 1]));
    assert_eq!(footprint.living[PRODUCER], 8);
    assert!(!footprint.holds(GRAZER), "a traveller is not yet a resident");
    let world = reading(&s.sim, &Scope::World).unwrap();
    assert_eq!(world.sites, BTreeSet::from([0, 1, 2]));
    assert_eq!((world.living[PRODUCER], world.living[GRAZER]), (12, 5));
    assert_eq!(world.biomass["life:producer"], footprint.biomass["life:producer"] * 3 / 2);
}

#[test]
fn absence_is_reported_at_the_smallest_scope_that_lost_the_lineage() {
    let s = line();
    let wide = BTreeSet::from([1, 2]);
    assert_eq!(absence(&s.sim, GRAZER, 2, &wide).unwrap(), Absence::Present);
    assert_eq!(absence(&s.sim, GRAZER, 1, &wide).unwrap(), Absence::Patch);
    assert_eq!(absence(&s.sim, GRAZER, 0, &BTreeSet::from([0, 1])).unwrap(), Absence::Regional);
    assert_eq!(absence(&s.sim, "lineage:9", 0, &wide.union(&BTreeSet::from([0])).copied().collect()).unwrap(), Absence::Global);
}

#[test]
fn unknown_and_empty_scopes_are_refused() {
    let s = line();
    assert!(reading(&s.sim, &Scope::Patch(7)).is_err());
    assert!(reading(&s.sim, &Scope::Footprint(BTreeSet::new())).is_err());
    assert!(reading(&s.sim, &Scope::Footprint(BTreeSet::from([0, 7]))).is_err());
    assert!(absence(&s.sim, GRAZER, 0, &BTreeSet::from([1, 2])).is_err(), "a footprint holds its patch");
    assert!(sources(&s.sim, GRAZER, 7).is_err());
}

#[test]
fn a_source_needs_living_residents_a_route_path_and_a_native_means() {
    let mut s = line();
    let far = sources(&s.sim, GRAZER, 0).unwrap();
    assert_eq!(far.len(), 1, "the traveller is no resident source");
    assert_eq!((far[0].site, far[0].members, far[0].hops), (2, 5, Some(2)));
    let travel = s.sim.state().sites[&2].routes.iter().find(|r| r.to == 1).unwrap().travel;
    assert_eq!(far[0].means, vec![Means::Walk { process: "move:2-1/lineage:1".into(), to: 1, travel }]);
    assert!(far[0].available());
    let seeds = sources(&s.sim, PRODUCER, 0).unwrap();
    assert_eq!(seeds.iter().map(|f| (f.site, f.hops)).collect::<Vec<_>>(), vec![(1, Some(1)), (2, Some(2))]);
    assert!(seeds.iter().all(|f| f.available()
        && f.means.iter().all(|m| matches!(m, Means::Scatter { process, strength: 100, .. } if process == "ecology:birth-0"))));
    // Nothing outside the grazers' own site holds them as residents.
    assert!(sources(&s.sim, GRAZER, 2).unwrap().is_empty());
    s.sim.state.sites.get_mut(&1).unwrap().routes.clear();
    let cut = sources(&s.sim, GRAZER, 0).unwrap();
    assert_eq!((cut[0].members, cut[0].hops), (5, None));
    assert!(cut[0].means.is_empty() && !cut[0].available(), "a remote, unconnected population is no source");
}
