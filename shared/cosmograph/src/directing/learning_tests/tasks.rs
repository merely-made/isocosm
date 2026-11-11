//! The M3.6 tasks (delayed reward, player demonstration), cohorts, rows and
//! replay.

use super::mechanics::{critter, slow};
use super::*;
use crate::directing::{Aim, Toward};
use crate::rules::{Binding, Query};

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

/// The delayed-reward world (M3.6): the consumer's home has no producers; the
/// first route out of it leads to the site that has them all, and the other
/// routes lead elsewhere. A move is a journey a few ticks long and pays
/// only when the critter has arrived and eats. To make it choose again and
/// again, feeding needs a hunger (fewer than 14 matter), so a fed critter
/// has nothing to eat and moves off, and goes home to choose once more.
/// Returns the home and the paying site.
fn travel(g: &mut crate::simulation::Genesis) -> (Id, Id) {
    for (id, p) in g.rules.processes.iter_mut() {
        if id.starts_with("move:") {
            p.priority = 500;
            p.period = Some(3);
        }
    }
    let hungry = Query::Below { who: Binding::Actor, key: "matter:1-0".into(), amount: 14 };
    g.rules.processes.get_mut("ecology:feed-1").unwrap().requires.push(hungry);
    let home = g.population.groups.values().find(|x| x.entity.method == Method::Deliberative).unwrap().entity.place;
    let pay = g.sites[&home].routes[0].to;
    for group in g.population.groups.values_mut().filter(|x| x.entity.lineage == "lineage:0") {
        group.entity_mut().place = pay;
    }
    (home, pay)
}

/// How often `c` set out from its home for the paying site, and for any
/// other, along a trail.
fn departures(t: &Trail, home: Id, pay: Id) -> (usize, usize) {
    let leaves = t.windows(2).filter(|w| w[0].0 == home && w[1].0 != home);
    let (paid, other): (Vec<_>, Vec<_>) = leaves.partition(|w| w[1].0 == pay);
    (paid.len(), other.len())
}

/// The delayed-reward task (M3.6): the act that pays is the move to the
/// site where producers feed it, and it pays only after the journey, on
/// arrival, when the critter eats. The served reading is taken as the move
/// ends, when nothing is better, so the move's own reading never credits
/// it; the pending credit does, when the meal that follows serves the
/// critter inside the horizon, and the weight on the move scores it, where
/// the move has no need term of its own to move. A twin that does not learn,
/// and one that learns with a horizon of nought, take the paying route and
/// the others as chance and the tie-break give them.
#[test]
fn a_delayed_reward_is_credited_to_the_move_that_earned_it() {
    let run = |learning, seed| {
        let mut ends = (0, 0);
        let mut s = build(seed, Setup::new(learning, 20), |g| ends = travel(g));
        let c = critter(&s);
        departures(&follow(&mut s, c, 200), ends.0, ends.1)
    };
    let (mut learner, mut twin, mut nought) = ((0, 0), (0, 0), (0, 0));
    for seed in 1..=12 {
        for (total, learning) in [
            (&mut learner, Some(slow())),
            (&mut twin, None),
            (&mut nought, Some(Learning { horizon: 0, ..slow() })),
        ] {
            let (paid, other) = run(learning, seed);
            total.0 += paid;
            total.1 += other;
        }
    }
    let share = |(paid, other): (usize, usize)| paid as f64 / (paid + other).max(1) as f64;
    let say = format!("paying moves of all, over twelve seeds: learner {learner:?}, twin {twin:?}, horizon nought {nought:?}");
    assert!(learner.0 > twin.0 && learner.0 > nought.0, "the learner sets out for the paying site more often: {say}");
    assert!(share(learner) > share(twin) && share(learner) > share(nought), "and more of its moves pay: {say}");
    assert_eq!(twin, nought, "a horizon of nought credits no move: {say}");
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
/// comes later is the delayed-reward task's.
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
