// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    bodied::Bodies,
    directing::{Encounter, tests::critter},
    schema::Visit,
};

#[test]
fn a_small_wound_changes_one_animals_own_destination_and_replays() {
    let mut founding = world(7, 8).sim.genesis().founding.clone().unwrap();
    founding.bodies = Some(Bodies::default());
    let mut g = founding.generate().unwrap();
    // Keep the pair at the same appetite and spare the control incidental predator
    // bites while waiting for the first scheduled travel decision.
    for process in g.rules.processes.values_mut() {
        if moves(process).is_empty() {
            process.period = None;
        }
    }
    let mut injured = Session::new(g.clone(), Execution::Individuals).unwrap();
    let mut control = Session::new(g, Execution::Individuals).unwrap();
    let c = critter(&injured);
    let here = place(&injured, c);
    let body = injured.sim.state().population.get(c).unwrap();
    let part = body
        .living()
        .find(|(id, p)| crate::anatomy::living_cells(body.extent(*id), p) > 4)
        .unwrap()
        .0;
    injured
        .command(Command::Wound {
            entity: c,
            part: Some(part),
            cells: 1,
        })
        .unwrap();
    assert_eq!(
        injured
            .sim
            .state()
            .population
            .get(c)
            .unwrap()
            .places
            .as_ref()
            .unwrap()
            .hurt,
        Some((here, 0))
    );
    assert_eq!(
        injured.sim.mood(c),
        control.sim.mood(c),
        "one cell leaves ordinary mood alone"
    );
    assert_eq!(choice_step(&control, c), None);
    assert!(choice_step(&injured, c).is_some());
    let mut left = false;
    for _ in 0..40 {
        injured.advance(1).unwrap();
        control.advance(1).unwrap();
        if injured
            .sim
            .state()
            .population
            .get(c)
            .unwrap()
            .resident_site()
            != Some(here)
        {
            assert_eq!(place(&control, c), here);
            left = true;
            break;
        }
    }
    assert!(left, "the injured animal actually departs");
    assert!(injured.sim.state().nudges.is_empty());
    let replay = Session::load(injured.save(), Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), injured.sim.state_hash());
}

#[test]
fn a_remembered_meal_changes_with_appetite_and_expires() {
    let mut s = world(7, 8);
    let c = critter(&s);
    let here = place(&s, c);
    let there = *s
        .sim
        .state()
        .sites
        .iter()
        .find(|(site, s)| **site != here && s.routes.iter().any(|r| r.to == here))
        .unwrap()
        .0;
    let rules = &s.sim.genesis.rules;
    let body = s.sim.state.population.lift(c).unwrap();
    body.remember(rules, here, 0, Encounter::Meal);
    let rules = s.sim.genesis.rules.directing_ref().place_memory.unwrap();
    assert_eq!(
        s.sim.place_preferences(c, None)[0].unwrap().aim,
        Aim::StayNear
    );
    let stay = s.sim.place_preferences(c, None)[0].unwrap();
    assert!(stay.vote(here, None) > stay.vote(here, Some(there)));
    let body = s.sim.state.population.lift(c).unwrap();
    body.place = there;
    body.accounts.insert("matter:1-0".into(), 0);
    let pref = s.sim.place_preferences(c, None)[0].unwrap();
    assert_eq!((pref.aim, pref.site), (Aim::MoveToward, here));
    let return_step = |s: &Session| {
        let options = s.sim.consider(c, false).options;
        options
            .into_iter()
            .find_map(|(p, (score, _))| {
                (moves(&s.sim.genesis().rules.processes[&p]) == [here]).then_some(score)
            })
            .unwrap()
    };
    let before = return_step(&s);
    s.sim.state.tick = rules.span;
    assert!(s.sim.place_preferences(c, None)[0].is_none());
    assert_eq!(before - return_step(&s), i64::from(rules.meal));
    assert!(s.sim.state().nudges.is_empty());
}

#[test]
fn danger_can_outweigh_a_players_request_without_crediting_it() {
    let mut s = world(7, 8);
    let (p, c) = played(&mut s);
    let here = place(&s, c);
    // A radius of zero makes staying at this site an actual alternative to leaving.
    let mut g = s.sim.genesis().clone();
    g.rules.directing.as_mut().unwrap().near_hops = 0;
    s = Session::new(g, Execution::Individuals).unwrap();
    assert_eq!(played(&mut s), (p, c));
    request(&mut s, p, c, Aim::StayNear, Toward::Site(here));
    assert_eq!(choice_step(&s, c), None);
    let rules = &s.sim.genesis.rules;
    s.sim
        .state
        .population
        .lift(c)
        .unwrap()
        .remember(rules, here, 0, Encounter::Hurt);
    let choice = s.sim.consider(c, false);
    assert!(choice_step(&s, c).is_some());
    assert!(choice.answers.is_empty());
    assert!(s.sim.state().nudges[0].answer.is_none());
}

#[test]
fn undirected_feeding_leaves_a_personal_memory_that_replays() {
    let mut s = world(7, 8);
    let mut meal = None;
    for _ in 0..60 {
        s.advance(1).unwrap();
        meal = s
            .sim
            .state()
            .population
            .groups
            .iter()
            .find_map(|(&id, group)| {
                let (site, tick) = group.entity.places.as_ref()?.meal?;
                Some((id, site, tick))
            });
        if meal.is_some() {
            break;
        }
    }
    let (id, site, tick) = meal.expect("a scheduled native meal transfers matter");
    assert!(s.sim.state().nudges.is_empty());
    assert!(
        s.sim
            .state()
            .population
            .groups
            .values()
            .any(|g| g.entity.places.is_none())
    );
    let replay = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), s.sim.state_hash());
    assert_eq!(
        replay
            .sim
            .state()
            .population
            .get(id)
            .unwrap()
            .places
            .as_ref()
            .unwrap()
            .meal,
        Some((site, tick))
    );
}

#[test]
fn a_return_can_follow_only_a_recent_familiar_step_or_a_visible_exit() {
    let mut s = world(7, 8);
    let c = critter(&s);
    let here = place(&s, c);
    let via = s.sim.state().sites[&here].routes[0].to;
    let far = *hops_from(&s.sim, here)
        .iter()
        .find(|(_, d)| **d > 1)
        .unwrap()
        .0;
    let rules = &s.sim.genesis.rules;
    let body = s.sim.state.population.lift(c).unwrap();
    body.place = far;
    body.remember(rules, far, 0, Encounter::Meal);
    body.place = here;
    body.accounts.insert("matter:1-0".into(), 0);
    assert!(s.sim.place_preferences(c, None)[0].is_none());
    let body = s.sim.state.population.lift(c).unwrap();
    body.visits.extend([
        Visit {
            place: far,
            from: 0,
            until: 1,
        },
        Visit {
            place: via,
            from: 2,
            until: 3,
        },
    ]);
    s.sim.state.tick = 4;
    assert_eq!(s.sim.place_preferences(c, None)[0].unwrap().site, via);
}
