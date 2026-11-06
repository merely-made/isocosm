// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Every authored Eat in a scheduled act keeps its own bounded share.
//! The ordinary world-time meal keeps two and sheds four out of six.

use cosmograph::{
    Command, Execution, Founding, Session, anatomy,
    bodied::Bodies,
    ecology::{Rates, Web},
    schema::Id,
    simulation::{Genesis, Receipt},
};

const FEED: &str = "ecology:feed-1";
const PREY: &str = "matter:0-0";
const BODY: &str = "matter:1-0";
const DETRITUS: &str = "world:detritus";

#[path = "scheduled_meals/controls.rs"]
mod controls;
#[path = "scheduled_meals/defence.rs"]
mod defence;
#[path = "scheduled_meals/eligible.rs"]
mod eligible;

fn world() -> (Genesis, Id, Id) {
    world_with_bodies(false)
}

fn world_with_bodies(rooted: bool) -> (Genesis, Id, Id) {
    let mut g = Founding {
        seed: 9,
        sites: 1,
        population: 60,
        cohort_size: 1,
        lineages: 5,
        ecology: true,
        ecology_rates: Rates::WorldTime,
        web: Web::Diets,
        bodies: rooted.then(Bodies::default),
        ..Founding::default()
    }
    .generate()
    .unwrap();
    let member = |line: &str| {
        *g.population
            .groups
            .iter()
            .find(|(_, group)| group.entity.lineage == line)
            .unwrap()
            .0
    };
    let (eater, prey) = (member("lineage:1"), member("lineage:0"));
    g.population
        .groups
        .retain(|id, _| *id == eater || *id == prey);
    g.population.lift(eater).unwrap().place = 0;
    g.population.lift(prey).unwrap().place = 0;
    g.rules.processes.retain(|id, _| id == FEED);
    g.rules.processes.get_mut(FEED).unwrap().period = Some(1);
    (g, eater, prey)
}

fn held(s: &Session, id: Id, account: &str) -> u64 {
    anatomy::held(
        s.sim.state().population.get(id).unwrap(),
        &s.sim.genesis().rules,
        account,
    )
}

fn balances(s: &Session, eater: Id, prey: Id) -> (u64, u64, u64) {
    (
        held(s, prey, PREY),
        held(s, eater, BODY),
        s.sim.state().sites[&0]
            .accounts
            .get(DETRITUS)
            .copied()
            .unwrap_or(0),
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Meal {
    taken: u64,
    kept: u64,
    dropped: u64,
}

fn change(before: (u64, u64, u64), after: (u64, u64, u64)) -> Meal {
    Meal {
        taken: before.0 - after.0,
        kept: after.1 - before.1,
        dropped: after.2 - before.2,
    }
}

fn direct(s: &mut Session, eater: Id, prey: Id) -> Receipt {
    serde_json::from_str(
        &s.command(Command::Act {
            actor: eater,
            target: Some(prey),
            process: FEED.into(),
            cause: None,
        })
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn scheduled_world_time_multi_eat_matches_the_authored_six_two_four_meal() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey) = world();
        let mut direct_session = Session::new(g.clone(), mode).unwrap();
        let before = balances(&direct_session, eater, prey);
        assert_eq!((before.0, before.1), (60, 12));
        let total = direct_session.sim.matter();
        let receipt = direct(&mut direct_session, eater, prey);
        assert!(receipt.accepted(), "ordinary direct meal: {receipt:?}");
        let expected = Meal {
            taken: 6,
            kept: 2,
            dropped: 4,
        };
        assert_eq!(
            change(before, balances(&direct_session, eater, prey)),
            expected
        );
        assert_eq!(direct_session.sim.matter(), total);

        let mut scheduled = Session::new(g, mode).unwrap();
        let work = scheduled.advance(1).unwrap();
        assert_eq!((work.evaluations, work.accepted), (1, 1), "{work:?}");
        assert_eq!(scheduled.sim.matter(), total);
        assert_eq!(
            Session::load(scheduled.save(), mode).unwrap().save(),
            scheduled.save()
        );
        assert_eq!(
            change(before, balances(&scheduled, eater, prey)),
            expected,
            "every scheduled Eat must retain its authored bound, {mode:?}"
        );
    }
}
