// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Citations found in text: "ruling 975", "(rulings 258, 286)", "(742)",
//! "rulings 971 to 974", "case 9", "sim-execution D5". Generated, never judged.

use crate::model::Key;
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

static WORDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:[Rr]ulings?|[Cc]ases?)\s+(\d{1,4}(?:(?:,\s*(?:and\s+)?|\s+and\s+|\s+or\s+|\s+to\s+)\d{1,4})*)")
        .unwrap()
});
static BARE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\((\d{2,4}(?:,\s*\d{2,4})*(?:,?\s+and\s+\d{2,4})?)\)").unwrap()
});
static PLAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b([a-z][a-z0-9-]*) D(\d{1,3})\b").unwrap());
static NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+|\bto\b").unwrap());

/// What a body of text cites, restricted to case numbers up to `max_case`
/// and to plan keywords in `plans`. Ranges ("971 to 974") expand when short.
/// `record` is the file name of the repository's case record, if it has one.
pub struct Citer<'a> {
    pub max_case: u32,
    pub plans: &'a BTreeSet<String>,
    pub record: Option<&'a str>,
}

impl Citer<'_> {
    pub fn cite(&self, text: &str) -> BTreeSet<Key> {
        let mut out = BTreeSet::new();
        for cap in WORDED.captures_iter(text) {
            let mut last: Option<u32> = None;
            let mut range = false;
            for m in NUM.find_iter(&cap[1]) {
                if m.as_str() == "to" {
                    range = true;
                    continue;
                }
                let Ok(n) = m.as_str().parse::<u32>() else { continue };
                match (range, last) {
                    (true, Some(a)) if n > a && n - a <= 40 => (a + 1..=n).for_each(|k| self.case(k, &mut out)),
                    _ => self.case(n, &mut out),
                }
                range = false;
                last = Some(n);
            }
        }
        for cap in BARE.captures_iter(text) {
            for m in NUM.find_iter(&cap[1]) {
                if let Ok(n) = m.as_str().parse::<u32>() {
                    self.case(n, &mut out);
                }
            }
        }
        for cap in PLAN.captures_iter(text) {
            if self.plans.contains(&cap[1]) {
                if let Ok(n) = cap[2].parse() {
                    out.insert(Key::Plan(cap[1].to_string(), n));
                }
            }
        }
        out
    }

    fn case(&self, n: u32, out: &mut BTreeSet<Key>) {
        if (1..=self.max_case).contains(&n) {
            out.insert(Key::Case(n));
        }
    }
}

/// The comment text of a Rust source line, if any.
pub fn rust_comment(line: &str) -> Option<&str> {
    line.find("//").map(|i| &line[i + 2..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cites(text: &str) -> Vec<String> {
        let plans = BTreeSet::from(["sim-execution".to_string()]);
        let c = Citer { max_case: 984, plans: &plans, record: Some("2026-09-18_wing_design_plan.md") };
        c.cite(text).iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn worded_bare_ranged_and_plan_citations() {
        assert_eq!(cites("ruling 742, as (258, 286) said"), ["258", "286", "742"]);
        assert_eq!(cites("rulings 971 to 974"), ["971", "972", "973", "974"]);
        assert_eq!(cites("rulings 54 and 975"), ["54", "975"]);
        assert_eq!(cites("see sim-execution D5 and other D3"), ["sim-execution D5"]);
    }

    #[test]
    fn enumerations_years_and_large_numbers_are_not_cases() {
        assert!(cites("Options: (1) a fixed sun (2) a field").is_empty());
        assert!(cites("in (2026) we measured (1500)").is_empty());
    }
}
