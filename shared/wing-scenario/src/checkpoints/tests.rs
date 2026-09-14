// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The four checkpoint verbs are pure arithmetic over two field maps, so they
//! are provable with no host at all.

use super::*;

fn taken() -> BTreeMap<String, Checkpoint> {
    let mut start = Checkpoint::new();
    for (field, value) in [
        ("position", "4,0,9"),
        ("hash", "abc"),
        ("step", "3"),
        ("target-vitality", "9.5"),
        ("action", "none"),
        ("saves", "0"),
    ] {
        start.insert(field.into(), value.into());
    }
    BTreeMap::from([("start".to_owned(), start)])
}

fn now() -> ProbeSnapshot {
    ProbeSnapshot::default()
        .with_field("position", "5,0,9")
        .with_field("hash", "def")
        .with_field("step", "11")
        .with_field("target-vitality", "4.25")
        .with_field("action", "none")
}

fn check(verb: &str, fields: &[&str]) -> Result<(), String> {
    Checkpoints(&taken()).compare(verb, "start", fields, &now())
}

#[test]
fn a_view_reads_the_names_and_fields_remember_took() {
    let held = taken();
    let view = Checkpoints(&held);
    assert!(!view.is_empty());
    assert_eq!(view.len(), 1);
    assert_eq!(view.names().collect::<Vec<_>>(), ["start"]);
    assert_eq!(view.field("start", "hash"), Some("abc"));
    assert_eq!(view.field("start", "absent"), None);
    assert_eq!(view.field("absent", "hash"), None);
    assert!(view.get("start").is_some());
}

#[test]
fn differs_holds_for_every_moved_field_and_names_the_one_that_did_not() {
    assert_eq!(check("differs", &["position", "hash", "step"]), Ok(()));
    assert_eq!(
        check("differs", &["hash", "action"]),
        Err("differs start action: none -> none".into()),
        "a miss names the verb, the checkpoint, the field and both values"
    );
}

#[test]
fn dropped_holds_only_for_a_number_that_fell() {
    assert_eq!(check("dropped", &["target-vitality"]), Ok(()));
    assert_eq!(
        check("dropped", &["step"]),
        Err("dropped start step: 3 -> 11".into()),
        "a reading that rose is a miss, not a drop"
    );
    assert_eq!(
        check("dropped", &["hash"]),
        Err("hash is not a number".into())
    );
}

#[test]
fn same_and_more_keep_the_meanings_the_new_verbs_mirror() {
    assert_eq!(check("same", &["action"]), Ok(()));
    assert_eq!(
        check("same", &["hash"]),
        Err("same start hash: abc -> def".into())
    );
    assert_eq!(check("more", &["step"]), Ok(()));
    assert_eq!(
        check("more", &["hash"]),
        Err("hash is not a counter".into()),
        "`more` stays integer-only; `dropped` is the one that reads decimals"
    );
    // `dropped` reads a decimal `more` would refuse, which is why the strike's
    // falling vitality needed a verb of its own.
    assert_eq!(
        check("more", &["target-vitality"]),
        Err("target-vitality is not a counter".into())
    );
}

#[test]
fn an_unknown_checkpoint_field_or_verb_fails_before_any_comparison() {
    let held = taken();
    let view = Checkpoints(&held);
    assert_eq!(
        view.compare("differs", "absent", &["hash"], &now()),
        Err("unknown checkpoint absent".into())
    );
    assert_eq!(
        view.compare("dropped", "start", &["absent"], &now()),
        Err("checkpoint start has no absent".into())
    );
    assert_eq!(
        view.compare("differs", "start", &["saves"], &now()),
        Err("no snapshot field saves".into())
    );
    assert_eq!(
        view.compare("bigger", "start", &["step"], &now()),
        Err("unknown checkpoint verb bigger".into())
    );
}
