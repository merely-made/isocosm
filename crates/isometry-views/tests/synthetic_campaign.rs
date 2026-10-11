// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::asserted::Assertion;
use isometry_views::{synth_world, SYNTH_PARTY};

#[test]
fn synthetic_campaign_keeps_authored_topology_and_costs() {
    let world = synth_world(5, 6);
    assert_eq!(world.places().len(), 5);
    assert_eq!(world.characters().len(), 5);
    assert_eq!(world.factions().len(), 3);
    assert_eq!(world.laws().len(), 3);
    assert_eq!(world.storylets.len(), 6);
    assert_eq!(world.party_at(SYNTH_PARTY), Some("p0"));
    assert_eq!(world.overmap_for(SYNTH_PARTY).nodes.len(), 5);
    let prefix: Vec<_> = world
        .assertions()
        .iter()
        .take(10)
        .map(Assertion::key)
        .collect();
    assert_eq!(
        prefix,
        ["p0", "c0", "p1", "c1", "p2", "c2", "p3", "c3", "p4", "c4"]
    );
    let routes: Vec<_> = world
        .routes()
        .values()
        .map(|route| {
            (
                route.key.as_str(),
                route.from.as_str(),
                route.to.as_str(),
                route.weight,
            )
        })
        .collect();
    assert_eq!(
        routes,
        [
            ("r0_0", "p0", "p1", 1),
            ("r0_1", "p0", "p3", 1),
            ("r1_0", "p1", "p2", 2),
            ("r1_1", "p1", "p4", 2),
            ("r3_0", "p3", "p4", 4),
        ],
        "synthetic route identities, endpoints, and authored costs"
    );
    let route_keys: Vec<_> = world
        .assertions()
        .iter()
        .filter_map(|assertion| match assertion {
            Assertion::Route(route) => Some(route.key.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(route_keys, ["r0_0", "r0_1", "r1_0", "r1_1", "r3_0"]);
    assert_eq!(world.overmap().route("p0", "p4").unwrap().1, 3);
    assert_eq!(world.characters()["c4"].place.as_deref(), Some("p4"));
    assert_eq!(world.characters()["c4"].faction.as_deref(), Some("f1"));
}
