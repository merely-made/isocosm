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

fn map(key: &str, place: &str) -> Assertion {
    let cell = CellEdit {
        at: [1, 0, 2],
        kind: "stone".into(),
        height: 2,
    };
    Assertion::Edit(MapEdit {
        key: key.into(),
        place: place.into(),
        cells: vec![cell],
    })
}

fn storylet(key: &str, asserts: Vec<Assertion>) -> Assertion {
    let source = "arrival".into();
    Assertion::Storylet(Applied {
        key: key.into(),
        source,
        asserts,
    })
}

#[test]
fn a_map_edit_is_held_on_an_asserted_place() {
    let mut s = session();
    assert!(s.command(Command::Assert(map("m", "ford"))).is_err());
    s.command(Command::Assert(place("ford"))).unwrap();
    assert_eq!(
        s.command(Command::Assert(map("m", "ford"))).unwrap(),
        "map:m"
    );
    assert_eq!(s.sim.state.asserted.maps["m"].cells.len(), 1);
    assert!(s.command(Command::Assert(map("m", "shrine"))).is_err());
}

#[test]
fn a_storylet_lands_whole_or_not_at_all() {
    let mut s = session();
    let before = (
        s.sim.state_hash(),
        s.sim.state.population.next_id,
        s.entries.len(),
    );
    let bad = storylet("s", vec![place("ford"), route("r", "ford", "nowhere")]);
    assert!(s.command(Command::Assert(bad)).is_err());
    assert!(
        s.sim.authored_site("ford").is_none(),
        "nothing half-applied"
    );
    assert_eq!(
        (
            s.sim.state_hash(),
            s.sim.state.population.next_id,
            s.entries.len()
        ),
        before
    );
    let good = storylet("s", vec![place("ford"), map("m", "ford")]);
    assert_eq!(
        s.command(Command::Assert(good.clone())).unwrap(),
        "storylet:s"
    );
    assert!(s.sim.authored_site("ford").is_some());
    assert!(s.sim.state.asserted.storylets.contains_key("s"));
    assert_eq!(s.command(Command::Assert(good)).unwrap(), "storylet:s");
    let pack = Assertion::PackForced(Applied {
        key: "p".into(),
        source: "watchtower".into(),
        asserts: vec![place("shrine")],
    });
    s.command(Command::Assert(pack)).unwrap();
    assert!(s.sim.state.asserted.packs.contains_key("p"));
    let loaded = Session::load(s.save(), Execution::Grouped).unwrap();
    assert_eq!(loaded.sim.state_hash(), s.sim.state_hash());
    assert_eq!(loaded.sim.state.asserted, s.sim.state.asserted);
}

#[test]
fn a_fold_holds_groups_whole() {
    let bad = [storylet("s", vec![place("ford"), map("m", "nowhere")])];
    assert_eq!(Asserted::fold(&bad), Err(Refused::Unplaced("m".into())));
    let good = [storylet("s", vec![place("ford"), map("m", "ford")])];
    let folded = Asserted::fold(&good).unwrap();
    assert!(folded.places.contains_key("ford"));
    assert!(folded.records.maps.contains_key("m"));
    let mut again = folded.clone();
    assert_eq!(again.apply(&good[0]), Ok(false));
}
