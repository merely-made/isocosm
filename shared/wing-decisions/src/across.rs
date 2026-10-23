// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Keys in other repositories (`mere:stack.1`), checked against the committed
//! manifest of the checkout beside this one (DOC_POLICY core §11, rule 1).

use crate::MANIFEST;
use crate::git::Repo;
use crate::lookup::Lookup;
use crate::manifest::Manifest;
use crate::model::Key;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Another repository's committed manifest, read at its HEAD from the
/// checkout beside `root`; None when there is no such checkout. A checkout
/// with no manifest declares nothing.
pub fn manifest(root: &Path, repo: &str) -> Option<Lookup> {
    let dir = root.parent()?.join(repo);
    if !dir.join(".git").exists() {
        return None;
    }
    let text = Repo { root: dir }.git(&["show", &format!("HEAD:{MANIFEST}")]).unwrap_or_default();
    Lookup::parse(&text).ok()
}

/// What checking found: how many keys were checked, those the other
/// repository does not declare (with what links to them here), and the
/// repositories with no checkout to check against.
#[derive(Default)]
pub struct Report {
    pub checked: usize,
    pub broken: Vec<String>,
    pub unchecked: Vec<String>,
}

pub fn check(root: &Path, m: &Manifest) -> Report {
    let mut by_repo: BTreeMap<&str, Vec<(&Key, BTreeSet<&Key>)>> = BTreeMap::new();
    for (k, item) in &m.items {
        if let Key::Foreign(repo, inner) = k {
            let from = [&item.mentioned_by, &item.drawn_on_by, &item.amended_by, &item.replaced_by, &item.contradicted_by]
                .into_iter()
                .flatten()
                .collect();
            by_repo.entry(repo).or_default().push((inner, from));
        }
    }
    let mut out = Report::default();
    for (repo, keys) in by_repo {
        let Some(theirs) = manifest(root, repo) else {
            out.unchecked.push(repo.to_string());
            continue;
        };
        out.checked += keys.len();
        for (inner, from) in keys.iter().filter(|(k, _)| !theirs.declares(k)) {
            let from: Vec<String> = from.iter().map(|k| k.to_string()).collect();
            out.broken.push(format!("{repo}:{inner}, linked from {}", from.join(", ")));
        }
    }
    out
}
