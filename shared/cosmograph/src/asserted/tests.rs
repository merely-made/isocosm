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
    assert_eq!((event.place.site().unwrap(), event.subject), (ford, mara));
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

#[test]
fn character_table_metadata_saves_and_replays_without_taking_or_moving_a_body() {
    let genesis = Founding {
        seed: 7,
        sites: 4,
        population: 48,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        bodies: Some(crate::bodied::Bodies::default()),
        played: Some(crate::directing::Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut s = Session::new(genesis, Execution::Grouped).unwrap();
    let joined = s.command(Command::Join).unwrap();
    let participant: Id = joined
        .strip_prefix("participant:")
        .unwrap()
        .parse()
        .unwrap();
    let critter = *s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method == Method::Deliberative)
        .unwrap()
        .0;
    s.command(Command::Take {
        participant,
        critter,
    })
    .unwrap();
    let body = s.sim.state().population.get(critter).unwrap().clone();
    assert!(
        body.body.is_some(),
        "the positive control has native body geometry"
    );
    assert_ne!(body.place, crate::directing::PLACELESS);
    assert_eq!(
        s.sim.plays(participant),
        Some(critter),
        "control was actually taken"
    );
    let relations = s.sim.state().relations.clone();
    let player = s.sim.state().population.get(participant).unwrap().clone();
    let character = Character {
        key: "mira".into(),
        name: "Mira".into(),
        table: Some(CharacterTable {
            cell: [-7, 2, 13],
            owner: Some(participant),
        }),
        ..Default::default()
    };
    let command = Command::Assert(Assertion::Character(character.clone()));
    s.command(command.clone()).unwrap();
    let (id, authored) = s.sim.authored_entity("mira").unwrap();
    assert_ne!(id, critter);
    assert_eq!(authored.authored.as_ref(), Some(&character));
    assert_eq!(authored.place, crate::directing::PLACELESS);
    assert!(authored.patch.is_none());
    assert!(authored.body.is_none());
    assert_eq!(s.sim.state().population.get(critter), Some(&body));
    assert_eq!(s.sim.state().population.get(participant), Some(&player));
    assert_eq!(s.sim.state().relations, relations);
    assert_eq!(s.sim.plays(participant), Some(critter));
    let hash = s.sim.state_hash();
    s.command(command).unwrap();
    assert_eq!(
        s.sim.state_hash(),
        hash,
        "identical metadata is a replay no-op"
    );
    let json = serde_json::to_vec(&s.save()).unwrap();
    let loaded = Session::load_json(&json, Execution::Grouped).unwrap();
    assert_eq!(
        loaded.sim.state_hash(),
        hash,
        "the actual saved command log replays"
    );
    assert_eq!(
        loaded
            .sim
            .authored_entity("mira")
            .unwrap()
            .1
            .authored
            .as_ref(),
        Some(&character)
    );
    assert_eq!(loaded.sim.state().population.get(critter), Some(&body));
    assert_eq!(
        loaded.sim.state().population.get(participant),
        Some(&player)
    );
    assert_eq!(loaded.sim.state().relations, relations);
    assert_eq!(loaded.sim.plays(participant), Some(critter));
}

#[test]
fn character_table_metadata_keeps_legacy_json_and_present_or_absent_wire_values() {
    #[derive(Serialize)]
    struct BeforeTable {
        key: Key,
        name: String,
        tags: BTreeSet<Key>,
        faction: Option<Key>,
        place: Option<Key>,
    }
    let legacy = BeforeTable {
        key: "mira".into(),
        name: "Mira".into(),
        tags: BTreeSet::new(),
        faction: None,
        place: None,
    };
    let bytes = serde_json::to_vec(&legacy).unwrap();
    let old: Character = serde_json::from_slice(&bytes).unwrap();
    assert!(
        old.table.is_none(),
        "non-table characters keep their existing path"
    );
    assert_eq!(serde_json::to_vec(&old).unwrap(), bytes);
    assert_eq!(crate::digest(&old), crate::digest(&legacy));
    assert_ne!(
        state_witness::hash_value(&old).unwrap(),
        state_witness::hash_value(&legacy).unwrap(),
        "the current postcard field is a new wire/witness format, not historical compatibility"
    );
    for table in [
        None,
        Some(CharacterTable {
            cell: [-7, 2, 13],
            owner: Some(77),
        }),
        Some(CharacterTable {
            cell: [-7, 2, 13],
            owner: None,
        }),
    ] {
        let character = Character {
            table,
            ..old.clone()
        };
        let bytes = postcard::to_allocvec(&character).unwrap();
        let wired: Character = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(wired, character);
        let json = serde_json::to_vec(&character).unwrap();
        let saved: Character = serde_json::from_slice(&json).unwrap();
        assert_eq!(saved, character);
    }
}
