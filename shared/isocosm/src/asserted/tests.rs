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

fn place(key: &str) -> Assertion {
    Assertion::Place(Place {
        key: key.into(),
        name: key.into(),
        ..Default::default()
    })
}

fn route(key: &str, from: &str, to: &str) -> Assertion {
    Assertion::Route(Route {
        key: key.into(),
        from: from.into(),
        to: to.into(),
        tags: BTreeSet::new(),
        weight: 3,
    })
}

#[test]
fn authored_nouns_land_on_native_nouns() {
    let mut s = session();
    let run = |s: &mut Session, a: Assertion| s.command(Command::Assert(a)).unwrap();
    run(&mut s, Assertion::Faction(eels()));
    run(&mut s, place("ford"));
    run(&mut s, place("shrine"));
    run(&mut s, route("ford-shrine", "ford", "shrine"));
    let mara = Character {
        key: "mara".into(),
        faction: Some("eel-cult".into()),
        ..Default::default()
    };
    run(&mut s, Assertion::Character(mara));
    let law = Law {
        key: "iron".into(),
        ..Default::default()
    };
    run(&mut s, Assertion::Law(law));
    let line = HistoryLine {
        key: "arrival".into(),
        participants: vec!["mara".into()],
        place: Some("ford".into()),
        ..Default::default()
    };
    assert_eq!(run(&mut s, Assertion::History(line)), "line:arrival");
    let sim = &s.sim;
    let (ford, site) = sim.authored_site("ford").unwrap();
    let (shrine, _) = sim.authored_site("shrine").unwrap();
    assert_eq!(site.routes[0].to, shrine);
    let (mara, entity) = sim.authored_entity("mara").unwrap();
    assert_eq!(entity.place, crate::directing::PLACELESS);
    let (polity, _) = sim.authored_polity("eel-cult").unwrap();
    assert!(related(&sim.state.relations, mara, "member", polity).is_some());
    assert!(sim.state.laws.contains_key("iron"));
    let event = &sim.state.events["line:arrival"];
    assert_eq!((event.place, event.subject), (ford, mara));
    assert!(
        s.command(Command::Assert(route("bad", "ford", "nowhere")))
            .is_err()
    );
}

#[test]
fn a_fold_holds_what_the_entries_assert() {
    let entries = [place("ford"), place("shrine"), route("r", "ford", "shrine")];
    let folded = Asserted::fold(&entries).unwrap();
    assert_eq!(folded.routes["r"].weight, 3);
    let mut again = folded.clone();
    assert_eq!(again.apply(&place("ford")), Ok(false));
    let early = [route("r", "ford", "shrine")];
    assert_eq!(Asserted::fold(&early), Err(Refused::Unplaced("r".into())));
}
