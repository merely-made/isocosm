//! Tests for this module, split out on 2026-07-24; unchanged.

use super::*;
use crate::asserted::Authored;
use crate::legacy::campaign::MapPoint;
use crate::legacy::campaign::MapProposalError;

#[test]
fn storylet_requires_private_fact_and_casts_existing_character() {
    let mut world = CampaignWorld::default();
    let faction = Faction {
        authored: Authored {
            key: "tide".into(),
            name: "Tide Court".into(),
            tags: ["river".into()].into(),
            claims: Default::default(),
        },
        ..Default::default()
    };
    world.assert(Assertion::Faction(faction)).unwrap();
    let mara = Character {
        key: "mara".into(),
        name: "Mara".into(),
        tags: ["warden".into()].into(),
        faction: Some("tide".into()),
        place: None,
    };
    world.assert(Assertion::Character(mara)).unwrap();
    let law = Law {
        key: "iron-remembers".into(),
        name: "Iron remembers".into(),
        text: "Iron keeps the name of its maker.".into(),
        tags: ["magic".into()].into(),
        parameters: BTreeMap::new(),
    };
    world.assert(Assertion::Law(law)).unwrap();
    let storylet = StoryletProposal {
        key: "sunken-vow".into(),
        entry: "The old oath surfaces.".into(),
        tags: vec![],
        requirements: StoryletRequirements {
            faction_tags: vec!["river".into()],
            hidden_facts: vec!["vow.secret".into()],
            world_laws: vec!["iron-remembers".into()],
        },
        roles: vec![RoleSlot {
            key: "warden".into(),
            tags: vec!["warden".into()],
        }],
        effects: vec![],
    };
    assert!(matches!(
        world.resolve_storylet(&storylet, []),
        Err(StoryletError::MissingHiddenFact(_))
    ));
    let resolved = world.resolve_storylet(&storylet, ["vow.secret"]).unwrap();
    assert_eq!(resolved.cast["warden"], "mara");
}

#[test]
fn campaign_draft_rejects_duplicate_map_ids_before_commit() {
    let mut world = CampaignWorld::default();
    world.storylets.insert(
        "finale".into(),
        StoryletProposal {
            key: "finale".into(),
            entry: "Finale".into(),
            tags: vec![],
            requirements: Default::default(),
            roles: vec![],
            effects: vec![],
        },
    );
    let map = LocalMapProposal {
        id: "same".into(),
        name: "Same".into(),
        width: 2,
        height: 2,
        default_ground: "grass".into(),
        cells: vec![],
        spawn_zones: vec![],
        transitions: vec![],
        encounter_anchors: vec![],
    };
    let draft = CampaignDraft {
        id: "draft".into(),
        name: "Draft".into(),
        world,
        maps: vec![
            DraftMap {
                scale: MapScale::Region,
                map: map.clone(),
                inhabitants: vec![],
            },
            DraftMap {
                scale: MapScale::Local,
                map,
                inhabitants: vec![],
            },
        ],
        secrets: vec![],
        rewards: vec![],
        starting_map: "same".into(),
        final_storylet: "finale".into(),
    };
    assert_eq!(
        draft.validate(),
        Err(WorldError::DuplicateMap("same".into()))
    );
}

#[test]
fn draft_map_validation_refuses_duplicate_inhabitants() {
    let map = DraftMap {
        scale: MapScale::Local,
        map: LocalMapProposal {
            id: "watchtower".into(),
            name: "Ruined Watchtower".into(),
            width: 2,
            height: 2,
            default_ground: "stone".into(),
            cells: vec![],
            spawn_zones: vec![],
            transitions: vec![],
            encounter_anchors: vec![],
        },
        inhabitants: vec![
            MapInhabitant {
                id: 4,
                name: "Warden".into(),
                sprite: "warden".into(),
                at: MapPoint { col: 0, row: 0 },
                system: "demo".into(),
                stats: BTreeMap::new(),
                owner: None,
            },
            MapInhabitant {
                id: 4,
                name: "Second Warden".into(),
                sprite: "warden".into(),
                at: MapPoint { col: 1, row: 1 },
                system: "demo".into(),
                stats: BTreeMap::new(),
                owner: None,
            },
        ],
    };

    assert_eq!(
        map.validate(),
        Err(MapProposalError::DuplicateInhabitantId(4))
    );
}

#[test]
fn storylet_effects_round_trip_over_the_binary_carrier() {
    let map = LocalMapProposal {
        id: "shore".into(),
        name: "Shore".into(),
        width: 2,
        height: 2,
        default_ground: "sand".into(),
        cells: vec![],
        spawn_zones: vec![],
        transitions: vec![],
        encounter_anchors: vec![],
    };
    let effects = vec![
        StoryletEffect::Fact {
            fact: WorldFact {
                id: "oath".into(),
                kind: "reveal".into(),
                text: "The oath is known.".into(),
                tags: vec!["river".into()],
            },
        },
        StoryletEffect::History {
            event: HistoryLine {
                key: "tide".into(),
                time: 1,
                kind: "tide".into(),
                text: "The tide turns.".into(),
                ..Default::default()
            },
        },
        StoryletEffect::Item {
            item: ItemProposal {
                template: "river-key".into(),
                name: "River Key".into(),
                tags: vec![],
            },
        },
        StoryletEffect::LocalMap { map },
    ];
    let bytes = postcard::to_allocvec(&effects).expect("encode storylet effects");
    assert_eq!(
        postcard::from_bytes::<Vec<StoryletEffect>>(&bytes).expect("decode storylet effects"),
        effects
    );
}

fn place(id: &str, name: &str) -> Assertion {
    Assertion::Place(Place {
        key: id.into(),
        name: name.into(),
        ..Default::default()
    })
}

