// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Inspect the native grammar without declaring a second effect evaluator.
//! A primitive's presence is evidence for a binding candidate, not proof of
//! a ruleset's targeting, cost, duration or complete mechanical behaviour.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Transfer,
    Transform,
    Sunlight,
    Condition,
    Relate,
    Trait,
    Practice,
    Move,
    Note,
    Birth,
    Bear,
    Bud,
    Death,
    Wound,
    Rot,
    Tell,
    FoundPolity,
    Record,
    Ease,
    Eat,
    When,
    Spend,
    Grow,
    Convert,
    Keep,
    Allocate,
    Carry,
    Heal,
    Terrain,
    ReserveProject,
    WorkProject,
    Step,
    ProjectDecision,
    MovePaced,
    InstitutionDecision,
    Graft,
    Institution,
    DirectOverflow,
}

impl Effect {
    /// This exhaustive match keeps inspection in step with the execution grammar.
    pub fn kind(&self) -> EffectKind {
        match self {
            Self::Transfer { .. } => EffectKind::Transfer,
            Self::Transform { .. } => EffectKind::Transform,
            Self::Sunlight { .. } => EffectKind::Sunlight,
            Self::Condition { .. } => EffectKind::Condition,
            Self::Relate { .. } => EffectKind::Relate,
            Self::Trait { .. } => EffectKind::Trait,
            Self::Practice { .. } => EffectKind::Practice,
            Self::Move { .. } => EffectKind::Move,
            Self::Note { .. } => EffectKind::Note,
            Self::Birth { .. } => EffectKind::Birth,
            Self::Bear { .. } => EffectKind::Bear,
            Self::Bud { .. } => EffectKind::Bud,
            Self::Death => EffectKind::Death,
            Self::Wound { .. } => EffectKind::Wound,
            Self::Rot { .. } => EffectKind::Rot,
            Self::Tell { .. } => EffectKind::Tell,
            Self::FoundPolity { .. } => EffectKind::FoundPolity,
            Self::Record { .. } => EffectKind::Record,
            Self::Ease { .. } => EffectKind::Ease,
            Self::Eat { .. } => EffectKind::Eat,
            Self::When { .. } => EffectKind::When,
            Self::Spend { .. } => EffectKind::Spend,
            Self::Grow { .. } => EffectKind::Grow,
            Self::Convert { .. } => EffectKind::Convert,
            Self::Keep { .. } => EffectKind::Keep,
            Self::Allocate { .. } => EffectKind::Allocate,
            Self::Carry { .. } => EffectKind::Carry,
            Self::Heal { .. } => EffectKind::Heal,
            Self::Terrain { .. } => EffectKind::Terrain,
            Self::ReserveProject { .. } => EffectKind::ReserveProject,
            Self::WorkProject => EffectKind::WorkProject,
            Self::Step { .. } => EffectKind::Step,
            Self::ProjectDecision => EffectKind::ProjectDecision,
            Self::MovePaced { .. } => EffectKind::MovePaced,
            Self::InstitutionDecision => EffectKind::InstitutionDecision,
            Self::Graft { .. } => EffectKind::Graft,
            Self::Institution { .. } => EffectKind::Institution,
            Self::DirectOverflow { .. } => EffectKind::DirectOverflow,
        }
    }

    fn collect_declarations<'a>(&'a self, into: &mut Vec<&'a Effect>) {
        into.push(self);
        if let Self::When {
            then, otherwise, ..
        } = self
        {
            for child in then.iter().chain(otherwise) {
                child.collect_declarations(into);
            }
        }
    }
}

impl Process {
    /// All declared possibilities, including commitments and both guarded and
    /// risk branches. This does not say which branch an accepted act took.
    pub fn effect_kinds(&self) -> BTreeSet<EffectKind> {
        self.effect_declarations()
            .into_iter()
            .map(Effect::kind)
            .collect()
    }

    /// The declarations behind `effect_kinds`, retaining parameters for binding
    /// inspection. Includes every possible branch, not just executed effects.
    pub fn effect_declarations(&self) -> Vec<&Effect> {
        let mut declarations = Vec::new();
        for effect in self
            .commitments
            .iter()
            .chain(&self.effects)
            .chain(self.risk.iter().flat_map(|risk| &risk.effects))
        {
            effect.collect_declarations(&mut declarations);
        }
        declarations
    }
}
