// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{
    Command, Execution, Founding, Session,
    asserted::{Assertion, Fact},
    simulation::Genesis,
};

fn genesis() -> Genesis {
    Founding {
        seed: 13,
        sites: 2,
        population: 4,
        lineages: 2,
        cohort_size: 2,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

#[test]
fn bare_note_kind_declarations_keep_native_admission_and_replay() {
    let mut genesis = genesis();
    let kinds = [
        "reveal",
        "narration",
        "history",
        "a-custom_kind/1.2",
        "custom:event",
    ];
    for kind in kinds {
        genesis.rules.note_kinds.insert(kind.into());
    }
    let proposed = Session::new(genesis.clone(), Execution::Grouped);
    assert!(
        proposed.is_ok(),
        "declared bare note names must found natively"
    );
    let mut grouped = proposed.unwrap();
    let mut individual = Session::new(genesis, Execution::Individuals).unwrap();
    for kind in kinds {
        let command = Command::Assert(Assertion::Fact(Fact {
            key: format!("note:{kind}"),
            kind: kind.into(),
            text: format!("The authored {kind}."),
            ..Default::default()
        }));
        grouped.command(command.clone()).unwrap();
        individual.command(command).unwrap();
    }
    assert_eq!(grouped.save(), individual.save());
    let saved = serde_json::from_slice(&serde_json::to_vec(&grouped.save()).unwrap()).unwrap();
    let loaded = Session::load(saved, Execution::Individuals).unwrap();
    assert_eq!(loaded.save(), grouped.save());
    for kind in kinds {
        assert!(loaded.sim.state().notes.iter().any(|n| n.core.kind == kind));
    }
}

#[test]
fn note_kind_declarations_reject_invalid_names_and_undeclared_assertions() {
    for bad in [
        "",
        " ",
        "two names",
        "bad\tname",
        ":empty",
        "empty:",
        "a:b c",
        "noté",
    ] {
        let mut rules = genesis().rules;
        rules.note_kinds.insert(bad.into());
        assert!(
            rules.validate().is_err(),
            "invalid declared note spelling must refuse"
        );
    }
    let mut rules = genesis().rules;
    rules.note_kinds.insert("a".repeat(257));
    assert!(rules.validate().is_err());
    let mut native = Session::new(genesis(), Execution::Grouped).unwrap();
    let before = native.save();
    let refused = native
        .command(Command::Assert(Assertion::Fact(Fact {
            key: "undeclared".into(),
            kind: "history".into(),
            ..Default::default()
        })))
        .unwrap_err();
    assert_eq!(refused, "unknown note kind");
    assert_eq!(native.save(), before);
}

#[test]
fn bare_note_kinds_do_not_relax_native_owner_key_validation() {
    let mut conditions = genesis().rules;
    conditions.note_kinds.insert("history".into());
    conditions.conditions.insert("storm".into());
    assert!(
        conditions.validate().is_err(),
        "native condition names remain namespaced"
    );
    let mut accounts = genesis().rules;
    accounts.note_kinds.insert("history".into());
    let account = accounts.accounts.values().next().unwrap().clone();
    accounts.accounts.insert("food".into(), account);
    assert!(
        accounts.validate().is_err(),
        "native account names remain namespaced"
    );
    for bad in [":left", "right:", "white:space here", "wide:é"] {
        let mut rules = genesis().rules;
        rules.conditions.insert(bad.into());
        assert!(rules.validate().is_err());
    }
}