fn route(id: &str, from: &str, to: &str, weight: u32) -> Assertion {
    Assertion::Route(Route {
        key: id.into(),
        from: from.into(),
        to: to.into(),
        tags: Default::default(),
        weight,
    })
}

#[test]
fn the_overmap_projects_from_places_and_routes() {
    let mut world = CampaignWorld::default();
    // The forest opens into a tactical map; the node carries it as its site.
    world.assert(place("village", "Village")).unwrap();
    let forest = Place {
        key: "forest".into(),
        name: "Forest".into(),
        map: Some("forest-map".into()),
        ..Default::default()
    };
    world.assert(Assertion::Place(forest)).unwrap();
    world.assert(place("ruins", "Ruins")).unwrap();
    world.assert(route("r1", "village", "forest", 2)).unwrap();
    world.assert(route("r2", "forest", "ruins", 3)).unwrap();
    // An unweighted route (weight 0) still costs 1 once projected.
    world.assert(route("r3", "village", "ruins", 0)).unwrap();

    let overmap = world.overmap();
    assert_eq!(overmap.nodes.len(), 3, "a node per place");
    assert_eq!(
        overmap.node("forest").and_then(|n| n.site.as_deref()),
        Some("forest-map"),
        "a place's tactical map becomes the node's site"
    );
    // The direct village->ruins route projects to cost 1 (weight 0 -> 1),
    // cheaper than through the forest (5). Pathfinding runs on the projection.
    let (path, cost) = overmap
        .route("village", "ruins")
        .expect("the ruins are reachable");
    assert_eq!(path, vec!["village", "ruins"]);
    assert_eq!(cost, 1, "an unweighted route projects to unit cost");
}

#[test]
fn a_party_sits_on_an_overmap_node_and_travels() {
    let mut world = CampaignWorld::default();
    world.assert(place("village", "Village")).unwrap();
    world.assert(place("forest", "Forest")).unwrap();
    world.assert(route("r1", "village", "forest", 2)).unwrap();

    assert_eq!(world.party_at("A"), None, "the party starts off the map");
    world
        .apply(&WorldEvent::PartyMoved {
            party: "A".into(),
            node: "village".into(),
        })
        .unwrap();
    assert_eq!(world.party_at("A"), Some("village"));
    // The projected overmap says the forest is reachable, so travel there.
    assert!(world.overmap().route("village", "forest").is_some());
    world
        .apply(&WorldEvent::PartyMoved {
            party: "A".into(),
            node: "forest".into(),
        })
        .unwrap();
    assert_eq!(
        world.party_at("A"),
        Some("forest"),
        "the party travelled the edge"
    );
}

#[test]
fn pace_scales_the_travel_cost() {
    let mut world = CampaignWorld::default();
    world.assert(place("village", "Village")).unwrap();
    world.assert(place("forest", "Forest")).unwrap();
    world.assert(route("r1", "village", "forest", 4)).unwrap();

    // Default pace is normal (100%): the cost is the route's weight.
    assert_eq!(world.pace("A"), 100);
    assert_eq!(world.travel_cost("A", "village", "forest"), Some(4));

    // Fast (50%) halves the time; slow (200%) doubles it. Same edge, same
    // party, different ticks.
    world
        .apply(&WorldEvent::PartyPaceSet {
            party: "A".into(),
            pace: 50,
        })
        .unwrap();
    assert_eq!(
        world.travel_cost("A", "village", "forest"),
        Some(2),
        "fast is half the time"
    );
    world
        .apply(&WorldEvent::PartyPaceSet {
            party: "A".into(),
            pace: 200,
        })
        .unwrap();
    assert_eq!(
        world.travel_cost("A", "village", "forest"),
        Some(8),
        "slow is double"
    );

    // A cost never rounds to zero, and an unreachable destination has none.
    assert_eq!(world.travel_cost("A", "village", "atlantis"), None);
}

#[test]
fn a_party_discovers_the_overmap_as_it_travels() {
    let mut world = CampaignWorld::default();
    for id in ["village", "forest", "ruins", "island"] {
        world.assert(place(id, id)).unwrap();
    }
    world.assert(route("r1", "village", "forest", 2)).unwrap();
    world.assert(route("r2", "forest", "ruins", 2)).unwrap();
    // The island has no route to it.

    // A party that knows nothing sees an empty overmap.
    assert!(
        world.overmap_for("A").nodes.is_empty(),
        "the unfound map is dark"
    );
    assert!(!world.knows("A", "village"));

    // Arriving at the village discovers it and its neighbour (the forest),
    // but not what is two steps on (the ruins).
    world
        .apply(&WorldEvent::PartyMoved {
            party: "A".into(),
            node: "village".into(),
        })
        .unwrap();
    assert!(world.knows("A", "village"));
    assert!(world.knows("A", "forest"), "and one step on");
    assert!(!world.knows("A", "ruins"), "but not two steps on");
    // The known overmap shows only what has been found, and refuses to route
    // through the dark.
    let known = world.overmap_for("A");
    assert_eq!(known.nodes.len(), 2);
    assert!(
        known.route("village", "ruins").is_none(),
        "cannot plot a course into the unknown"
    );

    // Travel on to the forest, and the ruins come into view.
    world
        .apply(&WorldEvent::PartyMoved {
            party: "A".into(),
            node: "forest".into(),
        })
        .unwrap();
    assert!(
        world.knows("A", "ruins"),
        "arriving at the forest reveals the ruins"
    );

    // A rumour reveals the island directly, though no road leads there.
    world
        .apply(&WorldEvent::NodeRevealed {
            party: "A".into(),
            node: "island".into(),
        })
        .unwrap();
    assert!(
        world.knows("A", "island"),
        "word of mouth reaches the unreachable"
    );
}
