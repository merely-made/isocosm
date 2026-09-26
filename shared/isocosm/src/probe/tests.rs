// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{aggregate, draws::Stream, *};
use crate::{Execution, Simulation, simulation::Outcome};

fn competition(margin: u64) -> Competition {
    Competition {
        ration: 4,
        contest: "leaning:contest".into(),
        margin,
        cost: 1,
        round: "probe:round".into(),
        upset: 0,
        advantage: 1,
        kinds: vec![],
    }
}

#[test]
fn allocation_spreads_the_shortfall_over_pairs() {
    assert_eq!(allocate(10, 10), None);
    assert_eq!(allocate(10, 12), None);
    // Ten hungry, five pairs: seven rations give two pairs a ration each
    // and leave three pairs contending for one.
    assert_eq!(
        allocate(10, 7),
        Some(Allocation {
            doubles: 2,
            contested: 3
        })
    );
    // Fewer rations than pairs: every ration is contended.
    assert_eq!(
        allocate(10, 3),
        Some(Allocation {
            doubles: 0,
            contested: 3
        })
    );
    assert_eq!(
        allocate(10, 0),
        Some(Allocation {
            doubles: 0,
            contested: 0
        })
    );
    // Odd count: the last member is unpaired and goes without.
    assert_eq!(
        allocate(11, 10),
        Some(Allocation {
            doubles: 5,
            contested: 0
        })
    );
    for hungry in 0..40u64 {
        for rations in 0..hungry {
            let a = allocate(hungry, rations).unwrap();
            assert_eq!(2 * a.doubles + a.contested, rations.min(2 * (hungry / 2)));
            assert!(a.doubles + a.contested <= hungry / 2);
        }
    }
}

#[test]
fn meetings_follow_the_ruling() {
    let c = competition(1);
    let (contest, share) = (true, false);
    // Two sharers split the ration.
    assert_eq!(meet(&c, [share, share], [3, 1]), Meeting::Settled([2, 2]));
    // A contester takes it from a sharer, whatever their reserves.
    assert_eq!(meet(&c, [share, contest], [9, 1]), Meeting::Settled([0, 4]));
    // Sizing up: a clear gap ends it at display, and the smaller yields.
    assert_eq!(
        meet(&c, [contest, contest], [5, 2]),
        Meeting::Settled([4, 0])
    );
    assert_eq!(
        meet(&c, [contest, contest], [2, 5]),
        Meeting::Settled([0, 4])
    );
    // A close match, an exact one included, escalates to a fight.
    assert_eq!(meet(&c, [contest, contest], [2, 3]), Meeting::Fight);
    assert_eq!(meet(&c, [contest, contest], [1, 1]), Meeting::Fight);
}

#[test]
fn matching_draws_follow_enumerated_probabilities() {
    // Two members in each of two bins: three matchings, one of them within
    // bins, so within-bin pairs come out a third of the time.
    let mut s = Stream::new(7);
    let trials = 3000;
    let within = (0..trials)
        .filter(|_| s.matching(&[2, 2]).contains_key(&(0, 0)))
        .count();
    assert!((900..=1100).contains(&within), "{within} of {trials}");
    // Three and one: the lone member always pairs across.
    for _ in 0..50 {
        let m = s.matching(&[3, 1]);
        assert_eq!(m, [((0, 0), 1), ((0, 1), 1)].into_iter().collect());
    }
}

#[test]
fn count_splits_keep_totals_and_follow_the_urn() {
    let mut s = Stream::new(11);
    let counts = [5, 0, 12, 3];
    let mut first = 0;
    for _ in 0..2000 {
        let split = s.split(&counts, 8);
        assert_eq!(split.iter().sum::<u64>(), 8);
        assert!(split.iter().zip(&counts).all(|(t, c)| t <= c));
        first += split[0];
    }
    // Expected 8 * 5 / 20 = 2 per draw.
    assert!((3700..=4300).contains(&first), "{first}");
    assert_eq!(s.split(&counts, 20), counts.to_vec());
}

#[test]
fn a_bin_moves_as_its_members_move_one_by_one() {
    let mut world = ProbeFounding {
        seed: 3,
        ..Default::default()
    }
    .generate()
    .unwrap();
    for site in world.genesis.sites.values_mut() {
        site.accounts.insert("world:food".into(), 1_000_000);
    }
    let mut sim = Simulation::new(world.genesis.clone(), Execution::Individuals).unwrap();
    let (&first, group) = sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.kingdom != "kingdom:world" && g.count > 1)
        .unwrap();
    let (entity, count) = (group.entity.clone(), group.count);
    let kind = world.competitions()["world:food"]
        .kinds
        .iter()
        .find(|k| entity.traits.contains(&k.identity))
        .unwrap()
        .clone();
    let mut site = sim.state().sites[&entity.place].clone();
    let round = world.competitions()["world:food"].round.clone();
    let needs = &world.mind().unwrap().needs;
    let processes = [&kind.eat, &kind.spend, &round, "mind:strain", "mind:relief"];
    for process in processes.map(String::from) {
        let p = &world.genesis.rules.processes[&process];
        let before = site.clone();
        let moved = aggregate::apply(p, &entity, &mut site, count, 1, needs).unwrap();
        for id in first..first + count {
            let outcome = sim.execute(id, None, &process, None).outcome;
            let Some(moved) = &moved else {
                assert!(matches!(outcome, Outcome::Blocked(_)), "{process}");
                continue;
            };
            assert_eq!(outcome, Outcome::Accepted, "{process}");
            assert_eq!(
                &aggregate::normalize(sim.state().population.get(id).unwrap().clone()),
                moved
            );
        }
        for (key, value) in &site.accounts {
            assert_eq!(
                sim.state().sites[&entity.place]
                    .accounts
                    .get(key)
                    .copied()
                    .unwrap_or(0),
                *value
            );
        }
        site = before;
        sim = Simulation::new(world.genesis.clone(), Execution::Individuals).unwrap();
    }
}
