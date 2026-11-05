// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cambium::GenetAppRunner;
use cosmograph::schema::PartId;
use genet_scripted_dom::ScriptedDom;
use isometer::mesh::BodyDependencyRevision;
use std::{cell::RefCell, rc::Rc};

fn state() -> State {
    State::new(super::super::super::HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        dev: true,
        ..Default::default()
    })
}

/// Uses the production keyed aside and logical inspector key. Focusable
/// descendants let the pinned runner expose the actual retained DOM identities
/// without a second diff engine or a layout/selector test dependency.
fn retained_panels(state: &State) -> Child {
    let row = |label: String| -> Child { Box::new(focusable(el("button", text(label)))) };
    let mut panels = vec![
        (PanelKey::Map, row("map".into())),
        (PanelKey::Vitals, row("vitals".into())),
    ];
    if state.host.config.dev {
        let inspector = (
            inspector_key(state),
            row(format!("inspector {}", state.tick_reading)),
        );
        if state.host.inspection.open {
            panels.insert(0, inspector);
        } else {
            panels.push(inspector);
        }
    }
    aside(panels)
}

#[test]
fn keyed_panel_reorder_retains_dom_focus_but_subject_source_or_unmount_restarts() {
    let mut state = state();
    state.tick_reading = 1;
    state.source_lifetime = 1;
    state.host.inspection.selected = Some(crate::section::BodySelection {
        organism: 7,
        part: PartId(2),
        revision: BodyDependencyRevision(1),
    });
    let mut runner = GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        retained_panels as Logic,
        state,
    );
    let before = runner.focusables();
    assert_eq!(before.len(), 3);
    runner.set_focus(Some(before[2]));
    runner.update(|state| {
        state.host.inspection.open = true;
        state.tick_reading = 2;
        state.host.inspection.selected.as_mut().unwrap().revision = BodyDependencyRevision(2);
    });
    assert_eq!(
        runner.focusables(),
        vec![before[2], before[0], before[1]],
        "same logical panels move their existing DOM nodes"
    );
    assert_eq!(runner.focus(), Some(before[2]));
    assert!(
        runner
            .dom()
            .borrow()
            .outer_html(before[2])
            .contains("inspector 2"),
        "retention must still rebuild the current values"
    );

    runner.update(|state| state.host.inspection.selected.as_mut().unwrap().part = PartId(3));
    let changed_part = runner.focusables();
    assert_ne!(
        changed_part[0], before[2],
        "another native part begins a new panel lifetime"
    );
    assert_eq!(&changed_part[1..], &before[..2]);
    runner.update(|state| state.host.inspection.selected.as_mut().unwrap().organism = 8);
    let changed_subject = runner.focusables();
    assert_ne!(
        changed_subject[0], changed_part[0],
        "another subject cannot inherit that panel"
    );
    assert_eq!(&changed_subject[1..], &before[..2]);

    runner.set_focus(Some(changed_subject[0]));
    runner.update(|state| state.source_lifetime += 1);
    let changed_source = runner.focusables();
    assert_ne!(
        changed_source[0], changed_subject[0],
        "a fresh world cannot inherit the same subject/part DOM lifetime"
    );
    assert_ne!(
        runner.focus(),
        Some(changed_subject[0]),
        "the previous source's focus route ended"
    );
    assert_eq!(&changed_source[1..], &before[..2]);

    runner.update(|state| state.host.config.dev = false);
    assert_eq!(runner.focusables(), before[..2]);
    runner.update(|state| state.host.config.dev = true);
    let remounted = runner.focusables();
    assert_eq!(&remounted[1..], &before[..2]);
    assert_ne!(
        remounted[0], changed_source[0],
        "true unmount does not retain a dead DOM lifetime"
    );
}

#[test]
fn root_build_uses_cached_tick_and_review_without_borrowing_native_world() {
    let mut state = state();
    state.tick_reading = 123_456;
    state.review_open = true;
    let world = state.host.runtime.world().share();
    let before = world.cut().unwrap();
    // The exclusive publication borrow makes any accidental runtime/native
    // read from view construction fail. The callback changes no native data.
    let html = world
        .product(before, |_| {
            let runner = GenetAppRunner::new(
                Rc::new(RefCell::new(ScriptedDom::new())),
                root as Logic,
                state,
            );
            Ok(runner.dom().borrow().outer_html(runner.root()))
        })
        .unwrap();
    assert!(html.contains("tick 123456"));
    assert!(html.contains("Next offer"));
    assert_eq!(
        world.cut().unwrap(),
        before,
        "DOM construction cannot change native authority"
    );
}
