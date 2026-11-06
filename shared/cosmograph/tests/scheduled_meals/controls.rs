// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::rules::{Amount, Binding, Effect, Expr, Query, Reading};

fn second_eater(g: &mut Genesis, eater: Id) -> Id {
    let e = g.population.get(eater).unwrap().clone();
    g.population.insert(e, 1).unwrap()
}

fn eat(amount: u64, into: &str, of: &[&str]) -> Effect {
    Effect::Eat {
        from: Binding::Target,
        amount: amount.into(),
        into: into.into(),
        of: of.iter().map(|k| (*k).into()).collect(),
        whole: false,
    }
}

/// The scarcity controls deposit only what their admitted meal actually
/// shed. Legacy compound meals can instead keep a fixed authored cost.
fn deposit_actual_loss() -> Effect {
    Effect::Transfer {
        from: Binding::Actor,
        to: Binding::Place,
        account: DETRITUS.into(),
        amount: Amount::Computed(Expr::Read(Reading::Account {
            who: Binding::Actor,
            key: DETRITUS.into(),
        })),
    }
}

fn scarcity(stock: u64) -> (Genesis, [Id; 2], Id) {
    let (mut g, eater, prey) = world();
    let e = g.population.lift(prey).unwrap();
    e.accounts.retain(|key, _| key == PREY);
    e.accounts.insert(PREY.into(), stock);
    let other = second_eater(&mut g, eater);
    let feed = g.rules.processes.get_mut(FEED).unwrap();
    for query in &mut feed.requires {
        if let Query::Holds {
            who: Binding::Target,
            at_least,
        } = query
        {
            *at_least = 1;
        }
    }
    feed.effects[2] = deposit_actual_loss();
    (g, [eater, other], prey)
}

fn conserved_and_replayed(s: &Session, total: u128) {
    assert_eq!(s.sim.matter(), total);
    let saved = s.save();
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(Session::load(saved.clone(), mode).unwrap().save(), saved);
    }
}

#[test]
fn separate_meals_keep_their_partial_and_zero_grants_without_fallback() {
    for (stock, kept, dropped, remaining) in [(6, 1, 4, 0), (3, 0, 0, 3)] {
        let mut hashes = vec![];
        for mode in [Execution::Individuals, Execution::Grouped] {
            let (mut g, eaters, prey) = scarcity(stock);
            if stock == 3 {
                // Each unshared path retains a positive food offering.
                // Four one-unit demands against three units all floor to
                // zero, so eligibility stays stable through replay.
                g.rules.processes.get_mut(FEED).unwrap().effects = vec![
                    eat(1, BODY, &[]),
                    eat(1, DETRITUS, &[]),
                    deposit_actual_loss(),
                ];
            }
            let mut s = Session::new(g.clone(), mode).unwrap();
            let total = s.sim.matter();
            let run = s.advance_tick_with_flows().unwrap();
            assert_eq!((run.result.accepted, run.result.blocked), (2, 0), "{run:?}");
            for eater in eaters {
                assert_eq!(held(&s, eater, BODY), 12 + kept);
            }
            assert_eq!(held(&s, prey, PREY), remaining);
            assert_eq!(balances(&s, eaters[0], prey).2, dropped);
            if kept == 0 {
                assert!(
                    run.flows.is_empty(),
                    "a retained zero grant moves no matter"
                );
                assert_eq!(s.sim.state().population.get(prey), g.population.get(prey));
            }
            conserved_and_replayed(&s, total);
            hashes.push(s.sim.state_hash());
        }
        assert_eq!(hashes[0], hashes[1]);
    }
}

#[test]
fn overlapping_food_selectors_cannot_reuse_another_meals_accounts() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = world();
        let other = "matter:2-0";
        let e = g.population.lift(prey).unwrap();
        e.accounts.insert(PREY.into(), 2);
        e.accounts.insert(other.into(), 4);
        g.rules.processes.get_mut(FEED).unwrap().effects = vec![
            eat(2, BODY, &[PREY]),
            eat(4, DETRITUS, &[PREY, other]),
            deposit_actual_loss(),
        ];
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        let work = s.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
        assert_eq!((held(&s, prey, PREY), held(&s, prey, other)), (0, 0));
        assert_eq!(held(&s, eater, BODY), 14);
        assert_eq!(balances(&s, eater, prey).2, 4);
        conserved_and_replayed(&s, total);
    }
}

