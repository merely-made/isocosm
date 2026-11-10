// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The drift (case 1208; lineage-rounds D9).

use super::tests::learned;
use super::{boundary, drift, revise};
use crate::directing::interim::{Happening, Interim, Pace, Start, no_candidates};
use crate::directing::{found::Played, policy, readings::Mode};
use crate::{Execution, Founding, Session, history::Command, rules::*, schema::*};
use std::sync::Arc;

const SINCE: Tick = 10;

fn rule(s: &mut Session, on: bool) {
    let g = Arc::make_mut(&mut s.sim.genesis);
    g.rules.directing.get_or_insert_with(Directing::default).drift = on;
}

/// A developing line with the drift rule as given, and its template's
/// first tagma's segments.
fn world(on: bool) -> (Session, Key, u8) {
    let (mut s, line, _) = learned();
    rule(&mut s, on);
    let segments = template(&s, &line).recipe.tagmata[0].segments;
    (s, line, segments)
}

fn template(s: &Session, line: &str) -> Development {
    s.sim.state().lineages[line].development.clone().unwrap()
}

/// The line's living groups' first ids.
fn ours(s: &Session, line: &str) -> Vec<Id> {
    let groups = &s.sim.state().population.groups;
    groups.iter().filter(|(_, g)| g.entity.alive && g.entity.lineage == line)
        .map(|(f, _)| *f).collect()
}

/// Every group draws as its own line's template says, none a policy draw.
fn uniform(s: &mut Session) {
    let lines = s.sim.state().lineages.clone();
    for g in s.sim.state.population.groups.values_mut() {
        let d = lines.get(&g.entity.lineage).and_then(|l| l.development.as_ref());
        let soma: Vec<u8> = d.map(|d| d.recipe.tagmata.iter().map(|t| t.segments).collect())
            .unwrap_or_default();
        let e = g.entity_mut();
        e.soma = soma;
        e.drawn = None;
    }
}

/// The group at `id` draws `segments` on tagma 0.
fn draw(s: &mut Session, id: Id, segments: u8) {
    let g = s.sim.state.population.groups.get_mut(&id).unwrap();
    g.entity_mut().soma[0] = segments;
}

/// Plants `n` dead young of `parent`, born at `born`, as a birth writes
/// them (`sim:child` and `sim:parent`).
fn bear(s: &mut Session, parent: Id, born: Tick, n: u64) {
    let mut young = s.sim.state().population.get(parent).unwrap().clone();
    young.alive = false;
    young.born = born;
    for _ in 0..n {
        let child = s.sim.state.population.insert(young.clone(), 1).unwrap();
        s.sim.state.relations.insert(Relation::new(child, "sim:parent", parent));
        s.sim.state.relations.insert(Relation::new(parent, "sim:child", child));
    }
}

fn count(s: &Session, id: Id) -> u64 {
    s.sim.state().population.groups[&id].count
}

/// A line of two groups, the rest dead: the first drawing `segments`, the
/// second `segments + 2`, each with the young given.
fn pair(on: bool, young: [u64; 2]) -> (Session, Key, [Id; 2], u8) {
    let (mut s, line, segments) = world(on);
    uniform(&mut s);
    let ids = ours(&s, &line);
    for id in &ids[2..] {
        s.sim.state.population.groups.get_mut(id).unwrap().entity_mut().alive = false;
    }
    let both = [ids[0], ids[1]];
    draw(&mut s, both[1], segments + 2);
    for (id, n) in both.iter().zip(young) {
        bear(&mut s, *id, SINCE, n);
    }
    (s, line, both, segments)
}

/// What weights `a` on `s` and `b` on `s + 2` average to, nearest.
fn nearest(a: u64, b: u64, s: u8) -> u8 {
    let (a, b) = (u128::from(a), u128::from(b));
    let (q, r) = ((2 * b) / (a + b), (2 * b) % (a + b));
    s + q as u8 + u8::from(2 * r > a + b)
}

