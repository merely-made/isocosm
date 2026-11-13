// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit port matching, shared by prospective selection and an actual meal.
//! No caller receives a no-port fallback or an implicit source selector.

use super::active;
use crate::{
    ecology::{Categories, Flesh, Spoilage},
    process::{DeadClass, IntakePort, NisKind, Tolerance},
    rules::{AccountKind, Rules},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

/// A declaration explicitly admits every source line or only the named ones.
/// An empty named set admits nothing; it is not an alias for every line.
#[derive(Clone, Copy, Debug)]
pub enum Sources<'a> {
    All,
    Only(&'a BTreeSet<Key>),
}

/// What is eaten beyond its provenance: living or dead, and, under the
/// categories rule, whether it is tough plant tissue. A bare `bool` is the
/// body's life with soft tissue, which is every meal before the rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eaten {
    pub alive: bool,
    /// Whether the part bitten is tough; none where the whole body is meant
    /// and either may be (a feed's reach is judged by either).
    pub tough: Option<bool>,
}

impl From<bool> for Eaten {
    fn from(alive: bool) -> Self {
        Self { alive, tough: Some(false) }
    }
}

impl Eaten {
    /// What a bite of `part` of `prey` is: soft where the world sets no
    /// categories rule or the bite is of the body's ledger.
    #[must_use]
    pub fn part(prey: &Entity, part: Option<PartId>, rules: &Rules) -> Self {
        let tough = rules.categories.is_some() && part.is_some_and(|id| crate::ecology::tough(prey, id));
        Self { alive: prey.alive, tough: Some(tough) }
    }

    /// Living or dead matter of any part, for judging what a mouth could
    /// ever reach (founding a feed's sources).
    #[must_use]
    pub fn any(alive: bool) -> Self {
        Self { alive, tough: None }
    }

    /// The whole of `prey`, any part of it.
    #[must_use]
    pub fn body(prey: &Entity) -> Self {
        Self { alive: prey.alive, tough: None }
    }
}

/// Why a bite harms its eater.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cause {
    /// Flesh the mouth does not tolerate (spoilage).
    Flesh,
    /// Tough plant matter the mouth cannot grind (categories).
    Tough,
}

/// Provenance resolved through the world's native account and lineage tables.
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    lineage: &'a str,
    kind: Option<NisKind>,
    alive: bool,
    /// The world's spoilage rule, which widens what a mouth admits and
    /// lets it harm instead of refuse.
    spoilage: Option<Spoilage>,
    /// The world's categories rule, which classes dead matter and makes
    /// tough plant matter harm a mouth that cannot grind it.
    categories: Option<Categories>,
    tough: Option<bool>,
}

impl<'a> Source<'a> {
    /// Death is a state of the body offering this account, not a new nis kind.
    pub fn of(
        account: &str,
        eaten: impl Into<Eaten>,
        rules: &'a Rules,
        lineages: &'a BTreeMap<Key, Lineage>,
    ) -> crate::Result<Self> {
        let Eaten { alive, tough } = eaten.into();
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
        Ok(Self { lineage, kind, alive, spoilage: rules.spoilage, categories: rules.categories, tough })
    }

