// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeSet;

fn part(id: u32) -> PartRef {
    PartRef {
        subject: 3,
        part: id,
    }
}
fn source(id: u32, charge: u64) -> Node {
    Node {
        id: NodeId(id),
        kind: NodeKind::Source {
            part: part(id),
            capacity: charge,
            charge,
        },
    }
}
fn effect(id: u32) -> Node {
    Node {
        id: NodeId(id),
        kind: NodeKind::Effect { part: part(id) },
    }
}
fn edge(from: u32, to: u32, capacity: u64) -> Edge {
    Edge {
        from: NodeId(from),
        to: NodeId(to),
        capacity,
    }
}
fn live(ids: &[u32]) -> BTreeSet<PartRef> {
    ids.iter().map(|i| part(*i)).collect()
}
/// A source of `charge` feeding a trunk of `trunk` room that branches to
/// two sites, 3 and 4.
fn forked(charge: u64, trunk: u64) -> FunctionalNetwork {
    FunctionalNetwork::new(
        vec![source(1, charge), effect(2), effect(3), effect(4)],
        vec![edge(1, 2, trunk), edge(2, 3, 100), edge(2, 4, 100)],
    )
    .unwrap()
}
fn charge(n: &FunctionalNetwork, id: u32) -> u64 {
    match n.nodes[&NodeId(id)].kind {
        NodeKind::Source { charge, .. } | NodeKind::Store { charge, .. } => charge,
        _ => panic!("not a supply"),
    }
}

#[test]
fn carriage_delivers_every_ask_where_routes_have_room() {
    let mut n = forked(20, 20);
    let c = n
        .carry(&[(NodeId(3), 5), (NodeId(4), 7)], 4, &live(&[1, 2, 3, 4]))
        .unwrap();
    assert_eq!(c.delivered, vec![(NodeId(3), 5), (NodeId(4), 7)]);
    assert_eq!(
        c.supplied_by,
        vec![SupplyDebit {
            source: NodeId(1),
            amount: 12
        }]
    );
    assert_eq!(charge(&n, 1), 8);
}

#[test]
fn sites_share_a_trunk_within_one_carriage() {
    // The trunk carries 6 in all, however the asks fall.
    let mut n = forked(20, 6);
    let c = n
        .carry(&[(NodeId(3), 5), (NodeId(4), 5)], 4, &live(&[1, 2, 3, 4]))
        .unwrap();
    assert_eq!(c.total(), 6);
    assert_eq!(charge(&n, 1), 14);
    // Control: carried one at a time, the room resets and each takes all.
    let mut alone = forked(20, 6);
    let a = alone
        .carry(&[(NodeId(3), 5)], 4, &live(&[1, 2, 3, 4]))
        .unwrap();
    let b = alone
        .carry(&[(NodeId(4), 5)], 4, &live(&[1, 2, 3, 4]))
        .unwrap();
    assert_eq!(a.total() + b.total(), 10);
}

#[test]
fn a_short_supply_delivers_what_it_holds() {
    let mut n = forked(4, 20);
    let c = n
        .carry(&[(NodeId(3), 5), (NodeId(4), 5)], 4, &live(&[1, 2, 3, 4]))
        .unwrap();
    assert_eq!(c.total(), 4);
    assert_eq!(charge(&n, 1), 0);
}

#[test]
fn a_cut_route_or_a_dead_site_takes_nothing() {
    let mut cut = forked(20, 20);
    let c = cut
        .carry(&[(NodeId(3), 5), (NodeId(4), 5)], 4, &live(&[1, 3, 4]))
        .unwrap();
    assert_eq!(c.delivered, vec![(NodeId(3), 0), (NodeId(4), 0)]);
    assert!(c.supplied_by.is_empty());
    assert_eq!(charge(&cut, 1), 20);
    let mut dead = forked(20, 20);
    let c = dead
        .carry(&[(NodeId(3), 5), (NodeId(4), 5)], 4, &live(&[1, 2, 4]))
        .unwrap();
    assert_eq!(c.delivered, vec![(NodeId(3), 0), (NodeId(4), 5)]);
}

#[test]
fn hops_bound_a_carriage() {
    let mut n = forked(20, 20);
    let c = n.carry(&[(NodeId(3), 5)], 1, &live(&[1, 2, 3, 4])).unwrap();
    assert_eq!(c.total(), 0);
}

#[test]
fn a_store_supplies_a_carriage() {
    let store = Node {
        id: NodeId(1),
        kind: NodeKind::Store {
            part: part(1),
            capacity: 9,
            charge: 9,
        },
    };
    let mut n = FunctionalNetwork::new(vec![store, effect(2)], vec![edge(1, 2, 9)]).unwrap();
    let c = n.carry(&[(NodeId(2), 4)], 2, &live(&[1, 2])).unwrap();
    assert_eq!(c.total(), 4);
    assert_eq!(charge(&n, 1), 5);
}

#[test]
fn a_carriage_carries_only_to_effect_sites() {
    let mut n = forked(20, 20);
    let unknown = n.carry(&[(NodeId(9), 1)], 4, &live(&[1, 2, 3, 4]));
    assert_eq!(unknown, Err(EvaluationError::UnknownTarget(NodeId(9))));
    let supply = n.carry(&[(NodeId(1), 1)], 4, &live(&[1, 2, 3, 4]));
    assert!(matches!(supply, Err(EvaluationError::WrongTarget { .. })));
}

#[test]
fn a_store_already_charged_still_takes_a_store_operation() {
    let store = Node {
        id: NodeId(2),
        kind: NodeKind::Store {
            part: part(2),
            capacity: 9,
            charge: 3,
        },
    };
    let mut n = FunctionalNetwork::new(vec![source(1, 5), store], vec![edge(1, 2, 9)]).unwrap();
    let rules = WorldRules {
        allowed_operators: [Operator::Store].into_iter().collect(),
        allowed_costs: [2].into_iter().collect(),
        max_range: 1,
        max_hops: 2,
    };
    let request = EvaluationRequest {
        operator: Operator::Store,
        target: NodeId(2),
        cost: 2,
        range: 0,
        max_hops: 2,
    };
    let e = n.evaluate(&request, &live(&[1, 2]), &rules).unwrap();
    assert_eq!(
        e.supplied_by,
        vec![SupplyDebit {
            source: NodeId(1),
            amount: 2
        }]
    );
    assert_eq!(charge(&n, 2), 5);
}
