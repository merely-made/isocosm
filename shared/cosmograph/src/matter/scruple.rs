// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A part's retained tissue, read by source lineage from its native ledger.
//! Reserve and birth provision remain separate from the scruple (TG3).

use crate::{Result, rules::{AccountKind, Rules}, schema::{Ledger, Part}};

/// The exact milligrams of each source line in this part's tissue.
/// This reading stores nothing and does not rewrite donor provenance.
pub fn of(part: &Part, rules: &Rules) -> Result<Ledger> {
    let mut mix = Ledger::new();
    for (account, &mg) in &part.matter {
        let kind = rules.accounts.get(account)
            .ok_or_else(|| format!("scruple has unknown account {account}"))?;
        if let AccountKind::Matter { lineage, reserve: false, provision: false } = kind {
            if mg > 0 {
                let held = mix.entry(lineage.clone()).or_default();
                *held = held.checked_add(mg)
                    .ok_or_else(|| format!("scruple overflows source lineage {lineage}"))?;
            }
        }
    }
    Ok(mix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Rules {
        crate::Founding::default().generate().unwrap().rules
    }

    fn tissue(lineage: &str) -> AccountKind {
        AccountKind::Matter { lineage: lineage.into(), reserve: false, provision: false }
    }

    #[test]
    fn source_lines_survive_account_aliases_and_reserve_is_not_tissue() {
        let mut rules = rules();
        rules.accounts = [
            ("tissue:a".into(), tissue("lineage:a")),
            ("tissue:a-graft".into(), tissue("lineage:a")),
            ("tissue:b".into(), tissue("lineage:b")),
            ("reserve:a".into(), AccountKind::Matter {
                lineage: "lineage:a".into(), reserve: true, provision: false,
            }),
            ("birth:a".into(), AccountKind::Matter {
                lineage: "lineage:a".into(), reserve: false, provision: true,
            }),
        ].into();
        let part = Part {
            matter: [
                ("tissue:a".into(), 3), ("tissue:a-graft".into(), 4),
                ("tissue:b".into(), 5), ("reserve:a".into(), 20), ("birth:a".into(), 30),
            ].into(),
            ..Part::default()
        };
        let before = part.clone();
        assert_eq!(of(&part, &rules).unwrap(), [
            ("lineage:a".into(), 7), ("lineage:b".into(), 5),
        ].into());
        assert_eq!(part, before, "inspection cannot rewrite a part");
    }

    #[test]
    fn unknown_accounts_and_source_overflow_are_reported() {
        let mut rules = rules();
        let mut part = Part { matter: [("missing".into(), 1)].into(), ..Part::default() };
        assert_eq!(of(&part, &rules).unwrap_err(), "scruple has unknown account missing");
        rules.accounts = [
            ("a".into(), tissue("lineage:a")), ("b".into(), tissue("lineage:a")),
        ].into();
        part.matter = [("a".into(), u64::MAX), ("b".into(), 1)].into();
        assert_eq!(of(&part, &rules).unwrap_err(), "scruple overflows source lineage lineage:a");
    }
}
