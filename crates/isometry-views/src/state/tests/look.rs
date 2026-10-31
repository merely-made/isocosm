//! The look editor: cancel leaves the table, accept writes it, reopen reads it.

use super::*;

fn first_token(ui: &UiState) -> Token {
    ui.map.tokens[0].clone()
}

#[test]
fn cancel_leaves_the_table_and_accept_writes_what_reopening_shows() {
    let mut ui = UiState::new(demo_map());
    let token = first_token(&ui);
    let before = ui.map.clone();

    ui.open_look(token.id);
    assert!(ui.look_open);
    assert_eq!(ui.look.sprite, token.sprite);
    ui.set_look_class("goblin");
    ui.set_look_layer("hair", "hair:long");
    ui.set_look_colour(1, [1, 2, 3]);
    ui.cancel_look();
    assert_eq!(ui.map, before, "cancel changes nothing at the table");

    ui.open_look(token.id);
    assert_eq!(ui.look.sprite, token.sprite, "the cancelled draft is gone");
    ui.set_look_class("goblin");
    ui.set_look_layer("hair", "hair:long");
    ui.set_look_colour(1, [1, 2, 3]);
    let accepted = ui.look.appearance.clone();
    ui.accept_look();
    assert!(!ui.look_open);
    assert_eq!(ui.map.token(token.id).unwrap().sprite, "goblin");
    assert_eq!(ui.map.appearances.get(&token.id), Some(&accepted));

    ui.open_look(token.id);
    assert_eq!(ui.look.sprite, "goblin");
    assert_eq!(ui.look.appearance, accepted, "reopening shows what was accepted");
}

#[test]
fn choosing_the_class_own_layer_clears_the_override() {
    let mut ui = UiState::new(demo_map());
    let token = first_token(&ui);
    ui.open_look(token.id);
    ui.set_look_class("hero");
    ui.set_look_layer("hair", "hair:long");
    assert_eq!(ui.look.appearance.layers.len(), 1);
    // hero's own hair: the slot follows the pack again.
    ui.set_look_layer("hair", "hair:short");
    assert!(ui.look.appearance.is_empty());
}

#[test]
fn a_remote_accept_sends_the_event_and_waits_for_the_echo() {
    let mut ui = UiState::new(demo_map());
    let token = first_token(&ui);
    ui.net_mode = NetMode::Remote;
    ui.open_look(token.id);
    ui.set_look_layer("hair", "hair:crop");
    ui.accept_look();
    assert_eq!(ui.map.appearances.get(&token.id), None, "no optimistic write");
    assert!(matches!(
        ui.net_outbox.as_slice(),
        [GameEvent::AppearanceSet { token: id, .. }] if *id == token.id
    ));
}

#[test]
fn a_seat_that_does_not_control_the_token_cannot_open_its_look() {
    let mut ui = UiState::new(demo_map());
    let mut token = first_token(&ui);
    token.owner = Some("someone-else".to_owned());
    ui.map.tokens[0] = token.clone();
    ui.can_edit_inventory = false;
    ui.viewer = Some("me".to_owned());
    ui.open_look(token.id);
    assert!(!ui.look_open);
    ui.viewer = Some("someone-else".to_owned());
    ui.open_look(token.id);
    assert!(ui.look_open, "the controlling seat may");
}

#[test]
fn a_turn_entry_is_called_by_its_characters_name() {
    let mut ui = UiState::new(demo_map());
    let token = first_token(&ui);
    let mut sheet = isometry_core::SheetData::new("5e-srd");
    sheet.set_text("name", "Mira".to_owned());
    ui.map.set_sheet(token.id, sheet);
    assert_eq!(ui.turn_name(token.id), "Mira");
    ui.map.sheets.clear();
    assert_eq!(ui.turn_name(token.id), token.sprite, "no named sheet: the class");
}
