// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Decisions as written: the design record's §0 cases and each plan's
//! Decisions entries, with their fields lines and what their text cites.

use crate::cite::Citer;
use crate::model::{self, Key, Links};
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

/// One decision as found in a document.
#[derive(Clone, Debug)]
pub struct Entry {
    pub key: Key,
    pub title: String,
    pub source: String,
    pub line: usize,
    pub links: Links,
    pub mentions: BTreeSet<Key>,
    pub errors: Vec<String>,
}

static CASE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d{1,4})\. \*\*(.+?)(?:\*\*|$)").unwrap());
static STATEMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^### (.+?) (\d{1,3})\. (.+)$").unwrap());
static DECISION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^- \*\*D(\d{1,3})\b[,:]?\s*(.*?)(?::?\*\*|$)").unwrap());

/// The case numbers a design record's §0 holds, cheaply, for the citer.
pub fn case_numbers(record: &str) -> BTreeSet<u32> {
    section(record, "## 0.", "## 1.")
        .filter_map(|(_, l)| CASE.captures(l).and_then(|c| c[1].parse().ok()))
        .collect()
}

/// The design record's §0 cases.
pub fn record_cases(path: &str, text: &str, citer: &Citer) -> Vec<Entry> {
    let lines: Vec<(usize, &str)> = section(text, "## 0.", "## 1.").collect();
    entries(path, &lines, citer, |l| {
        CASE.captures(l).and_then(|c| Some((Key::Case(c[1].parse().ok()?), title(&c[2]))))
    })
}

/// A plan's keyed decisions, under any heading that starts "## Decisions".
pub fn plan_decisions(path: &str, text: &str, keyword: &str, citer: &Citer) -> Vec<Entry> {
    let lines: Vec<(usize, &str)> = section(text, "## Decisions", "## ").collect();
    entries(path, &lines, citer, |l| {
        DECISION.captures(l).and_then(|c| Some((Key::Plan(keyword.to_string(), c[1].parse().ok()?), title(&c[2]))))
    })
}

/// The doctrine's statements, `### Naming 1. The family`, keyed `naming.1`.
pub fn doctrine_statements(path: &str, text: &str, citer: &Citer) -> Vec<Entry> {
    let lines: Vec<(usize, &str)> = text.lines().enumerate().map(|(i, l)| (i + 1, l)).collect();
    let mut found = entries(path, &lines, citer, |l| {
        STATEMENT.captures(l).and_then(|c| Some((Key::Statement(model::slug(&c[1]), c[2].parse().ok()?), title(&c[3]))))
    });
    // A statement's text ends at the next heading of any level.
    for e in &mut found {
        if e.links.area.is_none() {
            e.links.area = lines.iter().take(e.line).rev().find_map(|(_, l)| l.strip_prefix("## ")).map(|a| a.trim().to_string());
        }
    }
    found
}

/// Lines from the heading starting `from` to the next heading starting
/// `until`, numbered from 1.
fn section<'a>(text: &'a str, from: &'a str, until: &'a str) -> impl Iterator<Item = (usize, &'a str)> + 'a {
    let mut inside = false;
    text.lines().enumerate().filter_map(move |(i, l)| {
        if l.starts_with(from) {
            inside = true;
            return None;
        }
        if inside && l.starts_with(until) && !l.starts_with(from) {
            inside = false;
        }
        inside.then_some((i + 1, l))
    })
}

fn title(s: &str) -> String {
    s.trim().trim_end_matches(['.', ':']).trim().to_string()
}

fn entries(
    path: &str,
    lines: &[(usize, &str)],
    citer: &Citer,
    start: impl Fn(&str) -> Option<(Key, String)>,
) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    let mut body = String::new();
    let finish = |out: &mut Vec<Entry>, body: &mut String| {
        if let Some(e) = out.last_mut() {
            e.mentions = citer.cite(body);
            e.mentions.remove(&e.key);
        }
        body.clear();
    };
    for &(n, l) in lines {
        if let Some((key, title)) = start(l) {
            finish(&mut out, &mut body);
            out.push(Entry {
                key,
                title,
                source: path.to_string(),
                line: n,
                links: Links::default(),
                mentions: BTreeSet::new(),
                errors: Vec::new(),
            });
        }
        let Some(e) = out.last_mut() else { continue };
        // A bold title that wraps: carry it on until its closing `**`.
        if e.line != n && body.matches("**").count() % 2 == 1 {
            let part = l.trim().split("**").next().unwrap_or("");
            e.title = title(&format!("{} {part}", e.title));
        }
        if l.trim_start().starts_with(model::PREFIX) {
            match model::parse_links(l) {
                Ok(links) => e.links = links,
                Err(err) => e.errors.push(format!("{path}:{n}: {err}")),
            }
        } else {
            body.push_str(l);
            body.push('\n');
        }
    }
    finish(&mut out, &mut body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECORD: &str = "# R\n\n## 0. Cases\n\n1. **One thing.** Draws on nothing.\n\n2. **Two\n   things.** As ruling 1 said (1).\n\n   Links: area Ecology · draws on 1 · summary Two things\n\n## 1. Next\n\n3. **Not a case.**\n";

    #[test]
    fn record_cases_take_text_links_and_mentions() {
        let plans = BTreeSet::new();
        let c = Citer { max_case: 984, plans: &plans, record: Some("2026-09-18_wing_design_plan.md") };
        let cases = record_cases("r.md", RECORD, &c);
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[1].key, Key::Case(2));
        assert_eq!(cases[1].title, "Two things");
        assert_eq!(cases[1].line, 7);
        assert_eq!(cases[1].links.area.as_deref(), Some("Ecology"));
        assert_eq!(cases[1].mentions, BTreeSet::from([Key::Case(1)]));
        assert_eq!(case_numbers(RECORD), BTreeSet::from([1, 2]));
    }

    #[test]
    fn plan_decisions_are_keyed_by_plan() {
        let plans = BTreeSet::new();
        let c = Citer { max_case: 984, plans: &plans, record: Some("2026-09-18_wing_design_plan.md") };
        let text = "## Decisions\n\n- **D1, draw keys (2026-10-19):** \"Actor\".\n  More (975).\n- **D2, near:** x\n\n## Findings\n\n- **D9, no:** y\n";
        let d = plan_decisions("p.md", text, "sim-execution", &c);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].key.to_string(), "sim-execution D1");
        assert_eq!(d[0].title, "draw keys (2026-10-19)");
        assert!(d[0].mentions.contains(&Key::Case(975)));
    }
}
