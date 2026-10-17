// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{Founding, asserted::*, simulation::Genesis};
use isometry_campaign::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn genesis() -> Genesis {
    let mut genesis = Founding { seed: 41, sites: 2, population: 4, lineages: 2,
        cohort_size: 2, ..Default::default() }.generate().unwrap();
    genesis.rules.conditions.insert("weather:storm".into());
    genesis.sites.get_mut(&0).unwrap().conditions.insert("weather:storm".into(), -3);
    genesis
}

pub fn draft() -> CampaignDraft {
    let mut world = CampaignWorld::default();
    for assertion in [
        Assertion::Faction(Faction { authored: Authored { key: "river".into(),
            name: "The River".into(), tags: BTreeSet::from(["water".into()]),
            claims: BTreeSet::from(["ford".into()]) }, governance: "council".into(),
            focus: BTreeSet::from(["memory".into()]) }),
        Assertion::Place(Place { key: "ford".into(), name: "The Ford".into(),
            map: Some("river-map".into()), position: Some((-3, 8)), ..Default::default() }),
        Assertion::Place(Place { key: "shrine".into(), name: "The Shrine".into(),
            ..Default::default() }),
        Assertion::Route(Route { key: "ford-shrine".into(), from: "ford".into(),
            to: "shrine".into(), weight: 19, tags: BTreeSet::from(["water".into()]) }),
        Assertion::Character(Character { key: "mira".into(), name: "Mira".into(),
            faction: Some("river".into()), place: Some("ford".into()),
            table: Some(CharacterTable { cell: [-4, 2, 11], owner: Some(77) }),
            ..Default::default() }),
        Assertion::Law(Law { key: "oath".into(), name: "The Oath".into(),
            text: "Keep the river's memory.".into(),
            parameters: BTreeMap::from([("cost".into(), "three".into())]),
            ..Default::default() }),
        Assertion::History(HistoryLine { key: "arrival".into(), time: -7,
            kind: "arrived".into(), text: "Mira arrived.".into(),
            participants: vec!["mira".into()], place: Some("ford".into()),
            ..Default::default() }),
        Assertion::Fact(Fact { key: "oath-note".into(), kind: "history".into(),
            text: "The oath was spoken.".into(), ..Default::default() }),
    ] { world.assert(assertion).unwrap(); }
    world.storylets.insert("final".into(), StoryletProposal { key: "final".into(),
        entry: "The final oath.".into(), tags: vec!["river".into()],
        requirements: StoryletRequirements { faction_tags: vec!["water".into()],
            hidden_facts: vec!["secret-oath".into()], world_laws: vec!["oath".into()] },
        roles: vec![RoleSlot { key: "keeper".into(), tags: vec![] }],
        effects: vec![StoryletEffect::Item { item: ItemProposal { template: "demo:blade".into(),
            name: "Witness Blade".into(), tags: vec!["river".into()] } }] });
    world.faction_sheets.insert("river".into(), BTreeMap::from([("clock".into(), 23)]));
    world.faction_control.insert("river".into(), "player".into());
    world.party_node.insert("party".into(), "ford".into());
    world.party_pace.insert("party".into(), 50);
    world.party_known.insert("party".into(), BTreeSet::from(["shrine".into()]));
    world.party_resources.insert("party".into(), BTreeMap::from([("food".into(), 9)]));
    let secret = SecretFact { id: "secret-oath".into(), text: "HOST PRIVATE OATH".into(),
        tags: vec!["oath".into()], reveal: RevealCondition::TrustThreshold(-5) };
    CampaignDraft { id: "river".into(), name: "River Campaign".into(), world,
        maps: vec![DraftMap { scale: MapScale::Local, map: LocalMapProposal {
            id: "river-map".into(), name: "River Map".into(), width: 4, height: 3,
            default_ground: "grass".into(), cells: vec![MapCellProposal { col: 1, row: 1,
                ground: Some("stone".into()), prop: Some("shrine".into()), elevation: Some(4) }],
            spawn_zones: vec![SpawnZone { id: "entry".into(), cells: vec![MapPoint { col: 0, row: 0 }] }],
            transitions: vec![MapTransition { id: "gate".into(), at: MapPoint { col: 3, row: 2 },
                target_map: "river-map".into(), target_entry: Some("entry".into()) }],
            encounter_anchors: vec![EncounterAnchor { id: "ambush".into(), at: MapPoint { col: 2, row: 1 },
                tags: vec!["river".into()] }],
        }, inhabitants: vec![MapInhabitant { id: 1, name: "Mira".into(), sprite: "mira".into(),
            at: MapPoint { col: 0, row: 0 }, system: "5e-srd".into(), owner: Some("player".into()),
            stats: BTreeMap::from([("hp".into(), 17), ("speed".into(), 25)]) }] }],
        secrets: vec![secret], rewards: vec![ItemProposal { template: "demo:blade".into(),
            name: "Witness Blade".into(), tags: vec!["river".into()] }],
        starting_map: "river-map".into(), final_storylet: "final".into() }
}

pub fn record() -> GenerationRecord {
    GenerationRecord { id: "generated.river.1".into(), entropy: u64::MAX,
        request: GeneratorRequest { generator: "demo:campaign".into(),
            args: GenValue::Text { value: "river".into() },
            locks: BTreeMap::from([("season".into(), GenValue::Text { value: "winter".into() })]) },
        proposal: GenValue::Campaign { campaign: draft() } }
}
