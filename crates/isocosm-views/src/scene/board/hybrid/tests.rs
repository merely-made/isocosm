// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm_core::{Facing, IsoGeometry, MapDocument, TileCoord, Token, TokenId};
use isometer::hybrid::HybridStats;
use isometer::{FrameRequest, SceneSource};

use super::BoardBodyQuality;
use crate::scene::harness::device;
use crate::scene::{BoardPick, BoardProducer, BoardSource, BoardView};
use crate::state::UiState;

const SIZE: [u32; 2] = [320, 240];
const NEAR_TILE: TileCoord = (3, 3);
const RAISED_TILE: TileCoord = (6, 1);
const HIDDEN_TILE: TileCoord = (0, 7);

fn board() -> UiState {
    let mut map = MapDocument::new("far token test", 8, 8);
    let stone = map.intern_tile_kind("stone");
    for row in 0..8 {
        for col in 0..8 {
            map.ground.set(col, row, stone);
        }
    }
    map.elevation
        .set(RAISED_TILE.0 as u32, RAISED_TILE.1 as u32, 3);
    for (id, at) in [(1, NEAR_TILE), (2, RAISED_TILE), (3, HIDDEN_TILE)] {
        map.tokens.push(Token {
            id: TokenId(id),
            at,
            facing: Facing::East,
            sprite: "knight".into(),
            owner: None,
        });
    }
    let mut ui = UiState::new(map);
    ui.viewer = Some("viewer".into());
    ui.visible = (0..8)
        .flat_map(|row| (0..8).map(move |col| (col, row)))
        .filter(|&tile| tile != HIDDEN_TILE)
        .collect();
    ui.viewport = (SIZE[0] as f32, SIZE[1] as f32);
    zoom(&mut ui, 0.5);
    ui
}

fn zoom(ui: &mut UiState, factor: f32) {
    let base = IsoGeometry::default();
    ui.geo = IsoGeometry {
        tile_w: base.tile_w * factor,
        tile_h: base.tile_h * factor,
        elev_step: base.elev_step * factor,
    };
    let (x, y) = ui.geo.tile_to_screen(NEAR_TILE, 0);
    ui.camera = (ui.viewport.0 / 2.0 - x, ui.viewport.1 / 2.0 - y);
}

fn token_pixel(source: &BoardSource, tile: TileCoord, elevation: i32, id: u32) -> Option<[f32; 2]> {
    let feet = source.world()?.stand(tile, elevation);
    let [x, y] = source.presented_camera()?.ndc_of(feet)?;
    (-8..=8)
        .flat_map(|dx| (0..=28).map(move |dy| (dx, dy)))
        .find_map(|(dx, dy)| {
            let point = [
                x + 2.0 * dx as f32 / SIZE[0] as f32,
                y + 2.0 * dy as f32 / SIZE[1] as f32,
            ];
            (source.pick(point) == Some(BoardPick::Token(TokenId(id)))).then_some(point)
        })
}

fn settled(producer: &mut BoardProducer, request: &FrameRequest<'_>) -> HybridStats {
    assert!(
        producer.render_scene(request).unwrap().is_some(),
        "new inputs must draw"
    );
    for _ in 0..16 {
        let stats = producer.source().hybrid_stats().unwrap();
        assert!(stats.retained_bytes + stats.retiring_bytes <= 4 * 1024 * 1024);
        assert!(stats.refreshes <= 8);
        request
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        if producer.render_scene(request).unwrap().is_none() {
            return stats;
        }
    }
    panic!("an unchanged board must park after the hybrid transition settles");
}

#[test]
#[ignore = "headless receipt requires a real GPU"]
fn far_token_quality_preserves_board_picks_and_fog_across_zoom_reversal() {
    let (device, queue) = device().expect("this receipt requires a GPU");
    let mut ui = board();
    let map_before = ui.map.clone();
    let mut view = BoardView::new(ui.map.clone());
    view.sync(&ui);
    let handle = view.into_handle();
    let mut producer = BoardProducer::new(BoardSource::new(handle.clone()));
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size: SIZE,
        aspect: SIZE[0] as f32 / SIZE[1] as f32,
        color: None,
        needs_frame: false,
        render_scale: 1,
    };

    assert_eq!(settled(&mut producer, &request), HybridStats::default());
    let flat = token_pixel(producer.source(), NEAR_TILE, 0, 1).expect("live flat token pick");
    let raised =
        token_pixel(producer.source(), RAISED_TILE, 3, 2).expect("live elevated token pick");
    producer
        .source_mut()
        .set_body_quality(BoardBodyQuality::Hybrid);
    let initial = settled(&mut producer, &request);
    assert_eq!(
        initial.sprite_bodies, 2,
        "only the visible tokens use the atlas"
    );
    assert_eq!(initial.resident_slots, 2);
    assert_eq!(
        producer.source().pick(flat),
        Some(BoardPick::Token(TokenId(1)))
    );
    assert_eq!(
        producer.source().pick(raised),
        Some(BoardPick::Token(TokenId(2)))
    );
    assert_eq!(token_pixel(producer.source(), HIDDEN_TILE, 0, 3), None);

    ui.selected_token = Some(TokenId(3));
    handle.borrow_mut().sync(&ui);
    assert_eq!(settled(&mut producer, &request).sprite_bodies, 2);
    assert_eq!(token_pixel(producer.source(), HIDDEN_TILE, 0, 3), None);

    ui.selected_token = Some(TokenId(2));
    handle.borrow_mut().sync(&ui);
    assert_eq!(settled(&mut producer, &request).sprite_bodies, 1);
    assert_eq!(
        producer.source().pick(raised),
        Some(BoardPick::Token(TokenId(2)))
    );

    ui.selected_token = None;
    zoom(&mut ui, 4.0);
    handle.borrow_mut().sync(&ui);
    assert_eq!(settled(&mut producer, &request).sprite_bodies, 0);
    assert!(token_pixel(producer.source(), NEAR_TILE, 0, 1).is_some());

    zoom(&mut ui, 0.5);
    handle.borrow_mut().sync(&ui);
    let returned = settled(&mut producer, &request);
    assert_eq!(returned.sprite_bodies, 2);
    assert_eq!(
        producer.source().pick(flat),
        Some(BoardPick::Token(TokenId(1)))
    );
    assert_eq!(
        producer.source().pick(raised),
        Some(BoardPick::Token(TokenId(2)))
    );
    assert_eq!(token_pixel(producer.source(), HIDDEN_TILE, 0, 3), None);
    assert_eq!(ui.map, map_before);
    eprintln!("[far board] initial={initial:?} returned={returned:?}");
}
