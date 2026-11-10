// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Known lineages and regard (cases 1210, 1214; lineage-rounds LR5).

use super::interim::{Happening, Interim, Pace, Start, no_candidates, regard::{dealings, regarded}};
use super::readings::{self, Mode, known};
use super::tests::{founding, played};
use super::*;
use crate::{
    rules::Directing,
    simulation::Genesis,
    Execution, Founding, Session,
    flows::{Flow, Holder, MadeBy, Kind},
    history::Command,
    lineage::draft,
    rules::Regard,
};
use std::collections::BTreeSet;

const PACE: Pace = Pace { round: 4, scoring: 6 };

fn genesis(seed: u64, rule: bool) -> Genesis {
    let mut g = Founding {
        seed, sites: 5, population: 60, cohort_size: 4, lineages: 3, ecology: true,
        played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default()
    }.generate().unwrap();
    g.rules.epoch_ticks = 8;
    if rule {
        g.rules.directing.as_mut().unwrap().regard = Some(Regard::default());
    }
    g
}

fn interim(seed: u64, rule: bool) -> Interim {
    Interim::found(genesis(seed, rule), Start::default(), Mode::Creative, PACE, Execution::Individuals).unwrap()
}

/// The played critter's lineage.
fn line(i: &Interim) -> Key {
    i.session.sim.state().population.get(i.critter().unwrap()).unwrap().lineage.clone()
}

/// The lineages the participant knows by a `directing:knows` relation.
fn met(s: &Session, p: Id) -> BTreeSet<Key> {
    let pop = &s.sim.state().population;
    s.sim.state().relations.iter().filter(|r| r.subject == p && r.kind == KNOWS)
        .map(|r| pop.get(r.object).unwrap().lineage.clone()).collect()
}

fn regards(s: &Session) -> usize {
    s.entries.iter().filter(|e| matches!(e.command, Command::Regard { .. })).count()
}

fn set(keys: &[&str]) -> BTreeSet<Key> { keys.iter().map(|k| k.to_string()).collect() }

/// A move of `amount` a member for `count` members from one line's body to another's.
fn flow(from: &str, to: &str, amount: u64, count: u64) -> Flow {
    let kind = |l: &str| Some(Kind { lineage: l.into(), kingdom: "kingdom:fauna".into() });
    Flow {
        tick: 0, made_by: MadeBy::Process("test:give".into()),
        from: (Holder::Entity(0), "matter:x".into()), to: (Holder::Entity(1), "matter:x".into()),
        amount, count, from_kind: kind(from), to_kind: kind(to), composition: None,
    }
}

/// A round whose first tick also carries `planted` moves.
fn round_with(i: &mut Interim, planted: Vec<Flow>) -> Vec<Happening> {
    let mut once = Some(planted);
    i.round_with_flows_using(&no_candidates, &mut |s| {
        let mut flows = s.advance_tick_with_flows()?.flows;
        flows.extend(once.take().unwrap_or_default());
        Ok(flows)
    }).unwrap().0
}

/// A line that exists to be dealt with, holding no one.
fn ally(i: &mut Interim) -> Key {
    let mut l = i.session.sim.state().lineages["lineage:0"].clone();
    l.parent = None;
    i.session.sim.state.lineages.insert("lineage:ally".into(), l);
    "lineage:ally".into()
}

#[test]
fn a_world_without_the_rule_keeps_and_records_nothing() {
    let mut i = interim(5, false);
    for _ in 0..6 { round_with(&mut i, vec![flow("lineage:1", "lineage:0", 100, 1)]); }
    let (s, p) = (&i.session, i.participant);
    assert_eq!(regards(s), 0);
    assert!(s.sim.state().relations.iter().all(|r| r.kind != KNOWS));
    assert!(s.sim.state().lineages.values().all(|l| l.regard.is_empty() && l.ate.is_empty()));
    assert!(known(s, p).is_empty() && readings::regard(&s.sim, p).is_empty());
    let json = serde_json::to_string(&s.sim.state().lineages).unwrap();
    assert!(!json.contains("regard") && !json.contains("\"ate\""), "unset, the fields are not serialized");
    let mut twin = interim(5, false);
    for _ in 0..6 { round_with(&mut twin, vec![flow("lineage:1", "lineage:0", 100, 1)]); }
    assert_eq!(s.sim.state_hash(), twin.session.sim.state_hash());
    let command = Command::Regard { participant: p, knows: vec![], regard: vec![] };
    assert!(i.session.command(command).is_err(), "unset, a regard is refused");
}

