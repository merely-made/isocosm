// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{
    Command, Execution, Founding, Session,
    asserted::{
        Applied, Asserted, Assertion, Authored, CellEdit, Character, CharacterTable, Fact, Faction,
        HistoryLine, Law, MapEdit, Place, Refused, Route,
    },
    history::Saved,
    schema::{Id, Method, related},
    simulation::Genesis,
};
use isocosm_vtt::native::{Phase, Replay};
use std::collections::{BTreeMap, BTreeSet};

fn genesis(seed: u64) -> Genesis {
    Founding {
        seed,
        sites: 2,
        population: 6,
        cohort_size: 2,
        lineages: 2,
        ..Default::default()
    }
    .generate()
    .unwrap()
}

fn place(key: &str) -> Assertion {
    Assertion::Place(Place {
        key: key.into(),
        name: key.into(),
        map: Some(format!("map:{key}")),
        position: Some((3, -2)),
        ..Default::default()
    })
}

fn fact(key: &str) -> Assertion {
    Assertion::Fact(Fact {
        key: key.into(),
        about: "ford".into(),
        kind: "sim:observed".into(),
        text: "The water rises.".into(),
        tags: BTreeSet::from(["flood".into()]),
    })
}

fn edit(key: &str, place: &str) -> Assertion {
    Assertion::Edit(MapEdit {
        key: key.into(),
        place: place.into(),
        cells: vec![CellEdit {
            at: [1, 0, 2],
            kind: "stone".into(),
            height: 2,
        }],
    })
}

fn character(owner: Id) -> Character {
    Character {
        key: "mira".into(),
        name: "Mira".into(),
        faction: Some("eels".into()),
        place: Some("ford".into()),
        table: Some(CharacterTable {
            cell: [-7, 2, 13],
            owner: Some(owner),
        }),
        ..Default::default()
    }
}

fn assertions() -> Vec<Assertion> {
    vec![
        Assertion::Faction(Faction {
            authored: Authored {
                key: "eels".into(),
                name: "The Eels".into(),
                tags: BTreeSet::from(["cult".into()]),
                claims: BTreeSet::from(["ford".into()]),
            },
            governance: "council".into(),
            focus: BTreeSet::from(["secrecy".into()]),
        }),
        place("ford"),
        place("shrine"),
        Assertion::Route(Route {
            key: "ford-shrine".into(),
            from: "ford".into(),
            to: "shrine".into(),
            weight: 3,
            tags: BTreeSet::from(["river".into()]),
        }),
        Assertion::Character(character(77)),
        Assertion::Law(Law {
            key: "iron".into(),
            name: "Iron".into(),
            text: "Bring no iron.".into(),
            parameters: BTreeMap::from([("penalty".into(), "one".into())]),
            ..Default::default()
        }),
        Assertion::History(HistoryLine {
            key: "arrival".into(),
            time: -4,
            kind: "arrived".into(),
            text: "Mira arrived.".into(),
            participants: vec!["mira".into()],
            place: Some("ford".into()),
            ..Default::default()
        }),
        Assertion::Storylet(Applied {
            key: "arrival-story".into(),
            source: "arrival".into(),
            asserts: vec![
                Assertion::Character(Character {
                    key: "echo".into(),
                    name: "Echo".into(),
                    ..Default::default()
                }),
                fact("story-fact"),
            ],
        }),
        Assertion::PackForced(Applied {
            key: "river-pack".into(),
            source: "river".into(),
            asserts: vec![place("dock"), edit("ford-edit", "ford")],
        }),
        fact("arrival-summary"),
    ]
}

fn json_saved(replay: &Replay) -> Saved {
    serde_json::from_slice(&serde_json::to_vec(&replay.save()).unwrap()).unwrap()
}

