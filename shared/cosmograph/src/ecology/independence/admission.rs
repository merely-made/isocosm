// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, ecology::JointWitness, schema::Key, simulation::Genesis};
use crate::ecology::{AdmittedJointRun, ValidatedJointRun};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Produced only after native role proof, catalogue coverage and consumed mechanics.
/// Decoded records must validate against their exact joint native witness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    version: u32,
    basis: Genesis,
    basis_digest: Key,
    genesis_digest: Key,
    specification_digest: Key,
    witness_digest: Key,
    coverage: Vec<NativeReference>,
    evidence: Vec<MechanicalEvidence>,
    scalars: Vec<ConsumedScalar>,
}

fn evidence(witness: &JointWitness, spec: &IndependenceSpec, basis: &Genesis)
    -> Result<(Vec<NativeReference>, Vec<MechanicalEvidence>, Vec<ConsumedScalar>)> {
    let candidate = &witness.native.genesis;
    basis.validate()?;
    let founding = basis.founding.as_ref().ok_or("recorded ecology basis has no producer founding")?;
    inventory::ordinary(founding)?;
    if basis.rules.ecology_witness.is_some() { return Err("structural basis cannot exempt admission declarations".into()); }
    spec.validate_genesis(candidate)?;
    let coverage = inventory::changed(basis, candidate)?;
    let claimed: BTreeSet<_> = spec.extras.iter().map(|extra| extra.noun.clone()).collect();
    let required: BTreeSet<_> = coverage.iter().cloned().collect();
    if claimed != required {
        return Err("extra-noun evidence must cover every actual added/changed native definition exactly once".into());
    }
    let mut rows = vec![];
    for extra in &spec.extras {
        // Indirect changed definitions cannot borrow an old, unrelated difference.
        if !matches!(&extra.noun, NativeReference::Account(_))
            && !required.contains(&NativeReference::Process(extra.effect.process.clone())) {
            return Err("extra noun requires a changed consumed process; indirect definition-only axes are unsupported".into());
        }
        let references = references::References::new(candidate, &extra.equivalences)?;
        let actual = project::projection(candidate, &extra.effect, &references, true)?;
        let baseline = project::projection(candidate, &extra.compare, &references, false)?;
        references.complete(&extra.noun)?;
        let distinct = match extra.axis {
            MechanicalAxis::Cadence => (actual.cadence, actual.priority) != (baseline.cadence, baseline.priority),
            MechanicalAxis::PaidStock => {
                if extra.effect.phase != EffectPhase::Commitment || extra.compare.phase != EffectPhase::Commitment {
                    return Err("paid-stock evidence must point to actual native commitments".into());
                }
                structural::shape(&actual,true) != structural::shape(&baseline,true)
            },
            MechanicalAxis::TypedFlow => structural::shape(&actual,false) != structural::shape(&baseline,false),
        };
        if !distinct { return Err("extra noun has no distinct consumed mechanical effect; names/families/draws are insufficient".into()); }
        let accepted_actions: Vec<_> = witness.acts.iter().filter(|act| extra.consumer.matches(act))
            .map(|act| act.action.clone()).collect();
        if accepted_actions.is_empty() {
            return Err("extra noun's consumer never ran as an accepted scheduled action in the joint witness".into());
        }
        rows.push(MechanicalEvidence { noun: extra.noun.clone(), consumer: extra.consumer.clone(),
            effect: extra.effect.clone(), compared: extra.compare.clone(), family: actual.family,
            compared_family: baseline.family, axis: extra.axis, actual, baseline, accepted_actions });
    }
    Ok((coverage, rows, scalars::consumed(candidate, spec)?))
}

fn complete_roles(witness: &JointWitness) -> Result<()> {
    let missing = witness.spec.missing(&witness.totals);
    if missing.is_empty() { Ok(()) }
    else { Err(format!("joint ecology witness is incomplete: {}", missing.join("; "))) }
}

fn build(witness: &JointWitness, spec: &IndependenceSpec, basis: &GeneratedBasis) -> Result<Admission> {
    if witness.spec.independence.as_ref() != Some(spec) { return Err("independence input differs from the joint native declaration".into()); }
    let (coverage, evidence, scalars) = evidence(witness, spec, &basis.source)?;
    Ok(Admission { version: 1, basis: basis.source.clone(), basis_digest: basis.source_digest(),
        genesis_digest: crate::digest(&witness.native.genesis), specification_digest: crate::digest(spec),
        witness_digest: crate::digest(witness), coverage, evidence, scalars })
}

pub fn admit(witness: &JointWitness, spec: &IndependenceSpec, basis: &GeneratedBasis) -> Result<Admission> {
    // Four actual roles in one source trajectory precede every criterion check.
    witness.require_roles()?;
    build(witness, spec, basis)
}

/// Admit the exact run already replayed by `run_joint_validated`. The proof
/// object is unforgeable outside this crate and cannot survive serialization.
pub fn admit_validated(run: ValidatedJointRun, spec: &IndependenceSpec,
    basis: &GeneratedBasis) -> Result<AdmittedJointRun> {
    let (witness, session) = run.into_parts();
    complete_roles(&witness)?;
    let admission = build(&witness, spec, basis)?;
    Ok(AdmittedJointRun::new(witness, admission, session))
}

impl Admission {
    pub fn genesis_digest(&self) -> &str { &self.genesis_digest }
    pub fn basis_digest(&self) -> &str { &self.basis_digest }
    pub fn specification_digest(&self) -> &str { &self.specification_digest }
    pub fn witness_digest(&self) -> &str { &self.witness_digest }
    pub fn coverage(&self) -> &[NativeReference] { &self.coverage }
    pub fn evidence(&self) -> &[MechanicalEvidence] { &self.evidence }
    pub fn scalars(&self) -> &[ConsumedScalar] { &self.scalars }

    /// Uses the captured native basis, never the current generator on saved restore.
    pub fn validate(&self, witness: &JointWitness) -> Result<()> {
        witness.require_roles()?;
        let spec = witness.spec.independence.as_ref().ok_or("saved ecology seal lacks its independence declaration")?;
        if self.version != 1 || crate::digest(&self.basis) != self.basis_digest
            || crate::digest(&witness.native.genesis) != self.genesis_digest
            || crate::digest(spec) != self.specification_digest || crate::digest(witness) != self.witness_digest {
            return Err("saved ecology admission source seals differ".into());
        }
        let (coverage, evidence, scalars) = evidence(witness, spec, &self.basis)?;
        if coverage != self.coverage || evidence != self.evidence || scalars != self.scalars {
            return Err("saved ecology admission differs from its consumed native evidence".into());
        }
        Ok(())
    }
}