#[test]
fn the_rule_off_commits_nothing_whatever_members_hold() {
    let (mut s, line, _, _) = pair(false, [2, 5]);
    let before = s.sim.state().lineages[&line].revision;
    assert!(drift::development(&s.sim, &line, SINCE).is_some(), "there is a pull");
    assert!(drift::commands(&s.sim, &line, SINCE).is_empty());
    assert!(!drift::commit(&mut s, &line, SINCE).unwrap());
    assert_eq!(s.sim.state().lineages[&line].revision, before);
}

#[test]
fn without_variation_the_template_stays_and_a_boundary_hashes_as_before() {
    let (mut on, line, _) = world(true);
    uniform(&mut on);
    let mut off = on.clone();
    rule(&mut off, false);
    for id in ours(&on, &line) {
        bear(&mut on, id, SINCE, 3);
        bear(&mut off, id, SINCE, 3);
    }
    assert!(drift::commands(&on.sim, &line, SINCE).is_empty());
    let a = boundary::adapt(&mut on, None, &revise::revisions, 2).unwrap();
    let b = boundary::adapt(&mut off, None, &revise::revisions, 2).unwrap();
    assert!(a.iter().all(|t| !t.drifted));
    assert_eq!(a, b);
    assert_eq!(on.sim.state_hash(), off.sim.state_hash());
}

#[test]
fn a_group_weighs_its_count_and_its_young_born_in_the_epoch() {
    let (mut s, line, ids, _) = pair(true, [0, 0]);
    let n = count(&s, ids[0]);
    assert_eq!(drift::weights(&s.sim, &line, SINCE)[&ids[0]], n);
    bear(&mut s, ids[0], SINCE, 3);
    bear(&mut s, ids[0], SINCE + 4, 1);
    bear(&mut s, ids[0], SINCE - 1, 5);
    let w = drift::weights(&s.sim, &line, SINCE);
    assert_eq!(w[&ids[0]], n + 4, "the young born before the epoch do not count");
    assert_eq!(w[&ids[1]], count(&s, ids[1]));
    // A dead member has no weight, young or not.
    s.sim.state.population.groups.get_mut(&ids[0]).unwrap().entity_mut().alive = false;
    let w = drift::weights(&s.sim, &line, SINCE);
    assert!(!w.contains_key(&ids[0]));
    assert!(w.keys().all(|id| s.sim.state().population.get(*id).unwrap().alive));
}

#[test]
fn a_variant_moves_the_template_in_proportion_to_its_weight() {
    let (s, line, ids, segments) = pair(true, [1, 1]);
    let (a, b) = (count(&s, ids[0]) + 1, count(&s, ids[1]) + 1);
    let moved = drift::development(&s.sim, &line, SINCE).expect("the template moves");
    assert_eq!(moved.recipe.tagmata[0].segments, nearest(a, b, segments));
    let big = 10 * (a + b);
    let (more, _, _, _) = pair(true, [1, big]);
    let (a, b) = (count(&more, ids[0]) + 1, count(&more, ids[1]) + big);
    let further = drift::development(&more.sim, &line, SINCE).unwrap();
    assert_eq!(further.recipe.tagmata[0].segments, nearest(a, b, segments));
    assert!(further.recipe.tagmata[0].segments > moved.recipe.tagmata[0].segments);
    assert!(further.recipe.tagmata[0].segments <= segments + 2);
    // The commands are the revision, and nothing for a line with no policy.
    let sent = drift::commands(&s.sim, &line, SINCE);
    assert_eq!(sent, vec![Command::Revise { lineage: line.clone(), development: moved }]);
}