#[test]
fn playing_a_lineage_knows_it_and_a_fork_knows_its_kin() {
    let mut s = Session::new(rule(founding(3)), Execution::Individuals).unwrap();
    let p: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    assert!(known(&s, p).is_empty(), "nobody known before playing");
    let (kin, other) = pair(&s);
    s.command(Command::Take { participant: p, critter: other }).unwrap();
    assert_eq!(known(&s, p), set(&["lineage:1"]), "a played lineage, and nothing else");
    s.command(Command::Speciate { founder: kin, name: "split".into() }).unwrap();
    assert_eq!(known(&s, p), set(&["lineage:1", "lineage:split"]), "a descendant of a played one");
    // A second participant plays only the fork and knows its ancestor.
    let q: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    s.command(Command::Take { participant: q, critter: kin }).unwrap();
    assert_eq!(known(&s, q), set(&["lineage:1", "lineage:split"]), "an ancestor of a played one");
    assert!(!known(&s, q).contains("lineage:0") && !known(&s, q).contains("lineage:2"));
}

/// Two living deliberative members of the first such cohort, for a fork and a player.
fn pair(s: &Session) -> (Id, Id) {
    let (first, _) = s.sim.state().population.groups.iter()
        .find(|(_, g)| g.entity.alive && g.entity.method == Method::Deliberative && g.count >= 2).unwrap();
    (*first, *first + 1)
}

fn rule(mut g: Genesis) -> Genesis {
    g.rules.directing.as_mut().unwrap().regard = Some(Regard::default());
    g
}

#[test]
fn a_whole_meal_knows_the_donor_only_when_the_rule_is_set() {
    for on in [false, true] {
        let mut g = founding(3);
        if on { g = rule(g); }
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let (p, c) = played(&mut s);
        let eater = s.sim.state().population.get(c).unwrap().lineage.clone();
        let rules = s.sim.genesis().rules.clone();
        let kind: Key = "kind:frond".into();
        draft::learn(&mut s.sim.state.lineages, &rules, &eater, "lineage:0", &kind, 5);
        draft::learn(&mut s.sim.state.lineages, &rules, &eater, &eater.clone(), &kind, 6);
        if on {
            assert_eq!(s.sim.state().lineages[&eater].ate, set(&["lineage:0"]), "a meal of another line, not its own");
            assert_eq!(known(&s, p), set(&["lineage:0", &eater]));
        } else {
            assert!(s.sim.state().lineages[&eater].ate.is_empty() && known(&s, p).is_empty());
        }
    }
}

/// Moves every living body but the played line's to `far`, bar those named `stay` (by lineage, the first of each) to `here`.
fn crowd(i: &mut Interim, far: Id, stay: &[(&str, Id)]) {
    let own = line(i);
    let mut placed = BTreeSet::new();
    for g in i.session.sim.state.population.groups.values_mut() {
        let e = &g.entity;
        if !e.alive || is_participant(e) || e.lineage == own { continue; }
        let to = match stay.iter().find(|(l, _)| *l == e.lineage) {
            Some((l, here)) if placed.insert(*l) => *here,
            _ => far,
        };
        g.entity_mut().place = to;
    }
}

#[test]
fn a_lineage_at_the_played_site_is_met_and_one_elsewhere_is_not() {
    let mut i = interim(5, true);
    let here = i.session.sim.state().population.get(i.critter().unwrap()).unwrap().resident_site().unwrap();
    let far = *i.session.sim.state().sites.keys().find(|s| **s != here).unwrap();
    crowd(&mut i, far, &[("lineage:0", here)]);
    let at = |i: &Interim, l: &str| i.session.sim.state().population.groups.values()
        .any(|g| g.entity.alive && g.entity.lineage == l && g.entity.resident_site() == Some(here));
    assert!(at(&i, "lineage:0") && !at(&i, "lineage:2"));
    // Round without ticks, so nothing moves between planting and reading.
    i.pace = Pace { round: 0, ..PACE };
    round_with(&mut i, vec![]);
    assert_eq!(met(&i.session, i.participant), set(&["lineage:0"]));
    assert_eq!(known(&i.session, i.participant), set(&["lineage:0", "lineage:1"]));
    let n = regards(&i.session);
    round_with(&mut i, vec![]);
    assert_eq!(regards(&i.session), n, "a lineage already known is not recorded again");
}

