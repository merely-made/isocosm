// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored body-to-body succession eligibility (877). Control and directing
//! bonds never supply this social fact, and missing policy proves no absence.

use super::{AgreementState, EndReason, Standing};
use crate::{Result, rules::Rules, schema::{Id, Key}, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionPolicy { pub key: Key, pub rule: CompanionRule }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanionRule {
    Agreement { direction: Direction, crafts: BTreeSet<Key>, lives: Vec<AgreementLife> },
    Standing { direction: Direction, predicate: StandingRule },
    Any(Vec<CompanionRule>),
    All(Vec<CompanionRule>),
}

/// For agreements, left is asker and right holder. For standing, left is
/// holder and right the person toward whom its actual deeds are folded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction { CompanionToPredecessor, PredecessorToCompanion }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementLife { Standing, Ended(EndReason) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range { pub min: i16, pub max: i16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingRule { pub trust: Range, pub affinity: Range }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionProof {
    pub policy: Key, pub predecessor: Id, pub companion: Id, pub eligible: bool,
    pub agreements: BTreeSet<Id>, pub deeds: BTreeSet<Key>,
}

impl CompanionPolicy {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        crate::validation::key(&self.key)?;
        let vocabulary = rules.social.as_ref().ok_or("companion policy needs native social vocabulary")?;
        fn visit(rule: &CompanionRule, crafts: &BTreeSet<Key>, nodes: &mut usize) -> Result<()> {
            *nodes += 1;
            if *nodes > crate::rules::MAX_NODES { return Err("companion predicate exceeds native expression limit".into()); }
            match rule {
                CompanionRule::Agreement { crafts: wanted, lives, .. } => {
                    if wanted.is_empty() || lives.is_empty() || !wanted.is_subset(crafts) {
                        return Err("companion agreement needs declared crafts and explicit states".into());
                    }
                },
                CompanionRule::Standing { predicate, .. } => {
                    if predicate.trust.min > predicate.trust.max || predicate.affinity.min > predicate.affinity.max {
                        return Err("companion standing range is reversed".into());
                    }
                },
                CompanionRule::Any(items) | CompanionRule::All(items) => {
                    if items.is_empty() { return Err("companion predicate cannot be vacuous".into()); }
                    for item in items { visit(item, crafts, nodes)?; }
                },
            }
            Ok(())
        }
        let mut nodes=0;
        visit(&self.rule, &vocabulary.crafts, &mut nodes)
    }
}

impl Direction {
    fn pair(self, predecessor: Id, companion: Id) -> (Id, Id) {
        match self {
            Self::CompanionToPredecessor => (companion, predecessor),
            Self::PredecessorToCompanion => (predecessor, companion),
        }
    }
}

impl Simulation {
    pub fn companion(&self, predecessor: Id, companion: Id) -> Result<CompanionProof> {
        let policy = self.genesis().rules.companions.as_ref()
            .ok_or("world has no authored native companion predicate")?;
        policy.validate(&self.genesis().rules)?;
        if predecessor == companion || self.state().population.get(predecessor).is_none()
            || self.state().population.get(companion).is_none() {
            return Err("companion predicate requires two existing distinct native bodies".into());
        }
        let mut proof = CompanionProof { policy: crate::digest(policy), predecessor, companion,
            eligible: false, agreements: BTreeSet::new(), deeds: BTreeSet::new() };
        proof.eligible = evaluate(self, &policy.rule, predecessor, companion, &mut proof);
        Ok(proof)
    }

    /// The caller chooses the admissible population (for example living
    /// sophonts); eligibility itself is this world's explicit social predicate.
    pub fn companions<'a>(&self, predecessor: Id, candidates: impl IntoIterator<Item = &'a Id>)
        -> Result<Vec<CompanionProof>> {
        candidates.into_iter().filter(|id| **id != predecessor)
            .map(|id| self.companion(predecessor, *id)).collect::<Result<Vec<_>>>()
            .map(|proofs| proofs.into_iter().filter(|proof| proof.eligible).collect())
    }
}

fn evaluate(sim: &Simulation, rule: &CompanionRule, predecessor: Id, companion: Id,
    proof: &mut CompanionProof) -> bool {
    match rule {
        CompanionRule::Agreement { direction, crafts, lives } => {
            let (asker, holder) = direction.pair(predecessor, companion);
            let mut accepted = false;
            for agreement in sim.state().agreements.values() {
                let life = match agreement.state {
                    AgreementState::Standing => AgreementLife::Standing,
                    AgreementState::Ended { why, .. } => AgreementLife::Ended(why),
                };
                if agreement.asker == asker && agreement.holder == holder
                    && crafts.contains(&agreement.work.craft) && lives.contains(&life) {
                    proof.agreements.insert(agreement.id);
                    proof.deeds.extend(agreement.history.iter().map(|event| event.deed.clone()));
                    accepted = true;
                }
            }
            accepted
        },
        CompanionRule::Standing { direction, predicate } => {
            let (holder, toward) = direction.pair(predecessor, companion);
            let Standing { trust, affinity, from_deeds } = sim.standing(holder, toward);
            proof.deeds.extend(from_deeds.iter().cloned());
            // A stranger's implicit zero standing is not a body-to-body fact186.
            !from_deeds.is_empty()
                && (predicate.trust.min..=predicate.trust.max).contains(&trust)
                && (predicate.affinity.min..=predicate.affinity.max).contains(&affinity)
        },
        CompanionRule::Any(items) => {
            let mut result = false;
            for item in items { result |= evaluate(sim, item, predecessor, companion, proof); }
            result
        },
        CompanionRule::All(items) => {
            let mut result = true;
            for item in items { result &= evaluate(sim, item, predecessor, companion, proof); }
            result
        },
    }
}
