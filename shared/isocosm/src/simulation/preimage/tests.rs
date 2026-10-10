// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent former Character bytes, assembled into its native containers.
//! These controls never encode a current Character or its witness projection.

use super::*;
use crate::{Command, Execution, Founding, Session};
use state_witness::{hash_bytes, hash_value, label};

#[derive(Serialize)]
struct BeforeTable<'a> {
    key: &'a Key,
    name: &'a String,
    tags: &'a BTreeSet<Key>,
    faction: &'a Option<Key>,
    place: &'a Option<Key>,
}

fn bytes<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

fn former_character(c: &asserted::Character) -> Vec<u8> {
    assert!(c.table.is_none());
    bytes(&BeforeTable {
        key: &c.key,
        name: &c.name,
        tags: &c.tags,
        faction: &c.faction,
        place: &c.place,
    })
}

fn former_entity(entity: &crate::schema::Entity) -> Vec<u8> {
    let mut bare = entity.clone();
    bare.authored = None;
    let mut encoded = bytes(&bare);
    if let Some(c) = &entity.authored {
        // Authored is the final Entity field; absent is omitted, present
        // starts with the existing Option::Some marker, then the old fill.
        encoded.push(1);
        encoded.extend(former_character(c));
    }
    encoded
}

fn former_population(pop: &population::Population) -> Vec<u8> {
    // Native order: map length; each id/count/entity; next id.
    let mut encoded = bytes(&pop.groups.len());
    for (id, group) in &pop.groups {
        encoded.extend(bytes(id));
        encoded.extend(bytes(&group.count));
        encoded.extend(former_entity(&group.entity));
    }
    encoded.extend(bytes(&pop.next_id));
    encoded
}

fn former_applied(a: &asserted::Applied) -> Vec<u8> {
    let mut encoded = bytes(&a.key);
    encoded.extend(bytes(&a.source));
    encoded.extend(bytes(&a.asserts.len()));
    for assertion in &a.asserts {
        match assertion {
            asserted::Assertion::Character(c) => {
                // These are the former native Assertion discriminants.
                encoded.extend(bytes(&4u32));
                encoded.extend(former_character(c));
            },
            asserted::Assertion::Storylet(group) => {
                encoded.extend(bytes(&8u32));
                encoded.extend(former_applied(group));
            },
            asserted::Assertion::PackForced(group) => {
                encoded.extend(bytes(&9u32));
                encoded.extend(former_applied(group));
            },
            other => encoded.extend(bytes(other)),
        }
    }
    encoded
}

fn former_records(records: &asserted::Records) -> Vec<u8> {
    let mut encoded = bytes(&records.maps);
    for groups in [&records.storylets, &records.packs] {
        encoded.extend(bytes(&groups.len()));
        for (key, group) in groups {
            encoded.extend(bytes(key));
            encoded.extend(former_applied(group));
        }
    }
    encoded
}

fn character(key: &str) -> asserted::Character {
    asserted::Character {
        key: key.into(),
        name: key.into(),
        tags: BTreeSet::from(["speaker".into()]),
        ..Default::default()
    }
}

