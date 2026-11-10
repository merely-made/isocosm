// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::tests::fixture;
use super::*;
use crate::{Execution, Session, rules::{Expr, Reading}, schema::*, simulation::State};
use std::collections::BTreeMap;

/// The fixture's states, its rules and what it names.
struct Pair { rules: crate::rules::Rules, before: State, host: Id, surface: Identity }

fn pair() -> Pair {
    let (g, host, _, surface, _) = fixture();
    let s = Session::new(g, Execution::Individuals).unwrap();
    Pair { rules: s.sim.genesis().rules.clone(), before: s.sim.state().clone(), host, surface }
}

fn capped(p: &mut Pair, limit: i64) {
    let gene = Expr::Read(Reading::Kept { name: "capacity:gene".into() });
    p.rules.spread_capacity = Some(CapacityRules { key: "capacity:base".into(),
        trait_inputs: BTreeMap::from([("capacity:gene".into(), "function:fix".into())]),
        mechanic: Expr::Mul(vec![gene, Expr::Const(limit)]), layers: vec![] });
}

fn set(state: &mut State, at: Substrate, identity: &Identity, amount: u64) {
    let ledger = match at {
        Substrate::Site(id) => &mut state.sites.get_mut(&id).unwrap().accounts,
        Substrate::Host(id) => &mut state.population.lift(id).unwrap().accounts,
        Substrate::Place(_) => unreachable!("not planted here"),
    };
    if amount == 0 { ledger.remove(&identity.key()); } else { ledger.insert(identity.key(), amount); }
}

#[test]
fn matter_on_a_new_site_is_reached_and_only_for_surfaces() {
    let p = pair();
    let mut after = p.before.clone();
    set(&mut after, Substrate::Site(1), &p.surface, 3);
    set(&mut after, Substrate::Site(2), &Identity::Territory(p.host), 9);
    assert_eq!(happened(&p.rules, &p.before, &after),
        vec![Surfaced::Reached { identity: p.surface.clone(), at: Substrate::Site(1) }]);
}

#[test]
fn matter_moved_off_a_substrate_has_left_it() {
    let p = pair();
    let mut after = p.before.clone();
    set(&mut after, Substrate::Site(0), &p.surface, 0);
    assert_eq!(happened(&p.rules, &p.before, &after),
        vec![Surfaced::Left { identity: p.surface.clone(), at: Substrate::Site(0) }]);
}

#[test]
fn matter_raised_to_capacity_brims_unless_it_already_had() {
    let mut p = pair();
    capped(&mut p, 20);
    let site = Substrate::Site(0);
    let mut low = p.before.clone();
    set(&mut low, site, &p.surface, 15);
    let mut full = p.before.clone();
    set(&mut full, site, &p.surface, 20);
    let brim = Surfaced::Brimmed { identity: p.surface.clone(), at: site };
    assert_eq!(happened(&p.rules, &low, &full), vec![brim.clone()]);
    let mut over = p.before.clone();
    set(&mut over, site, &p.surface, 25);
    assert_eq!(happened(&p.rules, &low, &over), vec![brim]);
    assert!(happened(&p.rules, &full, &over).is_empty(), "already at capacity before");
    p.rules.spread_capacity = None;
    assert!(happened(&p.rules, &low, &full).is_empty(), "no capacity, no brim");
}

#[test]
fn a_dead_host_is_reported_once_and_not_as_a_departure() {
    let p = pair();
    let mut after = p.before.clone();
    after.population.lift(p.host).unwrap().alive = false;
    assert_eq!(happened(&p.rules, &p.before, &after),
        vec![Surfaced::HostDied { identity: p.surface.clone(), host: p.host }]);
    let mut gone = p.before.clone();
    gone.population.groups.remove(&p.host);
    assert_eq!(happened(&p.rules, &p.before, &gone),
        vec![Surfaced::HostDied { identity: p.surface.clone(), host: p.host }]);
}

#[test]
fn a_second_surface_on_a_substrate_meets_the_first_both_ways() {
    let p = pair();
    let other = Identity::Surface("zz:other".into());
    assert!(p.surface.key() < other.key());
    let mut after = p.before.clone();
    after.spreads.identities.insert(other.clone());
    set(&mut after, Substrate::Site(0), &other, 4);
    let at = Substrate::Site(0);
    assert_eq!(happened(&p.rules, &p.before, &after), vec![
        Surfaced::Met { identity: p.surface.clone(), other: other.clone(), at },
        Surfaced::Reached { identity: other.clone(), at },
        Surfaced::Met { identity: other.clone(), other: p.surface.clone(), at },
    ]);
    let mut again = after.clone();
    set(&mut again, Substrate::Site(0), &other, 9);
    assert!(happened(&p.rules, &after, &again).is_empty(), "they already shared it");
}

#[test]
fn identical_states_have_no_events_and_a_busy_step_is_in_a_stable_order() {
    let mut p = pair();
    capped(&mut p, 20);
    assert!(happened(&p.rules, &p.before, &p.before).is_empty());
    let other = Identity::Surface("zz:other".into());
    let mut after = p.before.clone();
    after.spreads.identities.insert(other.clone());
    set(&mut after, Substrate::Site(2), &p.surface, 5);
    set(&mut after, Substrate::Site(1), &other, 5);
    set(&mut after, Substrate::Site(2), &other, 5);
    set(&mut after, Substrate::Site(0), &p.surface, 0);
    after.population.lift(p.host).unwrap().alive = false;
    let found = happened(&p.rules, &p.before, &after);
    assert_eq!(found, happened(&p.rules, &p.before, &after));
    let (a, b) = (p.surface.clone(), other);
    assert_eq!(found, vec![
        Surfaced::Left { identity: a.clone(), at: Substrate::Site(0) },
        Surfaced::Reached { identity: a.clone(), at: Substrate::Site(2) },
        Surfaced::Met { identity: a.clone(), other: b.clone(), at: Substrate::Site(2) },
        Surfaced::HostDied { identity: a.clone(), host: p.host },
        Surfaced::Reached { identity: b.clone(), at: Substrate::Site(1) },
        Surfaced::Reached { identity: b.clone(), at: Substrate::Site(2) },
        Surfaced::Met { identity: b, other: a, at: Substrate::Site(2) },
    ]);
}

#[test]
fn simulations_read_the_same_as_their_states() {
    let (g, _, _, surface, _) = fixture();
    let before = Session::new(g, Execution::Individuals).unwrap();
    let mut after = before.clone();
    set(&mut after.sim.state, Substrate::Site(1), &surface, 2);
    assert_eq!(surfaced(&before.sim, &after.sim),
        vec![Surfaced::Reached { identity: surface, at: Substrate::Site(1) }]);
}