#[test]
fn native_reading_carries_all_nouns_and_groups() {
    let inputs = assertions();
    let replay = Replay::from_bench_assertions(genesis(7), &inputs, Execution::Grouped).unwrap();
    let reading = replay.reading();
    let (ford, site) = reading.authored_site("ford").unwrap();
    let (shrine, reverse) = reading.authored_site("shrine").unwrap();
    assert_eq!(site.routes[0].to, shrine);
    assert_eq!(site.routes[0].travel, 3);
    assert_eq!(reverse.routes[0].to, ford);
    assert_eq!(site.authored.as_ref().unwrap().position, Some((3, -2)));
    assert_eq!(
        site.authored.as_ref().unwrap().map.as_deref(),
        Some("map:ford")
    );
    let (polity, faction) = reading.authored_polity("eels").unwrap();
    assert_eq!(faction.authored.as_ref().unwrap().name, "The Eels");
    assert_eq!(faction.constitution.governance, "council");
    assert!(faction.constitution.members.is_empty());
    let (mira, entity) = reading.authored_entity("mira").unwrap();
    assert_eq!(entity.authored.as_ref(), Some(&character(77)));
    assert_eq!(entity.place, isocosm::directing::PLACELESS);
    assert!(entity.body.is_none());
    assert!(related(&reading.state().relations, mira, "member", polity).is_some());
    assert_eq!(reading.state().laws["iron"].parameters["penalty"], "one");
    let event = &reading.state().events["line:arrival"];
    assert_eq!((event.place.site().unwrap(), event.subject), (ford, mira));
    assert_eq!(event.authored.as_ref().unwrap().time, -4);
    assert!(reading.authored_entity("echo").is_some());
    assert_eq!(
        reading.state().asserted.maps["ford-edit"].cells[0].kind,
        "stone"
    );
    assert_eq!(
        reading.state().asserted.storylets["arrival-story"]
            .asserts
            .len(),
        2
    );
    assert_eq!(
        reading.state().asserted.packs["river-pack"].asserts.len(),
        2
    );
    assert!(
        reading
            .state()
            .notes
            .iter()
            .any(|n| n.core.cause == "assert:arrival-summary")
    );
    assert!(reading.state().edits.is_empty());
    assert_eq!(reading.execution(), Execution::Grouped);
    assert_eq!(reading.genesis().seed, 7);
    assert_eq!(reading.entries().len(), inputs.len());
    for (index, (entry, input)) in reading.entries().iter().zip(inputs).enumerate() {
        assert_eq!(entry.command, Command::Assert(input));
        assert_eq!(entry.order, index as u64);
    }
}

#[test]
fn individual_and_grouped_native_hashes_and_json_replay_agree() {
    for seed in [0, 7, u64::MAX] {
        let g = genesis(seed);
        let inputs = assertions();
        let individual =
            Replay::from_bench_assertions(g.clone(), &inputs, Execution::Individuals).unwrap();
        let grouped = Replay::from_bench_assertions(g, &inputs, Execution::Grouped).unwrap();
        assert_eq!(
            individual.reading().state_hash(),
            grouped.reading().state_hash()
        );
        assert_eq!(individual.save(), grouped.save());
        for (replay, mode) in [
            (&individual, Execution::Grouped),
            (&grouped, Execution::Individuals),
        ] {
            let loaded = Replay::load(json_saved(replay), mode).unwrap();
            assert_eq!(loaded.save(), replay.save());
            assert_eq!(loaded.reading().state(), replay.reading().state());
            assert_eq!(loaded.reading().execution(), mode);
        }
    }
}

#[test]
fn duplicate_native_assertions_stay_in_the_ordered_command_log() {
    let replay = Replay::from_bench_assertions(
        genesis(1),
        &[place("ford"), place("ford")],
        Execution::Grouped,
    )
    .unwrap();
    assert_eq!(replay.reading().entries().len(), 2);
    assert_eq!(
        replay.reading().entries()[0].outcome,
        replay.reading().entries()[1].outcome
    );
    assert_eq!(
        replay.reading().state().sites.len(),
        replay.reading().genesis().sites.len() + 1
    );
    assert_eq!(
        Replay::load(json_saved(&replay), Execution::Individuals)
            .unwrap()
            .save(),
        replay.save()
    );
}

#[test]
fn drawn_site_edit_uses_native_admission_not_the_sim_off_fold() {
    let inputs = [edit("drawn-edit", "site:0")];
    assert_eq!(
        Asserted::fold(&inputs),
        Err(Refused::Unplaced("drawn-edit".into()))
    );
    let g = genesis(3);
    assert!(g.sites.contains_key(&0));
    let sites = g.sites.clone();
    let replay = Replay::from_bench_assertions(g, &inputs, Execution::Grouped).unwrap();
    assert_eq!(
        replay.reading().state().asserted.maps["drawn-edit"].place,
        "site:0"
    );
    assert_eq!(replay.reading().state().sites, sites);
    assert!(replay.reading().state().edits.is_empty());
    assert_eq!(
        Replay::load(json_saved(&replay), Execution::Individuals)
            .unwrap()
            .save(),
        replay.save()
    );
}

