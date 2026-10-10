// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Execution, Founding, Session, history::Command};

/// A small ecology whose lineage 1, a consumer, is played.
pub(crate) fn world(seed: u64, mode: Execution) -> Session {
    let genesis = Founding {
        seed,
        sites: 4,
        population: 48,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    Session::new(genesis, mode).unwrap()
}

pub(crate) fn critter(s: &Session) -> Id {
    let groups = &s.sim.state().population.groups;
    let played = groups
        .iter()
        .find(|(_, g)| g.entity.method == Method::Deliberative);
    *played.unwrap().0
}

/// Joins a participant and takes up the first deliberative critter.
pub(crate) fn played(s: &mut Session) -> (Id, Id) {
    let joined = s.command(Command::Join).unwrap();
    let p: Id = joined
        .strip_prefix("participant:")
        .unwrap()
        .parse()
        .unwrap();
    let c = critter(s);
    s.command(Command::Take {
        participant: p,
        critter: c,
    })
    .unwrap();
    (p, c)
}

#[test]
fn a_relation_without_a_value_reads_as_nought_and_saves_as_before() {
    let r = Relation::new(3, "sim:parent", 4);
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("value"));
    let old: Relation =
        serde_json::from_str(r#"{"subject":3,"kind":"sim:parent","object":4}"#).unwrap();
    assert_eq!(old, r);
    let mut set = BTreeSet::from([r.clone()]);
    hold(
        &mut set,
        Relation {
            value: 9,
            ..r.clone()
        },
        true,
    );
    assert_eq!(set.len(), 1, "one relation per subject, kind and object");
    hold(&mut set, Relation { value: 9, ..r }, false);
    assert!(set.is_empty());
}

#[test]
fn a_run_without_directing_saves_no_new_fields() {
    let unplayed = Founding {
        seed: 7,
        ecology: true,
        ..Default::default()
    };
    let mut s = Session::new(unplayed.generate().unwrap(), Execution::Individuals).unwrap();
    s.advance(12).unwrap();
    let json = serde_json::to_string(&s.save()).unwrap();
    for field in ["\"value\"", "\"nudges\"", "\"directing\"", PARTICIPANT] {
        assert!(!json.contains(field), "{field} leaked into a plain save");
    }
    let loaded = Session::load_json(json.as_bytes(), Execution::Individuals).unwrap();
    assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
}

#[test]
fn a_participant_is_placeless_bonded_and_seeded_by_default() {
    let mut s = world(7, Execution::Grouped);
    let (p, c) = played(&mut s);
    let e = s.sim.state().population.get(p).unwrap();
    assert!(is_participant(e) && e.place == PLACELESS);
    assert_eq!(s.sim.bond(p, c), Some(250));
    assert_eq!(s.sim.plays(p), Some(c));
    assert_eq!(s.sim.bonded(c), vec![(p, 250)]);
    // Nothing takes a participant as its target.
    assert!(!s.sim.target_matches(
        c,
        Some(p),
        &crate::rules::Process {
            target: Some(crate::rules::Target {
                same_place: false,
                alive: None,
                lineage: None,
                among: Default::default(),
                weighted: false,
            }),
            ..crate::generate::process("test:any", crate::rules::Causation::Choice, vec![])
        }
    ));
}

#[test]
fn a_nudge_needs_a_bond_and_something_named() {
    let mut s = world(7, Execution::Individuals);
    let (p, c) = played(&mut s);
    let nudge = |toward| Command::Nudge {
        participant: p,
        critter: c,
        aim: Aim::Attend,
        toward,
    };
    assert!(s.command(nudge(Toward::Site(999))).is_err());
    assert!(s.command(nudge(Toward::Thing(p))).is_err());
    assert_eq!(s.command(nudge(Toward::Site(1))).unwrap(), "nudged");
    let stranger = Command::Nudge {
        participant: c,
        critter: c,
        aim: Aim::Act,
        toward: Toward::Site(1),
    };
    assert!(s.command(stranger).is_err());
    assert_eq!(s.sim.state().nudges.len(), 1);
    assert_eq!(s.sim.live_nudges(c, s.sim.state().tick), vec![0]);
}

#[test]
fn a_nudged_run_replays_to_the_same_hash() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = world(11, mode);
        let (p, c) = played(&mut s);
        for round in 0..4u64 {
            // A critter may die of age in the run; a dead one takes no nudge.
            if !s.sim.state().population.get(c).unwrap().alive {
                break;
            }
            s.command(Command::Nudge {
                participant: p,
                critter: c,
                aim: Aim::Attend,
                toward: Toward::Site(round % 4),
            })
            .unwrap();
            s.advance(9).unwrap();
        }
        let json = serde_json::to_vec(&s.save()).unwrap();
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
        let fork = s
            .fork_at(s.sim.state().tick, "branch:again".into())
            .unwrap();
        assert_eq!(fork.sim.state_hash(), s.sim.state_hash());
    }
}
