// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 936: observe the unchanged played seed 9, without tuning or a gate draw.

use super::*;
use std::collections::BTreeMap;

#[test]
#[ignore = "explicit read-only A1 seed-9 diagnosis, with an existing receipt owner"]
fn a1_seed9_accepted_choices_and_moves() {
    let path = receipt_directory().join("a1-2026-10-16-chooser-seed9.json");
    assert!(!path.exists(), "fresh receipt required before the full diagnostic runs");
    assert!(
        std::env::var_os("COSMOGRAPH_REPRO_TWEAK").is_none(),
        "unchanged A1 preset"
    );
    let g = diagnosed_seed9();
    let genesis = crate::digest(&g);
    let process_rules: BTreeMap<_, _> = g
        .rules
        .processes
        .iter()
        .filter(|(_, p)| p.causation == Causation::Choice)
        .map(|(key, p)| {
            (
                key.clone(),
                serde_json::json!({
                    "period": p.period, "priority": p.priority, "requires": p.requires,
                    "effects": p.effects, "need_account": p.need_account, "need_below": p.need_below
                }),
            )
        })
        .collect();
    let mut run = Interim::found(
        g,
        Start::default(),
        Mode::Survival,
        Pace {
            round: 1,
            scoring: 1,
        },
        Execution::Individuals,
    )
    .unwrap();
    for key in run
        .session
        .sim
        .genesis()
        .rules
        .processes
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        run.session.sim.watch(&key);
    }
    let matter = run.session.sim.matter();
    let mut rows = Vec::new();
    let mut boundaries = 0;
    let mut births = 0;
    for _ in 0..EPOCH * 3 {
        let before = run.session.sim.state().population.clone();
        let happenings = run.round(&no_candidates).unwrap();
        births += happenings
            .iter()
            .filter(|h| matches!(h, Happening::Born { .. }))
            .count();
        boundaries += happenings
            .iter()
            .filter(|h| matches!(h, Happening::Boundary { .. }))
            .count();
        let sim = &mut run.session.sim;
        assert_eq!(sim.matter(), matter, "unchanged conserved matter");
        let actual = sim.take_watched();
        let (choice_tick, choices) = sim.recorded_choices();
        let consumer = |id| {
            before
                .get(id)
                .or_else(|| sim.state().population.get(id))
                .is_some_and(|e| e.lineage == "lineage:1")
        };
        let decisions: Vec<_> = choices.iter().filter(|(id, _)| consumer(**id))
            .map(|(id, choice)| {
                let prior = before.get(*id);
                let after = sim.state().population.get(*id);
                serde_json::json!({ "actor": id, "before_site": prior.and_then(Entity::resident_site),
                    "after_site": after.and_then(Entity::resident_site),
                    "method": prior.map(|e| e.method), "choice": choice.process,
                    "target": choice.target, "options": choice.options,
                    "alive": after.is_some_and(|e| e.alive) })
            }).collect();
        let accepted: Vec<_> = actual.iter().filter(|act| consumer(act.actor)).collect();
        let mut census: BTreeMap<String, u64> = BTreeMap::new();
        let mut living = 0;
        for group in sim
            .state()
            .population
            .groups
            .values()
            .filter(|g| g.entity.alive)
        {
            let e = &group.entity;
            *census
                .entry(format!("{}@{}", e.lineage, e.place))
                .or_default() += group.count;
            if e.lineage == "lineage:1" {
                living += group.count;
            }
        }
        rows.push(
            serde_json::json!({ "tick": sim.state().tick, "choice_tick": choice_tick,
            "living": living, "census": census, "decisions": decisions, "accepted": accepted }),
        );
        if living == 0 {
            break;
        }
    }
    let saved = run.session.save();
    let mut replay_hashes = Vec::new();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let replay = Session::load(saved.clone(), execution).unwrap();
        assert_eq!(replay.sim.state_hash(), run.session.sim.state_hash());
        replay_hashes.push(serde_json::json!({"execution": format!("{execution:?}"),
            "state_hash": replay.sim.state_hash()}));
    }
    let report = serde_json::json!({ "purpose": "936 diagnosis; unchanged source behavior; not an A1 gate",
        "seed": 9, "form": "played", "rates": "A1", "genesis": genesis,
        "tick": run.session.sim.state().tick, "state_hash": run.session.sim.state_hash(),
        "choice_observation_valid": false, "choice_observation_limit": "cache cleared after advance; use chooser_window",
        "births_scope": "played-parent Happening::Born, not native consumer birth acts",
        "births": births, "boundaries": boundaries, "process_rules": process_rules,
        "replay_hashes": replay_hashes, "ticks": rows });
    let bytes = serde_json::to_vec(&report).unwrap();
    assert!(bytes.len() < 100 * 1024 * 1024, "953 receipt cap");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("fresh diagnostic receipt; never overwrite evidence");
    use std::io::Write;
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    eprintln!(
        "A1_DIAG genesis={genesis} tick={} births={births} boundaries={boundaries} bytes={} path={}",
        run.session.sim.state().tick,
        bytes.len(),
        path.display()
    );
}
