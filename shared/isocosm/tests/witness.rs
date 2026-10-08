// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The state witness's first phase (rulings 607, 608, 610, 634 to 636 and
//! 641): the FNV-1a witness, v2 saves, v1 saves still read, and the
//! per-tick trace.

use isocosm::{
    Execution, Founding, Session,
    history::{Command, SAVE_VERSION},
    rules::AccountKind,
    simulation::Genesis,
};

/// A v1 save written by the pre-witness core (isometry `81fb4162`): seed
/// 641, three sites, 24 members, a four-tick epoch, six ticks, one
/// inspection, six more ticks; checkpoints at ticks 4, 8 and 12.
const V1: &str = include_str!("data/v1-checkpointed-world.json");
const MODES: [Execution; 2] = [Execution::Individuals, Execution::Grouped];

fn genesis(seed: u64) -> Genesis {
    let mut genesis = Founding {
        seed,
        sites: 3,
        population: 24,
        cohort_size: 8,
        lineages: 2,
        ..Default::default()
    }
    .generate()
    .unwrap();
    genesis.rules.epoch_ticks = 4;
    genesis
}

fn edited(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
    let mut save: serde_json::Value = serde_json::from_str(V1).unwrap();
    edit(&mut save);
    serde_json::to_vec(&save).unwrap()
}

#[test]
fn a_v1_save_loads_and_saves_again_as_v2() {
    for mode in MODES {
        let session = Session::load_json(V1.as_bytes(), mode).unwrap();
        let saved = session.save();
        assert_eq!(saved.version, SAVE_VERSION);
        let ticks: Vec<_> = saved.checkpoints.iter().map(|c| c.tick).collect();
        assert_eq!(ticks, [4, 8, 12], "{mode:?}");
        let json = serde_json::to_vec(&saved).unwrap();
        let again = Session::load_json(&json, mode).unwrap();
        assert_eq!(again.sim.state_hash(), session.sim.state_hash());
        assert_eq!(again.save(), saved);
    }
}

#[test]
fn a_v1_save_is_checked_at_every_checkpoint() {
    let flip = |hash: &mut serde_json::Value| {
        let s = hash.as_str().unwrap();
        let first = if s.starts_with('0') { "1" } else { "0" };
        *hash = format!("{first}{}", &s[1..]).into();
    };
    let middle = edited(|s| flip(&mut s["checkpoints"][1]["state_hash"]));
    let dropped = edited(|s| {
        s["checkpoints"].as_array_mut().unwrap().remove(1);
    });
    let last = edited(|s| flip(&mut s["state_hash"]));
    for mode in MODES {
        for bad in [&middle, &dropped] {
            let why = Session::load_json(bad, mode).unwrap_err();
            assert_eq!(why, "checkpoint history mismatch", "{mode:?}");
        }
        let why = Session::load_json(&last, mode).unwrap_err();
        assert_eq!(why, "save state hash mismatch", "{mode:?}");
    }
}

#[test]
fn a_v2_save_refuses_a_planted_hash() {
    let session = Session::load_json(V1.as_bytes(), Execution::Grouped).unwrap();
    let saved = session.save();
    assert!(Session::load(saved.clone(), Execution::Individuals).is_ok());
    let mut middle = saved.clone();
    middle.checkpoints[1].state_hash ^= 1;
    let why = Session::load(middle, Execution::Grouped).unwrap_err();
    assert_eq!(why, "checkpoint history mismatch");
    let mut last = saved;
    last.state_hash ^= 1;
    let why = Session::load(last, Execution::Grouped).unwrap_err();
    assert_eq!(why, "save state hash mismatch");
}

#[test]
fn the_witness_is_deterministic_and_follows_the_world() {
    let hash = |seed| {
        let mut session = Session::new(genesis(seed), Execution::Grouped).unwrap();
        session.advance(8).unwrap();
        session.sim.state_hash()
    };
    assert_eq!(hash(1), hash(1));
    assert_ne!(hash(1), hash(2));
}

#[test]
fn a_traced_advance_reaches_the_same_world() {
    for mode in MODES {
        let mut whole = Session::new(genesis(3), mode).unwrap();
        whole.advance(12).unwrap();
        let mut traced = Session::new(genesis(3), mode).unwrap();
        let mut lines = vec![];
        traced
            .advance_traced(12, |tick, hash| lines.push((tick, hash)))
            .unwrap();
        let ticks: Vec<_> = lines.iter().map(|l| l.0).collect();
        assert_eq!(ticks, (1..=12).collect::<Vec<_>>());
        assert_eq!(lines[11].1, whole.sim.state_hash(), "{mode:?}");
        assert_eq!(traced.checkpoints, whole.checkpoints, "{mode:?}");
    }
}

#[test]
fn a_planted_unit_diverges_the_trace_after_it_and_not_before() {
    let genesis = genesis(4);
    let account = genesis
        .rules
        .accounts
        .iter()
        .find(|(_, kind)| matches!(kind, AccountKind::Matter { .. }))
        .map(|(key, _)| key.clone())
        .unwrap();
    let mut plain = Session::new(genesis.clone(), Execution::Grouped).unwrap();
    let site = *plain.sim.state().sites.keys().next().unwrap();
    let mut clean = vec![];
    plain
        .advance_traced(12, |_, hash| clean.push(hash))
        .unwrap();
    let mut planted = Session::new(genesis, Execution::Grouped).unwrap();
    let mut trace = vec![];
    planted
        .advance_traced(5, |_, hash| trace.push(hash))
        .unwrap();
    planted
        .command(Command::PlaceMatter {
            site,
            account,
            amount: 1,
        })
        .unwrap();
    planted
        .advance_traced(7, |_, hash| trace.push(hash))
        .unwrap();
    let first = clean.iter().zip(&trace).position(|(a, b)| a != b);
    assert_eq!(first, Some(5), "tick 6 is the first traced after the plant");
}
