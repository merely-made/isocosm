// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Answers from the committed manifest: a decision's neighbourhood, what cites
//! a path, an area's decisions.

use crate::model::Key;
use std::fmt::Write;
use toml::{Table, Value};

pub struct Lookup {
    pub cases: Table,
    pub decisions: Table,
    pub statements: Table,
    pub foreign: Table,
}

const FIELDS: &[&str] = &[
    "area",
    "summary",
    "draws_on",
    "amends",
    "replaces",
    "contradicts",
    "plans",
    "governs",
    "stack",
    "mentions",
    "mentioned_by",
    "drawn_on_by",
    "amended_by",
    "replaced_by",
    "contradicted_by",
    "cited_in_docs",
    "cited_in_code",
];

impl Lookup {
    pub fn parse(text: &str) -> Result<Lookup, String> {
        let mut t: Table = text.parse().map_err(|e| format!("manifest: {e}"))?;
        let take = |t: &mut Table, k: &str| match t.remove(k) {
            Some(Value::Table(x)) => x,
            _ => Table::new(),
        };
        Ok(Lookup {
            cases: take(&mut t, "cases"),
            decisions: take(&mut t, "decisions"),
            statements: take(&mut t, "statements"),
            foreign: take(&mut t, "foreign"),
        })
    }

    fn item(&self, key: &Key) -> Option<&Table> {
        match key {
            Key::Case(n) => self.cases.get(&n.to_string()),
            Key::Plan(..) => self.decisions.get(&key.to_string()),
            Key::Statement(..) => self.statements.get(&key.to_string()),
            Key::Foreign(..) => self.foreign.get(&key.to_string()),
        }
        .and_then(Value::as_table)
    }

    fn all(&self) -> impl Iterator<Item = (String, &Table)> {
        let c = self.cases.iter().map(|(k, v)| (k.clone(), v));
        let d = self.decisions.iter().map(|(k, v)| (k.clone(), v));
        let st = self.statements.iter().map(|(k, v)| (k.clone(), v));
        c.chain(d).chain(st).filter_map(|(k, v)| v.as_table().map(|t| (k, t)))
    }

    /// Whether the manifest declares `key` (a title, not only citations).
    pub fn declares(&self, key: &Key) -> bool {
        self.item(key).is_some_and(|t| t.contains_key("title"))
    }

    /// One decision, its declared links and everything pointing back.
    pub fn decision(&self, key: &Key) -> Option<String> {
        let t = self.item(key)?;
        let mut s = String::new();
        let title = t.get("title").and_then(Value::as_str).unwrap_or("(not live: cited, but declared nowhere)");
        let _ = writeln!(s, "{key}: {title}");
        if let Some(src) = t.get("source").and_then(Value::as_str) {
            let _ = writeln!(s, "  at {src}");
        }
        for f in FIELDS {
            match t.get(*f) {
                Some(Value::Array(a)) if *f == "cited_in_code" && a.len() > 8 => {
                    let shown: Vec<String> = a.iter().take(8).filter_map(|v| v.as_str().map(str::to_string)).collect();
                    let _ = writeln!(s, "  {f} ({}): {}, ...", a.len(), shown.join(", "));
                }
                Some(Value::Array(a)) => {
                    let v: Vec<String> = a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
                    let _ = writeln!(s, "  {f}: {}", v.join(", "));
                }
                Some(Value::String(v)) => {
                    let _ = writeln!(s, "  {f}: {v}");
                }
                Some(Value::Boolean(true)) => {
                    let _ = writeln!(s, "  {f}");
                }
                _ => {}
            }
        }
        Some(s)
    }

    /// A plan's decisions by its keyword, `sim-execution`, in D-number order;
    /// `None` when the keyword declares none.
    pub fn plan(&self, keyword: &str) -> Option<String> {
        let mut found: Vec<(u32, &str)> = self
            .decisions
            .iter()
            .filter_map(|(k, v)| match Key::parse(k)? {
                Key::Plan(p, n) if p == keyword => Some((n, v.get("title").and_then(Value::as_str).unwrap_or(""))),
                _ => None,
            })
            .collect();
        found.sort();
        let mut s = String::new();
        for (n, title) in &found {
            let _ = writeln!(s, "{keyword} D{n}: {title}");
        }
        (!found.is_empty()).then_some(s)
    }

    /// What cites, or is governed in, a file or directory.
    pub fn path(&self, path: &str) -> String {
        let p = path.replace('\\', "/");
        let mut s = String::new();
        for (k, t) in self.all() {
            let title = t.get("title").and_then(Value::as_str).unwrap_or("");
            let hits: Vec<String> = ["cited_in_code", "cited_in_docs"]
                .iter()
                .filter_map(|f| t.get(*f).and_then(Value::as_array))
                .flatten()
                .filter_map(Value::as_str)
                .filter(|v| v.starts_with(&p))
                .map(str::to_string)
                .collect();
            if !hits.is_empty() {
                let _ = writeln!(s, "{k}: {title}\n  {}", hits.join(", "));
            }
        }
        if s.is_empty() { format!("nothing cites {p}\n") } else { s }
    }

    /// The decisions declared in an area.
    pub fn area(&self, area: &str) -> String {
        let mut s = String::new();
        for (k, t) in self.all() {
            if t.get("area").and_then(Value::as_str).is_some_and(|a| a.eq_ignore_ascii_case(area)) {
                let title = t.get("title").and_then(Value::as_str).unwrap_or("");
                let _ = writeln!(s, "{k}: {title}");
            }
        }
        if s.is_empty() { format!("no decision declares area {area:?} yet\n") } else { s }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_plan_keyword_lists_its_decisions_in_number_order() {
        let l = Lookup::parse(
            "[decisions.\"p D10\"]\ntitle = \"ten\"\n[decisions.\"p D2\"]\ntitle = \"two\"\n[decisions.\"q D1\"]\ntitle = \"other\"\n",
        )
        .unwrap();
        assert_eq!(l.plan("p").as_deref(), Some("p D2: two\np D10: ten\n"));
        assert_eq!(l.plan("r"), None);
    }
}
