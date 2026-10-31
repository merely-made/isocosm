// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmograph::{Command, Execution, Founding};
use isocosm_campaign::{EntropyTape, GenValue, GeneratorRequest};
use isocosm_campaign::LowerDraftMap;
use isocosm_native::founding::Campaign;
use isocosm_system::{GeneratorLimits, GeneratorPack};
use std::{collections::BTreeMap, path::Path};

fn found_pack(pack_id: &str, generator: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs").join(pack_id);
    let pack = GeneratorPack::load(root).unwrap();
    let request = GeneratorRequest { generator: generator.into(),
        args: GenValue::Text { value: "river".into() }, locks: BTreeMap::new() };
    let mut tape = EntropyTape::from_seed(17);
    let limits = GeneratorLimits::default();
    let record = pack.generate("generated.native.1", &request, &mut tape, limits).unwrap();
    let record_before = record.clone();
    let draws = tape.draws.clone();
    let GenValue::Campaign { campaign: draft } = &record.proposal else { panic!("expected campaign"); };
    let genesis = Founding { seed: 17, sites: 2, population: 4, lineages: 2, cohort_size: 2,
        ..Default::default() }.generate().unwrap();
    let mut first = None;
    for mode in [Execution::Individuals, Execution::Grouped] {
        let prepared = Campaign::from_generation_bench(genesis.clone(), &record, limits.max_value_depth, mode).unwrap();
        let campaign = &prepared.campaign;
        assert_eq!(campaign.content().maps, draft.maps);
        assert_eq!(campaign.content().storylets, draft.world.storylets);
        assert_eq!(campaign.content().rewards, draft.rewards);
        assert_eq!(campaign.content().lowered_maps().unwrap(), draft.maps.iter().map(|m| m.lower().unwrap()).collect::<Vec<_>>());
        assert_eq!(campaign.reading().entries().len(), draft.world.assertions().len());
        for (entry, input) in campaign.reading().entries().iter().zip(draft.world.assertions()) {
            assert_eq!(entry.command, Command::Assert(input.clone()));
        }
        for (key, place) in draft.world.places() {
            assert_eq!(campaign.reading().authored_site(key).unwrap().1.authored.as_ref(), Some(place));
        }
        for (key, character) in draft.world.characters() {
            let reading = campaign.reading();
            let (_, entity) = reading.authored_entity(key).unwrap();
            assert_eq!(entity.authored.as_ref(), Some(character));
            assert_eq!(entity.place, cosmograph::directing::PLACELESS);
            assert!(entity.body.is_none());
        }
        for (key, law) in draft.world.laws() { assert_eq!(&campaign.reading().state().laws[key], law); }
        for history in draft.world.history() {
            let reading = campaign.reading();
            let event = &reading.state().events[&history.event_key()];
            assert_eq!(event.authored.as_ref(), Some(history));
            assert_eq!(event.strength, 0);
        }
        for route in draft.world.routes().values() {
            let reading = campaign.reading();
            let from = reading.authored_site(&route.from).unwrap().1;
            let native = from.routes.iter().find(|r| r.authored.as_ref() == Some(route)).unwrap();
            assert_eq!(native.travel, u64::from(route.weight));
            assert_eq!(native.transmission, 0);
        }
        for secret in &draft.secrets { assert_eq!(prepared.private.secret(&secret.id), Some(secret)); }
        assert_eq!(campaign.origin().unwrap().entropy, record.entropy);
        assert_eq!(campaign.origin().unwrap().request, record.request);
        let saved = campaign.save();
        if let Some(first) = &first { assert_eq!(&saved, first); }
        first = Some(saved.clone());
        let json = serde_json::to_vec(&saved).unwrap();
        let loaded = Campaign::load(serde_json::from_slice(&json).unwrap(), Execution::Grouped).unwrap();
        assert_eq!(loaded.save(), saved);
        for secret in &draft.secrets { assert!(!std::str::from_utf8(&json).unwrap().contains(&secret.text)); }
    }
    assert_eq!(record, record_before);
    assert_eq!(tape.draws, draws);
    assert_eq!(draws, vec![record.entropy]);
}

#[test]
fn native_demo_pack_founds_without_rerunning_or_changing_authored_content() {
    found_pack("demo", "demo:campaign");
}

#[test]
fn native_watchtower_pack_founds_without_rerunning_or_changing_authored_content() {
    found_pack("watchtower", "watchtower:ruined_tower");
}
