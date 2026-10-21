// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeMap;

/// Ruling 936: observed accepted acts, never a policy experiment or gate run.
#[test]
#[ignore = "bounded seed-9 diagnostic; set the reproduction receipt directory"]
fn seed9_accepted_movement_diagnosis() {
    let g = world(9, Form::Played, false);
    let genesis = crate::digest(&g);
    let mut i = Interim::found(g, Start::default(), Mode::Survival,
        Pace { round: 1, scoring: 1 }, Execution::Individuals).unwrap();
    let processes: Vec<_> = i.session.sim.genesis().rules.processes.keys().cloned().collect();
    for p in processes { i.session.sim.watch(&p); }
    let matter = i.session.sim.matter();
    let mut ticks = vec![];
    for _ in 0..180 {
        let before: BTreeMap<_, _> = i.session.sim.state().population.groups.iter()
            .filter(|(_, g)| g.entity.lineage == "lineage:1")
            .flat_map(|(first, g)| (*first..*first + g.count)
                .map(move |id| (id, (g.entity.place, format!("{:?}", g.entity.method)))))
            .collect();
        let happened = i.round(&no_candidates).unwrap();
        assert_eq!(i.session.sim.matter(), matter, "conservation every tick");
        let sim = &mut i.session.sim;
        let accepted: Vec<_> = sim.take_watched().into_iter()
            .filter(|a| before.contains_key(&a.actor))
            .map(|a| {
                let p = &sim.genesis().rules.processes[&a.process];
                let destinations = crate::directing::choice::moves(p);
                serde_json::json!({"act": a, "before": before[&a.actor],
                    "destinations": destinations})
            }).collect();
        let mut census = BTreeMap::<String, u64>::new();
        for g in sim.state().population.groups.values().filter(|g| g.entity.alive) {
            if g.entity.lineage.starts_with("lineage:") {
                *census.entry(format!("{}@{}", g.entity.lineage, g.entity.place)).or_default() += g.count;
            }
        }
        ticks.push(serde_json::json!({"tick": sim.state().tick,
            "accepted_consumers": accepted, "census": census,
            "happenings": format!("{happened:?}")}));
        if i.critter().is_none() { break; }
    }
    let bytes = serde_json::to_vec(&serde_json::json!({"genesis": genesis,
        "claim": "accepted acts and preceding-tick actor sites; not pass-start scoring",
        "ticks": ticks})).unwrap();
    assert!(bytes.len() < 20_000_000, "diagnostic budget");
    let path = receipt_directory().join(format!("a1-936-seed9-{genesis}.json"));
    use std::io::Write;
    std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap()
        .write_all(&bytes).unwrap();
    eprintln!("936 diagnostic {} bytes: {}", bytes.len(), path.display());
}
