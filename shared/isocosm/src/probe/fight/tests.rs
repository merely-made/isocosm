// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{super::aggregate, *};
use crate::{
    probe::{ProbeFounding, ProbeWorld},
    rules::Effect,
};

/// Two members of a drawn world fighting through the interpreter's
/// meanings, as the crowd's copies do, without remembering.
struct Plain<'a> {
    world: &'a ProbeWorld,
    states: [Entity; 2],
}

impl Sides for Plain<'_> {
    fn act(&mut self, side: usize, process: &str) -> Result<()> {
        let p = &self.world.genesis.rules.processes[process];
        let mut site = self.world.genesis.sites[&0].clone();
        let needs = &self.world.mind()?.needs;
        let next = aggregate::apply(p, &self.states[side], &mut site, 1, 1, needs)?;
        self.states[side] = next.ok_or("blocked")?;
        Ok(())
    }
    fn member(&self, side: usize) -> &Entity {
        &self.states[side]
    }
}

fn world(edit: impl Fn(&mut Competition, &mut Mind)) -> ProbeWorld {
    let mut w = ProbeFounding {
        seed: 11,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let rules = &mut w.genesis.rules;
    let c = rules.competitions.get_mut("world:food").unwrap();
    edit(c, rules.mind.as_mut().unwrap());
    w
}

/// A member of kind 0 that contests, with the given reserve and strain.
fn member(w: &ProbeWorld, reserve: u64, strain: u64) -> Entity {
    let c = &w.competitions()["world:food"];
    let group = w.genesis.population.groups.values();
    let mut e = group
        .map(|g| g.entity.clone())
        .find(|e| e.traits.contains(&c.kinds[0].identity))
        .unwrap();
    e.traits.insert(c.contest.clone());
    e.accounts.insert(c.kinds[0].body.clone(), reserve);
    e.accounts.insert(w.mind().unwrap().strain.clone(), strain);
    e
}

fn run(w: &ProbeWorld, reserves: [u64; 2], strains: [u64; 2], seed: u64) -> (Fight, [Entity; 2]) {
    let c = &w.competitions()["world:food"];
    let states = [0, 1].map(|x| member(w, reserves[x], strains[x]));
    let mut sides = Plain { world: w, states };
    let site = w.genesis.sites[&0].clone();
    let ground = Ground {
        site: &site,
        tick: 1,
    };
    let kinds = [&c.kinds[0], &c.kinds[0]];
    let mind = w.mind().unwrap();
    let f = fight(c, mind, kinds, &ground, &mut sides, &mut Stream::new(seed)).unwrap();
    (f, sides.states)
}

fn reserve(w: &ProbeWorld, e: &Entity) -> u64 {
    value(&e.accounts, &w.competitions()["world:food"].kinds[0].body)
}

#[test]
fn fights_end_when_a_side_yields_or_is_spent() {
    let w = world(|_, _| {});
    let c = &w.competitions()["world:food"];
    for seed in 0..300 {
        let reserves = [1 + seed % 7, 1 + (seed / 7) % 7];
        let (f, [a, b]) = run(&w, reserves, [seed % 5, seed % 3], seed);
        let loser = [&a, &b][1 - f.winner];
        let spent = reserve(&w, loser) == 0;
        let shift = |side: usize| match f.breaks[side] {
            Some(true) => c.advantage as i64,
            Some(false) => -(c.advantage as i64),
            None => 0,
        };
        let standing = |e, side| reserve(&w, e) as i64 + shift(side);
        let gap = standing(&a, 0).abs_diff(standing(&b, 1));
        assert!(spent || gap > c.margin, "seed {seed}: {f:?}");
        // Both take every round; only the loser of an exchange spends.
        let rounds = |side: usize| f.acts[side].iter().filter(|a| **a == Act::Round).count();
        assert_eq!(rounds(0), rounds(1));
        for (side, e) in [&a, &b].into_iter().enumerate() {
            let spends = f.acts[side].len() - rounds(side);
            assert_eq!(reserve(&w, e), reserves[side] - spends as u64);
        }
    }
}

#[test]
fn neither_side_wins_automatically() {
    // Equal reserves: either side can win. A reserve one below the other's
    // still wins some fights, through upsets and breaks.
    let w = world(|c, m| {
        c.margin = 2;
        c.upset = 200;
        m.bearing = 2;
    });
    let wins = |reserves| {
        (0..400)
            .filter(|&seed| run(&w, reserves, [0, 0], seed).0.winner == 0)
            .count()
    };
    let even = wins([4, 4]);
    assert!((160..=240).contains(&even), "{even} of 400");
    let under = wins([3, 4]);
    assert!((30..=130).contains(&under), "{under} of 400");
}

#[test]
fn without_upsets_or_breaks_the_higher_standing_wins() {
    let w = world(|c, m| {
        c.margin = 3;
        c.upset = 0;
        m.bearing = 1_000;
        m.bearing_traits.clear();
    });
    for seed in 0..100 {
        let (f, _) = run(&w, [4, 5], [0, 0], seed);
        assert_eq!(f.winner, 1);
        assert_eq!(f.breaks, [None, None]);
        assert!(f.acts[1].iter().all(|a| *a == Act::Round));
    }
}

#[test]
fn a_break_moves_standing_and_decides_nothing() {
    // Every mind is past its bearing after the first round.
    let past = |rise| {
        world(move |c, m| {
            c.margin = 3;
            m.bearing = -1;
            m.bearing_traits.clear();
            m.rise = rise;
            m.rise_traits.clear();
            m.stake = 0;
        })
    };
    let (up, down) = (past(1_000), past(0));
    let mut longer = 0;
    for seed in 0..100 {
        let (f, _) = run(&up, [4, 4], [0, 0], seed);
        assert_eq!(f.breaks, [Some(true), Some(true)]);
        let (f, _) = run(&down, [4, 4], [0, 0], seed);
        assert_eq!(f.breaks, [Some(false), Some(false)]);
        // Breaking both ways leaves the match as close as it was, so fights
        // go on past the first round.
        longer += usize::from(f.acts[0].len() > 1);
    }
    assert!(longer > 50, "{longer}");
}

#[test]
fn strain_grows_by_the_round_and_by_what_is_spent() {
    let w = world(|_, _| {});
    let rules = &w.genesis.rules;
    let per = |process: &str| -> u64 {
        let effects = &rules.processes[process].effects;
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Transform { give, .. } => give.get(&w.mind().unwrap().strain).copied(),
                _ => None,
            })
            .sum()
    };
    let c = &w.competitions()["world:food"];
    let (round, spend) = (per(&c.round), per(&c.kinds[0].spend));
    for seed in 0..100 {
        let (f, states) = run(&w, [5, 5], [1, 2], seed);
        for (side, e) in states.iter().enumerate() {
            let rounds = f.acts[side].iter().filter(|a| **a == Act::Round).count() as u64;
            let spends = f.acts[side].len() as u64 - rounds;
            let strain = value(&e.accounts, &w.mind().unwrap().strain);
            assert_eq!(strain, 1 + side as u64 + rounds * round + spends * spend);
        }
    }
}

#[test]
fn mood_is_read_from_needs_and_moves_the_way_a_break_goes() {
    let w = world(|_, _| {});
    let mind = w.mind().unwrap();
    let site = &w.genesis.sites[&0];
    let mood = |reserve| {
        let e = member(&w, reserve, 0);
        let seen = aggregate::Seen {
            member: Some(&e),
            site,
            tick: 1,
            needs: &mind.needs,
        };
        seen.mood().unwrap()
    };
    // Content with a full reserve; each need met lowers mood further.
    assert_eq!(mood(100), 0);
    assert!(mood(1) < mood(100));
    let weights: i64 = mind.needs.iter().take(3).map(|n| n.weight).sum();
    assert_eq!(mood(1), weights);
    let e = member(&w, 1, 0);
    let expected = (mind.rise
        + e.traits
            .iter()
            .filter_map(|t| mind.rise_traits.get(t))
            .sum::<i64>()
        + mind.stake * weights)
        .clamp(0, 1000) as u64;
    assert_eq!(rise(mind, &e, weights), expected);
}
