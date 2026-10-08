// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The state witness (rulings 607 to 610, 633 to 636, 641, 649 to 653): the
//! labelled entries and their digest, v3 saves, the kept v1 and v2 readers,
//! and the per-tick traces.

use isocosm::{
    Execution, Founding, Session,
    history::{Command, SAVE_VERSION},
    rules::AccountKind,
    simulation::Genesis,
};
use serde_json::Value;
use state_witness::{Trace, Witness, first_trace_divergence, label};

/// Written by the core at isometry `81fb4162`: seed 641, three sites, 24
/// members, a four-tick epoch, six ticks, one inspection, six more ticks;
/// checkpoints at ticks 4, 8 and 12.
const V1: &str = include_str!("data/v1-checkpointed-world.json");
/// The same run at seed 651, written by the pre-H2 core at `14d8d9af`.
const V2: &str = include_str!("data/v2-checkpointed-world.json");
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

fn edited(json: &str, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut save: Value = serde_json::from_str(json).unwrap();
    edit(&mut save);
    serde_json::to_vec(&save).unwrap()
}

/// A session advanced five ticks, then one unit placed at a site, if asked.
fn planted(seed: u64, plant: bool) -> (Session, u64) {
    let genesis = genesis(seed);
    let account = genesis
        .rules
        .accounts
        .iter()
        .find(|(_, kind)| matches!(kind, AccountKind::Matter { .. }))
        .map(|(key, _)| key.clone())
        .unwrap();
    let mut session = Session::new(genesis, Execution::Grouped).unwrap();
    let site = *session.sim.state().sites.keys().next().unwrap();
    session.advance(5).unwrap();
    if plant {
        let amount = 1;
        let place = Command::PlaceMatter {
            site,
            account,
            amount,
        };
        session.command(place).unwrap();
    }
    (session, site)
}

#[test]
fn old_saves_load_and_save_again_as_v3() {
    for old in [V1, V2] {
        for mode in MODES {
            let session = Session::load_json(old.as_bytes(), mode).unwrap();
            let saved = session.save();
            assert_eq!(saved.version, SAVE_VERSION);
            let ticks: Vec<_> = saved.checkpoints.iter().map(|c| c.tick).collect();
            assert_eq!(ticks, [4, 8, 12], "{mode:?}");
            let json = serde_json::to_vec(&saved).unwrap();
            let again = Session::load_json(&json, mode).unwrap();
            assert_eq!(again.save(), saved);
        }
    }
}

#[test]
fn old_saves_are_checked_at_every_checkpoint() {
    // A v1 hash is a hex string, a v2 hash a number; either is flipped.
    let flip = |hash: &mut Value| {
        let flipped = match &*hash {
            Value::String(s) => {
                let first = if s.starts_with('0') { "1" } else { "0" };
                format!("{first}{}", &s[1..]).into()
            },
            other => (other.as_u64().unwrap() ^ 1).into(),
        };
        *hash = flipped;
    };
    for old in [V1, V2] {
        let middle = edited(old, |s| flip(&mut s["checkpoints"][1]["state_hash"]));
        let dropped = edited(old, |s| {
            s["checkpoints"].as_array_mut().unwrap().remove(1);
        });
        let last = edited(old, |s| flip(&mut s["state_hash"]));
        for mode in MODES {
            for bad in [&middle, &dropped] {
                let why = Session::load_json(bad, mode).unwrap_err();
                assert_eq!(why, "checkpoint history mismatch", "{mode:?}");
            }
            let why = Session::load_json(&last, mode).unwrap_err();
            assert_eq!(why, "save state hash mismatch", "{mode:?}");
        }
    }
}

#[test]
fn the_state_hash_is_the_entries_digest() {
    let mut session = Session::new(genesis(5), Execution::Grouped).unwrap();
    session.advance(8).unwrap();
    let witness = session.sim.witness();
    assert_eq!(session.sim.state_hash(), witness.digest());
    assert_eq!(
        witness.len(),
        17,
        "seed, revision, world and fourteen fields"
    );
    let saved = session.save();
    assert_eq!(saved.state_hash, saved.witness.digest());
    assert_eq!(saved.checkpoints.len(), 2);
}

