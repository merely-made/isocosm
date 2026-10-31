// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph_overlay::{AttentionChange, PlaceHandle};

#[test]
fn spatial_routes_keep_seat_filtering_and_native_close_custody() {
    let f = fixture();
    let site = body(&f, f.subjects[0]).place;
    let probe = Window::new(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let hidden = f.world.read(|s| *cosmograph::directing::readings::view(&s.sim, f.subjects[0],
        cosmograph::directing::readings::Mode::Creative).sites.difference(&probe.projection().unwrap().visible.sites)
        .next().expect("declared fixture supplies an unknown native site")).unwrap();
    let mut requested = binding("mesocosm");
    requested.attention.examined = Some(PlaceHandle(site));
    let mut player = Window::attach(&f.world, f.participants[0], Seat::Player, requested).unwrap();
    let mut requested = binding("vtt");
    requested.attention.examined = Some(PlaceHandle(hidden));
    let dm = Window::attach(&f.world, f.participants[0], Seat::Dm, requested).unwrap();
    let player_cut = player.projection().unwrap();
    let dm_cut = dm.projection().unwrap();
    let player_handle = player.handle().unwrap();
    let dm_handle = dm.handle().unwrap();
    let saved = f.world.save().unwrap();
    f.world.read(|s| {
        assert!(!crate::spatial::far(s, &player_cut, player_handle).unwrap().iter().any(|c| c.site == hidden));
        assert!(crate::spatial::region(s, &player_cut, player_handle, hidden).is_err());
        assert!(crate::spatial::region(s, &dm_cut, dm_handle, hidden).is_ok());
        assert!(crate::spatial::inspect(s, &player_cut, player_handle, site, f.subjects[0]).is_ok());
    }).unwrap();
    assert_eq!(f.world.save().unwrap(), saved);
    let changed = player.window_attention_request(AttentionChange::StopExamining).unwrap();
    player.apply_attention(changed).unwrap();
    f.world.read(|s| assert!(crate::spatial::inspect(s, &player_cut, player_handle, site, f.subjects[0]).is_err())).unwrap();
    player.close().unwrap();
    let saved = f.world.save().unwrap();
    f.world.read(|s| {
        assert!(crate::spatial::far(s, &player_cut, player_handle).is_err());
        assert!(crate::spatial::region(s, &dm_cut, dm_handle, hidden).is_ok());
    }).unwrap();
    assert_eq!(f.world.save().unwrap(), saved);
}

#[test]
fn actual_take_invalidates_an_old_spatial_cut_without_a_second_world() {
    let f = fixture();
    let native = Window::attach(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let old = native.projection().unwrap();
    let handle = native.handle().unwrap();
    assert_eq!(old.access.subject.unwrap().0, f.subjects[0]);
    f.world.command(Command::Take { participant: f.participants[0].0, critter: f.subjects[1] }).unwrap();
    assert_eq!(native.projection().unwrap().access.subject.unwrap().0, f.subjects[1]);
    let saved = f.world.save().unwrap();
    f.world.read(|s| assert!(crate::spatial::far(s, &old, handle).is_err(),
        "a real native Take invalidates the old spatial subject cut")).unwrap();
    let fresh = native.projection().unwrap();
    assert!(f.world.read(|s| crate::spatial::far(s, &fresh, handle)).unwrap().is_ok());
    assert_eq!(f.world.save().unwrap(), saved);
}