fn controlled() -> (Saved, Id, Id) {
    let g = Founding {
        seed: 7,
        sites: 4,
        population: 48,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        bodies: Some(isocosm::bodied::Bodies::default()),
        played: Some(isocosm::directing::Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut session = isocosm_host::founding::found(g, Execution::Grouped, isocosm_host::founding::Purpose::Bench, None).unwrap().into_parts().unwrap().0;
    let participant = session
        .command(Command::Join)
        .unwrap()
        .strip_prefix("participant:")
        .unwrap()
        .parse()
        .unwrap();
    let critter = *session
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, g)| g.entity.method == Method::Deliberative)
        .unwrap()
        .0;
    session
        .command(Command::Take {
            participant,
            critter,
        })
        .unwrap();
    assert_eq!(session.sim.plays(participant), Some(critter));
    let body = session.sim.state().population.get(critter).unwrap();
    assert!(body.body.is_some());
    assert_ne!(body.place, isocosm::directing::PLACELESS);
    (session.save(), participant, critter)
}

#[test]
fn late_assertion_failure_is_atomic_and_metadata_keeps_taken_control() {
    let (saved, participant, critter) = controlled();
    let original = Replay::load(saved, Execution::Individuals).unwrap();
    let before = original.save();
    let state = original.reading().state().clone();
    let mut bad = fact("bad");
    if let Assertion::Fact(f) = &mut bad {
        f.kind = "vtt:undeclared".into();
    }
    let err = original.with_assertions(&[place("ford"), bad]).unwrap_err();
    assert_eq!(err.phase, Phase::Assertion { index: 1 });
    assert_eq!(err.reason, "unknown note kind");
    assert_eq!(original.save(), before);
    assert_eq!(original.reading().state(), &state);
    assert!(original.reading().authored_site("ford").is_none());
    let mut table_character = character(participant);
    table_character.faction = None;
    let updated = original
        .with_assertions(&[place("ford"), Assertion::Character(table_character.clone())])
        .unwrap();
    let reading = updated.reading();
    let (_, authored) = reading.authored_entity("mira").unwrap();
    assert_eq!(authored.authored.as_ref(), Some(&table_character));
    assert!(authored.body.is_none());
    assert_eq!(authored.place, isocosm::directing::PLACELESS);
    assert_eq!(
        reading.state().population.get(critter),
        state.population.get(critter)
    );
    assert_eq!(
        reading.state().population.get(participant),
        state.population.get(participant)
    );
    assert_eq!(reading.state().relations, state.relations);
    let native = Session::load(updated.save(), Execution::Grouped).unwrap();
    assert_eq!(native.sim.plays(participant), Some(critter));
    assert_eq!(original.save(), before);
    assert_eq!(
        Replay::load(json_saved(&updated), Execution::Grouped)
            .unwrap()
            .save(),
        updated.save()
    );
}

#[test]
fn a_failed_nested_group_cannot_publish_earlier_top_level_assertions() {
    let original = Replay::from_bench_assertions(genesis(2), &[], Execution::Grouped).unwrap();
    let before = original.save();
    let batch = [
        place("earlier"),
        Assertion::Storylet(Applied {
            key: "broken".into(),
            source: "river".into(),
            asserts: vec![place("half"), edit("bad-map", "missing")],
        }),
    ];
    let err = original.with_assertions(&batch).unwrap_err();
    assert_eq!(err.phase, Phase::Assertion { index: 1 });
    assert_eq!(err.reason, "map bad-map names an unasserted place");
    assert_eq!(original.save(), before);
    assert!(original.reading().authored_site("earlier").is_none());
    assert!(original.reading().authored_site("half").is_none());
    assert!(original.reading().state().asserted.storylets.is_empty());
}

