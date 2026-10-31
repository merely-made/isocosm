// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmograph::{
    Execution, Founding,
    asserted::{Assertion as Native, Authored, Faction, HistoryLine, Place},
    directing::PLACELESS,
};
use cosmograph_overlay::{
    EntityHandle, EventHandle, FactionHandle, ParticipantHandle, PlaceHandle, Pointable, WorldPoint,
    vtt::{Assertion, Cell, CellEdit, Fact, MapEdit, NewCharacter},
};
use isocosm_native::{contract, native::{ContractError, Replay}};
use std::collections::BTreeSet;

fn replay(mode: Execution) -> Replay {
    let genesis = Founding {
        seed: 7,
        sites: 2,
        population: 4,
        lineages: 2,
        cohort_size: 2,
        ..Default::default()
    }
    .generate()
    .unwrap();
    Replay::from_bench_assertions(genesis, &[
        Native::Faction(Faction {
            authored: Authored {
                key: "eels".into(), name: "The Eels".into(),
                tags: BTreeSet::new(), claims: BTreeSet::new(),
            },
            governance: "council".into(), focus: BTreeSet::new(),
        }),
        Native::Place(Place {
            key: "ford".into(), name: "Ford".into(), ..Default::default()
        }),
    ], mode).unwrap()
}

fn character(base: &Replay, key: &str) -> Assertion {
    Assertion::Character(NewCharacter {
        key: key.into(), name: key.into(),
        faction: Some(FactionHandle(base.reading().authored_polity("eels").unwrap().0)),
        at: Cell {
            site: PlaceHandle(base.reading().authored_site("ford").unwrap().0),
            at: WorldPoint([-7, 2, 13]),
        },
        owner: Some(ParticipantHandle(77)),
    })
}

fn fact(key: &str, about: Pointable) -> Assertion {
    Assertion::Fact(Fact {
        key: key.into(), kind: "sim:observed".into(), about: Some(about),
        text: "The water rises.".into(), tags: vec!["flood".into()],
    })
}

fn batch(base: &Replay) -> Vec<Assertion> {
    let allocated = base.reading().state().population.next_id;
    vec![character(base, "mira"), fact("arrival-summary", Pointable::Entity(EntityHandle(allocated)))]
}

#[test]
fn native_contract_batch_uses_allocated_ids_in_order() {
    let original = replay(Execution::Grouped);
    let before = original.save();
    let allocated = original.reading().state().population.next_id;
    assert_ne!(allocated, contract::handle("mira"));
    let accepted = original.with_contract_assertions(&batch(&original));
    assert!(accepted.is_ok(), "contract lowering observes preceding accepted native identities");
    let accepted = accepted.unwrap();
    let (id, entity) = accepted.reading().authored_entity("mira").unwrap();
    assert_eq!(id, allocated);
    assert_eq!(entity.place, PLACELESS);
    let table = entity.authored.as_ref().unwrap().table.as_ref().unwrap();
    assert_eq!((table.cell, table.owner), ([-7, 2, 13], Some(77)));
    let state = accepted.reading().state();
    let note = state.notes.iter().find(|note| note.core.object == "arrival-summary");
    assert_eq!(note.map(|note| note.core.subject.as_str()), Some("mira"),
        "contract batch retains its final note after allocating the native character");
    assert_eq!(original.save(), before);
}

#[test]
fn native_contract_groups_retain_staged_keys_and_table_metadata() {
    let original = replay(Execution::Grouped);
    let group = Assertion::Storylet {
        key: "arrival-story".into(), storylet: "river".into(),
        asserts: vec![character(&original, "echo"),
            fact("echo-note", Pointable::Entity(EntityHandle(contract::handle("echo"))))],
    };
    let forced = Assertion::PackForced {
        key: "river-pack".into(), pack: "river".into(),
        asserts: vec![fact("ford-note", Pointable::Place(PlaceHandle(
            original.reading().authored_site("ford").unwrap().0)))],
    };
    let accepted = original.with_contract_assertions(&[group, forced]).unwrap();
    let state = accepted.reading().state();
    assert_eq!(state.asserted.storylets["arrival-story"].source, "river");
    assert_eq!(state.asserted.packs["river-pack"].source, "river");
    let (echo, entity) = accepted.reading().authored_entity("echo").unwrap();
    let polity = accepted.reading().authored_polity("eels").unwrap().0;
    assert!(state.relations.iter().any(|r| r.subject == echo && r.kind == "member" && r.object == polity));
    assert_eq!(entity.place, PLACELESS);
    assert_eq!(entity.authored.as_ref().unwrap().table.as_ref().unwrap().owner, Some(77));
    assert!(state.notes.iter().any(|note| note.core.object == "echo-note" && note.core.subject == "echo"));
    assert!(state.notes.iter().any(|note| note.core.object == "ford-note" && note.core.subject == "ford"));
}

#[test]
fn native_contract_translation_refusal_keeps_original_unpublished() {
    let original = replay(Execution::Grouped);
    let before = original.save();
    let bad = Assertion::Edit(MapEdit {
        key: "missing-site".into(), site: PlaceHandle(u64::MAX), cells: vec![],
    });
    let inputs = [fact("earlier", Pointable::Place(PlaceHandle(
        original.reading().authored_site("ford").unwrap().0))), bad];
    let result = original.with_contract_assertions(&inputs);
    assert!(result.is_err(), "contract lowering refusal prevents publishing an accepted prefix");
    assert_eq!(result.unwrap_err(), ContractError::Lowering {
        index: 1, refusal: contract::Refusal::Unnamed(Pointable::Place(PlaceHandle(u64::MAX))),
    });
    assert_eq!(original.save(), before);
    assert!(!original.reading().state().notes.iter().any(|n| n.core.object == "earlier"));
}

