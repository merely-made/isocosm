//! The M3.6 tasks (delayed reward, player demonstration), cohorts, rows and
//! replay.

use super::mechanics::{critter, slow};
use super::*;
use crate::directing::{Aim, Toward};

/// Where `c` stood, tick by tick: its place, its matter, its stored delta
/// and the tick of the journey it was on, if any.
type Trail = Vec<(Id, u64, Option<i32>, Option<Tick>)>;

fn follow(s: &mut Session, c: Id, ticks: u32) -> Trail {
    (0..ticks)
        .map(|_| {
            s.advance(1).unwrap();
            let e = s.sim.state().population.get(c).unwrap();
            (e.place, *e.accounts.get("matter:1-0").unwrap_or(&0), stored(s, c), e.journey.map(|j| j.departed))
        })
        .collect()
}

fn moves(t: &Trail) -> usize {
    t.windows(2).filter(|w| w[0].0 != w[1].0).count()
}

/// The delayed-reward task (M3.6): the act that pays is a move to a site
/// where producers feed it, and it pays only when the critter has arrived
/// and eats. With the served reading as built, the move itself is never
/// credited: the reading is taken as the act ends, the move feeds nothing
/// so its terms name no need for the delta to move, and the meal that pays
/// is credited to the meal. A move is a journey here, a few ticks long. So the learner goes no more often than its
/// twin. This test states that limit; a credit that spans the delay is
/// owed (see the report of this cut).
#[test]
fn a_delayed_reward_is_not_credited_to_the_act_that_earned_it() {
    let travel = |g: &mut crate::simulation::Genesis| {
        for (id, p) in g.rules.processes.iter_mut() {
            if id.starts_with("move:") {
                p.priority = 500;
                p.period = Some(3);
            }
        }
    };
    let run = |learning, seed| {
        let mut s = build(seed, Setup::new(learning, 20), travel);
        let c = critter(&s);
        follow(&mut s, c, 60)
    };
    let (mut travelled, mut departures) = (0, 0);
    for seed in 1..8 {
        let (learner, twin) = (run(Some(slow()), seed), run(None, seed));
        assert_eq!(moves(&learner), moves(&twin), "seed {seed}: no more moves with learning on");
        assert_eq!(
            learner.iter().map(|t| (t.0, t.1)).collect::<Vec<_>>(),
            twin.iter().map(|t| (t.0, t.1)).collect::<Vec<_>>(),
            "seed {seed}: the same life"
        );
        // The tick it set out on a journey, its delta did not move.
        let mut was = None;
        for (k, now) in learner.iter().enumerate() {
            if now.3 == Some(k as Tick + 1) {
                assert_eq!(was, Some(now.2), "seed {seed}: a move was credited");
                departures += 1;
            }
            was = Some(now.2);
        }
        assert!(learner.last().unwrap().2.is_some(), "its meals were credited");
        travelled += moves(&learner);
    }
    assert!(travelled > 0 && departures > 0, "the fixture never sent a critter travelling");
}

fn nudge_toward_prey(s: &mut Session, c: Id) {
    let pop = &s.sim.state().population;
    let place = pop.get(c).unwrap().place;
    let thing = *pop.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.place == place && g.entity.lineage == "lineage:0").unwrap().0;
    let p = s.sim.bonded(c)[0].0;
    s.command(Command::Nudge { act: Some("ecology:feed-1".into()), participant: p, critter: c, aim: Aim::Act, toward: Toward::Thing(thing) }).unwrap();
}

/// The player-demonstration task (M3.6): feeding scores below the act that
/// competes with it, so left alone a critter does not feed. A player nudges
/// one toward feeding, which serves it; after the nudges end it feeds
/// un-nudged, which a twin never nudged does not, and nor does one nudged
/// as often but not learning. The act serves it at once here; a payoff that
/// comes later is the delayed-reward task's limit.
#[test]
fn a_member_nudged_toward_an_act_that_serves_it_takes_it_up_unprompted() {
    let both = |g: &mut crate::simulation::Genesis| {
        for id in ["ecology:feed-1", "ecology:upkeep-1"] {
            g.rules.processes.get_mut(id).unwrap().period = Some(1);
        }
    };
    let make = |seed, learning| build(seed, Setup::new(learning, 1300), both);
    let (seed, _) = fed_seed(|seed| make(seed, Some(Learning { bound: 32, span: 1_000_000, ..Learning::default() })));
    let learning = Some(Learning { bound: 32, span: 1_000_000, ..Learning::default() });
    let meals = |learning, nudged: bool| {
        let mut s = make(seed, learning);
        let c = critter(&s);
        for _ in 0..6 {
            if nudged {
                nudge_toward_prey(&mut s, c);
            }
            s.advance(1).unwrap();
        }
        let learnt = stored(&s, c);
        let trail = follow(&mut s, c, 8);
        let ate = trail.windows(2).filter(|w| w[1].1 > w[0].1).count();
        (ate, learnt)
    };
    let (taught, learnt) = meals(learning, true);
    let (never, none) = meals(learning, false);
    let (unlearning, _) = meals(None, true);
    assert_eq!(learnt, Some(24), "six served acts of a step of four");
    assert_eq!(none, None);
    assert!(taught >= 6, "the nudged learner feeds on its own: {taught} of 8");
    assert_eq!((never, unlearning), (0, 0));
}
