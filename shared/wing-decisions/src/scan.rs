// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which files carry decisions or cite them, and what each one says: entries
//! from the record and plans, citations from design docs and Rust comments.

use crate::cite::{Citer, rust_comment};
use crate::docs::{self, Entry};
use crate::model::{self, Key, anchor_id, slug};
use std::collections::BTreeSet;

/// Whether `path` is the case record named `record`, wherever it lives (a
/// file name, so the record's history reads the same across moves).
pub fn is_record(path: &str, record: Option<&str>) -> bool {
    record.is_some_and(|r| path.ends_with(r)) && !path.contains("archive_docs/")
}

/// Files this tool writes; they cite everything, so they cite nothing.
pub const GENERATED: &[&str] = &["case_register.md"];

pub fn is_doc(path: &str) -> bool {
    path.ends_with(".md")
        && path.contains("design_docs/")
        && !path.contains("archive_docs/")
        && !GENERATED.iter().any(|g| path.ends_with(&format!("/{g}")))
}

pub fn is_rust(path: &str) -> bool {
    path.ends_with(".rs") && !path.starts_with("vendor/") && !path.contains("/target/")
}

pub fn relevant(path: &str) -> bool {
    is_doc(path) || is_rust(path)
}

/// What one file says.
#[derive(Default)]
pub struct Facts {
    pub entries: Vec<Entry>,
    /// Keys cited, with the 1-based line they are cited on.
    pub cites: Vec<(Key, usize)>,
}

pub fn facts(path: &str, text: &str, citer: &Citer) -> Facts {
    let mut f = Facts::default();
    if is_record(path, citer.record) {
        f.entries = docs::record_cases(path, text, citer);
    } else if path.ends_with("/doctrine.md") && is_doc(path) {
        f.entries = docs::doctrine_statements(path, text, citer);
    } else if let Some(k) = model::plan_keyword(path).filter(|_| is_doc(path)) {
        f.entries = docs::plan_decisions(path, text, &k, citer);
    }
    if is_rust(path) {
        for (i, line) in text.lines().enumerate() {
            if let Some(c) = rust_comment(line) {
                f.cites.extend(citer.cite(c).into_iter().map(|k| (k, i + 1)));
            }
        }
    } else if is_doc(path) && !is_record(path, citer.record) && !path.ends_with("/doctrine.md") {
        for (i, line) in text.lines().enumerate() {
            f.cites.extend(citer.cite(line).into_iter().map(|k| (k, i + 1)));
        }
    }
    f
}

/// A file's facts as impresa triples (subject, kind, object), for the
/// journal. Line numbers are left out: they move with every edit.
pub fn triples(path: &str, f: &Facts) -> BTreeSet<(String, String, String)> {
    let mut out = BTreeSet::new();
    let file = format!("file:{path}");
    for (k, _) in &f.cites {
        out.insert((file.clone(), "link:cites".into(), k.id()));
    }
    for e in &f.entries {
        let s = e.key.id();
        let mut add = |kind: &str, object: String| {
            out.insert((s.clone(), format!("link:{kind}"), object));
        };
        add("declared-in", file.clone());
        e.mentions.iter().for_each(|k| add("mentions", k.id()));
        let l = &e.links;
        if let Some(a) = &l.area {
            add("area", format!("area:{}", slug(a)));
        }
        l.draws_on.iter().for_each(|k| add("draws-on", k.id()));
        l.amends.iter().for_each(|k| add("amends", k.id()));
        l.replaces.iter().for_each(|k| add("replaces", k.id()));
        l.contradicts.iter().for_each(|k| add("contradicts", k.id()));
        l.plans.iter().for_each(|p| add("plans", format!("plan:{p}")));
        l.governs.iter().for_each(|g| add("governs", anchor_id(g)));
        if l.stack {
            add("scope", "scope:stack".into());
        }
    }
    out
}

/// Every kind a triple can carry, for the impresa kind set.
pub const KINDS: &[&str] = &[
    "link:cites",
    "link:declared-in",
    "link:mentions",
    "link:area",
    "link:draws-on",
    "link:amends",
    "link:replaces",
    "link:contradicts",
    "link:plans",
    "link:governs",
    "link:scope",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_comments_cite_and_code_does_not() {
        let plans = BTreeSet::new();
        let c = Citer { max_case: 984, plans: &plans, record: Some("2026-09-18_wing_design_plan.md") };
        let f = facts("shared/a.rs", "let x = f(42); // the share rule (975)\n/// ruling 338\n", &c);
        let keys: Vec<String> = f.cites.iter().map(|(k, l)| format!("{k}@{l}")).collect();
        assert_eq!(keys, ["975@1", "338@2"]);
        assert!(!relevant("vendor/taplo/src/lib.rs"));
        assert!(!relevant("mesocosm/design_docs/archive_docs/x/y.md"));
    }
}
