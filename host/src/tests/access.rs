// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::directing::readings::{view, Mode};

#[test]
fn product_mode_does_not_widen_a_players_native_view() {
    let f = fixture();
    let first = window(&f, 0, Seat::Player, "mesocosm");
    let other_mode = window(&f, 0, Seat::Player, "vtt");
    let expected = f.world.read(|session| view(&session.sim, f.subjects[0], Mode::Survival)).unwrap();
    let truth = f.world.read(|session| view(&session.sim, f.subjects[0], Mode::Creative)).unwrap();
    assert!(expected.sites.len() < truth.sites.len(), "fixture must contain a hidden site");
    assert_eq!(first.projection().unwrap().visible, expected,
        "a player seat must use native survival visibility");
    assert_eq!(other_mode.projection().unwrap().visible, expected,
        "another product mode cannot widen the player's native visibility");
    assert_eq!(first.projection().unwrap().access, other_mode.projection().unwrap().access);
}

#[test]
fn creative_and_dm_seats_read_truth_without_command_authority() {
    let f = fixture();
    let expected = f.world.read(|session| view(&session.sim, f.subjects[0], Mode::Creative)).unwrap();
    let before = f.world.save().unwrap();
    for seat in [Seat::CreativeObserver, Seat::Dm] {
        let observer = window(&f, 0, seat, "mesocosm");
        assert_eq!(observer.projection().unwrap().visible, expected);
        assert_eq!(f.world.save().unwrap(), before, "reading a seat cannot submit native commands");
    }
}

#[test]
fn player_without_a_played_subject_has_an_empty_view() {
    let f = fixture();
    let joined = f.world.command(Command::Join).unwrap();
    let participant = ParticipantHandle(joined.strip_prefix("participant:").unwrap().parse().unwrap());
    let empty = Window::new(&f.world, participant, Seat::Player, binding("mesocosm")).unwrap();
    let projected = empty.projection().unwrap();
    assert!(projected.access.subject.is_none());
    assert_eq!(projected.visible, Default::default());
    assert!(projected.attention.played.is_empty());
}

#[test]
fn a_non_participant_cannot_supply_a_window_access_identity() {
    let f = fixture();
    assert!(Window::new(&f.world, ParticipantHandle(f.subjects[0]), Seat::Dm, binding("vtt")).is_err());
}

#[test]
fn ordinary_access_round_trips_without_importing_a_product_mode() {
    let f = fixture();
    let projected = window(&f, 0, Seat::Player, "mesocosm").projection().unwrap();
    let bytes = serde_json::to_vec(&projected.access).unwrap();
    let restored: isocosm_overlay::OrdinaryAccess = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, projected.access);
    let fields: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(fields.get("mode").is_none());
}
