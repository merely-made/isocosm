// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmograph::{
    Command, Execution, Founding, Session,
    asserted::{Assertion, Character, CharacterTable, HistoryLine, Place, Route},
    schema::{Id, Route as SiteRoute, Tick},
    simulation::{Genesis, Outcome, Receipt},
};
use isocosm_native::native::Replay;
use std::collections::BTreeSet;

fn genesis(sites: u32) -> Genesis {
    Founding {
        seed: 1, sites, population: 24, cohort_size: 8, lineages: 2,
        ..Default::default()
    }.generate().unwrap()
}

fn route(to: Id, travel: Tick) -> SiteRoute {
    SiteRoute { to, travel, transmission: 1_000_000, border: None, authored: None }
}

fn empty_routes(g: &mut Genesis) {
    for site in g.sites.values_mut() {
        site.routes.clear();
    }
}

#[test]
fn native_geography_borrows_authored_labels_and_deduplicates_route_fill() {
    let places = [Place {
        key: "ford".into(), name: "Old Ford".into(), map: Some("river.map".into()),
        position: Some((-7, 12)), tags: BTreeSet::from(["flood".into()]),
    }, Place {
        key: "shrine".into(), name: "Shrine".into(), position: Some((5, -2)),
        ..Default::default()
    }];
    let road = Route {
        key: "river-road".into(), from: "ford".into(), to: "shrine".into(),
        weight: 17, tags: BTreeSet::from(["bridge".into()]),
    };
    let inputs = [
        Assertion::Place(places[0].clone()), Assertion::Place(places[1].clone()),
        Assertion::Route(road.clone()),
        Assertion::Character(Character {
            key: "traveler".into(), name: "Traveler".into(), place: Some("ford".into()),
            table: Some(CharacterTable { cell: [2, 1, 8], owner: Some(81) }),
            ..Default::default()
        }),
    ];
    let replay = Replay::from_bench_assertions(genesis(2), &inputs, Execution::Grouped).unwrap();
    let before = replay.save();
    for mode in [Execution::Grouped, Execution::Individuals] {
        let loaded = Replay::load(before.clone(), mode).unwrap();
        let read = loaded.reading();
        let labels = read.authored_places();
        assert_eq!(labels.keys().copied().collect::<Vec<_>>(), vec!["ford", "shrine"]);
        assert_eq!(labels["ford"].1, &places[0]);
        assert_eq!(labels["shrine"].1, &places[1]);
        let (ford, site) = read.authored_site("ford").unwrap();
        let (shrine, _) = read.authored_site("shrine").unwrap();
        assert_eq!(labels["ford"].0, ford);
        assert!(std::ptr::eq(labels["ford"].1, site.authored.as_ref().unwrap()));
        let routes = read.authored_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes["river-road"], &road);
        assert!(read.state().sites.values().flat_map(|s| &s.routes).any(|r| {
            r.authored.as_ref().is_some_and(|a| std::ptr::eq(a, routes["river-road"]))
        }));
        assert_eq!(read.travel_path(ford, shrine), Some((vec![ford, shrine], 17)));
        assert_eq!(read.travel_path(shrine, ford), Some((vec![shrine, ford], 17)));
        let (_, traveler) = read.authored_entity("traveler").unwrap();
        assert_eq!(traveler.place, cosmograph::directing::PLACELESS);
        assert!(traveler.body.is_none());
        assert_eq!(loaded.save(), before);
    }
    assert_eq!(replay.save(), before);
}

#[test]
fn native_path_uses_weighted_generated_routes_without_narrowing_ticks() {
    let mut g = genesis(4);
    empty_routes(&mut g);
    let wide = u64::from(u32::MAX) + 10;
    g.sites.get_mut(&0).unwrap().routes = vec![route(1, 2), route(2, wide + 100)];
    g.sites.get_mut(&1).unwrap().routes = vec![route(2, wide)];
    g.sites.get_mut(&2).unwrap().routes = vec![route(3, 1)];
    let replay = Replay::from_bench_assertions(g, &[], Execution::Grouped).unwrap();
    let before = replay.save();
    let read = replay.reading();
    assert_eq!(read.travel_path(0, 3), Some((vec![0, 1, 2, 3], wide + 3)));
    assert!(read.authored_places().is_empty());
    assert!(read.authored_routes().is_empty());
    assert_eq!(replay.save(), before);
}

