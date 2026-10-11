//! Atomic character authoring receipts.

use super::*;

fn created() -> GameEvent {
    GameEvent::CharacterCreated {
        token: Token {
            id: TokenId(40),
            at: (3, 3),
            facing: Facing::South,
            sprite: "hero".to_owned(),
            owner: Some("Mira".to_owned()),
        },
        sheet: sheet("Mira", 12, 14),
    }
}

#[test]
fn character_creation_replays_as_one_complete_token_and_sheet() {
    let mut sim = Sim::new(HostSession::new(snapshot()));
    sim.connect(PeerId(10));

    sim.host_event(created());

    for state in std::iter::once(sim.host.state()).chain(
        sim.clients
            .values()
            .map(|client| client.state().expect("connected client has a snapshot")),
    ) {
        let token = state.map.token(TokenId(40)).expect("character token");
        assert_eq!(token.at, (3, 3));
        assert_eq!(token.owner.as_deref(), Some("Mira"));
        assert_eq!(
            state.map.sheet(TokenId(40)).and_then(|s| s.text("name")),
            Some("Mira")
        );
    }
    assert_eq!(
        sim.host.history().len(),
        1,
        "one replay record owns both halves"
    );
    assert_converged(&sim);
}

#[test]
fn refused_character_placement_does_not_attach_an_orphan_sheet() {
    let mut state = snapshot();
    let before = state.clone();
    let mut event = created();
    let GameEvent::CharacterCreated { token, .. } = &mut event else {
        unreachable!()
    };
    token.at = (-1, 0);
    assert!(isonetry::apply_game(&mut state, &event).is_err());
    assert_eq!(state, before);
}

/// The DM's character through the contract (795): every peer folds the
/// native entry it was lowered to (768).
#[test]
fn a_contract_character_replicates_as_its_native_entry() {
    use isocosm_overlay::vtt::{Assertion as Contract, Cell, NewCharacter};
    use isocosm_overlay::{ParticipantHandle, PlaceHandle, WorldPoint};
    let mut snap = snapshot();
    let ford = Place {
        key: "ford".into(),
        name: "Ford".into(),
        ..Default::default()
    };
    snap.world.assert(Assertion::Place(ford)).unwrap();
    let original_map = snap.map.clone();
    let mut sim = Sim::new(HostSession::new(snap));
    sim.connect(PeerId(10));
    let site = PlaceHandle(isometry_campaign::contract::handle("ford"));
    let mira = Contract::Character(NewCharacter {
        key: "mira".into(),
        name: "Mira".into(),
        faction: None,
        at: Cell {
            site,
            at: WorldPoint([-7, 2, 13]),
        },
        owner: Some(ParticipantHandle(10)),
    });
    sim.host_assertion(&mira).expect("the character commits");
    sim.connect(PeerId(11)); // The late snapshot keeps metadata too.
    let expected = Some(isocosm::asserted::CharacterTable {
        cell: [-7, 2, 13],
        owner: Some(10),
    });
    for state in std::iter::once(sim.host.state())
        .chain(sim.clients.values().map(|client| client.state().unwrap()))
    {
        let character = &state.world.characters()["mira"];
        assert_eq!(character.place.as_deref(), Some("ford"));
        assert_eq!(character.table, expected);
        assert_eq!(
            state.map, original_map,
            "authoring neither moves nor takes a table token"
        );
        let bytes = postcard::to_allocvec(state).unwrap();
        let wired: GameSnapshot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(wired, *state, "replicated snapshot bytes retain metadata");
        let json = serde_json::to_vec(state).unwrap();
        let saved: GameSnapshot = serde_json::from_slice(&json).unwrap();
        assert_eq!(saved, *state, "saved table state retains metadata");
    }
    let elsewhere = Contract::Character(NewCharacter {
        key: "elian".into(),
        name: "Elian".into(),
        faction: None,
        at: Cell {
            site: PlaceHandle(1),
            at: WorldPoint([0, 0, 0]),
        },
        owner: None,
    });
    assert!(
        sim.host_assertion(&elsewhere).is_err(),
        "an unnamed place is refused"
    );
    assert_converged(&sim);
}

#[test]
fn a_contract_character_announcement_retains_metadata_through_wire_delivery() {
    use isocosm_overlay::vtt::{Assertion as Contract, Cell, NewCharacter};
    use isocosm_overlay::{ParticipantHandle, PlaceHandle, WorldPoint};
    let mut snap = snapshot();
    snap.world
        .assert(Assertion::Place(Place {
            key: "ford".into(),
            ..Default::default()
        }))
        .unwrap();
    let mut host = HostSession::new(snap);
    let mut client = ClientSession::new();
    for (_, message) in host.on_connect(PeerId(10)) {
        let bytes = postcard::to_allocvec(&message).unwrap();
        let wired: NetMessage = postcard::from_bytes(&bytes).unwrap();
        assert!(client.on_message(wired).is_empty());
    }
    let assertion = Contract::Character(NewCharacter {
        key: "mira".into(),
        name: "Mira".into(),
        faction: None,
        at: Cell {
            site: PlaceHandle(isometry_campaign::contract::handle("ford")),
            at: WorldPoint([-7, 2, 13]),
        },
        owner: Some(ParticipantHandle(10)),
    });
    let out = host.commit_assertion(&assertion).unwrap();
    assert!(!out.is_empty(), "the accepted entry is actually broadcast");
    for (recipient, message) in out {
        assert_eq!(recipient, Recipient::All);
        let bytes = postcard::to_allocvec(&message).unwrap();
        let wired: NetMessage = postcard::from_bytes(&bytes).unwrap();
        assert!(client.on_message(wired).is_empty());
    }
    assert_eq!(client.state(), Some(host.state()));
    assert_eq!(client.log_hash(), host.log_hash());
    assert_eq!(client.applied(), host.seq());
    assert_eq!(
        client.state().unwrap().world.characters()["mira"].table,
        Some(isocosm::asserted::CharacterTable {
            cell: [-7, 2, 13],
            owner: Some(10)
        })
    );
}
