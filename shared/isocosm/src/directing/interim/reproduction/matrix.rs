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
    #[serde(skip_serializing_if = "is_false")]
    pub recovery: bool,
}

fn is_false(value: &bool) -> bool { !value }

impl Draw {
    fn prefix(self, genesis: &str) -> String {
        let form = self.form.name();
        let arm = if self.original { "original" } else { "candidate" };
        let owner = if self.recovery {
            "a1-seeded-958-recovery"
        } else if compact() {
            "a1-seeded-958-matrix"
        } else {
            "q3-primary-final-state-matrix"
        };
        format!("{owner}-{form}-{arm}-seed{}-{genesis}", self.seed)
    }
}

fn compact() -> bool {
    std::env::var("ISOCOSM_REPRO_COMPACT").as_deref() == Ok("1")
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
    stream: Option<File>,
    buffered_rows: Vec<TickReading>,
    sequence: Option<Sequence>,
    replays: Vec<ReplayReading>,
    founding: crate::Founding,
    profile_adjustment: Option<String>,
}

fn new_file(name: &str) -> File {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(receipt_directory().join(name))
        .expect("fresh matrix receipt path; never overwrite")
}

fn write_new_bytes(name: &str, bytes: &[u8]) {
    let mut file = new_file(name);
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn retain_new<T: serde::Serialize>(name: &str, value: &T) {
    static RETAIN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = RETAIN.lock().unwrap();
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    let recovery_receipt = name.starts_with("a1-seeded-958-recovery-");
    if compact() || recovery_receipt {
        let retained: u64 = std::fs::read_dir(receipt_directory()).unwrap()
            .map(|entry| entry.unwrap())
            .filter(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.starts_with("a1-seeded-958-matrix-")
                    || name.starts_with("a1-seeded-958-recovery-")
            })
            .map(|entry| entry.metadata().unwrap().len()).sum();
        // Reserve 4 MB for all 120 bounded conservation streams and run logs.
        assert!(retained + bytes.len() as u64 <= 96_000_000, "compact matrix receipt cap");
    }
    let mut file = new_file(name);
    file.write_all(&bytes).unwrap();
    file.sync_all().unwrap();
}

