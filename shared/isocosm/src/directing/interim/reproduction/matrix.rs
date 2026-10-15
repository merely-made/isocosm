// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub(super) struct Draw {
    pub seed: u64,
    pub form: Form,
    pub original: bool,
}

impl Draw {
    fn prefix(self, genesis: &str) -> String {
        let form = self.form.name();
        let arm = if self.original { "original" } else { "candidate" };
        format!("q3-primary-final-state-matrix-{form}-{arm}-seed{}-{genesis}", self.seed)
    }
}

#[derive(Debug)]
struct Sequence {
    initial_tick: Tick,
    matter: u128,
    ticks: u64,
}

impl Sequence {
    fn accept(&mut self, tick: Tick, matter: u128) -> std::result::Result<(), &'static str> {
        if tick != self.initial_tick + self.ticks + 1 {
            return Err("noncontiguous actual ticks");
        }
        if matter != self.matter {
            return Err("actual matter changed");
        }
        self.ticks += 1;
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct TickReading {
    kind: &'static str,
    tick: Tick,
    matter: u128,
}

#[derive(serde::Serialize)]
struct ReplayReading {
    mode: String,
    expected_state_hash: u64,
    actual_state_hash: u64,
}

pub(super) struct Record {
    draw: Draw,
    genesis: String,
    prefix: String,
    stream: File,
    sequence: Option<Sequence>,
    replays: Vec<ReplayReading>,
}

fn new_file(name: &str) -> File {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(receipt_directory().join(name))
        .expect("fresh matrix receipt path; never overwrite")
}

fn retain_new<T: serde::Serialize>(name: &str, value: &T) {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    let mut file = new_file(name);
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
}

fn validate_bodied(g: &Genesis) {
    assert_eq!(g.founding.as_ref().unwrap().bodies, Some(Bodies::default()));
    assert_eq!(g.rules.body(), BodyRules::default());
    assert!(!g.rules.kinds.is_empty() && !g.rules.functions.is_empty()
        && !g.rules.shapes.is_empty(), "native matrix body catalogue");
    for key in ["lineage:0", "lineage:1"] {
        let d = g.lineages[key].development.as_ref().expect("native matrix development");
        assert!(!d.recipe.tagmata.is_empty(), "native matrix body recipe");
        assert!(d.recipe.kinds().is_subset(&d.lexicon), "recipe kind lexicon");
        assert!(d.lexicon.iter().all(|k| g.rules.kinds.contains_key(k)),
            "catalogued development kinds");
    }
    let mut founders = 0u64;
    let mut consumers = 0u64;
    for c in g.population.groups.values() {
        let e = &c.entity;
        if g.lineages[&e.lineage].development.is_none() {
            continue;
        }
        founders += c.count;
        if e.lineage == "lineage:1" {
            consumers += c.count;
        }
        assert!(e.body.is_some() && !e.parts.is_empty(), "attached matrix founder anatomy");
        assert!(e.parts.keys().all(|id| e.bodied(*id) && e.situs(*id).is_some()),
            "sited matrix founder parts");
        assert!(e.parts.values().flat_map(|p| &p.functions)
            .all(|f| g.rules.functions.contains_key(f)), "catalogued founder functions");
    }
    assert!(founders > 0 && consumers > 0, "bodied matrix founders and played consumers");
}

impl Record {
    pub fn begin(draw: Draw, g: &Genesis) -> Self {
        assert_eq!(draw.seed, g.seed, "actual matrix seed");
        if draw.form.bodied() {
            validate_bodied(g);
        } else {
            assert!(g.founding.as_ref().unwrap().bodies.is_none(), "bare matrix form");
        }
        let genesis = crate::digest(g);
        let prefix = draw.prefix(&genesis);
        retain_new(&format!("{prefix}-genesis.json"), g);
        let stream = new_file(&format!("{prefix}-conservation.jsonl"));
        Self { draw, genesis, prefix, stream, sequence: None, replays: Vec::new() }
    }

    fn line(&mut self, row: &TickReading) {
        serde_json::to_writer(&mut self.stream, row).unwrap();
        self.stream.write_all(b"\n").unwrap();
    }

    pub fn start(&mut self, tick: Tick, matter: u128) {
        assert!(self.sequence.is_none(), "matrix baseline is written once");
        self.line(&TickReading { kind: "baseline", tick, matter });
        self.sequence = Some(Sequence { initial_tick: tick, matter, ticks: 0 });
    }

    pub fn tick(&mut self, tick: Tick, matter: u128) {
        // Preserve the observed row even if the subsequent existing conservation
        // gate refuses it. No complete evidence marker is written on that failure.
        self.line(&TickReading { kind: "tick", tick, matter });
        self.sequence.as_mut().expect("matrix baseline")
            .accept(tick, matter).expect("actual matrix tick conservation");
    }

    pub fn replay(&mut self, mode: Execution, expected: u64, actual: u64) {
        assert_eq!(actual, expected, "actual final matrix replay");
        self.replays.push(ReplayReading {
            mode: format!("{mode:?}"), expected_state_hash: expected, actual_state_hash: actual,
        });
    }

    pub fn finish<S: serde::Serialize, T: serde::Serialize>(
        self, saved: &S, state: &T, out: &Outcome,
    ) {
        let sequence = self.sequence.as_ref().expect("matrix baseline");
        assert_eq!(out.genesis, self.genesis);
        assert_eq!(sequence.initial_tick + sequence.ticks, out.tick);
        assert_eq!(self.replays.iter().map(|r| r.mode.as_str()).collect::<Vec<_>>(),
            vec!["Individuals", "Grouped"]);
        self.stream.sync_all().unwrap();
        retain_new(&format!("{}-session.json", self.prefix), saved);
        retain_new(&format!("{}-state.json", self.prefix), state);
        retain_new(&format!("{}-outcome.json", self.prefix), out);
        let summary = serde_json::json!({
            "schema": "q3-matrix-draw-complete-v1", "draw": self.draw,
            "genesis": self.genesis, "baseline_tick": sequence.initial_tick,
            "baseline_matter": sequence.matter, "actual_ticks": sequence.ticks,
            "replays": self.replays, "per_tick_replay_claimed": false,
        });
        retain_new(&format!("{}-evidence.json", self.prefix), &summary);
        eprintln!("MATRIX SAVED form={} original={} seed={} genesis={} ticks={}",
            self.draw.form.name(), self.draw.original, self.draw.seed, out.genesis, sequence.ticks);
    }
}

#[test]
fn matrix_keys_separate_all_120_arms() {
    let mut keys = std::collections::BTreeSet::new();
    for form in [Form::Bare, Form::Bodied, Form::Played] {
        for original in [false, true] {
            for seed in 0..20 {
                assert!(keys.insert(Draw { seed, form, original }.prefix("same-genesis")));
            }
        }
    }
    assert_eq!(keys.len(), 120);
}

#[test]
fn matrix_sequence_refuses_gaps_and_matter_drift() {
    let mut s = Sequence { initial_tick: 0, matter: 123, ticks: 0 };
    assert_eq!(s.accept(1, 123), Ok(()));
    assert_eq!(s.accept(3, 123), Err("noncontiguous actual ticks"));
    assert_eq!(s.accept(2, 122), Err("actual matter changed"));
    assert_eq!(s.ticks, 1);
    assert_eq!(s.accept(2, 123), Ok(()));
}
