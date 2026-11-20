// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! The refusal limit (E10, D77-D79): a search stops at the limit with no
//! route; the same world without the rule keeps going.
use super::*;
use crate::spatial::embodiment::tests::terrain;
use turn_tests::control_input;

/// The turn control with its food moved west and three one-cell obstacles
/// whose strides the node check admits and the step check refuses: an
/// unlimited search finds a route after 5 refusals.
fn refusing(limit: Option<u32>) -> Input {
    let sim = terrain();
    let mut input = control_input(&sim);
    let cell = |x, y, z| [[x, y, z], [x + 1, y + 1, z + 1]];
    let food = cell(371, 17, 384);
    input.foods = vec![(2, vec![food])];
    input.blocked = vec![food, cell(374, 26, 386), cell(373, 24, 379), cell(373, 20, 391)];
    input.refusal_limit = limit;
    input
}

#[test]
fn a_search_ends_with_no_route_at_the_limit_and_an_unlimited_one_keeps_going() {
    let free = find(&refusing(None)).unwrap();
    assert!(free.route.is_some(), "positive control: unlimited finds its route");
    assert_eq!(free.refused_edges, 5, "positive control: it needs more refusals than the limit");
    let capped = find(&refusing(Some(2))).unwrap();
    assert!(capped.route.is_none());
    assert_eq!(capped.refused_edges, 2);
    assert_eq!(find(&refusing(Some(5))).unwrap().refused_edges, 5);
    assert_eq!(find(&refusing(Some(6))).unwrap(), free, "a limit not reached changes nothing");
}

#[test]
fn the_cache_keeps_a_limited_failure_apart_from_an_unlimited_search() {
    let cache = cache::Cache::default();
    let capped = cache.find(refusing(Some(2))).unwrap();
    let free = cache.find(refusing(None)).unwrap();
    assert!(capped.route.is_none() && free.route.is_some());
    assert_eq!(cache.find(refusing(Some(2))).unwrap(), capped);
}

#[test]
fn the_rule_is_absent_from_serialised_rules_unless_set() {
    use crate::Founding;
    let rules = |limit| {
        let mut g = Founding { seed: 8, population: 16, sites: 2, ..Default::default() }.generate().unwrap();
        g.rules.route_refusal_limit = limit;
        serde_json::to_string(&g.rules).unwrap()
    };
    let (off, on) = (rules(None), rules(Some(32)));
    assert!(!off.contains("route_refusal_limit"));
    assert_eq!(on.replace(",\"route_refusal_limit\":32", ""), off);
}
