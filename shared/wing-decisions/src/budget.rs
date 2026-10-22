// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The doctrine's size budget (`DOC_POLICY.md`, "Decisions and doctrine",
//! rule 5): a cap on each area's statements and one on every statement's
//! body lines. Fields lines and notes (paragraphs opening with `*`) do not
//! count. Overruns are reported, never refused.

use serde::Deserialize;
use std::collections::BTreeMap;

pub const BUDGET: &str = "design_docs/doctrine_budget.toml";

#[derive(Debug, Deserialize)]
pub struct Budget {
    pub lines_per_statement: usize,
    /// Area heading to its statement cap.
    pub statements: BTreeMap<String, usize>,
}

/// One area's statements and their body lines, in doctrine order.
#[derive(Debug, Default, PartialEq)]
pub struct Area {
    pub name: String,
    pub statements: Vec<(String, usize)>,
}

impl Budget {
    pub fn parse(text: &str) -> Result<Budget, String> {
        toml::from_str(text).map_err(|e| format!("{BUDGET}: {e}"))
    }

    /// Every overrun, one line each; empty when the doctrine fits.
    pub fn overruns(&self, areas: &[Area]) -> Vec<String> {
        let mut out = Vec::new();
        for a in areas {
            match self.statements.get(&a.name) {
                None => out.push(format!("{}: no budget", a.name)),
                Some(&cap) if a.statements.len() > cap => {
                    out.push(format!("{}: {} statements, budget {cap}", a.name, a.statements.len()))
                }
                Some(_) => {}
            }
            for (title, lines) in a.statements.iter().filter(|(_, n)| *n > self.lines_per_statement) {
                out.push(format!("{title}: {lines} lines, budget {}", self.lines_per_statement));
            }
        }
        out
    }
}

/// Counts each statement's body lines, area by area.
pub fn measure(doctrine: &str) -> Vec<Area> {
    let mut areas: Vec<Area> = Vec::new();
    let (mut opens, mut body) = (true, false);
    for l in doctrine.lines() {
        if let Some(name) = l.strip_prefix("## ") {
            areas.push(Area { name: name.trim().to_string(), statements: Vec::new() });
            opens = true;
        } else if let Some(title) = l.strip_prefix("### ") {
            if let Some(a) = areas.last_mut() {
                a.statements.push((title.trim().to_string(), 0));
            }
            opens = true;
        } else if l.trim().is_empty() {
            opens = true;
        } else {
            if opens {
                let t = l.trim_start();
                body = !(t.starts_with('*') || t.starts_with(crate::model::PREFIX));
                opens = false;
            }
            if let Some(s) = areas.last_mut().and_then(|a| a.statements.last_mut()).filter(|_| body) {
                s.1 += 1;
            }
        }
    }
    areas
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCTRINE: &str = "# D\n\nStatus.\n\n## Naming\n\n### Naming 1. The family\n\nOne line\nand two.\n\n*Not yet built:* a\nnote.\n\nLinks: draws on 1 · summary S\n\n*Reviewed, new.*\n\n### Naming 2. Ladder\n\nOne.\n\n## Method\n";

    #[test]
    fn bodies_count_and_notes_and_fields_do_not() {
        let a = measure(DOCTRINE);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].statements, [("Naming 1. The family".to_string(), 2), ("Naming 2. Ladder".to_string(), 1)]);
        assert!(a[1].statements.is_empty());
    }

    #[test]
    fn overruns_name_the_area_and_the_statement() {
        let b = Budget::parse("lines_per_statement = 1\n[statements]\nNaming = 1\n").unwrap();
        let over = b.overruns(&measure(DOCTRINE));
        assert_eq!(over, ["Naming: 2 statements, budget 1", "Naming 1. The family: 2 lines, budget 1", "Method: no budget"]);
    }
}
