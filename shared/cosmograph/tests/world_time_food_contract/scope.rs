// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::schema::{Method, Movement, Route};

fn offering_count(s: &Session, eater: Id, prey: Id) -> usize {
    let g = s.sim.genesis();
    let state = s.sim.state();
    g.rules.processes["ecology:feed-3"]
        .food_offerings(
            state.population.get(eater).unwrap(),
            state.population.get(prey).unwrap(),
            &g.rules,
            &state.lineages,
        )
        .unwrap()
        .len()
}

#[test]
fn a_participant_classified_holder_is_never_food_even_with_eligible_matter() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(3, 8);
        // Deliberately retain the physical animal tissue: exclusion must
        // come from native participant meaning, not from an empty ledger.
        let e = g.population.lift(prey).unwrap();
        e.kingdom = cosmograph::directing::PARTICIPANT.into();
        e.method = Method::Inert;
        let mut s = Session::new(g, mode).unwrap();
        let total = s.sim.matter();
        assert_eq!(held(&s, prey, "matter:8-0"), 15);
        assert_eq!(offering_count(&s, eater, prey), 0);
        assert_eq!(s.sim.available_food(eater, "ecology:feed-3").unwrap(), 0);
        let before = (books(&s, eater), books(&s, prey));
        let r = act(&mut s, eater, Some(prey), "ecology:feed-3");
        assert!(!r.accepted(), "a participant is not a target body: {r:?}");
        assert_eq!((books(&s, eater), books(&s, prey)), before);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}

#[test]
fn native_journey_prey_offer_nothing_until_arrival_even_to_a_global_feed() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, eater, prey) = controlled(3, 8);
        let mut destination = g.sites[&0].clone();
        destination.routes.clear();
        g.sites.insert(1, destination);
        g.sites.get_mut(&0).unwrap().routes.push(Route {
            to: 1,
            travel: 3,
            transmission: 1_000_000,
            border: None,
            authored: None,
        });
        let feed = g.rules.processes.get_mut("ecology:feed-3").unwrap();
        feed.period = None;
        feed.target.as_mut().unwrap().same_place = false;
        auxiliary(
            &mut g,
            "ecology:feed-3",
            "test:depart",
            vec![Effect::Move {
                destination: 1,
                via: None,
            }],
        );
        let mut s = Session::new_with_movement(g, mode, Movement::Journey).unwrap();
        let total = s.sim.matter();
        assert!(offering_count(&s, eater, prey) > 0);
        assert_eq!(s.sim.available_food(eater, "ecology:feed-3").unwrap(), 1);
        assert!(act(&mut s, prey, None, "test:depart").accepted());
        assert!(
            s.sim
                .state()
                .population
                .get(prey)
                .unwrap()
                .journey
                .is_some()
        );
        assert_eq!(offering_count(&s, eater, prey), 0);
        assert_eq!(s.sim.available_food(eater, "ecology:feed-3").unwrap(), 0);
        let before = (books(&s, eater), books(&s, prey));
        assert!(!act(&mut s, eater, Some(prey), "ecology:feed-3").accepted());
        assert_eq!((books(&s, eater), books(&s, prey)), before);
        assert_eq!(s.sim.matter(), total);
        replay(&s);
        assert_eq!(s.advance(3).unwrap().arrivals, 1);
        assert!(
            offering_count(&s, eater, prey) > 0,
            "the same body is food after native arrival"
        );
        assert_eq!(
            s.sim.available_food(eater, "ecology:feed-3").unwrap(),
            0,
            "per-head food remains local to the actor site"
        );
        assert!(act(&mut s, eater, Some(prey), "ecology:feed-3").accepted());
        assert_eq!(s.sim.matter(), total);
        replay(&s);
    }
}
