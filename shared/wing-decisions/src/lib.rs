// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The wing's decisions as linked structure (isocosm `DOC_POLICY.md`,
//! "Decisions and doctrine"; plan
//! `design_docs/2026-10-20_decision_links_plan.md`). Reads the design
//! record's cases, plans' keyed decisions, fields lines and citations; writes
//! the manifest and register; keeps the link journal; answers lookups; checks
//! anchors against rust-analyzer's index.

pub mod across;
pub mod anchors;
pub mod budget;
pub mod cite;
pub mod docs;
pub mod git;
pub mod journal;
pub mod lookup;
pub mod manifest;
pub mod model;
pub mod scan;

use cite::Citer;
use git::Repo;
use std::collections::BTreeSet;

pub const MANIFEST: &str = "design_docs/decisions.toml";
pub const DOCTRINE: &str = "design_docs/doctrine.md";
/// Per-repository settings; absent, a repository has no case record.
pub const CONFIG: &str = "design_docs/decisions_config.toml";

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The case record's file name, matched wherever it lives.
    pub record: Option<String>,
}

/// Everything relevant in the index, read once.
pub struct Snapshot {
    pub files: Vec<(String, String)>,
    pub max_case: u32,
    pub plans: BTreeSet<String>,
    pub config: Config,
}

impl Snapshot {
    pub fn from_index(repo: &Repo) -> Result<Snapshot, String> {
        let config: Config = match repo.git(&["show", &format!(":{CONFIG}")]) {
            Ok(text) => toml::from_str(&text).map_err(|e| format!("{CONFIG}: {e}"))?,
            Err(_) => Config::default(),
        };
        let record = config.record.as_deref();
        let mut blobs = repo.blobs()?;
        let mut files = Vec::new();
        for (path, blob) in repo.index()? {
            if scan::relevant(&path) {
                if let Some(text) = blobs.text(&blob)? {
                    files.push((path, text));
                }
            }
        }
        let max_case = files
            .iter()
            .filter(|(p, _)| scan::is_record(p, record))
            .flat_map(|(_, t)| docs::case_numbers(t))
            .max()
            .unwrap_or(0);
        let plans = files.iter().filter_map(|(p, _)| model::plan_keyword(p).filter(|_| scan::is_doc(p))).collect();
        Ok(Snapshot { files, max_case, plans, config })
    }

    pub fn citer(&self) -> Citer<'_> {
        Citer { max_case: self.max_case, plans: &self.plans, record: self.config.record.as_deref() }
    }

    pub fn manifest(&self) -> manifest::Manifest {
        let citer = self.citer();
        let facts: Vec<(String, scan::Facts)> =
            self.files.iter().map(|(p, t)| (p.clone(), scan::facts(p, t, &citer))).collect();
        manifest::Manifest::build(&facts)
    }

    /// The doctrine's area headings, in order.
    pub fn areas(&self) -> Vec<String> {
        self.files
            .iter()
            .find(|(p, _)| p == DOCTRINE)
            .map(|(_, t)| t.lines().filter_map(|l| l.strip_prefix("## ")).map(|a| a.trim().to_string()).collect())
            .unwrap_or_default()
    }
}