    pub fn lineage(self) -> &'a str {
        self.lineage
    }

    /// Declared foods, independent of whether an organ currently holds its
    /// supporting cells. Runtime admission checks readiness separately.
    ///
    /// Under the spoilage rule a mouth that admits living animal or
    /// decomposer matter is a flesh mouth, and also bites dead flesh and
    /// (where decomposer flesh is toxic) decomposer flesh; whether the bite
    /// harms is [`Source::harms`], not a refusal (D36, D37).
    pub(crate) fn admitted_by(self, port: IntakePort) -> Result<(), Refusal> {
        let flesh_mouth = port.admits_live(NisKind::Consumer) || port.admits_live(NisKind::Decomposer);
        let bites_decomposer = self.spoilage.is_some_and(|s| s.decomposer_flesh_toxic)
            && self.kind == Some(NisKind::Decomposer)
            && flesh_mouth;
        if self.alive {
            let admitted = self.kind.is_some_and(|kind| port.admits_live(kind));
            if !admitted && !bites_decomposer {
                return Err(Refusal::LivingKind);
            }
        } else if !port.admits_deadstock() {
            let carrion = self.spoilage.is_some()
                && flesh_mouth
                && match self.kind {
                    Some(NisKind::Consumer) => true,
                    Some(NisKind::Decomposer) => bites_decomposer || port.admits_live(NisKind::Decomposer),
                    _ => false,
                };
            if !carrion {
                return Err(Refusal::Deadstock);
            }
        } else if self.categories.is_some() && !self.classes().any(|class| port.takes(class)) {
            return Err(Refusal::Deadstock);
        }
        Ok(())
    }

    /// The classes this dead matter may be, by its kingdom and the part
    /// bitten: a plant's soft part is litter and its tough wood, either where
    /// the part is not known, and the flesh of anything else carrion.
    fn classes(self) -> impl Iterator<Item = DeadClass> {
        let plant = self.kind == Some(NisKind::Producer);
        DeadClass::ALL.into_iter().filter(move |class| match (plant, class) {
            (false, class) => *class == DeadClass::Carrion,
            (true, DeadClass::Litter) => self.tough != Some(true),
            (true, DeadClass::Wood) => self.tough != Some(false),
            (true, DeadClass::Carrion) | (_, DeadClass::Dung) => false,
        })
    }

    /// Whether eating this matter at `class` harms a mouth with `port`: it
    /// is admitted, and it is decomposer flesh the mouth neither admits as
    /// living food nor tolerates, or it is dead animal or decomposer flesh
    /// older than the mouth tolerates, or it is tough plant matter the mouth
    /// cannot grind. None where no rule is set.
    pub(crate) fn harms(self, port: IntakePort, class: Flesh) -> Option<Cause> {
        // A mouth gained after founding (by an edit, drift or arrival) has
        // the tolerance its kind founds with, read here rather than stored.
        let port = self.spoilage.map_or(port, |rule| rule.founded(port));
        let port = self.categories.map_or(port, |rule| rule.founded(port));
        if self.admitted_by(port).is_err() {
            return None;
        }
        if self.categories.is_some()
            && self.kind == Some(NisKind::Producer)
            && self.tough == Some(true)
            && !port.tolerates(Tolerance::ToughPlant)
        {
            return Some(Cause::Tough);
        }
        self.flesh_harms(port, class).then_some(Cause::Flesh)
    }

    fn flesh_harms(self, port: IntakePort, class: Flesh) -> bool {
        let Some(rule) = self.spoilage else { return false };
        let decomposer = self.kind == Some(NisKind::Decomposer);
        if decomposer
            && rule.decomposer_flesh_toxic
            && !port.admits_live(NisKind::Decomposer)
            && !port.tolerates(Tolerance::DecomposerFlesh)
        {
            return true;
        }
        let flesh = matches!(self.kind, Some(NisKind::Consumer | NisKind::Decomposer));
        if self.alive || !flesh {
            return false;
        }
        match class {
            Flesh::Fresh => false,
            Flesh::Aged => !port.tolerates(Tolerance::AgedFlesh),
            Flesh::Toxic => !port.tolerates(Tolerance::ToxicFlesh),
        }
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
    alive: impl Into<Eaten>,
    rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>,
) -> crate::Result<Ledger> {
    ready(intake).map_err(|why| format!("intake port refused: {why:?}"))?;
    let eaten: Eaten = alive.into();
    let mut admitted = Ledger::new();
    for (account, &mg) in holdings.iter().filter(|(_, mg)| **mg > 0) {
        match rules.accounts.get(account) {
            None => return Err(format!("intake holdings have unknown account {account}")),
            Some(AccountKind::Matter { .. }) => {},
            Some(_) => continue,
        }
        let source = Source::of(account, eaten, rules, lineages)?;
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
        Source { lineage: line, kind: Some(kind), alive, spoilage: None, categories: None, tough: Some(false) }
    }

    fn kind_of(kind: Option<NisKind>, alive: bool, tough: Option<bool>) -> Source<'static> {
        Source {
            lineage: "lineage:a",
            kind,
            alive,
            spoilage: None,
            categories: Some(Categories::default()),
            tough,
        }
    }

    fn classes(source: Source<'_>) -> Vec<DeadClass> {
        source.classes().collect()
    }

    #[test]
    fn dead_matter_reads_its_class_from_what_it_came_from() {
        let plant = |tough| kind_of(Some(NisKind::Producer), false, tough);
        assert_eq!(classes(plant(Some(false))), [DeadClass::Litter]);
        assert_eq!(classes(plant(Some(true))), [DeadClass::Wood]);
        assert_eq!(classes(plant(None)), [DeadClass::Litter, DeadClass::Wood], "either, where the part is not known");
        for kind in [Some(NisKind::Consumer), Some(NisKind::Decomposer), None] {
            for tough in [Some(false), Some(true), None] {
                assert_eq!(classes(kind_of(kind, false, tough)), [DeadClass::Carrion], "{kind:?}");
            }
        }
    }

    #[test]
    fn a_class_limited_mouth_takes_only_its_classes_and_a_root_takes_all() {
        let root = IntakePort::deadstock();
        let litter = root.taking(DeadClass::Litter);
        let wood = root.taking(DeadClass::Wood);
        let carrion = root.taking(DeadClass::Carrion);
        let plant = |tough| kind_of(Some(NisKind::Producer), false, tough);
        let animal = kind_of(Some(NisKind::Consumer), false, Some(false));
        for (port, soft, hard, flesh) in [(root, true, true, true), (litter, true, false, false), (wood, false, true, false), (carrion, false, false, true)] {
            assert_eq!(plant(Some(false)).admitted_by(port).is_ok(), soft);
            assert_eq!(plant(Some(true)).admitted_by(port).is_ok(), hard);
            assert_eq!(animal.admitted_by(port).is_ok(), flesh);
            // A feed that reaches the whole body can use a mouth for either.
            assert_eq!(plant(None).admitted_by(port).is_ok(), soft || hard);
        }
        // Without the rule a limit is not read: the mouth takes all dead stock as before.
        let unruled = Source { categories: None, ..plant(Some(true)) };
        assert!(unruled.admitted_by(litter).is_ok());
    }

    #[test]
    fn a_dung_mouth_admits_no_body_and_a_root_still_admits_every_dead_one() {
        let dung = IntakePort::deadstock().taking(DeadClass::Dung);
        for tough in [Some(false), Some(true), None] {
            assert!(kind_of(Some(NisKind::Producer), false, tough).admitted_by(dung).is_err(), "dung is a site's, not a body's");
            assert!(kind_of(Some(NisKind::Producer), false, tough).admitted_by(IntakePort::deadstock()).is_ok());
        }
        assert!(kind_of(Some(NisKind::Consumer), false, Some(false)).admitted_by(dung).is_err());
        assert!(dung.takes(DeadClass::Dung) && !dung.takes(DeadClass::Carrion));
    }

    #[test]
    fn tough_plant_matter_harms_a_mouth_without_grinding_and_nothing_else_does() {
        let bulk = IntakePort::live(NisKind::Producer);
        let tough = kind_of(Some(NisKind::Producer), true, Some(true));
        assert_eq!(tough.harms(bulk, Flesh::Fresh), Some(Cause::Tough));
        assert_eq!(tough.harms(bulk.tolerating(Tolerance::ToughPlant), Flesh::Fresh), None);
        assert_eq!(kind_of(Some(NisKind::Producer), true, Some(false)).harms(bulk, Flesh::Fresh), None, "soft tissue is not harmful");
        assert_eq!(kind_of(Some(NisKind::Producer), true, None).harms(bulk, Flesh::Fresh), None, "an unknown part is not judged");
        // Dead wood to the root, which grinds by founding.
        let wood = kind_of(Some(NisKind::Producer), false, Some(true));
        assert_eq!(wood.harms(IntakePort::deadstock(), Flesh::Fresh), None);
        // The rule unset, nothing harms.
        assert_eq!(Source { categories: None, ..tough }.harms(bulk, Flesh::Fresh), None);
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