#[test]
fn equal_pulls_either_way_leave_the_template_where_it_is() {
    let (mut s, line, ids, segments) = pair(true, [0, 0]);
    // Equal weight on S-1 and S+1: the young make up the difference.
    draw(&mut s, ids[0], segments - 1);
    draw(&mut s, ids[1], segments + 1);
    let (a, b) = (count(&s, ids[0]), count(&s, ids[1]));
    bear(&mut s, ids[0], SINCE, b.saturating_sub(a));
    bear(&mut s, ids[1], SINCE, a.saturating_sub(b));
    let w = drift::weights(&s.sim, &line, SINCE);
    assert_eq!(w[&ids[0]], w[&ids[1]]);
    assert_eq!(drift::development(&s.sim, &line, SINCE), None);
    assert!(drift::commands(&s.sim, &line, SINCE).is_empty());
}

#[test]
fn an_exact_half_rounds_toward_the_value_now_held() {
    // Equal weight on S and S+1: the mean is S plus a half.
    for held_high in [false, true] {
        let (mut s, line, ids, segments) = pair(true, [0, 0]);
        draw(&mut s, ids[1], segments + 1);
        let (a, b) = (count(&s, ids[0]), count(&s, ids[1]));
        bear(&mut s, ids[0], SINCE, b.saturating_sub(a));
        bear(&mut s, ids[1], SINCE, a.saturating_sub(b));
        if held_high {
            let mut d = template(&s, &line);
            d.recipe.tagmata[0].segments = segments + 1;
            s.command(Command::Revise { lineage: line.clone(), development: d }).unwrap();
        }
        // Held at S it stays at S; held at S+1 it stays at S+1.
        assert_eq!(drift::development(&s.sim, &line, SINCE), None, "held high: {held_high}");
    }
    // Held at S, the half between S+1 and S+2 goes to S+1, the nearer.
    let (mut s, line, ids, segments) = pair(true, [0, 0]);
    draw(&mut s, ids[0], segments + 1);
    draw(&mut s, ids[1], segments + 2);
    let (a, b) = (count(&s, ids[0]), count(&s, ids[1]));
    bear(&mut s, ids[0], SINCE, b.saturating_sub(a));
    bear(&mut s, ids[1], SINCE, a.saturating_sub(b));
    let moved = drift::development(&s.sim, &line, SINCE).expect("S is not the mean");
    assert_eq!(moved.recipe.tagmata[0].segments, segments + 1);
}

/// A line with a policy, its members each with a draw of their own.
fn minded() -> (Session, Key) {
    let mut g = Founding {
        seed: 11, sites: 4, population: 48, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Default::default()
    }.generate().unwrap();
    let d = g.rules.directing.get_or_insert_with(Directing::default);
    d.drift = true;
    d.policy = PolicyVariation { founding: 12, expression: 12, candidates: 2 };
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let line = s.sim.state().lineages.keys()
        .find(|l| boundary::standing(&s, l).1 > 0).unwrap().clone();
    let found = policy::founding(&s, &line);
    s.command(found).unwrap();
    (s, line)
}

#[test]
fn drawn_members_with_young_pull_the_policy_to_their_expression() {
    let (mut s, line) = minded();
    let had = s.sim.state().lineages[&line].policy.clone().unwrap();
    let ids = ours(&s, &line);
    assert!(ids.len() >= 2);
    assert!(ids.iter().all(|id| s.sim.state().population.get(*id).unwrap().drawn.is_some()));
    bear(&mut s, ids[0], SINCE, 9);
    let w = drift::weights(&s.sim, &line, SINCE);
    let size = s.sim.genesis().rules.directing().policy.expression;
    let express = |id: &Id| had.varied(s.sim.state().population.get(*id).unwrap().drawn.unwrap(), size);
    let total: i128 = w.values().map(|&v| i128::from(v)).sum();
    let sum: i128 = w.iter().map(|(id, &v)| i128::from(v) * i128::from(express(id).priority)).sum();
    let moved = drift::policy(&s.sim, &line, SINCE).expect("the policy moves");
    // Nearest to the weighted mean, whichever way a half goes.
    assert!((2 * (i128::from(moved.priority) * total - sum)).abs() <= total);
    let mut sums = vec![0i128; had.drives.len()];
    for (id, &v) in &w {
        for (k, d) in express(id).drives.iter().enumerate() {
            sums[k] += i128::from(v) * i128::from(*d);
        }
    }
    for (k, d) in moved.drives.iter().enumerate() {
        assert!((2 * (i128::from(*d) * total - sums[k])).abs() <= total, "drive {k}");
    }
    assert!(moved != had, "the members' draws pulled it");
    let sent = drift::commands(&s.sim, &line, SINCE);
    assert!(sent.contains(&Command::Policy { lineage: line.clone(), policy: moved }));
    // With no draws there is nothing to average but the policy itself.
    for g in s.sim.state.population.groups.values_mut() {
        g.entity_mut().drawn = None;
    }
    assert_eq!(drift::policy(&s.sim, &line, SINCE), None);
}

