// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "ecology_independence_879/fixture.rs"]
mod fixture;

use fixture::candidate;
use cosmograph::{ecology::*, rules::*, Execution, Session};

#[test]
fn one_actual_native_trajectory_precedes_same_family_admission() {
    let candidate = candidate("test:joint-cycle", 1, false);
    let witness = candidate.witness();
    witness.require_roles().unwrap();
    let before = Session::new(candidate.genesis.clone(), Execution::Individuals).unwrap().sim.matter();
    let after = Session::load(witness.native.clone(), Execution::Individuals).unwrap().sim.matter();
    assert_eq!(after, before);
    assert!(witness.totals.captured > 0 && witness.totals.returned > 0
        && witness.totals.births > 0 && witness.totals.counter > 0);
    assert!(witness.acts.iter().all(|act| act.process == candidate.process));
    assert!(witness.acts.iter().all(|act| act.tick == 1 && act.target != Some(act.actor)));
    let seal = candidate.admission(&witness).unwrap();
    assert_eq!(seal.coverage(), &[NativeReference::Process(candidate.process.clone())]);
    assert_eq!(seal.evidence()[0].family, OperationFamily::Transform);
    assert_eq!(seal.evidence()[0].compared_family, OperationFamily::Transform);
    assert!(!seal.evidence()[0].accepted_actions.is_empty());
    assert_eq!(seal.scalars().len(), 10);
    seal.validate(&witness).unwrap();
}

#[test]
fn process_name_and_reference_pair_order_cannot_create_mechanics() {
    for name in ["test:plain-name", "test:renamed-name"] {
        let mut candidate = candidate(name, 0, false);
        candidate.spec.independence.as_mut().unwrap().extras[0].equivalences.reverse();
        candidate.sync();
        let witness = candidate.witness();
        witness.require_roles().unwrap();
        assert!(candidate.admission(&witness).unwrap_err().contains("no distinct consumed mechanical effect"));
    }
}

#[test]
fn account_alias_and_token_ordinals_do_not_count_as_typed_flow() {
    let mut candidate = candidate("test:alias-cycle", 0, true);
    for extra in &mut candidate.spec.independence.as_mut().unwrap().extras {
        extra.equivalences.rotate_left(2);
    }
    candidate.sync();
    let witness = candidate.witness();
    witness.require_roles().unwrap();
    assert!(candidate.admission(&witness).unwrap_err().contains("no distinct consumed mechanical effect"));
}

#[test]
fn explicit_account_rename_preserves_a_real_quantity_difference() {
    let candidate = candidate("test:alias-with-effect", 1, true);
    let witness = candidate.witness();
    let seal = candidate.admission(&witness).unwrap();
    assert_eq!(seal.coverage().len(), 2);
    assert!(seal.coverage().contains(&NativeReference::Account("test:renamed-stock".into())));
    seal.validate(&witness).unwrap();
}

#[test]
fn omitted_added_catalogue_row_cannot_hide_behind_a_valid_joint_witness() {
    let mut candidate = candidate("test:coverage-cycle", 1, false);
    candidate.genesis.rules.traits.insert("test:unconsumed-extra".into());
    let witness = candidate.witness();
    witness.require_roles().unwrap();
    assert!(candidate.admission(&witness).unwrap_err().contains("cover every actual added/changed"));
}

#[test]
fn empty_extras_cannot_bypass_actual_changed_definition_coverage() {
    let mut candidate = candidate("test:empty-coverage", 1, false);
    let independence = candidate.spec.independence.as_mut().unwrap();
    independence.extras.clear();
    independence.scalars.clear();
    candidate.sync();
    let witness = candidate.witness();
    witness.require_roles().unwrap();
    assert!(candidate.admission(&witness).unwrap_err().contains("cover every actual added/changed"));
}

#[test]
fn removed_basis_definition_refuses_instead_of_disappearing_from_coverage() {
    let mut candidate = candidate("test:removed-definition", 1, false);
    candidate.genesis.rules.processes.remove("sim:remember");
    let witness = candidate.witness();
    assert!(candidate.admission(&witness).unwrap_err().contains("removal/replacement"));
}

#[test]
fn scalar_omission_and_incomplete_correspondence_refuse() {
    let mut candidate = candidate("test:missing-scalar", 1, false);
    candidate.spec.independence.as_mut().unwrap().scalars.pop();
    candidate.sync();
    assert!(candidate.spec.validate_genesis(&candidate.genesis).unwrap_err()
        .contains("cover every supported consumed field"));
    let mut candidate = fixture::candidate("test:missing-correspondence", 1, false);
    candidate.spec.independence.as_mut().unwrap().extras[0].equivalences.pop();
    candidate.sync();
    assert!(candidate.spec.validate_genesis(&candidate.genesis).unwrap_err()
        .contains("lacks explicit typed correspondence"));
}

#[test]
fn a_forged_role_total_cannot_replace_actual_scheduled_native_evidence() {
    let candidate = candidate("test:joint-proof", 1, false);
    let mut witness = candidate.witness();
    witness.totals.births += 1;
    assert!(candidate.admission(&witness).unwrap_err().contains("actual native trajectory"));
}

#[test]
fn an_incomplete_real_joint_trajectory_cannot_reach_independence_admission() {
    let mut candidate = candidate("test:missing-role", 1, false);
    candidate.spec.reproduction.at_least = 100;
    candidate.sync();
    let witness = candidate.witness();
    assert!(candidate.admission(&witness).unwrap_err().contains("joint ecology witness is incomplete"));
}

#[test]
fn saved_seal_validates_retained_basis_and_rejects_source_or_evidence_changes() {
    let candidate = candidate("test:retained-basis", 1, false);
    let witness = candidate.witness();
    let seal = candidate.admission(&witness).unwrap();
    let bytes = serde_json::to_vec(&seal).unwrap();
    let restored: Admission = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, seal);
    restored.validate(&witness).unwrap();
    let mut saved = serde_json::to_value(&seal).unwrap();
    saved["basis"]["seed"] = serde_json::json!(999);
    let corrupted: Admission = serde_json::from_value(saved).unwrap();
    assert!(corrupted.validate(&witness).unwrap_err().contains("source seals differ"));
    let mut saved = serde_json::to_value(&seal).unwrap();
    saved["evidence"][0]["accepted_actions"] = serde_json::json!([]);
    let corrupted: Admission = serde_json::from_value(saved).unwrap();
    assert!(corrupted.validate(&witness).unwrap_err().contains("consumed native evidence"));
}

#[test]
fn dynamic_amounts_are_an_explicit_evidence_gate() {
    let mut candidate = candidate("test:dynamic-quantity", 1, false);
    candidate.genesis.rules.processes.get_mut(&candidate.process).unwrap().effects[0] =
        Effect::Transfer { from: Binding::Actor, to: Binding::Place, account: "world:soil".into(),
            amount: Amount::Computed(Expr::Read(Reading::Account { who: Binding::Actor,
                key: "world:soil".into() })) };
    candidate.sync();
    assert!(candidate.spec.validate_genesis(&candidate.genesis).unwrap_err().contains("dynamic mechanical amount"));
}
