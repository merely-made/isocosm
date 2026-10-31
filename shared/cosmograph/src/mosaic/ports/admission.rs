// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit port matching, shared by prospective selection and an actual meal.
//! No caller receives a no-port fallback or an implicit source selector.

use super::active;
use crate::{process::NisKind, rules::{AccountKind, Rules}, schema::*};
use std::collections::{BTreeMap, BTreeSet};

/// A declaration explicitly admits every source line or only the named ones.
/// An empty named set admits nothing; it is not an alias for every line.
#[derive(Clone, Copy, Debug)]
pub enum Sources<'a> {
    All,
    Only(&'a BTreeSet<Key>),
}

/// Provenance resolved through the world's native account and lineage tables.
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    lineage: &'a str,
    kind: Option<NisKind>,
    alive: bool,
}

impl<'a> Source<'a> {
    /// Death is a state of the body offering this account, not a new nis kind.
    pub fn of(
        account: &str,
        alive: bool,
        rules: &'a Rules,
        lineages: &'a BTreeMap<Key, Lineage>,
    ) -> crate::Result<Self> {
        let Some(AccountKind::Matter { lineage, .. }) = rules.accounts.get(account) else {
            return Err(format!("intake source {account} is not declared matter"));
        };
        let line = lineages.get(lineage)
            .ok_or_else(|| format!("intake source has unknown lineage {lineage}"))?;
        let kind = match line.kingdom.as_str() {
            "kingdom:flora" => Some(NisKind::Producer),
            "kingdom:fauna" => Some(NisKind::Consumer),
            "kingdom:myco" => Some(NisKind::Decomposer),
            _ => None,
        };
        Ok(Self { lineage, kind, alive })
    }

    pub fn lineage(self) -> &'a str {
        self.lineage
    }

    /// Declared foods, independent of whether an organ currently holds its
    /// supporting cells. Runtime admission checks readiness separately.
    pub(crate) fn admitted_by(self, port: crate::process::IntakePort) -> Result<(), Refusal> {
        if self.alive {
            if !self.kind.is_some_and(|kind| port.admits_live(kind)) {
                return Err(Refusal::LivingKind);
            }
        } else if !port.admits_deadstock() {
            return Err(Refusal::Deadstock);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    MissingPort,
    DormantPort,
    SourceLine,
    LivingKind,
    Deadstock,
}

fn ready(part: &Part) -> Result<crate::process::IntakePort, Refusal> {
    if part.port.is_none() {
        return Err(Refusal::MissingPort);
    }
    active(part).ok_or(Refusal::DormantPort)
}

/// The same answer for a proposed meal and the matter it actually takes.
/// The caller binds the addressed living intake part and supplies its selector.
pub fn check(part: &Part, sources: Sources<'_>, source: Source<'_>) -> Result<(), Refusal> {
    let port = ready(part)?;
    if let Sources::Only(lines) = sources {
        if !lines.contains(source.lineage) {
            return Err(Refusal::SourceLine);
        }
    }
    source.admitted_by(port)
}

/// Admitted positive holdings, for the same addressed source part or body
/// ledger that a prospective target and its eventual meal will read.
pub fn offered(
    intake: &Part,
    sources: Sources<'_>,
    holdings: &Ledger,
    alive: bool,
    rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>,
) -> crate::Result<Ledger> {
    ready(intake).map_err(|why| format!("intake port refused: {why:?}"))?;
    let mut admitted = Ledger::new();
    for (account, &mg) in holdings.iter().filter(|(_, mg)| **mg > 0) {
        match rules.accounts.get(account) {
            None => return Err(format!("intake holdings have unknown account {account}")),
            Some(AccountKind::Matter { .. }) => {},
            Some(_) => continue,
        }
        let source = Source::of(account, alive, rules, lineages)?;
        if check(intake, sources, source).is_ok() {
            admitted.insert(account.clone(), mg);
        }
    }
    Ok(admitted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{IntakePort, Process, Registry};

    fn source(line: &str, kind: NisKind, alive: bool) -> Source<'_> {
        Source { lineage: line, kind: Some(kind), alive }
    }

    #[test]
    fn named_sources_narrow_the_port_without_an_eater_prey_table() {
        let part = Part { port: Some(IntakePort::live(NisKind::Producer)), ..Part::default() };
        let lines = BTreeSet::from(["lineage:a".into()]);
        let a = source("lineage:a", NisKind::Producer, true);
        let b = source("lineage:b", NisKind::Producer, true);
        assert_eq!(check(&part, Sources::Only(&lines), a), Ok(()));
        assert_eq!(check(&part, Sources::Only(&lines), b), Err(Refusal::SourceLine));
        assert_eq!(check(&part, Sources::All, b), Ok(()));
        assert_eq!(check(&part, Sources::Only(&BTreeSet::new()), a), Err(Refusal::SourceLine));
        assert_eq!(check(&part, Sources::All, source("lineage:a", NisKind::Consumer, true)),
            Err(Refusal::LivingKind));
    }

    #[test]
    fn missing_or_dormant_support_refuses_and_death_is_not_a_fourth_kind() {
        let mut part = Part::default();
        let live = source("lineage:a", NisKind::Producer, true);
        assert_eq!(check(&part, Sources::All, live), Err(Refusal::MissingPort));
        let g = crate::Founding::default().generate().unwrap();
        assert_eq!(offered(&part, Sources::All, &Ledger::new(), true, &g.rules, &g.lineages)
            .unwrap_err(), "intake port refused: MissingPort");
        let support = Registry::native().of_native(Process::Intake).reference();
        part.port = Some(IntakePort::live(NisKind::Producer).supported_by(support));
        assert_eq!(check(&part, Sources::All, live), Err(Refusal::DormantPort));
        part.cells.insert("function:intake".into(), 1);
        assert_eq!(check(&part, Sources::All, live), Ok(()));
        let dead = source("lineage:a", NisKind::Producer, false);
        assert_eq!(check(&part, Sources::All, dead), Err(Refusal::Deadstock));
        part.port = Some(IntakePort::deadstock().supported_by(support));
        assert_eq!(check(&part, Sources::All, dead), Ok(()));
        assert_eq!(check(&part, Sources::All, live), Err(Refusal::LivingKind));
    }

    #[test]
    fn provenance_comes_from_the_offered_account_not_the_eaters_line() {
        let g = crate::Founding::default().generate().unwrap();
        let source = Source::of("matter:0-0", true, &g.rules, &g.lineages).unwrap();
        assert_eq!(source.lineage(), "lineage:0");
        assert!(Source::of("sim:energy", true, &g.rules, &g.lineages).is_err());
    }

    #[test]
    fn prospective_and_taken_holdings_use_the_same_port_answer() {
        let g = crate::Founding::default().generate().unwrap();
        let part = Part { port: Some(IntakePort::live(NisKind::Producer)), ..Part::default() };
        let lines = BTreeSet::from(["lineage:0".into()]);
        let holdings = Ledger::from([
            ("matter:0-0".into(), 7), ("matter:1-0".into(), 8), ("sim:energy".into(), 9),
        ]);
        let admitted = offered(&part, Sources::Only(&lines), &holdings, true,
            &g.rules, &g.lineages).unwrap();
        assert_eq!(admitted, Ledger::from([("matter:0-0".into(), 7)]));
        let source = Source::of("matter:0-0", true, &g.rules, &g.lineages).unwrap();
        assert_eq!(check(&part, Sources::Only(&lines), source), Ok(()));
        assert_eq!(holdings["matter:1-0"], 8, "selection must not consume rejected matter");
    }
}