#[test]
fn a_changed_guard_cannot_lend_its_skipped_meal_quota_to_a_later_effect() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eaters, prey) = scarcity(6);
        g.rules.processes.get_mut(FEED).unwrap().effects = vec![
            eat(2, BODY, &[]),
            Effect::When {
                guard: Expr::AtLeast(
                    Box::new(Expr::Read(Reading::Account {
                        who: Binding::Actor,
                        key: BODY.into(),
                    })),
                    Box::new(Expr::Const(14)),
                ),
                then: vec![eat(4, DETRITUS, &[])],
                otherwise: vec![],
            },
            eat(1, BODY, &[]),
            deposit_actual_loss(),
        ];
        // The authored path is admissible: the unshared first meal raises
        // the body from twelve to fourteen and runs the guarded meal.
        let mut plain = Session::new(g.clone(), mode).unwrap();
        assert!(direct(&mut plain, eaters[0], prey).accepted());
        assert_eq!(held(&plain, eaters[0], BODY), 14);

        // Planning asks seven per eater. Six shared units floor the three
        // meals to zero, one, zero. The first zero changes the guard, so
        // this plan must refuse rather than consume a different address.
        let mut s = Session::new(g.clone(), mode).unwrap();
        let total = s.sim.matter();
        let run = s.advance_tick_with_flows().unwrap();
        assert_eq!((run.result.accepted, run.result.blocked), (0, 2), "{run:?}");
        assert!(run.flows.is_empty());
        for id in [eaters[0], eaters[1], prey] {
            assert_eq!(s.sim.state().population.get(id), g.population.get(id));
        }
        conserved_and_replayed(&s, total);
    }
}

/// A controlled part-ledger founding leaves one exposed native part
/// holding food. Six exceeds this seed's adult part share but fits its
/// native geometric capacity; it qualifies coincident-part contention,
/// while the separate fresh-founder controls qualify partitioned meals.
/// Root geometry is unchanged; this fixture explicitly authors the old
/// producer-only port and two-Eat meal to retain its compatibility coverage.
fn rooted(exposed: u64, two_eaters: bool) -> (Genesis, Vec<Id>, Id) {
    let (mut g, eater, prey) = world_with_bodies(true);
    eligible::authored_compound_meal(&mut g, eater, FEED, 1, 2);
    let e = g.population.get(prey).unwrap();
    let root = e.body.as_ref().unwrap().root;
    let part = *e
        .parts
        .keys()
        .filter(|&&id| id != root && anatomy::root_store::exposed(e, &g.rules, id))
        .max_by_key(|&&id| anatomy::bound(e.extent(id), &e.parts[&id], &g.rules, PREY))
        .unwrap();
    assert!(anatomy::bound(e.extent(part), &e.parts[&part], &g.rules, PREY) >= exposed);
    let e = g.population.lift(prey).unwrap();
    for id in e.parts.keys().copied().collect::<Vec<_>>() {
        if id != root {
            e.part_mut(&id).unwrap().matter.insert(PREY.into(), 0);
        }
    }
    e.part_mut(&part)
        .unwrap()
        .matter
        .insert(PREY.into(), exposed);
    assert_eq!(e.parts[&root].matter.get(PREY), Some(&20));
    let mut eaters = vec![eater];
    if two_eaters {
        eaters.push(second_eater(&mut g, eater));
    }
    (g, eaters, prey)
}

#[test]
fn authored_rooted_compound_feed_schedules_three_taken_one_kept_two_shed() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eaters, prey) = rooted(6, false);
        let eater = eaters[0];
        let mut plain = Session::new(g.clone(), mode).unwrap();
        let before = balances(&plain, eater, prey);
        let total = plain.sim.matter();
        assert!(direct(&mut plain, eater, prey).accepted());
        let expected = Meal {
            taken: 3,
            kept: 1,
            dropped: 2,
        };
        assert_eq!(change(before, balances(&plain, eater, prey)), expected);
        let mut s = Session::new(g, mode).unwrap();
        let work = s.advance(1).unwrap();
        assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
        assert_eq!(change(before, balances(&s, eater, prey)), expected);
        conserved_and_replayed(&s, total);
    }
}

#[test]
fn two_authored_compound_grazers_share_the_exposed_part_and_leave_the_root() {
    for (exposed, accepted, kept, left, dropped) in [(6, 2, 1, 20, 4), (3, 0, 0, 23, 0)] {
        let mut hashes = vec![];
        for mode in [Execution::Individuals, Execution::Grouped] {
            let (g, eaters, prey) = rooted(exposed, true);
            let root = g.population.get(prey).unwrap().body.as_ref().unwrap().root;
            let mut s = Session::new(g, mode).unwrap();
            let total = s.sim.matter();
            let work = s.advance(1).unwrap();
            assert_eq!(
                (work.accepted, work.blocked),
                (accepted, 2 - accepted),
                "{work:?}"
            );
            for eater in eaters {
                assert_eq!(held(&s, eater, BODY), 12 + kept);
            }
            assert_eq!(held(&s, prey, PREY), left);
            assert_eq!(
                s.sim.state().population.get(prey).unwrap().parts[&root]
                    .matter
                    .get(PREY),
                Some(&20)
            );
            assert_eq!(s.sim.state().sites[&0].accounts[DETRITUS], dropped);
            conserved_and_replayed(&s, total);
            hashes.push(s.sim.state_hash());
        }
        assert_eq!(hashes[0], hashes[1]);
    }
}
