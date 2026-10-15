// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{products::Mode, ui::model::{App, Preferences}, viewport::Registrations};
use cambium_rootstock::ProducerRegistry;

#[test]
fn real_mode_switch_and_close_retire_only_the_local_native_section() {
    let f = fixture();
    let mut app = App::new(f.world.share(), None);
    let first = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let second = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    assert!(app.windows[&first].scene.snapshot().is_none(), "opening a mode does not invent a close-up watch");
    let site = body(&f, f.subjects[0]).place;
    app.examine(first, Some(site)).unwrap(); app.examine(second, Some(site)).unwrap();
    let mut registry = ProducerRegistry::new();
    let mut owned = Registrations::default();
    assert!(owned.sync(&app, &mut registry).is_empty());
    let slots = [crate::viewport::key(first).unwrap(), crate::viewport::key(second).unwrap()];
    assert!(slots.into_iter().all(|slot| registry.contains(slot)));
    let before = app.world.save().unwrap();
    app.turn(first).unwrap();
    assert_ne!(app.windows[&first].scene.snapshot().unwrap().camera(1.0),
        app.windows[&second].scene.snapshot().unwrap().camera(1.0));
    assert_eq!(app.world.save().unwrap(), before, "turning a real Section changes no native command");
    let played = body(&f, f.subjects[0]);
    app.switch(first, Mode::Vtt).unwrap();
    assert!(app.windows[&first].scene.snapshot().is_none());
    assert!(owned.sync(&app, &mut registry).is_empty());
    assert!(!registry.contains(slots[0]), "a switched mode cannot retain its previous spatial image");
    assert!(registry.contains(slots[1]));
    assert_eq!(body(&f, f.subjects[0]), played);
    app.close(second).unwrap();
    assert!(owned.sync(&app, &mut registry).is_empty());
    assert!(!registry.contains(slots[1]), "closing withdraws only its own producer registration");
    assert!(app.windows.contains_key(&first));
}

#[test]
fn native_watch_withdrawal_removes_the_leaf_and_foreign_region_never_publishes_a_picture() {
    let f = fixture();
    let mut app = App::new(f.world.share(), None);
    let first = app.open(f.participants[0], Seat::Player, Mode::Mesocosm, Preferences::default(), None).unwrap();
    let site = body(&f, f.subjects[0]).place;
    app.examine(first, Some(site)).unwrap();
    let cut = app.windows[&first].native.projection().unwrap();
    let expected = app.world.read(|s| crate::spatial::region(s, &cut,
        app.windows[&first].native.handle().unwrap(), site)).unwrap().unwrap();
    assert_eq!(app.windows[&first].scene.snapshot().unwrap().body_ids(), expected.bodies.iter().map(|b|b.id).collect::<Vec<_>>());
    let hidden = app.world.read(|s| *s.sim.state().sites.keys().find(|id|
        !cut.visible.sites.contains(id)).unwrap()).unwrap();
    let saved = app.world.save().unwrap();
    assert!(app.examine(first, Some(hidden)).is_err());
    assert_eq!(app.world.save().unwrap(), saved);
    app.examine(first, None).unwrap();
    assert!(app.windows[&first].scene.snapshot().is_none());
    let mut registry = ProducerRegistry::new();
    let mut owned = Registrations::default();
    assert!(owned.sync(&app, &mut registry).is_empty());
    assert!(!registry.contains(crate::viewport::key(first).unwrap()));
}
