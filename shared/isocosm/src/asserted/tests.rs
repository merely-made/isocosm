// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Command, Execution, Founding, Session};

fn session() -> Session {
    Session::new(Founding::default().generate().unwrap(), Execution::Grouped).unwrap()
}

fn eels() -> Faction {
    Faction {
        authored: Authored {
            key: "eel-cult".into(),
            name: "The Eel Cult".into(),
            tags: BTreeSet::from(["cult".into()]),
            claims: BTreeSet::from(["drowned-ford".into()]),
        },
        governance: "theocracy".into(),
        focus: BTreeSet::from(["secrecy".into()]),
    }
}

#[test]
fn an_authored_faction_is_a_polity_with_no_members_yet() {
    let mut s = session();
    let next = s.sim.state.population.next_id;
    let out = s
        .command(Command::Assert(Assertion::Faction(eels())))
        .unwrap();
    assert_eq!(out, format!("polity:{next}"));
    let (id, polity) = s.sim.authored_polity("eel-cult").unwrap();
    assert_eq!(id, next);
    assert!(polity.constitution.members.is_empty());
    assert_eq!(polity.authored.as_ref().unwrap().name, "The Eel Cult");
    // Again is nothing new; otherwise is refused.
    assert_eq!(
        s.command(Command::Assert(Assertion::Faction(eels())))
            .unwrap(),
        out
    );
    let mut other = eels();
    other.governance = "council".into();
    assert!(
        s.command(Command::Assert(Assertion::Faction(other)))
            .is_err()
    );
    assert_eq!(s.entries.len(), 2, "a refused assertion is no entry");
}

#[test]
fn an_authored_fact_is_a_note_of_a_declared_kind() {
    let mut s = session();
    let kind = s
        .sim
        .genesis
        .rules
        .note_kinds
        .iter()
        .next()
        .unwrap()
        .clone();
    let fact = |kind: &str| Fact {
        about: "place:drowned-ford".into(),
        key: "ford-is-cursed".into(),
        kind: kind.into(),
        text: "The ford is cursed.".into(),
        tags: BTreeSet::new(),
    };
    let notes = s.sim.state.notes.len();
    assert!(
        s.command(Command::Assert(Assertion::Fact(fact("vtt:none"))))
            .is_err()
    );
    s.command(Command::Assert(Assertion::Fact(fact(&kind))))
        .unwrap();
    s.command(Command::Assert(Assertion::Fact(fact(&kind))))
        .unwrap();
    assert_eq!(s.sim.state.notes.len(), notes + 1);
    assert_eq!(s.sim.state.notes[notes].core.cause, "assert:ford-is-cursed");
}