fn validate_bodied(g: &Genesis, form: Form) {
    assert_eq!(g.founding.as_ref().unwrap().bodies, form.bodies());
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
            validate_bodied(g, draw.form);
        } else {
            assert!(g.founding.as_ref().unwrap().bodies.is_none(), "bare matrix form");
        }
        let genesis = crate::digest(g);
        let founding = g.founding.clone().expect("generated A1 founding");
        let profile_adjustment = draw.recovery.then(|| {
            let tweak = std::env::var("ISOCOSM_REPRO_TWEAK").unwrap_or_default();
            assert!(tweak.is_empty(), "bodied-stock recovery is the untuned stock control");
            tweak
        });
        let prefix = draw.prefix(&genesis);
        if draw.recovery {
            let retained: u64 = std::fs::read_dir(receipt_directory()).unwrap()
                .map(|entry| entry.unwrap())
                .filter(|entry| {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    name.starts_with("a1-seeded-958-matrix-")
                        || name.starts_with("a1-seeded-958-recovery-")
                })
                .map(|entry| entry.metadata().unwrap().len()).sum();
            assert!(retained + 4_000_000 <= 100_000_000,
                "recovery prewrite cap reserves stream and log space");
        }
        if !draw.recovery { retain_new(&format!("{prefix}-genesis.json"), g); }
        let stream = (!draw.recovery).then(|| new_file(&format!("{prefix}-conservation.jsonl")));
        Self { draw, genesis, prefix, stream, buffered_rows: Vec::new(), sequence: None, replays: Vec::new(), founding, profile_adjustment }
    }

    fn line(&mut self, row: &TickReading) {
        if let Some(stream) = &mut self.stream {
            serde_json::to_writer(&mut *stream, row).unwrap();
            stream.write_all(b"\n").unwrap();
        } else {
            self.buffered_rows.push(TickReading { kind: row.kind, tick: row.tick, matter: row.matter });
        }
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
        let (baseline_tick, baseline_matter, actual_ticks) =
            (sequence.initial_tick, sequence.matter, sequence.ticks);
        assert_eq!(out.genesis, self.genesis);
        assert_eq!(baseline_tick + actual_ticks, out.tick);
        assert_eq!(self.replays.iter().map(|r| r.mode.as_str()).collect::<Vec<_>>(),
            vec!["Individuals", "Grouped"]);
        if let Some(stream) = &self.stream {
            stream.sync_all().unwrap();
        }
        if self.draw.recovery {
            self.finish_recovery(saved, out, baseline_tick, baseline_matter, actual_ticks);
            return;
        }
        retain_new(&format!("{}-session.json", self.prefix), saved);
        // A native save plus its two verified replays reproduces the state.
        // Full explored states can be hundreds of MB; a declared compact gate
        // keeps the replay recipe and exact witness without duplicating it.
        let compact = compact() || self.draw.recovery;
        if !compact {
            retain_new(&format!("{}-state.json", self.prefix), state);
        }
        retain_new(&format!("{}-outcome.json", self.prefix), out);
        let summary = serde_json::json!({
            "schema": "q3-matrix-draw-complete-v1", "draw": self.draw,
            "genesis": self.genesis, "baseline_tick": baseline_tick,
            "baseline_matter": baseline_matter, "actual_ticks": actual_ticks,
            "replays": self.replays, "per_tick_replay_claimed": false,
            "full_state_retained": !compact,
        });
        retain_new(&format!("{}-evidence.json", self.prefix), &summary);
        eprintln!("MATRIX SAVED form={} original={} seed={} genesis={} ticks={}",
            self.draw.form.name(), self.draw.original, self.draw.seed, out.genesis, actual_ticks);
    }

    fn finish_recovery<S: serde::Serialize>(
        self, saved: &S, out: &Outcome, baseline_tick: Tick, baseline_matter: u128, actual_ticks: u64,
    ) {
        let recipe_name = format!("{}-recipe.json", self.prefix);
        let stream_name = format!("{}-conservation.jsonl", self.prefix);
        let outcome_name = format!("{}-outcome.json", self.prefix);
        let evidence_name = format!("{}-evidence.json", self.prefix);
        let recipe_bytes = recovery::validated_recipe_bytes(
            &self.founding, self.draw.form, self.draw.original,
            self.profile_adjustment.as_deref(), &self.genesis, saved,
        );
        let mut stream_bytes = Vec::new();
        for row in &self.buffered_rows {
            serde_json::to_writer(&mut stream_bytes, row).unwrap();
            stream_bytes.push(b'\n');
        }
        let outcome_bytes = serde_json::to_vec_pretty(out).unwrap();
        let summary = serde_json::json!({
            "schema": "a1-seeded-958-recovery-draw-complete-v1", "draw": self.draw,
            "genesis": self.genesis, "baseline_tick": baseline_tick,
            "baseline_matter": baseline_matter, "actual_ticks": actual_ticks,
            "replays": self.replays, "per_tick_replay_claimed": false,
            "full_state_retained": false, "recipe_file": recipe_name,
            "native_saved_full_genesis_omitted": true,
        });
        let evidence_bytes = serde_json::to_vec_pretty(&summary).unwrap();
        let outputs = [
            (&recipe_name, recipe_bytes.as_slice()),
            (&stream_name, stream_bytes.as_slice()),
            (&outcome_name, outcome_bytes.as_slice()),
            (&evidence_name, evidence_bytes.as_slice()),
        ];
        let retained: u64 = std::fs::read_dir(receipt_directory()).unwrap()
            .map(|entry| entry.unwrap())
            .filter(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.starts_with("a1-seeded-958-matrix-")
                    || name.starts_with("a1-seeded-958-recovery-")
            })
            .map(|entry| entry.metadata().unwrap().len()).sum();
        let planned: u64 = outputs.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        assert!(retained + planned <= 96_000_000,
            "recovery draw prewrite cap preserves 4 MB for stream and run logs");
        for (name, _) in &outputs {
            assert!(!receipt_directory().join(name).exists(), "recovery writes only fresh receipts");
        }
        for (name, bytes) in outputs {
            write_new_bytes(name, bytes);
        }
        eprintln!("MATRIX SAVED form=bodied original=true seed={} genesis={} ticks={}",
            self.draw.seed, out.genesis, actual_ticks);
    }
}

#[test]
fn matrix_keys_separate_all_120_arms() {
    let mut keys = std::collections::BTreeSet::new();
    for form in [Form::Bare, Form::Bodied, Form::Played] {
        for original in [false, true] {
            for seed in 0..20 {
                assert!(keys.insert(Draw { seed, form, original, recovery: false }.prefix("same-genesis")));
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

mod recovery;

pub(super) fn run_recovery() {
    recovery::run_missing_bodied_original();
}
