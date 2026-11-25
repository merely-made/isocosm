// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::directing::choice::moves;
use crate::{
    Command, Execution, Founding, Session,
    directing::{Played, tests::played},
};

mod personal;

fn world(seed: u64, sites: u32) -> Session {
    let g = Founding {
        seed,
        sites,
        population: 48,
        lineages: 3,
        cohort_size: 4,
        ecology: true,
        place_memory: Some(Default::default()),
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    Session::new(g, Execution::Individuals).unwrap()
}

fn request(s: &mut Session, p: Id, c: Id, aim: Aim, toward: Toward) {
    s.command(Command::Nudge {
        participant: p,
        critter: c,
        aim,
        toward,
        act: None,
    })
    .unwrap();
}

fn place(s: &Session, c: Id) -> Id {
    s.sim
        .state()
        .population
        .get(c)
        .unwrap()
        .resident_site()
        .unwrap()
}

fn choice_step(s: &Session, c: Id) -> Option<Id> {
    let choice = s.sim.consider(c, false).process?;
    moves(&s.sim.genesis().rules.processes[&choice])
        .first()
        .copied()
}

#[test]
fn generated_creatures_weigh_each_direction_and_the_players_bond() {
    for seed in [7, 11, 19] {
        let mut s = world(seed, 8);
        let (p, c) = played(&mut s);
        let here = place(&s, c);
        let normal = s.sim.consider(c, false);
        let goal = *hops_from(&s.sim, here)
            .iter()
            .max_by_key(|(_, d)| *d)
            .unwrap()
            .0;
        let distance = hops_to(&s.sim, goal);
        assert!(distance[&here] >= 2);
        let body = s
            .sim
            .state()
            .population
            .groups
            .iter()
            .find(|(_, g)| g.entity.resident_site() == Some(goal) && g.entity.alive)
            .unwrap()
            .0
            .to_owned();
        request(&mut s, p, c, Aim::MoveToward, Toward::Thing(body));
        let step = choice_step(&s, c).expect("trusted request can motivate an actual movement");
        assert!(distance[&step] < distance[&here]);
        request(&mut s, p, c, Aim::StayNear, Toward::Site(goal));
        let step = choice_step(&s, c).unwrap();
        assert!(distance[&step] < distance[&here]);
        request(&mut s, p, c, Aim::MoveAway, Toward::Site(here));
        assert_ne!(choice_step(&s, c).unwrap(), here);
        s.sim.set_bond(p, c, 0);
        let ignored = s.sim.consider(c, false);
        assert_eq!(
            (ignored.process, ignored.options),
            (normal.process, normal.options)
        );
        assert!(ignored.answers.is_empty());
    }
}

#[test]
fn staying_near_tracks_the_body_and_credits_one_answer() {
    let mut s = world(7, 8);
    let (p, c) = played(&mut s);
    let here = place(&s, c);
    let target = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(id, g)| **id != c && g.entity.resident_site() == Some(here))
        .unwrap()
        .0
        .to_owned();
    request(&mut s, p, c, Aim::StayNear, Toward::Thing(target));
    let fields = Guidance::new(&s.sim, &s.sim.live_nudges(c, 0));
    let r = &fields.requests[0];
    let distances = hops_to(&s.sim, here);
    let one = *distances.iter().find(|(_, d)| **d == 1).unwrap().0;
    let two = *distances.iter().find(|(_, d)| **d == 2).unwrap().0;
    assert_eq!(fields.vote(r, one, None), 1);
    assert_eq!(fields.vote(r, one, Some(two)), -1);
    assert_eq!(fields.vote(r, two, Some(one)), 1);
    let stationary = crate::generate::process("local", Causation::Choice, Vec::new());
    let mut answers = Vec::new();
    assert!(fields.score(&s.sim, &stationary, here, &mut answers) > 0);
    let wellbeing = s.sim.wellbeing(c);
    s.sim.answered(c, "local", &answers, wellbeing, true);
    let bond = s.sim.bond(p, c);
    s.sim.answered(c, "local", &answers, wellbeing, true);
    assert_eq!(s.sim.bond(p, c), bond);
    answers.clear();
    let live = s.sim.live_nudges(c, 1);
    assert_eq!(live, [0]);
    assert!(Guidance::new(&s.sim, &live).score(&s.sim, &stationary, here, &mut answers) > 0);
    assert!(answers.is_empty());
    s.sim.state.population.lift(target).unwrap().place = two;
    let moved = Guidance::new(&s.sim, &live);
    assert_eq!(moved.requests[0].field.0, two);
    s.sim.state.population.lift(target).unwrap().alive = false;
    assert_eq!(
        Guidance::new(&s.sim, &live).requests[0].field.0,
        two,
        "resident remains are a placeable target"
    );
    s.sim.state.population.lift(target).unwrap().journey = Some(crate::schema::Journey {
        from: two,
        to: one,
        departed: 0,
        arrives: 2,
        border: None,
    });
    assert!(Guidance::new(&s.sim, &live).requests.is_empty());
    request(&mut s, p, c, Aim::Attend, Toward::Site(here));
    assert_eq!(s.sim.live_nudges(c, 1), [1]);
    assert!(
        s.sim
            .live_nudges(c, s.sim.genesis().rules.directing().span)
            .is_empty()
    );
}

