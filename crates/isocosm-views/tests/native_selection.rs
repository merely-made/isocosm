// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../isocosm-native/tests/native_founding/fixture.rs"]
mod fixture;
#[path = "../../isocosm-native/tests/native_setup/fixture.rs"]
mod setup_fixture;
use cambium::TextInput;
use cosmograph::Execution;
use isocosm_views::{NativeAcceptance as Acceptance, NativeUi};

#[test]
fn ordinary_table_acceptance_needs_no_native_profile_and_native_acceptance_requires_preview() {
    let record = fixture::record();
    let mut ui = NativeUi::default();
    assert!(matches!(ui.acceptance(&record).unwrap(), Acceptance::TableOnly),
        "table-only acceptance was forced to select a native profile");
    ui.enabled = true;
    assert!(ui.acceptance(&record).unwrap_err().contains("preview a world profile"),
        "native acceptance fabricated an unselected world");
    let setup = setup_fixture::setup(Execution::Grouped);
    ui.profile_path = TextInput::new("world-profile.json");
    ui.preview_for(
        &serde_json::to_vec(&setup).unwrap(),
        None,
        &record,
        isocosm_native::founding::Purpose::Bench,
        None,
    )
    .unwrap();
    let Acceptance::Native(selected) = ui.acceptance(&record).unwrap() else { panic!("selected world was ignored") };
    assert_eq!(selected.setup, setup);
    assert_eq!(selected.preview.load(Execution::Individuals).unwrap().reading().authored_site("ford").unwrap().0, 0);
    assert_eq!(selected.sites.len(), 2);
}

#[test]
fn malformed_preview_and_stale_path_or_draft_cannot_replace_an_inspected_world() {
    let record = fixture::record();
    let mut ui = NativeUi::default(); ui.enabled = true;
    ui.profile_path = TextInput::new("world-profile.json");
    let bytes = serde_json::to_vec(&setup_fixture::setup(Execution::Individuals)).unwrap();
    ui.preview_for(
        &bytes,
        None,
        &record,
        isocosm_native::founding::Purpose::Bench,
        None,
    )
    .unwrap();
    let before = ui.selection.as_ref().unwrap().preview.clone();
    assert!(ui
        .preview_for(
            b"{}",
            None,
            &record,
            isocosm_native::founding::Purpose::Bench,
            None,
        )
        .is_err());
    assert_eq!(ui.selection.as_ref().unwrap().preview, before, "refused profile replaced native preview");
    let mut rerolled = record.clone(); rerolled.entropy ^= 1;
    assert!(ui.acceptance(&rerolled).unwrap_err().contains("another generated draft"),
        "stale draft preview was accepted");
    ui.profile_path = TextInput::new("other-profile.json");
    assert!(ui.acceptance(&record).unwrap_err().contains("changed since preview"),
        "unpreviewed profile selection was accepted");
    assert!(ui.world.is_none(), "preview published a native world");
}

#[test]
fn actual_ui_controls_arm_preview_and_commit_only_for_the_host() {
    use isocosm_views::{GenerationRequest, UiState};
    let mut ui = UiState::new(isocosm_core::MapDocument::new("table", 4, 4));
    ui.can_edit_inventory = false;
    ui.choose_native_world(true);
    assert!(!ui.native.enabled, "client selected the host's native world");
    ui.generator_preview = Some(fixture::record());
    ui.preview_native_world();
    assert!(!ui.native.preview_request, "client armed native world preview");
    ui.can_edit_inventory = true;
    ui.choose_native_world(true); ui.preview_native_world();
    assert!(ui.native.enabled && ui.native.preview_request);
    ui.choose_native_world(false);
    ui.commit_generation_preview();
    assert_eq!(ui.generation_request, Some(GenerationRequest::Commit),
        "ordinary table-only commit is no longer reachable through the real UI");
    assert!(ui.native.selection.is_none());
}
