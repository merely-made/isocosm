// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1057: an arrival enters through the border it crossed. A torus two
//! sites high joins each pair of rows by two routes, which stay distinct.

use cosmograph::{Execution, Founding, bodied::Bodies, map::{Grid, Layout}, rules::Effect,
    schema::{Journey, Movement}, simulation::{Genesis, Simulation}, spatial::entering};

fn torus(seed: u64, played: bool) -> Genesis {
    Founding { seed, population: 256, sites: 6, ecology: true, bodies: Some(Bodies::default()),
        played: played.then_some(cosmograph::directing::Played { lineage: 1, region_sites: 2 }),
        map: Some(Layout::Grid(Grid { shape: "shape:torus".into(), width: 3, height: 2, ..Grid::drawn(seed) })),
        ..Default::default() }.generate().unwrap()
}

#[test]
fn a_torus_pair_gives_two_crossings_at_opposite_edges() {
    let g = torus(1, false);
    let t = g.rules.spatial_templates.as_ref().unwrap();
    let (from, to) = (0, 3);
    let pair: Vec<_> = t.crossings.iter().filter(|c| c.from == from && c.to == to).collect();
    assert_eq!(pair.len(), 2, "both routes of the pair cross");
    let [a, b] = [pair[0], pair[1]];
    let (ba, bb) = (a.border.unwrap(), b.border.unwrap());
    assert_eq!((ba.enters + 2) % 4, bb.enters, "the two routes enter by opposite sides");
    // Rows run along z: entering by side 0 lands at low z, by side 2 at high z.
    let z = |c: &cosmograph::spatial::Crossing| c.stand.at[2];
    let (low, high) = if ba.enters == 0 { (a, b) } else { (b, a) };
    assert!(z(low) < z(high), "{} !< {}", z(low), z(high));
    for c in [a, b] {
        let edge = entering(&t.window, c.border.unwrap().enters, 4);
        let near = (c.stand.at[0] - edge[0]).abs().max((c.stand.at[2] - edge[1]).abs());
        assert!(near <= 12, "crossing stand {:?} lies at its entering edge {edge:?}", c.stand.at);
    }
    t.validate(&g).unwrap();
}

#[test]
fn repeated_crossings_are_refused() {
    let g = torus(1, false);
    let mut t = g.rules.spatial_templates.clone().unwrap();
    let first = t.crossings[0].clone();
    t.crossings.push(first);
    assert!(t.validate(&g).unwrap_err().contains("repeat"));
}

#[test]
fn each_parallel_route_has_its_own_act_and_records_its_border() {
    let g = torus(1, true);
    let moves: Vec<_> = g.rules.processes.values().filter_map(|p| p.effects.iter().find_map(|e| match e {
        Effect::Move { destination: 3, via } if p.id.starts_with("move:0-") => Some((p.id.clone(), *via)),
        _ => None,
    })).collect();
    assert_eq!(moves.len(), 2, "{moves:?}");
    assert!(moves.iter().all(|(id, via)| via.is_some_and(|side| id == &format!("move:0-3-{side}"))), "{moves:?}");
    // A lone route keeps its plain name and no side.
    assert!(g.rules.processes.values().any(|p| p.effects.iter()
        .any(|e| matches!(e, Effect::Move { destination: 1, via: None })) && p.id == "move:0-1"));
    let sim = Simulation::new(g.clone(), Execution::Grouped).unwrap();
    assert_eq!(sim.state().movement, Movement::Routed);
    let site = &g.sites[&0];
    let mut sides: Vec<_> = moves.iter().map(|(_, via)| {
        let route = cosmograph::journey::route(site, 3, *via).unwrap();
        assert_eq!(route.border.map(|b| b.side), *via);
        route.border.unwrap().enters
    }).collect();
    sides.sort();
    assert_eq!(sides, vec![0, 2]);
}

#[test]
fn a_journey_without_a_border_keeps_its_old_bytes() {
    let plain = Journey { from: 0, to: 1, departed: 2, arrives: 5, border: None };
    assert_eq!(serde_json::to_string(&plain).unwrap(), r#"{"from":0,"to":1,"departed":2,"arrives":5}"#);
    let place = cosmograph::schema::EventPlace::Journey(plain);
    assert_eq!(postcard::to_allocvec(&place).unwrap(), vec![1, 0, 1, 2, 5]);
    let routed = Journey { border: Some(cosmograph::schema::Border { side: 2, enters: 0, flipped: false }), ..plain };
    for j in [plain, routed] {
        let wire = postcard::to_allocvec(&cosmograph::schema::EventPlace::Journey(j)).unwrap();
        assert_eq!(postcard::from_bytes::<cosmograph::schema::EventPlace>(&wire).unwrap(),
            cosmograph::schema::EventPlace::Journey(j));
    }
}
