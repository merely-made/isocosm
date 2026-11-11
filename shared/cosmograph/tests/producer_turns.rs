// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Producers that share ground take turns (a lineage's index must never
//! decide who eats first). `ecology:produce-{i}` of a founding with more than
//! one producer names one turn group; a lone producer names none, so its world
//! is founded as it was. The controls run the seed-9 played A1 world, whose
//! producers (lineages 0 and 3) share a site, without dormancy, and count the
//! soil each captures there: the fast one over the first 20 ticks (debug runs are slow), the
//! ignored one (release) to tick 200, when lineage 3 used to be starved out.

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::Rates,
    flows::MadeBy,
    simulation::Genesis,
};

fn a1(seed: u64, lineages: u32) -> Founding {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    Founding {
        seed,
        population: 256,
        sites: grid.width * grid.height,
        lineages,
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies {
            hazard: cosmograph::harm::hazard::RATE,
            wound: cosmograph::harm::hazard::CELLS,
            ..Bodies::default()
        }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Founding::default()
    }
}

fn turns(g: &Genesis) -> Vec<(String, Option<String>)> {
    g.rules.processes.values()
        .filter(|p| p.id.starts_with("ecology:produce-"))
        .map(|p| (p.id.clone(), p.turn.clone()))
        .collect()
}

#[test]
fn two_producers_name_one_turn_group_on_their_growth() {
    let g = a1(9, 4).generate().unwrap();
    let named = turns(&g);
    assert_eq!(named.len(), 2, "{named:?}");
    assert!(named.iter().all(|(_, t)| t.as_deref() == Some("turn:world:soil")), "{named:?}");
    for p in g.rules.processes.values().filter(|p| !p.id.starts_with("ecology:produce-")) {
        assert_eq!(p.turn, None, "{} takes no turn", p.id);
    }
}

#[test]
fn a_lone_producer_names_no_turn_and_is_founded_as_before() {
    let g = a1(9, 2).generate().unwrap();
    assert_eq!(turns(&g).len(), 1);
    assert!(g.rules.processes.values().all(|p| p.turn.is_none()));
    // Nothing is saved for it, so the genesis serializes as it did.
    assert!(!serde_json::to_string(&g.rules.processes).unwrap().contains("\"turn\""));
}

/// Soil lineages 0 and 3 capture over `ticks`, with the turn group named or not.
fn captured(ticks: u64, turned: bool) -> [u64; 2] {
    let mut g = a1(9, 4).generate().unwrap();
    if !turned {
        g.rules.processes.values_mut().for_each(|p| p.turn = None);
    }
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut soil = [0u64; 2];
    for _ in 0..ticks {
        for f in &s.advance_tick_with_flows().unwrap().flows {
            let MadeBy::Process(p) = &f.made_by else { continue };
            for (j, i) in [(0, 0), (1, 3)] {
                if *p == format!("ecology:produce-{i}") && f.from.1 == "world:soil" {
                    soil[j] += f.amount * f.count;
                }
            }
        }
    }
    soil
}

/// Neither producer takes less than a third of the other's soil, where the
/// first by index used to take it all it could: lineage 3's share must also
/// beat what it had with no turns.
fn fair(ticks: u64, against_none: bool) {
    let [l0, l3] = captured(ticks, true);
    eprintln!("soil over {ticks} ticks: L0 {l0}, L3 {l3}");
    assert!(l0 > 0 && l3 > 0);
    assert!(l0 * 3 >= l3 && l3 * 3 >= l0, "L0 {l0}, L3 {l3}");
    if against_none {
        let [_, l3_before] = captured(ticks, false);
        eprintln!("L3 without turns {l3_before}");
        assert!(l3 > l3_before, "L3 {l3} against {l3_before} without turns");
    }
}

#[test]
fn neither_producer_is_privileged_over_the_first_twenty_ticks() {
    fair(20, false);
}

#[test]
#[ignore = "release: 200 ticks of the played A1 world"]
fn neither_producer_is_privileged_to_tick_200() {
    fair(200, true);
}