#[test]
fn a_v3_save_names_the_entry_that_was_changed() {
    let (mut session, _) = planted(6, false);
    session.advance(7).unwrap();
    let json = serde_json::to_string(&session.save()).unwrap();
    let flip_sites = |witness: &mut Value| {
        let entry = witness
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["label"] == "sites")
            .unwrap();
        entry["digest"] = (entry["digest"].as_u64().unwrap() ^ 1).into();
    };
    let middle = edited(&json, |s| flip_sites(&mut s["checkpoints"][1]["witness"]));
    let why = Session::load_json(&middle, Execution::Grouped).unwrap_err();
    assert_eq!(why, "checkpoint at tick 8 diverges first at sites");
    // The final entries changed with a consistent digest: named, not hashed.
    let last = edited(&json, |s| {
        flip_sites(&mut s["witness"]);
        let witness: Witness = serde_json::from_value(s["witness"].clone()).unwrap();
        s["state_hash"] = witness.digest().into();
    });
    let why = Session::load_json(&last, Execution::Grouped).unwrap_err();
    assert_eq!(why, "save state diverges first at sites");
    let unfolded = edited(&json, |s| flip_sites(&mut s["witness"]));
    let why = Session::load_json(&unfolded, Execution::Grouped).unwrap_err();
    assert_eq!(why, "save state hash mismatch");
}

#[test]
fn a_planted_site_change_is_named_on_load() {
    let (mut session, _) = planted(7, true);
    session.advance(7).unwrap();
    let mut saved = session.save();
    // Drop the placement from the history: the replay no longer makes it.
    assert_eq!(saved.entries.len(), 1);
    saved.entries.clear();
    let why = Session::load(saved, Execution::Grouped).unwrap_err();
    assert_eq!(why, "checkpoint at tick 8 diverges first at sites");
}

#[test]
fn a_planted_site_change_is_named_in_the_entity_trace() {
    let trace = |plant: bool| {
        let (mut session, site) = planted(8, plant);
        let mut trace = Trace::new();
        let mut push = |sim: &isocosm::Simulation| {
            trace.push(sim.state().tick, sim.entity_witness()).unwrap();
        };
        session.advance_traced(7, &mut push).unwrap();
        (trace, site)
    };
    let (clean, _) = trace(false);
    let (planted, site) = trace(true);
    let at = first_trace_divergence(&clean, &planted).unwrap();
    assert_eq!(at.tick, 6, "the first traced tick after the plant");
    assert_eq!(at.divergence.unwrap().label, label("site", [site]));
    let framed = planted.to_framed().unwrap();
    assert_eq!(Trace::from_framed(&framed).unwrap(), planted);
}

#[test]
fn a_planted_unit_diverges_the_text_trace_after_it_and_not_before() {
    let hashes = |plant: bool| {
        let (mut session, _) = planted(4, plant);
        let mut hashes = vec![];
        session
            .advance_traced(7, |sim| hashes.push(sim.state_hash()))
            .unwrap();
        hashes
    };
    let (clean, planted) = (hashes(false), hashes(true));
    let first = clean.iter().zip(&planted).position(|(a, b)| a != b);
    assert_eq!(first, Some(0), "tick 6 is the first traced after the plant");
}

#[test]
fn every_critter_is_labelled_alike_in_both_modes() {
    let witness = |mode| {
        let mut session = Session::new(genesis(9), mode).unwrap();
        session.advance(8).unwrap();
        let count = session.sim.state().population.count();
        (session.sim.entity_witness(), count)
    };
    let (individuals, count) = witness(Execution::Individuals);
    let (grouped, _) = witness(Execution::Grouped);
    let critters = |w: &Witness| {
        let entities = w
            .entries()
            .iter()
            .filter(|e| e.label.starts_with("entity:"));
        entities.count() as u64
    };
    assert_eq!(critters(&grouped), count);
    assert_eq!(individuals, grouped);
}

#[test]
fn a_traced_advance_reaches_the_same_world() {
    for mode in MODES {
        let mut whole = Session::new(genesis(3), mode).unwrap();
        whole.advance(12).unwrap();
        let mut traced = Session::new(genesis(3), mode).unwrap();
        let mut ticks = vec![];
        traced
            .advance_traced(12, |sim| ticks.push(sim.state().tick))
            .unwrap();
        assert_eq!(ticks, (1..=12).collect::<Vec<_>>());
        assert_eq!(traced.sim.state_hash(), whole.sim.state_hash(), "{mode:?}");
        assert_eq!(traced.checkpoints, whole.checkpoints, "{mode:?}");
    }
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