#[test]
fn errors_retain_native_text_and_typed_calling_context() {
    let mut invalid = genesis(1);
    invalid.version = 0;
    let native = isocosm_host::founding::found(
        invalid.clone(),
        Execution::Grouped,
        isocosm_host::founding::Purpose::Bench,
        None,
    )
    .err()
    .expect("invalid native founding is refused");
    let err = Replay::from_bench_assertions(invalid, &[], Execution::Grouped).unwrap_err();
    assert_eq!(err.phase, Phase::Founding);
    assert_eq!(err.reason, native);
    let invalid_order = [
        Assertion::Route(Route {
            key: "early".into(),
            from: "ford".into(),
            to: "shrine".into(),
            weight: 3,
            ..Default::default()
        }),
        place("ford"),
    ];
    let err = Replay::from_bench_assertions(genesis(1), &invalid_order, Execution::Grouped).unwrap_err();
    assert_eq!(err.phase, Phase::Assertion { index: 0 });
    assert_eq!(err.reason, "route early names an unasserted place");
    let replay = Replay::from_bench_assertions(genesis(1), &assertions(), Execution::Grouped).unwrap();
    let saved = replay.save();
    let mut version = saved.clone();
    version.version = 0;
    let mut digest = saved.clone();
    digest.genesis.seed ^= 1;
    let mut hash = saved.clone();
    hash.state_hash ^= 1;
    let mut omitted = saved.clone();
    omitted.entries.pop();
    let mut outcome = saved;
    outcome.entries[0].outcome = "wrong".into();
    for tampered in [version, digest, hash, omitted, outcome] {
        let native = Session::load(tampered.clone(), Execution::Individuals).unwrap_err();
        let err = Replay::load(tampered, Execution::Individuals).unwrap_err();
        assert_eq!(err.phase, Phase::Load);
        assert_eq!(err.reason, native);
    }
}

fn unknown_loss() -> (Genesis, Id, isocosm::schema::PartId) {
    let g = isocosm::probe::BodyFounding {
        sites: [1, 1],
        producers: [0, 0],
        grazers: [1, 1],
        cohort: 1,
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis;
    let (entity, part) = g
        .population
        .groups
        .iter()
        .find_map(|(&id, group)| {
            group
                .entity
                .parts
                .iter()
                .find(|(_, p)| p.cells.values().sum::<u32>() > 1)
                .map(|(&part, _)| (id, part))
        })
        .unwrap();
    let mut session = isocosm_host::founding::found(
        g,
        Execution::Individuals,
        isocosm_host::founding::Purpose::Bench,
        None,
    )
    .unwrap()
    .into_parts()
    .unwrap()
    .0;
    session
        .command(Command::Wound {
            entity,
            part: Some(part),
            cells: 1,
        })
        .unwrap();
    let mut older = session.sim.genesis().clone();
    older.population = session.sim.state().population.clone();
    older.sites = session.sim.state().sites.clone();
    let wounded = older
        .population
        .lift(entity)
        .unwrap()
        .parts
        .get_mut(&part)
        .unwrap();
    assert_eq!(wounded.lost.len(), 1);
    assert_eq!(
        wounded.losses.len(),
        1,
        "native wound supplies allocation before legacy omission"
    );
    wounded.losses.clear();
    (older, entity, part)
}

#[test]
fn unknown_loss_allocation_survives_native_replay_without_becoming_free_pool() {
    let (g, entity, part) = unknown_loss();
    let older = &g.population.get(entity).unwrap().parts[&part];
    assert!(!older.lost.is_empty());
    assert!(older.losses.is_empty());
    let lost = older.lost.clone();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let replay = Replay::from_bench_assertions(g.clone(), &[place("ford")], mode).unwrap();
        let loaded = Replay::load(json_saved(&replay), mode).unwrap();
        let reading = loaded.reading();
        let wounded = &reading.state().population.get(entity).unwrap().parts[&part];
        assert_eq!(wounded.lost, lost);
        assert!(wounded.losses.is_empty());
        assert!(
            reading
                .state()
                .population
                .get(entity)
                .unwrap()
                .body
                .is_some()
        );
        assert_eq!(loaded.save(), replay.save());
        let mut repaired_without_digest = replay.save();
        repaired_without_digest
            .genesis
            .population
            .lift(entity)
            .unwrap()
            .parts
            .get_mut(&part)
            .unwrap()
            .losses
            .insert(lost[0], None);
        let err = Replay::load(repaired_without_digest, mode).unwrap_err();
        assert_eq!(err.phase, Phase::Load);
        assert_eq!(err.reason, "save version or genesis digest mismatch");
    }
}