#[test]
fn a_boundary_knows_the_lineages_the_played_one_outholds() {
    for seed in 1..80 {
        let mut i = interim(seed, true);
        let epoch = 8;
        while (i.session.sim.state().tick + PACE.round) / epoch == i.session.sim.state().tick / epoch {
            round_with(&mut i, vec![]);
        }
        let own = line(&i);
        let far = *i.session.sim.state().sites.keys().next_back().unwrap();
        crowd(&mut i, far, &[]);
        let before = met(&i.session, i.participant);
        let happened = round_with(&mut i, vec![]);
        if !happened.iter().any(|h| matches!(h, Happening::Boundary { .. })) { continue; }
        let held = |l: &str| crate::lineage::boundary::standing(&i.session, l).0;
        let alive: BTreeSet<Key> = i.session.sim.state().population.groups.values()
            .filter(|g| g.entity.alive && !is_participant(&g.entity) && g.entity.lineage != own
                && g.entity.lineage != "world:ground")
            .map(|g| g.entity.lineage.clone()).collect();
        let lower: BTreeSet<Key> = alive.iter().filter(|l| held(l) < held(&own)).cloned().collect();
        let higher: BTreeSet<Key> = alive.iter().filter(|l| held(l) >= held(&own)).cloned().collect();
        let (lower, higher): (BTreeSet<Key>, BTreeSet<Key>) =
            (lower.difference(&before).cloned().collect(), higher.difference(&before).cloned().collect());
        if lower.is_empty() || higher.is_empty() { continue; }
        let now = met(&i.session, i.participant);
        assert!(lower.is_subset(&now), "seed {seed}: the outheld are known");
        assert!(higher.is_disjoint(&now), "seed {seed}: the outholding are not");
        return;
    }
    panic!("no seed gave a boundary with one line outheld and one not");
}

#[test]
fn matter_given_raises_regard_and_matter_taken_lowers_it_within_bounds() {
    let mut i = interim(5, true);
    let l = ally(&mut i);
    let (p, own) = (i.participant, line(&i));
    let regard = |i: &Interim| i.session.sim.state().lineages["lineage:ally"].regard.get(&p).copied();
    assert_eq!(regard(&i), None);
    round_with(&mut i, vec![flow(&own, &l, 7, 5), flow(&own, &l, 4, 1)]);
    assert_eq!(regard(&i), Some(250 + 3), "35 given is 3 per 10 at gain 1, from fresh; 4 more is lost to rounding");
    round_with(&mut i, vec![flow(&l, &own, 9, 5), flow(&l, &own, 3, 1)]);
    assert_eq!(regard(&i), Some(253 - 2 * 4), "48 taken is 4 per 10 at loss 2");
    round_with(&mut i, vec![flow(&own, &l, 1, 100_000)]);
    assert_eq!(regard(&i), Some(1000), "kept at the most");
    round_with(&mut i, vec![flow(&l, &own, 1, 100_000)]);
    assert_eq!(regard(&i), Some(0), "kept at nought");
    assert_eq!(readings::regard(&i.session.sim, p).get(&l), Some(&0));
    assert!(!met(&i.session, p).contains(&l), "dealing alone is not knowing");
    let n = regards(&i.session);
    round_with(&mut i, vec![flow(&own, &own, 99, 99), flow(&own, &l, 9, 1)]);
    assert_eq!(regards(&i.session), n, "a move within a line, or below a unit, records nothing");
}

#[test]
fn the_tally_sums_each_line_over_the_step_and_skips_participants() {
    let (rule, d) = (Regard::default(), Directing::default());
    let mut to_participant = flow("lineage:1", "participant:player", 50, 1);
    to_participant.to_kind.as_mut().unwrap().kingdom = PARTICIPANT.into();
    let mut site = flow("lineage:1", "lineage:0", 50, 1);
    site.to_kind = None;
    let flows = [flow("lineage:1", "lineage:0", 6, 2), flow("lineage:1", "lineage:0", 3, 3),
        flow("lineage:0", "lineage:1", 5, 4), flow("lineage:2", "lineage:0", 99, 9), to_participant, site];
    let tally = dealings(&flows, "lineage:1");
    assert_eq!(tally.len(), 1);
    assert_eq!(tally["lineage:0"], (21, 20));
    assert_eq!(regarded(None, 21, 20, &rule, &d), 250 + 2 - 2 * 2);
    assert_eq!(regarded(Some(1), 0, 99, &rule, &d), 0);
}

