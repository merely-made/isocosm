// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A bounded seed-9 window: capture choices where made, before cache clearing.

use super::*;
use crate::directing::choice::trace;
use std::collections::BTreeMap;

#[test]
#[ignore = "explicit unchanged seed-9 choice window, 24 ticks and compact receipt"]
fn a1_seed9_first_24_ticks_of_choices() {
    window("a1-2026-10-16-seed9-first24.json", 24);
}

#[test]
#[ignore = "explicit feeding-pass diagnosis, unchanged seed9 and 24 ticks"]
fn a1_seed9_first24_feeding_passes() {
    window("a1-2026-10-16-seed9-feeding-passes.json", 24);
}

#[test]
#[ignore = "explicit prey-departure diagnosis, unchanged seed9 and 16 ticks"]
fn a1_seed9_departure_window() {
    window("a1-2026-10-16-seed9-departure.json", 16);
}

fn window(receipt: &str, limit: u64) {
    let path = receipt_directory().join(receipt);
    assert!(!path.exists(), "diagnosis requires a fresh receipt path before running");
    assert!(std::env::var_os("ISOCOSM_REPRO_TWEAK").is_none());
    let genesis = world(9, Form::Played, false);
    let digest = crate::digest(&genesis);
    assert!(genesis.population.groups.values().filter(|g| g.entity.lineage == "lineage:1")
        .all(|g| g.entity.method == Method::Deliberative));
    let mut run = Interim::found(genesis, Start::default(), Mode::Survival,
        Pace { round: 1, scoring: 1 }, Execution::Individuals).unwrap();
    for key in run.session.sim.genesis().rules.processes.keys().cloned().collect::<Vec<_>>() {
        run.session.sim.watch(&key);
    }
    trace::start("lineage:1", "ecology:feed-1");
    let matter = run.session.sim.matter();
    let mut rows = Vec::new();
    let mut attempts = 0;
    for _ in 0..limit {
        let before = run.session.sim.state().population.clone();
        run.round(&no_candidates).unwrap();
        let sim = &mut run.session.sim;
        assert_eq!(sim.matter(), matter);
        let choices = trace::take();
        attempts += choices.len();
        let decisions: Vec<_> = choices.into_iter().map(|a| serde_json::json!({
            "phase": a.phase, "tick": a.tick, "actor": a.actor, "site": a.site, "pass": a.pass,
            "mood": a.mood, "actor_held": a.actor_held, "selected_target": a.selected_target, "choice": a.choice.process, "target": a.choice.target,
            "options": a.choice.options, "food_due": a.food_due,
            "food_target": a.food_target, "food_feasible": a.food_feasible
        })).collect();
        let watched = sim.take_watched();
        let mut other_acts: BTreeMap<String, u64> = BTreeMap::new();
        for act in &watched {
            if let Some(body) = before.get(act.actor).or_else(|| sim.state().population.get(act.actor)) {
                if body.lineage != "lineage:1" {
                    *other_acts.entry(format!("{}:{}", body.lineage, act.process)).or_default() += act.count;
                }
            }
        }
        let accepted: Vec<_> = watched.into_iter().filter(|act| {
            before.get(act.actor).or_else(|| sim.state().population.get(act.actor))
                .is_some_and(|body| body.lineage == "lineage:1")
        }).collect();
        let mut census: BTreeMap<String, u64> = BTreeMap::new();
        for group in sim.state().population.groups.values().filter(|g| g.entity.alive) {
            *census.entry(format!("{}@{}", group.entity.lineage, group.entity.place))
                .or_default() += group.count;
        }
        rows.push(serde_json::json!({ "tick": sim.state().tick,
            "decisions": decisions, "accepted": accepted, "other_accepted": other_acts, "census": census }));
    }
    trace::stop();
    assert!(attempts > 0, "observation must happen before the chooser cache is cleared");
    let saved = run.session.save();
    let hash = run.session.sim.state_hash();
    for execution in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(Session::load(saved.clone(), execution).unwrap().sim.state_hash(), hash);
    }
    let report = serde_json::json!({ "purpose": "936; unchanged played seed9; bounded choice/feeding window",
        "genesis": digest, "tick": limit, "hash": hash, "attempts": attempts,
        "observation": "choice evaluation attempts; accepted acts recorded separately",
        "replays": ["Individuals", "Grouped"], "ticks": rows });
    let bytes = serde_json::to_vec(&report).unwrap();
    assert!(bytes.len() < 100 * 1024 * 1024);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
    eprintln!("A1_WINDOW attempts={attempts} hash={hash:016x} bytes={} path={}", bytes.len(), path.display());
}
