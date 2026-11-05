// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Durable-session controls independent of the native revision token API.

use super::*;
use cosmograph::history::LifeHistory;

#[test]
fn a_native_only_life_history_change_is_published() {
    let world = crate::fixtures::fixture().world;
    let before = world.cut().unwrap();
    let revision = world.revision().unwrap();
    assert_eq!(
        world.save().unwrap().native.life_history,
        Some(LifeHistory::WorldTime)
    );
    world
        .product_current(|session| {
            session.life_history = LifeHistory::Ticks;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        world.save().unwrap().native.life_history,
        Some(LifeHistory::Ticks)
    );
    assert_eq!(world.revision().unwrap(), revision + 1);
    assert_eq!(
        world.cut().unwrap(),
        before,
        "saved metadata need not change the native cut"
    );
}

#[test]
fn life_history_and_product_sections_publish_together() {
    let world = crate::fixtures::fixture().world;
    let before = world.cut().unwrap();
    let revision = world.revision().unwrap();
    world
        .product_sections(before, |session, sections| {
            session.life_history = LifeHistory::Ticks;
            let mut sections = sections.clone();
            sections.insert("authority-control".into(), vec![0, 255]);
            Ok(((), sections))
        })
        .unwrap();
    let saved = world.save().unwrap();
    assert_eq!(saved.native.life_history, Some(LifeHistory::Ticks));
    assert_eq!(saved.sections["authority-control"], vec![0, 255]);
    assert_eq!(world.revision().unwrap(), revision + 1);
    for mode in [Execution::Individuals, Execution::Grouped] {
        assert_eq!(
            World::resume(saved.clone(), mode).unwrap().save().unwrap(),
            saved
        );
    }
}
