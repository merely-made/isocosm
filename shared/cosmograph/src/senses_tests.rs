// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The plant readings a pass memoises are the direct computation, for every
//! plant at a site, and a pass never outlives its pass.

use crate::{
    Execution, Founding, Session,
    bodied::Bodies,
    ecology::{Rates, Web},
    schedule::Frame,
    schema::{Id, Key},
};
use std::collections::BTreeSet;

const READINGS: [&str; 5] =
    ["sense:light", "sense:soil", "sense:crowding", "sense:light-share", "sense:soil-share"];

fn session() -> Session {
    let grid = crate::map::Grid { width: 2, height: 2, ..crate::map::Grid::drawn(9) };
    let genesis = Founding {
        seed: 9, sites: 4, population: 240, cohort_size: 1, lineages: 10, ecology: true,
        web: Web::Diets, ecology_rates: Rates::WorldTime, experimental_food_contract: true,
        bodies: Some(Bodies { clutch: [1, 1], sensing: true, grazing: true,
            hazard: [0, 0], wound: [0, 0], ..Bodies::default() }),
        map: Some(crate::map::Layout::Grid(grid)), ..Founding::default()
    }.generate().unwrap();
    let mut s = Session::new(genesis, Execution::Individuals).unwrap();
    s.advance(400).unwrap();
    s
}

fn plants(s: &Session) -> BTreeSet<Key> {
    s.sim.genesis.lineages.iter().filter(|(_, l)| l.kingdom == "kingdom:flora")
        .map(|(k, _)| k.clone()).collect()
}

/// Every live plant's readings, each asked twice, against the direct ones.
fn agree(s: &mut Session) -> Vec<(Id, u64, u64, u64)> {
    let among = plants(s);
    let ids: Vec<Id> = s.sim.state.population.groups.iter()
        .filter(|(_, g)| g.entity.alive && among.contains(&g.entity.lineage)).map(|(&id, _)| id).collect();
    assert!(!ids.is_empty(), "the world has plants");
    s.sim.frame = Some(Frame::starting(s.sim.state.notes.len()));
    let mut seen = vec![];
    for id in ids {
        let site = s.sim.state.population.get(id).and_then(|b| b.resident_site());
        let Some(site) = site else { continue };
        let Some(direct) = s.sim.plant_readings_direct(site, &among) else { continue };
        for _ in 0..2 {
            for (reading, want) in READINGS.iter().zip([
                direct[0], direct[1], direct[2], direct[0] / direct[2], direct[1] / direct[2],
            ]) {
                assert_eq!(s.sim.sensed(id, reading), Some(want), "{reading} of {id} at {site}");
            }
        }
        seen.push((id, direct[0], direct[1], direct[2]));
    }
    s.sim.frame = None;
    seen
}

#[test]
fn memoised_plant_readings_equal_the_direct_computation() {
    let mut s = session();
    let seen = agree(&mut s);
    assert!(seen.iter().any(|r| r.1 > 0 || r.2 > 0), "{seen:?}");
}

#[test]
fn a_later_pass_reads_the_world_as_it_then_began() {
    let mut s = session();
    let before = agree(&mut s);
    // Soil is spent at every site between two passes of one tick.
    for site in s.sim.state.sites.values_mut() {
        if let Some(soil) = site.accounts.get_mut("world:soil") {
            *soil /= 2;
        }
    }
    let after = agree(&mut s);
    assert!(before.iter().zip(&after).any(|(b, a)| b.2 != a.2), "soil moved the readings");
}

#[test]
fn a_clone_of_a_frame_starts_with_no_memo() {
    let mut s = session();
    let among = plants(&s);
    let (&id, _) = s.sim.state.population.groups.iter()
        .filter(|(_, g)| g.entity.alive && among.contains(&g.entity.lineage))
        .filter_map(|(id, g)| Some((id, g.entity.resident_site()?))).next().unwrap();
    s.sim.frame = Some(Frame::starting(0));
    let first = s.sim.sensed(id, "sense:soil").unwrap();
    let cloned = s.sim.frame.clone();
    for place in s.sim.state.sites.values_mut() {
        place.accounts.insert("world:soil".into(), first + 7);
    }
    s.sim.frame = cloned;
    assert_eq!(s.sim.sensed(id, "sense:soil"), Some(first + 7));
}