#[test]
fn a_critter_taken_up_starts_from_its_lineages_regard_where_it_has_one() {
    let mut s = Session::new(rule(founding(3)), Execution::Individuals).unwrap();
    let p: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    let (a, b) = pair(&s);
    let c = b + 1;
    let line = s.sim.state().population.get(a).unwrap().lineage.clone();
    s.command(Command::Take { participant: p, critter: a }).unwrap();
    assert_eq!(s.sim.bond(p, a), Some(250), "no regard: as inherited today");
    s.sim.set_bond(p, a, 850);
    s.command(Command::Take { participant: p, critter: b }).unwrap();
    assert_eq!(s.sim.bond(p, b), Some(550), "no regard: seeded from the forebear's bond");
    s.sim.state.lineages.get_mut(&line).unwrap().regard.insert(p, 700);
    s.sim.set_bond(p, a, 100);
    s.command(Command::Take { participant: p, critter: a }).unwrap();
    assert_eq!(s.sim.bond(p, a), Some(100), "a bond already held is kept");
    if s.sim.state().population.get(c).is_some_and(|e| e.lineage == line) {
        s.command(Command::Take { participant: p, critter: c }).unwrap();
        assert_eq!(s.sim.bond(p, c), Some(700), "from the lineage's regard");
    }
    // The rule unset ignores whatever regard a line holds.
    let mut s = Session::new(founding(3), Execution::Individuals).unwrap();
    let p: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
    s.sim.state.lineages.get_mut(&line).unwrap().regard.insert(p, 700);
    s.command(Command::Take { participant: p, critter: a }).unwrap();
    assert_eq!(s.sim.bond(p, a), Some(250));
}

#[test]
fn a_critter_taken_at_no_regard_has_no_bond_to_sway_a_nudge() {
    let mut ran = 0;
    for seed in 1..10 {
        let swayed = |regard: Option<i64>| {
            let mut s = Session::new(rule(founding(seed)), Execution::Individuals).unwrap();
            let p: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
            let c = super::tests::critter(&s);
            let line = s.sim.state().population.get(c).unwrap().lineage.clone();
            if let Some(r) = regard { s.sim.state.lineages.get_mut(&line).unwrap().regard.insert(p, r); }
            s.command(Command::Take { participant: p, critter: c }).unwrap();
            let pop = &s.sim.state().population;
            let place = pop.get(c).unwrap().place;
            let prey = pop.groups.iter().find(|(_, g)| g.entity.alive && g.entity.place == place
                && g.entity.lineage == "lineage:0").map(|(f, _)| *f)?;
            let nudge = Command::Nudge { participant: p, critter: c, aim: Aim::Act, toward: Toward::Thing(prey), act: None, };
            s.command(nudge).unwrap();
            s.sim.state.tick = 15;
            Some((s.sim.bond(p, c), s.sim.choose(c).process))
        };
        let (Some(none), Some(zero)) = (swayed(None), swayed(Some(0))) else { continue };
        ran += 1;
        assert_eq!(zero, (Some(0), Some("ecology:upkeep-1".into())), "no regard, no bond, no sway");
        assert_eq!(none, (Some(250), Some("ecology:feed-1".into())), "the default bond still sways");
    }
    assert!(ran > 0, "no seed put prey beside the critter");
}

#[test]
fn a_fork_starts_with_its_parents_regard_by_the_heritable_share() {
    for on in [true, false] {
        let g = if on { rule(founding(3)) } else { founding(3) };
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let p: Id = s.command(Command::Join).unwrap().strip_prefix("participant:").unwrap().parse().unwrap();
        let (kin, _) = pair(&s);
        let line = s.sim.state().population.get(kin).unwrap().lineage.clone();
        let parent = s.sim.state.lineages.get_mut(&line).unwrap();
        parent.regard.insert(p, 800);
        parent.regard.insert(p + 100, 30);
        parent.ate.insert("lineage:0".into());
        s.command(Command::Speciate { founder: kin, name: "split".into() }).unwrap();
        let fork = &s.sim.state().lineages["lineage:split"];
        if on {
            assert_eq!((fork.regard[&p], fork.regard[&(p + 100)]), (400, 15), "half of each, by 500 thousandths");
            assert_eq!(fork.ate, set(&["lineage:0"]));
        } else {
            assert!(fork.regard.is_empty() && fork.ate.is_empty(), "unset, a fork keeps neither");
        }
        assert_eq!(s.sim.state().lineages[&line].regard[&p], 800, "the parent keeps its own");
    }
}

#[test]
fn known_lineages_and_regard_read_the_same_from_a_replay() {
    let mut i = interim(11, true);
    let own = line(&i);
    for round in 0..10 {
        let given = if round % 3 == 2 { flow("lineage:0", &own, 15, 2) } else { flow(&own, "lineage:0", 30, 2) };
        round_with(&mut i, vec![given]);
    }
    let p = i.participant;
    assert!(regards(&i.session) > 0, "the run recorded regard");
    assert!(!readings::regard(&i.session.sim, p).is_empty());
    let json = serde_json::to_vec(&i.session.save()).unwrap();
    let r = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(i.session.sim.state_hash(), r.sim.state_hash());
    assert_eq!(known(&i.session, p), known(&r, p));
    assert_eq!(readings::regard(&i.session.sim, p), readings::regard(&r.sim, p));
    assert!(known(&r, p).contains(&own));
}
