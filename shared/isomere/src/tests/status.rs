// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! M3: the status lines, the help line, and the keymap the help line comes
//! from, as tests.

use super::*;
use crate::status::{
    Binding, HELP_CLASS, Keymap, STATUS_LINE_CLASS, STATUS_LINES_CLASS, StatusPanel, help_attrs,
    help_line, status_attrs, status_panel,
};

/// The session's press vocabulary as its handler lowers it: a character key
/// with its Ctrl state, or a named key. A pinned fixture standing in for the
/// product's own type, which is the point of the generic.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Press {
    Char(&'static str, bool),
    Named(&'static str),
}

/// What a press asks the session to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    Move(usize),
    Aim(usize),
    Charge,
    Take,
    Save,
    Close,
}

/// The session's keymap as it stands on 2026-09-15, in the order its help line
/// reads. The whole point of M3: this list and the handler's `match` are one
/// declaration, so the sentence below cannot outlive a binding.
fn session_keymap() -> Keymap<'static, Press, Command> {
    Keymap::new(vec![
        Binding::group(
            "WASD",
            "move",
            vec![
                (Press::Char("w", false), Command::Move(0)),
                (Press::Char("s", false), Command::Move(1)),
            ],
        ),
        Binding::group(
            "arrows",
            "aim",
            vec![(Press::Named("ArrowUp"), Command::Aim(0))],
        ),
        Binding::one(
            "Space",
            "charge, Space again to strike",
            Press::Named("Space"),
            Command::Charge,
        ),
        Binding::said("left mouse hold on the scene", "to charge"),
        Binding::one("E", "take", Press::Char("e", false), Command::Take),
        Binding::one("Ctrl+S", "save", Press::Char("s", true), Command::Save),
        Binding::one("Esc", "close", Press::Named("Escape"), Command::Close),
    ])
}

/// Every entry reads as its keys then its words, joined by the separator both
/// products already wrote their help line with.
#[test]
fn the_help_line_is_the_keymaps_own_words() {
    assert_eq!(
        session_keymap().help(),
        "WASD move · arrows aim · Space charge, Space again to strike · \
         left mouse hold on the scene to charge · E take · Ctrl+S save · Esc close"
    );
}

/// A binding the keyboard cannot reach still reads in the line. The session's
/// pointer verb is the only one today, and leaving it out of the declaration
/// is how a help line starts drifting again.
#[test]
fn a_binding_with_no_press_still_reads() {
    let keymap = session_keymap();
    let said = keymap
        .bindings
        .iter()
        .find(|binding| binding.presses.is_empty())
        .expect("the pointer verb is declared");
    assert_eq!(said.words(" "), "left mouse hold on the scene to charge");
    assert!(keymap.help().contains(&said.words(" ")));
}

/// Dispatch runs through the same declaration: a claimed press yields its
/// command, an unclaimed one yields nothing, and the Ctrl state is part of the
/// press rather than a branch above it — which is what keeps `S` and `Ctrl+S`
/// two different bindings.
#[test]
fn the_declaration_is_what_dispatches() {
    let keymap = session_keymap();
    assert_eq!(
        keymap.command(&Press::Char("s", false)),
        Some(&Command::Move(1))
    );
    assert_eq!(
        keymap.command(&Press::Char("s", true)),
        Some(&Command::Save)
    );
    assert_eq!(
        keymap.command(&Press::Named("Escape")),
        Some(&Command::Close)
    );
    assert_eq!(keymap.command(&Press::Char("z", false)), None);
    assert_eq!(
        keymap.command(&Press::Char("e", true)),
        None,
        "a modifier the declaration never claimed is not the plain key"
    );
}

/// Every claimed press is reachable, in declaration order — what a product's
/// own test walks to prove its handler answers all of them.
#[test]
fn every_claimed_press_is_listed_once() {
    let keymap = session_keymap();
    let presses: Vec<&Press> = keymap.presses().collect();
    assert_eq!(presses.len(), 7);
    assert_eq!(presses[0], &Press::Char("w", false));
    assert_eq!(presses.last(), Some(&&Press::Named("Escape")));
    for press in &presses {
        assert!(keymap.command(press).is_some(), "{press:?} dispatches");
    }
}

/// The status line keeps the product's id — three scenarios read one by id —
/// and gains the shared class, exactly as M1's error line did.
#[test]
fn the_status_line_keeps_its_id_and_gains_the_shared_class() {
    for id in ["specimen-status", "selection", "journal-summary"] {
        let attrs = status_attrs(id);
        assert_eq!(attr(&attrs, "id"), Some(id));
        assert_eq!(attr(&attrs, "class"), Some(STATUS_LINE_CLASS));
        assert_eq!(attr(&attrs, "role"), Some("status"), "it is a live region");
    }
}

/// The help line carries the class the M0 sheet already styles beside
/// `#controls-help`, so adopting it moves no pixel — and it is *not* a live
/// region: what the keys are does not change while anyone reads it.
#[test]
fn the_help_line_rides_the_sheets_existing_rule() {
    let attrs = help_attrs("controls-help");
    assert_eq!(attr(&attrs, "id"), Some("controls-help"));
    assert_eq!(attr(&attrs, "class"), Some(HELP_CLASS));
    assert_eq!(attr(&attrs, "role"), None);
    let rules = shared();
    let rule = rules
        .lines()
        .find(|line| line.starts_with("#controls-help"))
        .expect("the sheet styles the help line");
    assert!(
        rule.contains(&format!(".{HELP_CLASS}")),
        "the id and the class must be one rule: {rule}"
    );
}

/// The panel defaults to what the session does: a heading over a
/// `.status-lines` stack, and no class on the section unless the product names
/// one.
#[test]
fn the_status_panel_defaults_to_the_shared_shape() {
    let panel = StatusPanel {
        lines: vec![
            "Action: idle".to_owned(),
            "Movement supports 2/2".to_owned(),
        ],
        class: Some("panel status"),
        ..StatusPanel::new("Status")
    };
    assert_eq!(panel.lines_class, None);
    assert_eq!(panel.title, Some("Status"));
    let bare = StatusPanel::new("Status");
    assert_eq!(bare.class, None);
    assert!(bare.lines.is_empty());
    assert_eq!(
        StatusPanel::default()
            .lines_class
            .unwrap_or(STATUS_LINES_CLASS),
        "status-lines"
    );
    let _: crate::viewport::Child<u32, ()> = status_panel(&panel);
}

/// A crib names the entries it carries and gets their own words back, in
/// declaration order and with the product's own punctuation. Isometry's key
/// hint is four of eight bindings joined by " / ".
#[test]
fn a_crib_narrows_the_line_without_rewriting_it() {
    let keymap = Keymap {
        separator: " / ",
        infix: ": ",
        ..session_keymap()
    };
    assert_eq!(
        keymap.help_where(|binding| matches!(binding.keys, "arrows" | "E" | "Esc")),
        "arrows: aim / E: take / Esc: close"
    );
    assert_eq!(keymap.help_where(|_| false), "");
}

/// The lines build for a product with its own state and no action type. A
/// compile-level check, as the cards and the chips have.
#[test]
fn the_lines_build_for_a_products_own_state() {
    struct Product {
        seen: usize,
    }
    let _: crate::viewport::Child<Product, ()> =
        crate::status::status_line("specimen-status", "Seed 7 · Candidate 1 of 3 · 6 parts");
    let _: crate::viewport::Child<Product, ()> =
        help_line("controls-help", session_keymap().help());
    assert_eq!(Product { seen: 0 }.seen, 0);
}