#[test]
fn a_boundary_commits_the_drift_before_scoring() {
    let (s, line, _, _) = pair(true, [3, 8]);
    let before = s.sim.state().lineages[&line].revision;
    let stay_wins = &mut |_: &Session, _: &str, c: &boundary::Candidate, ticks: Tick| {
        let held = if c.commands.is_empty() { 10 } else { 1 };
        Ok(boundary::Score { held, members: 1, ticks })
    };
    let mut run = |mut s: Session| {
        let turns = boundary::adapt_since_using(&mut s, None, SINCE, &revise::revisions, 2, stay_wins)
            .unwrap();
        let turn = turns.into_iter().find(|t| t.lineage == line);
        (s.sim.state().lineages[&line].revision, turn)
    };
    let (revision, turn) = run(s.clone());
    let turn = turn.expect("the line took a turn");
    assert!(turn.drifted);
    assert_eq!(turn.committed, None, "no candidate beat staying");
    assert!(turn.considered.len() > 1, "candidates were weighed");
    assert_eq!(revision, before + 1, "the drift alone");
    let mut off = s;
    rule(&mut off, false);
    let (revision, turn) = run(off);
    assert_eq!(revision, before);
    assert!(!turn.unwrap().drifted);
}

const PACE: Pace = Pace { round: 4, scoring: 6 };

fn playing(seed: u64, on: bool) -> Interim {
    let policy = PolicyVariation { founding: 12, expression: 12, candidates: 0 };
    let mut g = Founding {
        seed, sites: 5, population: 60, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }),
        ..Default::default()
    }.generate().unwrap();
    g.rules.epoch_ticks = 8;
    let d = g.rules.directing.get_or_insert_with(Directing::default);
    d.drift = on;
    d.policy = policy;
    for l in g.lineages.values_mut() {
        if let Some(d) = l.development.as_mut() {
            d.recipe.variance = 2;
        }
    }
    let start = Start { within: 0, epochs: 1, ..Start::default() };
    let mut i = Interim::found(g, start, Mode::Creative, PACE, Execution::Individuals).unwrap();
    i.ending.since = i.session.sim.state().tick;
    // The played line founds a policy, each member drawing around it.
    let played = i.session.sim.plays(i.participant).unwrap();
    let line = i.session.sim.state().population.get(played).unwrap().lineage.clone();
    let found = policy::founding(&i.session, &line);
    i.session.command(found).unwrap();
    i
}

#[test]
fn the_played_line_drifts_at_a_boundary_too() {
    for on in [true, false] {
        let mut i = playing(101, on);
        let played = i.session.sim.plays(i.participant).unwrap();
        let line = i.session.sim.state().population.get(played).unwrap().lineage.clone();
        let had = i.session.sim.state().lineages[&line].policy.clone().unwrap();
        let mut fired = false;
        for _ in 0..3 {
            let out = i.round(&no_candidates).unwrap();
            fired |= out.iter().any(|h| matches!(h, Happening::Boundary { .. }));
            if fired { break; }
        }
        assert!(fired, "a boundary fired");
        let now = i.session.sim.state().lineages[&line].policy.clone().unwrap();
        // Its members drew around the policy, so their average is not it.
        assert_eq!(now != had, on, "the played line's template drifted: {on}");
    }
}

