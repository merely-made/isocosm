// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn close_up(site: Id, pinned: &[Id]) -> WindowAttention {
    WindowAttention {
        pinned: pinned.iter().copied().map(Target::Entity).collect(),
        examined: Some(Region::Site(site)),
    }
}

fn care(profile: &str) -> BTreeSet<Target> {
    match profile {
        "lineage" => BTreeSet::from([Target::Lineage("one".into())]),
        "known" => BTreeSet::from([Target::Entity(41)]),
        _ => BTreeSet::new(),
    }
}

#[test]
fn two_active_windows_union_pins_regions_and_native_profile_care() {
    let mut windows = Windows::default();
    windows.attach(10, "lineage".into(), close_up(1, &[21])).unwrap();
    windows.attach(10, "known".into(), close_up(2, &[22])).unwrap();
    windows.attach(11, "known".into(), close_up(3, &[23])).unwrap();
    let set = windows.for_player(10, [20], care);
    assert_eq!(set.played, BTreeSet::from([20]));
    assert_eq!(set.pinned, BTreeSet::from([Target::Entity(21), Target::Entity(22)]),
        "both active same-player windows must contribute their pins");
    assert_eq!(set.examined, BTreeSet::from([Region::Site(1), Region::Site(2)]),
        "both active same-player close-up regions must remain examined");
    assert_eq!(set.care, BTreeSet::from([Target::Lineage("one".into()), Target::Entity(41)]));
    assert!(!set.pinned.contains(&Target::Entity(23)));
}

#[test]
fn closing_one_window_withdraws_only_its_contribution() {
    let mut windows = Windows::default();
    let first = windows.attach(10, "lineage".into(), close_up(1, &[21, 22])).unwrap();
    let second = windows.attach(10, "known".into(), close_up(2, &[22, 23])).unwrap();
    windows.close(10, first).unwrap();
    let set = windows.for_player(10, [20], care);
    assert_eq!(set.pinned, BTreeSet::from([Target::Entity(22), Target::Entity(23)]),
        "closing one window must preserve the other window's shared and unique pins");
    assert_eq!(set.examined, BTreeSet::from([Region::Site(2)]));
    assert_eq!(set.care, BTreeSet::from([Target::Entity(41)]));
    windows.close(10, second).unwrap();
    let set = windows.for_player(10, [20], care);
    assert!(set.pinned.is_empty() && set.examined.is_empty() && set.care.is_empty());
    assert_eq!(set.played, BTreeSet::from([20]), "closing a window cannot change native play");
}

#[test]
fn replacing_one_mode_preserves_the_other_window_and_native_play() {
    let mut windows = Windows::default();
    let first = windows.attach(10, "lineage".into(), close_up(1, &[21])).unwrap();
    windows.attach(10, "known".into(), close_up(2, &[22])).unwrap();
    windows.replace(10, first, "known".into(), close_up(3, &[23])).unwrap();
    let set = windows.for_player(10, [20], care);
    assert_eq!(set.played, BTreeSet::from([20]));
    assert_eq!(set.pinned, BTreeSet::from([Target::Entity(22), Target::Entity(23)]));
    assert_eq!(set.examined, BTreeSet::from([Region::Site(2), Region::Site(3)]),
        "a mode replacement must leave the other window's examined region intact");
}

#[test]
fn stale_or_foreign_window_requests_refuse_without_mutation() {
    let mut windows = Windows::default();
    let first = windows.attach(10, "lineage".into(), close_up(1, &[21])).unwrap();
    let before = windows.clone();
    assert_eq!(windows.resolve(10, first.id()).unwrap(), first);
    assert!(windows.resolve(11, first.id()).is_err());
    assert!(windows.replace(11, first, "known".into(), close_up(2, &[22])).is_err());
    assert_eq!(windows, before);
    assert!(windows.close(11, first).is_err());
    assert_eq!(windows, before);
    windows.close(10, first).unwrap();
    let closed = windows.clone();
    assert!(windows.resolve(10, first.id()).is_err());
    assert!(windows.close(10, first).is_err());
    assert!(windows.replace(10, first, "known".into(), close_up(2, &[22])).is_err());
    assert_eq!(windows, closed);
    let next = windows.attach(10, "known".into(), close_up(2, &[22])).unwrap();
    assert_ne!(next, first, "a closed native window identity is never reused");
}

#[test]
fn unchanged_region_and_pins_are_an_unchanged_contribution() {
    let mut windows = Windows::default();
    let attention = close_up(1, &[21]);
    let first = windows.attach(10, "lineage".into(), attention.clone()).unwrap();
    let before = windows.clone();
    assert!(!windows.replace(10, first, "lineage".into(), attention).unwrap());
    assert_eq!(windows, before, "the driver must not log an unchanged region for a camera move");
}

#[test]
fn native_profile_care_is_derived_from_current_facts_each_read() {
    let mut windows = Windows::default();
    windows.attach(10, "known".into(), close_up(1, &[21])).unwrap();
    let before = windows.clone();
    let first = windows.for_player(10, [20], |_| BTreeSet::from([Target::Entity(41)]));
    let next = windows.for_player(10, [20], |_| BTreeSet::from([Target::Entity(42)]));
    assert_ne!(first.care, next.care);
    assert_eq!(first.pinned, next.pinned);
    assert_eq!(windows, before, "derived care is not a second saved caller-authored ledger");
}

#[test]
fn active_window_union_roundtrips_as_native_values() {
    let mut windows = Windows::default();
    windows.attach(10, "lineage".into(), close_up(1, &[21])).unwrap();
    windows.attach(10, "known".into(), close_up(2, &[22])).unwrap();
    let bytes = serde_json::to_vec(&windows).unwrap();
    let restored: Windows = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, windows);
    assert_eq!(restored.for_player(10, [20], care), windows.for_player(10, [20], care));
    // A current value roundtrip is not a native Session replay or an old-save receipt.
}
