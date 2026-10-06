use std::collections::BTreeMap;

use isocosm::legacy::campaign::{MapCellProposal, MapInhabitant, MapPoint};

use super::*;

#[test]
fn sparse_proposal_lowers_to_playable_document_and_metadata() {
    let proposal = LocalMapProposal {
        id: "demo:river-cache".to_owned(),
        name: "River Cache".to_owned(),
        width: 4,
        height: 3,
        default_ground: "grass".to_owned(),
        cells: vec![MapCellProposal {
            col: 2,
            row: 1,
            ground: Some("stone".to_owned()),
            prop: Some("tree".to_owned()),
            elevation: Some(2),
        }],
        spawn_zones: vec![SpawnZone {
            id: "party".to_owned(),
            cells: vec![MapPoint { col: 0, row: 1 }],
        }],
        transitions: Vec::new(),
        encounter_anchors: vec![EncounterAnchor {
            id: "guardian".to_owned(),
            at: MapPoint { col: 3, row: 1 },
            tags: vec!["undead".to_owned()],
        }],
    };
    let map = proposal.lower(MapScale::Local).unwrap();
    assert_eq!(map.document.ground.width(), 4);
    assert_eq!(map.document.elevation.get(2, 1), Some(&2));
    assert_eq!(map.spawn_zones[0].id, "party");
    assert_eq!(map.encounter_anchors[0].tags, vec!["undead"]);
}

fn watchtower(width: u32, height: u32) -> LocalMapProposal {
    LocalMapProposal {
        id: "watchtower".into(),
        name: "Ruined Watchtower".into(),
        width,
        height,
        default_ground: "stone".into(),
        cells: vec![],
        spawn_zones: vec![],
        transitions: vec![],
        encounter_anchors: vec![],
    }
}

fn inhabitant(id: u32, name: &str, at: MapPoint, stats: BTreeMap<String, i64>) -> MapInhabitant {
    MapInhabitant {
        id,
        name: name.into(),
        sprite: "warden".into(),
        at,
        system: "demo".into(),
        stats,
        owner: None,
    }
}

#[test]
fn draft_map_lowers_public_inhabitants_into_tokens_and_sheets() {
    let mut stats = BTreeMap::new();
    stats.insert("hp_current".into(), 12);
    stats.insert("ac".into(), 14);
    let mut warden = inhabitant(7, "Tower Warden", MapPoint { col: 2, row: 1 }, stats);
    warden.owner = Some("tower".into());
    let map = DraftMap {
        scale: MapScale::Local,
        map: watchtower(3, 2),
        inhabitants: vec![warden],
    };

    let lowered = map.lower().unwrap();
    assert_eq!(lowered.document.tokens.len(), 1);
    let token = lowered.document.token(TokenId(7)).unwrap();
    assert_eq!(token.at, (2, 1));
    assert_eq!(token.owner.as_deref(), Some("tower"));
    let sheet = lowered.document.sheet(TokenId(7)).unwrap();
    assert_eq!(sheet.system, "demo");
    assert_eq!(sheet.text("name"), Some("Tower Warden"));
    assert_eq!(sheet.int("hp_current"), Some(12));
    assert_eq!(sheet.int("ac"), Some(14));
}

#[test]
fn draft_map_refuses_inhabitants_before_exposing_a_partial_document() {
    let map = DraftMap {
        scale: MapScale::Local,
        map: watchtower(2, 2),
        inhabitants: vec![
            inhabitant(4, "Warden", MapPoint { col: 0, row: 0 }, BTreeMap::new()),
            inhabitant(
                4,
                "Second Warden",
                MapPoint { col: 1, row: 1 },
                BTreeMap::new(),
            ),
        ],
    };

    assert_eq!(map.lower(), Err(MapProposalError::DuplicateInhabitantId(4)));
}