fn nested(table: Option<asserted::CharacterTable>) -> Session {
    let mut session = Session::new(
        Founding {
            seed: 7,
            population: 24,
            sites: 3,
            lineages: 2,
            ..Default::default()
        }
        .generate()
        .unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    let mut mira = character("mira");
    mira.table = table;
    let group = |key: &str, assertion| asserted::Applied {
        key: key.into(),
        source: "arrival".into(),
        asserts: vec![assertion],
    };
    for assertion in [
        asserted::Assertion::Storylet(group(
            "story:outer",
            asserted::Assertion::PackForced(group(
                "pack:inner",
                asserted::Assertion::Character(mira),
            )),
        )),
        asserted::Assertion::PackForced(group(
            "pack:outer",
            asserted::Assertion::Storylet(group(
                "story:inner",
                asserted::Assertion::Character(character("omar")),
            )),
        )),
    ] {
        session.command(Command::Assert(assertion)).unwrap();
    }
    session
}

#[test]
fn former_character_bytes_survive_in_population_entity_and_every_nested_group() {
    let session = nested(None);
    let state = session.sim.state();
    let pop = state.population.normalized();
    let population_bytes = former_population(&pop);
    assert_eq!(bytes(&Population::from(&pop)), population_bytes);
    assert_eq!(
        session.sim.witness().get("population"),
        Some(hash_bytes(&population_bytes))
    );
    let entities = session.sim.entity_witness();
    for (id, group) in &pop.groups {
        let former = former_entity(&group.entity);
        assert_eq!(bytes(&Entity::from(&group.entity)), former);
        let digest = hash_bytes(&former);
        assert_eq!(
            entities.get(&label("entity", [id])),
            Some(hash_value(&digest).unwrap())
        );
    }
    let records_bytes = former_records(&state.asserted);
    assert_eq!(bytes(&Records::from(&state.asserted)), records_bytes);
    assert_eq!(
        session.sim.witness().get("asserted"),
        Some(hash_bytes(&records_bytes))
    );
    for (kind, groups) in [
        ("storylet", &state.asserted.storylets),
        ("pack", &state.asserted.packs),
    ] {
        for (key, group) in groups {
            let former = former_applied(group);
            assert_eq!(bytes(&Applied::from(group)), former);
            assert_eq!(entities.get(&label(kind, [key])), Some(hash_bytes(&former)));
        }
    }
}

#[test]
fn an_archive_with_former_character_preimages_replays_with_unchanged_integrity_checks() {
    let session = nested(None);
    let mut saved = session.save();
    let pop = session.sim.state().population.normalized();
    let mut entries = saved.witness.entries().to_vec();
    for entry in &mut entries {
        match entry.label.as_str() {
            "population" => entry.digest = hash_bytes(&former_population(&pop)),
            "asserted" => entry.digest = hash_bytes(&former_records(&session.sim.state().asserted)),
            _ => {},
        }
    }
    saved.witness = serde_json::from_slice(&serde_json::to_vec(&entries).unwrap()).unwrap();
    saved.state_hash = saved.witness.digest();
    let json = serde_json::to_vec(&saved).unwrap();
    assert!(!String::from_utf8_lossy(&json).contains("\"table\""));
    for mode in [Execution::Individuals, Execution::Grouped] {
        let loaded = Session::load_json(&json, mode).unwrap();
        assert_eq!(loaded.save(), saved);
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
    }
    saved.state_hash ^= 1;
    assert_eq!(
        Session::load(saved, Execution::Grouped).unwrap_err(),
        "save state hash mismatch"
    );
}

#[test]
fn present_cell_and_owner_metadata_are_witnessed_in_nouns_and_nested_groups() {
    let none = nested(None);
    let table = asserted::CharacterTable {
        cell: [-7, 2, 13],
        owner: Some(77),
    };
    let present = nested(Some(table.clone()));
    assert_ne!(present.sim.state_hash(), none.sim.state_hash());
    for other in [
        asserted::CharacterTable {
            cell: [-8, 2, 13],
            ..table.clone()
        },
        asserted::CharacterTable {
            owner: None,
            ..table
        },
    ] {
        let changed = nested(Some(other));
        for field in ["population", "asserted"] {
            assert_ne!(
                changed.sim.witness().get(field),
                present.sim.witness().get(field)
            );
        }
        let (id, _) = present.sim.authored_entity("mira").unwrap();
        assert_ne!(
            changed.sim.entity_witness().get(&label("entity", [id])),
            present.sim.entity_witness().get(&label("entity", [id]))
        );
        assert_ne!(
            changed.sim.entity_witness().get("storylet:story:outer"),
            present.sim.entity_witness().get("storylet:story:outer")
        );
        assert_ne!(
            changed.sim.entity_witness().get("pack:pack:inner"),
            present.sim.entity_witness().get("pack:pack:inner")
        );
    }
    let json = serde_json::to_vec(&present.save()).unwrap();
    let loaded = Session::load_json(&json, Execution::Grouped).unwrap();
    assert_eq!(loaded.save(), present.save());
}

#[test]
fn all_unchanged_entity_fields_and_assertion_variants_keep_their_native_encoding() {
    let genesis = Founding {
        seed: 7,
        sites: 3,
        population: 24,
        lineages: 2,
        bodies: Some(crate::bodied::Bodies::default()),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut entity = genesis
        .population
        .groups
        .values()
        .find(|group| group.entity.body.is_some())
        .unwrap()
        .entity
        .clone();
    assert!(entity.body.is_some());
    assert!(!entity.parts.is_empty());
    entity.patch = Some(isometer_space::places::PlaceId {
        site: entity.place,
        cell: [-3, 2, 1],
    });
    entity.varied.push(Varied {
        situs: [1, 2, 3],
        from: "function:sense".into(),
        to: "function:move".into(),
    });
    for authored in [
        None,
        Some(asserted::Character {
            table: Some(asserted::CharacterTable {
                cell: [-7, 2, 13],
                owner: Some(77),
            }),
            ..character("mira")
        }),
    ] {
        entity.authored = authored;
        assert_eq!(bytes(&Entity::from(&entity)), bytes(&entity));
    }
    let empty_group = asserted::Applied::default();
    let group = asserted::Applied {
        key: "all".into(),
        source: "all".into(),
        asserts: vec![
            asserted::Assertion::Faction(Default::default()),
            asserted::Assertion::Fact(Default::default()),
            asserted::Assertion::Place(Default::default()),
            asserted::Assertion::Route(Default::default()),
            asserted::Assertion::Law(Default::default()),
            asserted::Assertion::History(Default::default()),
            asserted::Assertion::Edit(Default::default()),
            asserted::Assertion::Storylet(empty_group.clone()),
            asserted::Assertion::PackForced(empty_group),
        ],
    };
    assert_eq!(bytes(&Applied::from(&group)), bytes(&group));
}