#[test]
fn native_contract_native_refusal_keeps_original_unpublished() {
    let original = replay(Execution::Grouped);
    let before = original.save();
    let site = Pointable::Place(PlaceHandle(original.reading().authored_site("ford").unwrap().0));
    let mut unsupported = fact("unsupported-kind", site);
    if let Assertion::Fact(fact) = &mut unsupported {
        fact.kind = "history".into();
    }
    let inputs = [fact("earlier", site), unsupported];
    let result = original.with_contract_assertions(&inputs);
    assert!(result.is_err(), "native refusal prevents publishing an accepted prefix");
    assert_eq!(result.unwrap_err(), ContractError::Native {
        index: 1, reason: "unknown note kind".into(),
    });
    assert_eq!(original.save(), before);
    assert!(!original.reading().state().notes.iter().any(|n| n.core.object == "earlier"));
}

#[test]
fn native_contract_modes_replay_and_duplicates_keep_one_state() {
    let individual = replay(Execution::Individuals);
    let grouped = replay(Execution::Grouped);
    let inputs = batch(&grouped);
    let a = individual.with_contract_assertions(&inputs).unwrap();
    let b = grouped.with_contract_assertions(&inputs).unwrap();
    assert_eq!(a.reading().state_hash(), b.reading().state_hash());
    let again = b.with_contract_assertions(&inputs).unwrap();
    assert_eq!(again.reading().state_hash(), b.reading().state_hash());
    assert_eq!(again.reading().entries().len(), b.reading().entries().len() + 2);
    let saved = serde_json::from_slice(&serde_json::to_vec(&again.save()).unwrap()).unwrap();
    let loaded = Replay::load(saved, Execution::Individuals).unwrap();
    assert_eq!(loaded.reading().state_hash(), again.reading().state_hash());
    assert_eq!(loaded.reading().entries(), again.reading().entries());
}

#[test]
fn native_contract_event_handles_and_drawn_site_edits_keep_their_keys() {
    let original = replay(Execution::Grouped);
    let history = HistoryLine {
        key: "arrival".into(), time: -4, kind: "arrived".into(),
        text: "The travelers arrived.".into(), place: Some("ford".into()),
        ..Default::default()
    };
    let event_key = history.event_key();
    let original = original.with_assertions(&[Native::History(history)]).unwrap();
    let drawn = original.reading().state().sites.iter()
        .find(|(_, site)| site.authored.is_none()).unwrap().0;
    let drawn = *drawn;
    let edit = Assertion::Edit(MapEdit {
        key: "drawn-edit".into(), site: PlaceHandle(drawn),
        cells: vec![CellEdit { at: WorldPoint([1, 0, 2]), kind: "stone".into(), height: 2 }],
    });
    let about = Pointable::Event(EventHandle(contract::handle(&event_key)));
    let accepted = original.with_contract_assertions(&[edit, fact("event-note", about)]).unwrap();
    let state = accepted.reading().state();
    assert_eq!(state.asserted.maps["drawn-edit"].place, format!("site:{drawn}"));
    assert_eq!(state.asserted.maps["drawn-edit"].cells[0].at, [1, 0, 2]);
    assert!(state.notes.iter().any(|n| n.core.object == "event-note" && n.core.subject == event_key));
}


#[test]
fn native_contract_declared_table_kinds_survive_modes_and_replay() {
    let kinds = ["reveal", "narration", "history"];
    let mut genesis = Founding {
        seed: 7, sites: 2, population: 4, lineages: 2, cohort_size: 2,
        ..Default::default()
    }.generate().unwrap();
    for kind in kinds {
        assert!(!genesis.rules.note_kinds.contains(kind));
        genesis.rules.note_kinds.insert(kind.into());
    }
    let inputs = kinds.map(|kind| Assertion::Fact(Fact {
        key: format!("table-{kind}"), kind: kind.into(), about: None,
        text: format!("Authored {kind}."), tags: vec!["table".into()],
    }));
    let mut hash = None;
    for (mode, reload_mode) in [(Execution::Individuals, Execution::Grouped),
        (Execution::Grouped, Execution::Individuals)] {
        let original = Replay::from_bench_assertions(genesis.clone(), &[], mode).unwrap();
        let before = original.save();
        let accepted = original.with_contract_assertions(&inputs).unwrap();
        assert_eq!(original.save(), before);
        if let Some(expected) = hash {
            assert_eq!(accepted.reading().state_hash(), expected);
        }
        hash = Some(accepted.reading().state_hash());
        let saved = accepted.save();
        assert_eq!(saved.genesis.rules.note_kinds, genesis.rules.note_kinds);
        let saved = serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap();
        let loaded = Replay::load(saved, reload_mode).unwrap();
        assert_eq!(loaded.reading().state_hash(), accepted.reading().state_hash());
        assert_eq!(loaded.reading().entries(), accepted.reading().entries());
        assert_eq!(loaded.reading().genesis().rules.note_kinds, genesis.rules.note_kinds);
        for reading in [accepted.reading(), loaded.reading()] {
            for kind in kinds {
                let key = format!("table-{kind}");
                let note = reading.state().notes.iter().find(|n| n.core.object == key).unwrap();
                assert_eq!(note.core.kind, kind);
                assert_eq!(note.djot, format!("Authored {kind}."));
            }
        }
        let before_refusal = accepted.save();
        let undeclared = Assertion::Fact(Fact {
            key: "table-undeclared".into(), kind: "unlisted-table-kind".into(),
            about: None, text: "Still refused.".into(), tags: vec![],
        });
        assert_eq!(accepted.with_contract_assertions(&[undeclared]).unwrap_err(),
            ContractError::Native { index: 0, reason: "unknown note kind".into() });
        assert_eq!(accepted.save(), before_refusal);
    }
}