#[test]
fn native_path_keeps_one_way_edges_and_refuses_unknown_sites() {
    let mut g = genesis(3);
    empty_routes(&mut g);
    g.sites.get_mut(&0).unwrap().routes = vec![route(1, 5)];
    let replay = Replay::from_bench_assertions(g, &[], Execution::Individuals).unwrap();
    let before = replay.save();
    let read = replay.reading();
    assert_eq!(read.travel_path(1, 0), None);
    assert_eq!(read.travel_path(0, 1), Some((vec![0, 1], 5)));
    assert_eq!(read.travel_path(0, 2), None);
    assert_eq!(read.travel_path(0, 0), Some((vec![0], 0)));
    assert_eq!(read.travel_path(u64::MAX, 0), None);
    assert_eq!(read.travel_path(0, u64::MAX), None);
    assert_eq!(replay.save(), before);
}

#[test]
fn native_path_refuses_overflow_without_losing_exact_maximum_cost() {
    let mut g = genesis(4);
    empty_routes(&mut g);
    g.sites.get_mut(&0).unwrap().routes = vec![route(1, u64::MAX - 5), route(3, u64::MAX)];
    g.sites.get_mut(&1).unwrap().routes = vec![route(2, 10)];
    let replay = Replay::from_bench_assertions(g, &[], Execution::Grouped).unwrap();
    let before = replay.save();
    let read = replay.reading();
    assert_eq!(read.travel_path(0, 2), None);
    assert_eq!(read.travel_path(0, 3), Some((vec![0, 3], u64::MAX)));
    assert_eq!(replay.save(), before);
}

#[test]
fn native_knowing_keeps_field_answers_refusals_and_replayed_memory() {
    let mut g = genesis(3);
    empty_routes(&mut g);
    g.sites.get_mut(&0).unwrap().routes = vec![route(1, 4)];
    for (entity, site) in [(3, 0), (4, 1), (5, 2)] {
        g.population.lift(entity).unwrap().place = site;
    }
    let mut session = cosmograph_host::founding::found(
        g,
        Execution::Grouped,
        cosmograph_host::founding::Purpose::Bench,
        None,
    )
    .unwrap()
    .into_parts()
    .unwrap()
    .0;
    let result = session.command(Command::Act {
        actor: 3, target: None, process: "sim:remember".into(), cause: None,
    }).unwrap();
    let receipt: Receipt = serde_json::from_str(&result).unwrap();
    assert_eq!(receipt.outcome, Outcome::Accepted);
    let initial = Replay::load(session.save(), Execution::Individuals).unwrap();
    let before = initial.save();
    let read = initial.reading();
    assert_eq!(read.knows(5, &receipt.id), Ok(false));
    assert_eq!(read.knows(4, &receipt.id), Ok(false));
    assert_eq!(read.knows(u64::MAX, &receipt.id), Err("unknown entity".into()));
    assert_eq!(read.knows(4, "missing-event"), Err("unknown event".into()));
    assert_eq!(initial.save(), before);
    session.advance(4).unwrap();
    assert_eq!(session.command(Command::Learn { entity: 4, event: receipt.id.clone() }).unwrap(), "true");
    session.advance(150).unwrap();
    session.command(Command::Assert(Assertion::Place(Place {
        key: "authored".into(), ..Default::default()
    }))).unwrap();
    session.command(Command::Assert(Assertion::History(HistoryLine {
        key: "table-history".into(), participants: vec![], place: Some("authored".into()),
        ..Default::default()
    }))).unwrap();
    let saved = session.save();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Replay::load(saved.clone(), mode).unwrap();
        let read = loaded.reading();
        assert_eq!(read.knows(4, &receipt.id), Ok(true));
        assert_eq!(read.knows(5, &receipt.id), Ok(false));
        assert_eq!(read.knows(4, "line:table-history"), Ok(false));
        assert_eq!(read.knows(4, &receipt.id), session.sim.knows(4, &receipt.id));
        let seen = read.geography(&BTreeSet::from([4])).unwrap();
        assert_eq!(seen.places.keys().copied().collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(seen.residents[&4], Some(1));
        assert_eq!(seen.routes.len(), 1);
        assert_eq!((seen.routes[0].0, seen.routes[0].1.to, seen.routes[0].1.travel), (0, 1, 4));
        let unseen = read.geography(&BTreeSet::from([5])).unwrap();
        assert_eq!(unseen.places.keys().copied().collect::<Vec<_>>(), vec![2]);
        assert!(unseen.routes.is_empty());
        assert!(read.geography(&BTreeSet::from([u64::MAX])).is_err());
        assert!(read.geography(&BTreeSet::new()).unwrap().places.is_empty());
        assert_eq!(loaded.save(), saved);
    }
}
