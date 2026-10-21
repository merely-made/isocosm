// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The anchor probe's answer key (`Code/testing/isometry/anchor-probe-2026-10-20`)
//! replayed through the resolver: every definition of 2026-10-08 judged
//! present, moved or removed against a SCIP index of `afbce8a0`.
//! `cargo run --release --example answer_key -- <index.scip> <truth.json>`

use std::collections::BTreeMap;
use wing_decisions::anchors::{Found, Index};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let index = Index::load(&std::fs::read(&args[0]).expect("index")).expect("decode");
    let truth: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(&args[1]).expect("truth")).expect("json");
    let files = |paths: &[String]| -> Vec<String> {
        paths.iter().flat_map(|p| index.by_path[p].iter().map(|(f, _)| format!("shared/isocosm/{f}"))).collect()
    };
    let mut tally: BTreeMap<(String, String), u32> = BTreeMap::new();
    let mut wrong = Vec::new();
    for row in &truth {
        let status = row["status"].as_str().unwrap();
        if status == "renamed_or_kind_changed" {
            continue;
        }
        let old = row["old_file"].as_str().unwrap();
        let Some(rel) = old.strip_prefix("shared/isocosm/src/").and_then(|r| r.strip_suffix(".rs")) else { continue };
        if rel.starts_with("bin/") {
            continue;
        }
        let mods: Vec<&str> = rel.split('/').filter(|m| *m != "mod" && *m != "lib").collect();
        let anchor = std::iter::once("isocosm").chain(mods).chain([row["name"].as_str().unwrap()]).collect::<Vec<_>>().join("::");
        let found_in = match index.resolve(&anchor) {
            Found::Present(at) => at.iter().map(|(f, _)| format!("shared/isocosm/{f}")).collect(),
            Found::Moved(paths) => files(&paths),
            Found::Gone => Vec::new(),
        };
        let judged = if found_in.iter().any(|f| f == old) {
            "present"
        } else if found_in.is_empty() {
            "removed"
        } else {
            "moved"
        };
        *tally.entry((status.to_string(), judged.to_string())).or_default() += 1;
        if judged != status && wrong.len() < 10 {
            wrong.push(format!("{status} judged {judged}: {anchor}"));
        }
    }
    for ((want, got), n) in &tally {
        println!("{want:8} -> {got:8} {n}");
    }
    for w in &wrong {
        println!("  {w}");
    }
    for a in ["isocosm::rules::effect::Effect::Sunlight", "isocosm::simulation::Receipt::foregone"] {
        println!("{a}: {:?}", index.resolve(a));
    }
}
