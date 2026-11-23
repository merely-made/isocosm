//! The family table page through Cambium layout, clip-aware picking and dispatch.

use super::*;
use isocosm_core::encounter_board::{BADGE, BoardCell, BoardToken, EncounterBoard, VALUE};
use isocosm_views::{
    VttActionRequest, VttActionView, VttCardValue, VttCardView, VttProjectionView,
};

fn table(actions: bool) -> VttProjectionView {
    let board = EncounterBoard {
        site: 17,
        cells_per_square: 4,
        cells: vec![
            BoardCell {
                at: [-4, 100, 8],
                stride: None,
            },
            BoardCell {
                at: [0, 100, 8],
                stride: Some("native-stride".into()),
            },
        ],
        tokens: vec![
            BoardToken {
                body: 63,
                name: "Linn".into(),
                at: [-4, 100, 8],
                active: true,
            },
            BoardToken {
                body: 90,
                name: "Tavi".into(),
                at: [i64::MAX, 0, i64::MIN],
                active: false,
            },
        ],
    };
    VttProjectionView {
        id: isocosm_native::TABLE.into(),
        label: "Table".into(),
        epoch: 41,
        revision: 8,
        cards: vec![VttCardView {
            id: "site-17".into(),
            title: "Encounter".into(),
            badges: vec![BADGE.into()],
            values: vec![VttCardValue {
                label: VALUE.into(),
                value: serde_json::to_string(&board).unwrap(),
            }],
            actions: if actions {
                vec![VttActionView {
                    id: "native-stride".into(),
                    label: "Stride east".into(),
                    explanation: String::new(),
                    payload_schema: r#"{"type":"null"}"#.into(),
                }]
            } else {
                vec![]
            },
        }],
    }
}

#[test]
fn disclosed_board_routes_only_the_painted_stride_to_its_card_and_cut() {
    let mut ui = UiState::new(isocosm_core::MapDocument::new("unused legacy map", 2, 2));
    ui.set_vtt_projections(vec![table(true)]);
    let mut harness = Harness::new(board_css(), ui, board_root as Logic);
    harness.layout_at(WINDOW.0, WINDOW.1);
    let tile = harness.with_dom(|dom| {
        let nodes = |class| taproot::matching_classes(dom, &Selector::class(class));
        assert_eq!(nodes("vtt-board-cell").len(), 2);
        assert_eq!(nodes("vtt-board-token").len(), 1);
        assert_eq!(nodes("vtt-board-turn").len(), 1);
        assert_eq!(
            nodes("vtt-board-legend").len(),
            2,
            "distant token remains listed"
        );
        assert!(
            nodes("vtt-action").is_empty(),
            "Stride is rendered on its landing"
        );
        let stride = nodes("vtt-board-stride");
        assert_eq!(stride.len(), 1);
        stride[0]
    });
    let (x, y, w, h) = harness.painted_rect(tile).expect("visible landing");
    assert!(w > 0. && h > 0.);
    harness.tab(true);
    harness.tab(true);
    assert_eq!(
        harness.focus(),
        Some(tile),
        "Tab reaches the disclosed landing"
    );
    harness.press_key(&KeyPress::named(NamedKey::Enter));
    harness.update(|ui| {
        assert_eq!(
            ui.take_vtt_action_request().unwrap().action,
            "native-stride"
        )
    });
    harness.click_at(x + 0.9 * w, y + 0.1 * h);
    harness.update(|ui| {
        assert!(
            ui.take_vtt_action_request().is_none(),
            "outside the diamond"
        )
    });
    harness.click_at(x + w / 2., y + h / 2.);
    harness.update(|ui| {
        assert_eq!(
            ui.take_vtt_action_request(),
            Some(VttActionRequest {
                projection: isocosm_native::TABLE.into(),
                card: "site-17".into(),
                action: "native-stride".into(),
                payload: vec![],
                epoch: 41,
                revision: 8,
            })
        );
        let mut refreshed = table(true);
        refreshed.revision += 1;
        ui.set_vtt_projections(vec![refreshed]);
    });
    harness.relayout();
    harness.click_at(x + w / 2., y + h / 2.);
    harness.update(|ui| {
        let request = ui
            .take_vtt_action_request()
            .expect("unchanged landing still invokes its displayed action");
        assert_eq!((request.epoch, request.revision), (41, 9));
        ui.set_vtt_projections(vec![table(false)]);
    });
    harness.relayout();
    harness.with_dom(|dom| {
        assert!(taproot::matching_classes(dom, &Selector::class("vtt-board-stride")).is_empty());
        assert_eq!(
            taproot::matching_classes(dom, &Selector::class("vtt-board-cell")).len(),
            2
        );
    });
    harness.click_at(x + w / 2., y + h / 2.);
    harness.update(|ui| assert!(ui.take_vtt_action_request().is_none()));
}

#[test]
fn a_disconnected_invited_table_shows_its_status_and_clears_the_board() {
    let mut ui = UiState::new(isocosm_core::MapDocument::new("local", 2, 2));
    ui.invited_table = true;
    ui.set_vtt_projections(vec![table(true)]);
    let mut harness = Harness::new(board_css(), ui, board_root as Logic);
    harness.layout_at(WINDOW.0, WINDOW.1);
    harness.update(|ui| {
        ui.set_vtt_projections(Vec::new());
        ui.status = "Waiting for the host".into();
    });
    harness.relayout();
    harness.with_dom(|dom| {
        assert_eq!(
            taproot::matching_classes(dom, &Selector::class("vtt-status")).len(),
            1
        );
        for class in ["vtt-board-cell", "vtt-action", "board"] {
            assert!(taproot::matching_classes(dom, &Selector::class(class)).is_empty());
        }
    });
}
