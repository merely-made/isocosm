// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;
use std::path::Path;

use isocosm::asserted::Assertion;
use isometry_campaign::{
    CampaignDraft, EntropyTape, GenValue, GeneratorRequest, StoryletEffect,
};
use isometry_system::{GeneratorLimits, GeneratorPack};

fn generate(pack: &str, generator: &str, seed: u64) -> CampaignDraft {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/packs")
        .join(pack);
    let pack = GeneratorPack::load(root).unwrap();
    let request = GeneratorRequest {
        generator: generator.to_owned(),
        args: GenValue::Text {
            value: "authored fixture".to_owned(),
        },
        locks: BTreeMap::new(),
    };
    let mut tape = EntropyTape::from_seed(seed);
    let record = pack
        .generate(
            "authored.fixture",
            &request,
            &mut tape,
            GeneratorLimits::default(),
        )
        .unwrap();
    assert_eq!(tape.draws.len(), 1);
    let GenValue::Campaign { campaign } = record.proposal else {
        panic!("campaign draft");
    };
    campaign.validate().unwrap();
    campaign
}

fn effect_key(effect: &StoryletEffect) -> &str {
    match effect {
        StoryletEffect::Fact { fact } => &fact.id,
        StoryletEffect::History { event } => &event.key,
        _ => panic!("authored fixture outcome"),
    }
}

#[test]
fn demo_pack_carries_authored_keys_maps_and_outcome() {
    let campaign = generate("demo", "demo:campaign", 17);
    let world = &campaign.world;
    let keys: Vec<_> = world.assertions().iter().map(Assertion::key).collect();
    assert_eq!(
        keys,
        [
            "ash-choir",
            "tide-court",
            "ford",
            "glass-shrine",
            "river-region",
            "mara",
            "ford-shrine",
            "region-ford",
            "iron-remembers",
            "choir-arrives",
            "oath-broken"
        ]
    );
    for (key, map) in [
        ("ford", "river-ford"),
        ("glass-shrine", "glass-shrine-map"),
        ("river-region", "river-region"),
    ] {
        assert_eq!(world.places()[key].map.as_deref(), Some(map));
    }
    assert_eq!(
        world.characters()["mara"].faction.as_deref(),
        Some("tide-court")
    );
    assert_eq!(world.characters()["mara"].place.as_deref(), Some("ford"));
    assert_eq!(
        world
            .overmap()
            .route("river-region", "glass-shrine")
            .unwrap()
            .1,
        2
    );
    assert!(world.routes().values().all(|route| route.weight == 0));
    assert_eq!(world.history()[0].key, "choir-arrives");
    assert_eq!(world.history()[0].time, -8);
    assert_eq!(world.history()[1].key, "oath-broken");
    assert_eq!(world.history()[1].time, -3);
    let storylet = &world.storylets[&campaign.final_storylet];
    assert!(world.resolve_storylet(storylet, []).is_err());
    let resolution = world
        .resolve_storylet(storylet, ["witness-blade.secret"])
        .unwrap();
    assert_eq!(resolution.cast["warden"], "mara");
    assert_eq!(
        effect_key(&resolution.effects[0]),
        "river-oath.restored",
        "the demo outcome retains its public fact ID"
    );
}

#[test]
fn watchtower_pack_carries_authored_sites_people_and_outcomes() {
    let campaign = generate("watchtower", "watchtower:ruined_tower", 91);
    let world = &campaign.world;
    let keys: Vec<_> = world.assertions().iter().map(Assertion::key).collect();
    assert_eq!(
        keys,
        [
            "bellwardens",
            "road-scavengers",
            "watchtower:forest-entry",
            "watchtower:forest-region",
            "watchtower:ruined-watchtower",
            "watchtower:stream-clearing",
            "brin",
            "elian",
            "mira",
            "otta",
            "forest-entry-region",
            "stream-region",
            "tower-region",
            "bell-cracked",
            "scavengers-arrived"
        ]
    );
    for (key, position) in [
        ("watchtower:forest-entry", (0, 4)),
        ("watchtower:forest-region", (5, 4)),
        ("watchtower:ruined-watchtower", (5, 1)),
        ("watchtower:stream-clearing", (10, 4)),
    ] {
        let place = &world.places()[key];
        assert_eq!(
            place.map.as_deref(),
            Some(key),
            "site {key} keeps its authored map link"
        );
        assert_eq!(place.position, Some(position));
    }
    assert_eq!(
        world.characters()["brin"].faction.as_deref(),
        Some("road-scavengers")
    );
    assert_eq!(
        world.characters()["mira"].place.as_deref(),
        Some("watchtower:ruined-watchtower")
    );
    assert_eq!(world.history()[0].participants, ["bellwardens"]);
    assert_eq!(
        world.history()[1].participants,
        ["brin", "otta", "road-scavengers"]
    );
    assert!(world.routes().values().all(|route| route.weight == 0));
    assert_eq!(
        world
            .overmap()
            .route("watchtower:forest-entry", "watchtower:stream-clearing")
            .unwrap()
            .1,
        2
    );
    let parley = world
        .resolve_storylet(&world.storylets["parley-at-the-breach"], [])
        .unwrap();
    assert_eq!(parley.cast["speaker"], "mira");
    assert_eq!(parley.cast["scavenger"], "brin");
    assert_eq!(effect_key(&parley.effects[0]), "watchtower:parley");
    assert_eq!(
        effect_key(&world.storylets["steel-in-the-stones"].effects[0]),
        "watchtower:stone-fight",
        "the tower fight retains its authored history key"
    );
    assert_eq!(
        effect_key(&world.storylets["the-last-bell"].effects[0]),
        "watchtower:last-bell"
    );
}