#[test]
fn generated_travel_executes_and_replays_beside_an_untreated_world() {
    let mut departures = 0;
    for seed in [7, 11, 19] {
        let mut s = world(seed, 8);
        let (p, c) = played(&mut s);
        let mut control = Session::load(s.save(), Execution::Individuals).unwrap();
        let here = place(&s, c);
        let goal = s.sim.state().sites[&here].routes[0].to;
        let identity = s.sim.player_seed(p).unwrap();
        request(&mut s, p, c, Aim::MoveToward, Toward::Site(goal));
        for _ in 0..45 {
            s.advance(1).unwrap();
            control.advance(1).unwrap();
            if s.sim.state().nudges[0].answer.is_some() {
                break;
            }
        }
        if let Some(answer) = &s.sim.state().nudges[0].answer {
            let step = moves(&s.sim.genesis().rules.processes[&answer.process]);
            assert_eq!(step, [goal]);
            let position = |s: &Session| {
                let body = s.sim.state().population.get(c).unwrap();
                body.journey.map(|j| j.to).or_else(|| body.resident_site())
            };
            if position(&s) == Some(goal) && position(&control) == Some(here) {
                departures += 1;
            }
        }
        let replay = Session::load(s.save(), Execution::Individuals).unwrap();
        assert_eq!(replay.sim.state_hash(), s.sim.state_hash());
        assert_eq!(replay.sim.player_seed(p).unwrap(), identity);
    }
    assert!(
        departures >= 2,
        "{departures} treated departures with controls still home"
    );
}

#[test]
fn a_named_targetless_act_can_be_suggested_at_its_site() {
    let mut s = world(7, 8);
    let (p, c) = played(&mut s);
    let here = place(&s, c);
    let body = s.sim.state.population.lift(c).unwrap();
    body.accounts.insert("matter:1-0".into(), 0);
    let baseline = s.sim.consider(c, false);
    let walk = baseline
        .options
        .keys()
        .find(|key| !moves(&s.sim.genesis().rules.processes[*key]).is_empty())
        .expect("a hungry resident can consider migration")
        .clone();
    let command = |act: String| Command::Nudge {
        participant: p,
        critter: c,
        aim: Aim::ActHere,
        toward: Toward::Site(here),
        act: Some(act),
    };
    assert!(s.command(command("missing".into())).is_err());
    s.command(command(walk.clone())).unwrap();
    let hinted = s.sim.consider(c, false);
    assert!(hinted.options[&walk].0 > baseline.options[&walk].0);
    for (other, score) in baseline.options.iter().filter(|(name, _)| **name != walk) {
        assert_eq!(&hinted.options[other], score);
    }
}

#[test]
fn two_players_have_separate_influence_on_the_same_creature() {
    let mut s = world(11, 8);
    let (first, c) = played(&mut s);
    let (second, same) = played(&mut s);
    assert_eq!(c, same);
    assert_ne!(
        s.sim.player_seed(first).unwrap(),
        s.sim.player_seed(second).unwrap()
    );
    let here = place(&s, c);
    s.sim.set_bond(first, c, 0);
    request(&mut s, first, c, Aim::StayNear, Toward::Site(here));
    request(&mut s, second, c, Aim::MoveAway, Toward::Site(here));
    let choice = s.sim.consider(c, false);
    assert_eq!(choice.answers, [1]);
    assert_ne!(choice_step(&s, c).unwrap(), here);
    assert!(s.sim.bond(second, c).unwrap() > 0);
}

#[test]
fn a_direction_cannot_waive_a_hard_requirement() {
    let mut s = world(7, 8);
    let (p, c) = played(&mut s);
    let here = place(&s, c);
    let to = s.sim.state().sites[&here].routes[0].to;
    request(&mut s, p, c, Aim::MoveToward, Toward::Site(to));
    let mut walk = crate::generate::process(
        "authored-walk",
        Causation::Choice,
        vec![crate::rules::Effect::Move {
            destination: to,
            via: None,
        }],
    );
    walk.requires = vec![Query::MigrationMotive { amount: i64::MIN }];
    assert!(s.sim.infeasible(c, None, &walk, here).is_none());
    walk.requires.push(Query::MoodBelow { amount: i64::MIN });
    assert!(s.sim.infeasible(c, None, &walk, here).is_some());
}

#[test]
#[ignore = "release timing of one native consideration"]
fn direction_cost() {
    use std::{hint::black_box, time::Instant};
    for sites in [8, 32] {
        let mut idle = world(7, sites);
        let (p, c) = played(&mut idle);
        let mut guided = Session::load(idle.save(), Execution::Individuals).unwrap();
        let here = place(&idle, c);
        request(
            &mut guided,
            p,
            c,
            Aim::MoveToward,
            Toward::Site(idle.sim.state().sites[&here].routes[0].to),
        );
        let measure = |session: &Session| {
            let start = Instant::now();
            for _ in 0..512 {
                black_box(session.sim.choose(black_box(c)));
            }
            start.elapsed().as_nanos() / 512
        };
        let mut plain = Vec::new();
        let mut active = Vec::new();
        for round in 0..11 {
            if round % 2 == 0 {
                plain.push(measure(&idle));
                active.push(measure(&guided));
            } else {
                active.push(measure(&guided));
                plain.push(measure(&idle));
            }
        }
        plain.sort_unstable();
        active.sort_unstable();
        println!(
            "{sites} sites, median ns per scheduled choice: idle {} directed {}",
            plain[5], active[5]
        );
    }
}
