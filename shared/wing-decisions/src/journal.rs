// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The link journal: every association and lapse, rebuilt from git history as
//! impresa records in a muniment journal. A local cache under
//! `target/wing-decisions/`, never committed; rebuilt when missing or when
//! HEAD no longer descends from where it stopped.

use crate::cite::Citer;
use crate::git::Repo;
use crate::scan::{self, KINDS};
use muniment::{DirectoryBackend, Journal, JsonSlots};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wing_impresa::{KindSet, KindSetLimits, KindSetSpec, Record, Stance};

const DOCTRINE: &str = "design_docs/doctrine.md";

type Triple = (String, String, String);

#[derive(Default, Serialize, Deserialize)]
struct State {
    /// The last first-parent commit folded in.
    last: Option<String>,
    tick: u64,
    /// Commits that changed the doctrine so far: a record's canon revision.
    revision: u64,
    per_file: BTreeMap<String, BTreeSet<Triple>>,
    /// How many files assert each triple; rebuilt from `per_file` on load.
    #[serde(skip)]
    count: BTreeMap<Triple, u32>,
}

pub fn kinds() -> KindSet {
    KindSet::new(KindSetSpec {
        version: wing_impresa::SCHEMA_VERSION,
        id: "wing:decision-links".into(),
        kinds: KINDS.iter().map(|k| k.to_string()).collect(),
        limits: KindSetLimits::default(),
    })
    .expect("the decision-link kinds are valid")
}

pub struct Updated {
    pub commits: usize,
    pub records: usize,
    pub total: usize,
}

/// Brings the journal up to HEAD and returns it.
pub fn update(repo: &Repo, citer: &Citer) -> Result<(Journal<Record>, Updated), String> {
    let dir = repo.root.join("target").join("wing-decisions");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let slots = JsonSlots::new(DirectoryBackend::open(&dir).map_err(|e| e.to_string())?);
    let load = || -> Result<(Journal<Record>, State), String> {
        pollster::block_on(async {
            let j = Journal::load(&slots, "journal").await.map_err(|e| e.to_string())?;
            let s: Option<State> = slots.load("state").await.map_err(|e| e.to_string())?;
            Ok((j, s.unwrap_or_default()))
        })
    };
    let (mut journal, mut state) = load()?;
    for t in state.per_file.values().flatten() {
        *state.count.entry(t.clone()).or_insert(0) += 1;
    }
    let commits = repo.first_parent()?;
    let start = match &state.last {
        Some(last) => match commits.iter().position(|(c, _)| c == last) {
            Some(i) => i + 1,
            None => {
                (journal, state) = (Journal::new(), State::default());
                0
            }
        },
        None => 0,
    };
    let kinds = kinds();
    let mut blobs = repo.blobs()?;
    let before = journal.len();
    for (commit, files) in commits.iter().skip(start) {
        let changed: Vec<String> = files.iter().filter(|p| scan::relevant(p)).cloned().collect();
        state.tick += 1;
        if changed.iter().any(|p| p == DOCTRINE) {
            state.revision += 1;
        }
        let mut added = BTreeSet::new();
        let mut removed = BTreeSet::new();
        for path in changed {
            let new = match blobs.text(&format!("{commit}:{path}"))? {
                Some(text) => scan::triples(&path, &scan::facts(&path, &text, citer)),
                None => BTreeSet::new(),
            };
            let old = state.per_file.remove(&path).unwrap_or_default();
            for t in new.difference(&old) {
                let c = state.count.entry(t.clone()).or_insert(0);
                *c += 1;
                if *c == 1 {
                    added.insert(t.clone());
                }
            }
            for t in old.difference(&new) {
                if let Some(c) = state.count.get_mut(t) {
                    *c -= 1;
                    if *c == 0 {
                        state.count.remove(t);
                        removed.insert(t.clone());
                    }
                }
            }
            if !new.is_empty() {
                state.per_file.insert(path, new);
            }
        }
        // A triple dropped in one file and added in another nets to nothing.
        let moved: BTreeSet<Triple> = added.intersection(&removed).cloned().collect();
        for (set, stance) in [(&removed, Stance::Lapse), (&added, Stance::Associate)] {
            for (subject, kind, object) in set.difference(&moved) {
                let r = Record {
                    subject: subject.clone(),
                    object: object.clone(),
                    kind: kind.clone(),
                    canon_revision: state.revision,
                    cause: commit.clone(),
                    tick: state.tick,
                    stance,
                };
                if r.validate(&kinds).is_ok() {
                    journal.append(r);
                }
            }
        }
        state.last = Some(commit.clone());
    }
    let records = journal.len() - before;
    pollster::block_on(async {
        journal.save(&slots, "journal").await.map_err(|e| e.to_string())?;
        slots.save("state", &state).await.map_err(|e| e.to_string())
    })?;
    let total = journal.len();
    Ok((journal, Updated { commits: commits.len() - start, records, total }))
}