#[test]
fn a_run_with_the_rule_on_replays_to_the_same_hash() {
    for seed in [101u64, 202] {
        let mut i = playing(seed, true);
        let policies = |i: &Interim| i.session.sim.state().lineages.values()
            .map(|l| l.policy.clone()).collect::<Vec<_>>();
        let founded = policies(&i);
        let mut drifted = false;
        for _ in 0..6 {
            let out = i.round(&no_candidates).unwrap();
            let turns = out.iter().filter_map(|h| match h {
                Happening::Boundary { turns, .. } => Some(turns),
                _ => None,
            });
            drifted |= turns.flatten().any(|t| t.drifted);
            if out.iter().any(|h| matches!(h, Happening::LineageEnded { .. })) { break; }
        }
        let json = serde_json::to_vec(&i.session.save()).unwrap();
        let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
        assert_eq!(replayed.sim.state_hash(), i.session.sim.state_hash(), "seed {seed}");
        assert_ne!(policies(&i), founded, "seed {seed}: the played line's policy drifted");
        eprintln!("seed {seed}: an unplayed line drifted: {drifted}");
    }
}

/// The group at `id` drew an absence at each of `n` segments of tagma 0.
fn lack(s: &mut Session, id: Id, n: u8) {
    s.sim.state.population.groups.get_mut(&id).unwrap().entity_mut().absent = (0..n).map(|k| (0, k)).collect();
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn odds(s: &Session, line: &str) -> [u32; 2] {
    template(s, line).recipe.absence
}

/// A world with the rule on, and a developing line whose tagmata could
/// draw an absence, its first two groups alive with the young given.
fn lacking(young: [u64; 2]) -> (Session, Key, [Id; 2]) {
    let (mut s, _, _) = world(true);
    uniform(&mut s);
    let line = s.sim.state().lineages.iter().find(|(k, l)| {
        let rules = &s.sim.genesis().rules;
        l.development.as_ref().is_some_and(|d| d.recipe.tagmata.iter().any(|t| crate::development::may_lack(rules, t)))
            && ours(&s, k).len() >= 2
    }).map(|(k, _)| k.clone()).expect("a line that could lack");
    let ids = ours(&s, &line);
    for id in &ids[2..] {
        s.sim.state.population.groups.get_mut(id).unwrap().entity_mut().alive = false;
    }
    for (id, n) in ids.iter().zip(young) {
        bear(&mut s, *id, SINCE, n);
    }
    (s, line, [ids[0], ids[1]])
}

/// How many tagmata of the line's recipe could draw an absence.
fn eligible(s: &Session, line: &str) -> u128 {
    let rules = &s.sim.genesis().rules;
    template(s, line).recipe.tagmata.iter().filter(|t| crate::development::may_lack(rules, t)).count() as u128
}

/// A seed whose soma draws an absence from the line's recipe.
fn lacking_seed(s: &Session, line: &str) -> u64 {
    let (rules, d) = (&s.sim.genesis().rules, template(s, line));
    (0..4000).find(|&seed| !crate::development::soma(rules, &d.recipe, seed).absent.is_empty())
        .expect("a seed that draws an absence")
}

/// A newborn's drawn absences, as developed on arrival, under the rule as given.
fn arrive(on: bool) -> [Vec<(u8, u8)>; 2] {
    let (mut s, line, _) = lacking([0, 0]);
    rule(&mut s, on);
    let seed = lacking_seed(&s, &line);
    let id = ours(&s, &line)[0];
    let e = s.sim.state.population.groups.get_mut(&id).unwrap().entity_mut();
    (e.body, e.parts, e.accounts) = (None, Default::default(), Default::default());
    s.sim.develop_arrival(id, seed).unwrap();
    let drew = crate::development::soma(&s.sim.genesis().rules, &template(&s, &line).recipe, seed).absent;
    [s.sim.state().population.groups[&id].entity.absent.clone(), drew]
}

#[test]
fn absences_are_recorded_only_where_the_rule_reads_them() {
    let [off, drew] = arrive(false);
    assert!(off.is_empty() && !drew.is_empty(), "rule off: nothing recorded");
    let [on, drew] = arrive(true);
    assert_eq!(on, drew, "rule on: the newborn records what it drew");
}

#[test]
fn absences_drawn_by_breeders_raise_the_odds_to_the_weighted_frequency() {
    let (mut s, line, ids) = lacking([2, 5]);
    let w = drift::weights(&s.sim, &line, SINCE);
    let (w0, w1) = (u128::from(w[&ids[0]]), u128::from(w[&ids[1]]));
    lack(&mut s, ids[0], 1);
    lack(&mut s, ids[1], 0);
    let (drawn, of) = (w0, (w0 + w1) * eligible(&s, &line));
    let g = gcd(drawn, of);
    let moved = drift::development(&s.sim, &line, SINCE).expect("the odds move");
    assert_eq!(moved.recipe.absence, [(drawn / g) as u32, (of / g) as u32]);
    assert!(s.sim.revisable(&line, &moved).is_ok());
    assert!(drift::commands(&s.sim, &line, SINCE).iter().any(|c| matches!(c, Command::Revise { .. })));
}

#[test]
fn no_absences_drawn_in_a_recorded_world_drops_the_odds_to_zero() {
    let (mut s, line, ids) = lacking([0, 0]);
    let other = *s.sim.state().population.groups.keys().find(|i| !ids.contains(i)).unwrap();
    lack(&mut s, other, 1);
    let moved = drift::development(&s.sim, &line, SINCE).expect("the odds move");
    assert_eq!(moved.recipe.absence, [0, 1]);
    assert!(s.sim.revisable(&line, &moved).is_ok());
}

#[test]
fn an_observed_frequency_equal_to_the_odds_moves_nothing() {
    let (mut s, line, ids) = lacking([0, 0]);
    lack(&mut s, ids[0], 1);
    let w = drift::weights(&s.sim, &line, SINCE);
    let (drawn, of) = (u128::from(w[&ids[0]]), w.values().map(|&n| u128::from(n)).sum::<u128>() * eligible(&s, &line));
    let g = gcd(drawn, of);
    let same = [(drawn / g) as u32, (of / g) as u32];
    s.sim.state.lineages.get_mut(&line).unwrap().development.as_mut().unwrap().recipe.absence = same;
    assert_eq!(odds(&s, &line), same);
    assert!(drift::development(&s.sim, &line, SINCE).is_none());
}

#[test]
#[ignore = "a readout, not a check"]
fn drift_cost() {
    let mut g = Founding {
        seed: 31, lineages: 1, population: 600, cohort_size: 1,
        bodies: Some(crate::bodied::Bodies::default()), ..Default::default()
    }.generate().unwrap();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let line = s.sim.state().lineages.iter().find(|(_, l)| l.development.is_some())
        .map(|(k, _)| k.clone()).unwrap();
    let ids = ours(&s, &line);
    for (k, id) in ids.iter().enumerate() {
        let g = s.sim.state.population.groups.get_mut(id).unwrap();
        for x in g.entity_mut().soma.iter_mut() { *x = x.saturating_add((k % 3) as u8); }
        bear(&mut s, *id, SINCE, (k % 4) as u64);
    }
    let start = std::time::Instant::now();
    let sent = drift::commands(&s.sim, &line, SINCE);
    let took = start.elapsed();
    eprintln!("drift_cost: {} members, {} commands, {took:?}", ids.len(), sent.len());
}
