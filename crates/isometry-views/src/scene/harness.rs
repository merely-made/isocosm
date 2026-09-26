//! The scene board drawn through its shipping path, for the receipts that
//! need a real frame.
//!
//! One `UiState`, the snapshot the host's `SceneBoard::sync` pushes into, and
//! the [`BoardSource`] the producer draws from — assembled here rather than in
//! each test file, because B5's cost receipts ([`super::cost_tests`]) measure
//! exactly what B4's edit receipts ([`super::edit_tests`]) assert and a second
//! copy of the fixture would be a second thing to keep true.
//!
//! Without a wgpu adapter every receipt built on this skips **loudly** through
//! [`board_or_skip`] and asserts nothing.

use isometer::{FrameRequest, SceneSource};
use isometry_core::TileCoord;

use super::board::{BoardPick, BoardSource};
use super::ground::BoardGround;
use super::view::BoardHandle;
use crate::demo::{demo_map, synth_map};
use crate::state::UiState;

/// The pane every receipt draws into, logical px.
pub(super) const PANE: (f32, f32) = (640.0, 480.0);

/// The board pane of B5's headed session, logical px: 1,782 by 1,504 physical
/// at device scale 2 and interface zoom 0.917.
pub(super) const HEADED_PANE: (f32, f32) = (972.0, 820.0);

/// The stress board with relief: `synth_map`'s ground and pieces, every tile
/// raised in steps of 0 to 7, so its bricks stand two layers tall.
pub(super) fn relief_map(edge: u32) -> isometry_core::MapDocument {
    let mut map = synth_map(edge, edge);
    for row in 0..edge {
        for col in 0..edge {
            map.elevation.set(col, row, ((col / 6 + row / 9) % 8) as u8);
        }
    }
    map
}

pub(super) fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

/// A board drawn through the shipping path: one `UiState`, the snapshot it
/// syncs into, and the source over it.
pub(super) struct Board {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub(super) ui: UiState,
    handle: BoardHandle,
    source: BoardSource,
    pane: (f32, f32),
}

impl Board {
    pub(super) fn new() -> Option<Self> {
        Self::of(demo_map())
    }

    /// The same fixture over any map, so a cost receipt can ask what a board
    /// other than the demo's costs.
    pub(super) fn of(map: isometry_core::MapDocument) -> Option<Self> {
        Self::sized(map, PANE)
    }

    /// The fixture at another pane size, framed on the map's centre tile.
    pub(super) fn sized(map: isometry_core::MapDocument, pane: (f32, f32)) -> Option<Self> {
        let (device, queue) = device()?;
        let mut ui = UiState::new(map);
        ui.viewport = pane;
        let centre = (
            ui.map.ground.width() as i32 / 2,
            ui.map.ground.height() as i32 / 2,
        );
        let (x, y) = ui.geo.tile_to_screen(centre, 0);
        ui.camera = (pane.0 / 2.0 - x, pane.1 / 2.0 - y);
        let mut view = super::view::BoardView::new(ui.map.clone());
        view.sync(&ui);
        let handle = view.into_handle();
        let source = BoardSource::new(handle.clone());
        Some(Self {
            device,
            queue,
            ui,
            handle,
            source,
            pane,
        })
    }

    /// Pushes the state into the snapshot and draws one frame, the way the
    /// host's own `SceneBoard::sync` plus the producer do.
    pub(super) fn draw(&mut self) {
        self.handle.borrow_mut().sync(&self.ui);
        let request = FrameRequest {
            device: &self.device,
            queue: &self.queue,
            size: [self.pane.0 as u32, self.pane.1 as u32],
            aspect: self.pane.0 / self.pane.1,
            color: None,
            needs_frame: true,
            render_scale: 1,
        };
        self.source
            .frame(&request)
            .expect("the scene board draws one frame");
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the frame completes");
    }

    /// What the tracer did with the terrain on the last frame.
    pub(super) fn terrain(&self) -> isometer::lens::BrickDiagnostics {
        self.source
            .terrain_diagnostics()
            .expect("a drawn frame leaves a terrain receipt")
    }

    /// What the last ground change cost this crate.
    pub(super) fn cost(&self) -> super::ground::GroundCost {
        self.source.ground_cost().expect("the board grew a ground")
    }

    /// The board's ground: its columns and the bricks the scene holds.
    pub(super) fn ground(&self) -> &BoardGround {
        self.source.ground().expect("a drawn board read its ground")
    }

    pub(super) fn pick(&self, px: f32, py: f32) -> Option<BoardPick> {
        self.source
            .pick([2.0 * px / self.pane.0 - 1.0, 1.0 - 2.0 * py / self.pane.1])
    }

    /// The pane this board draws into.
    pub(super) fn pane(&self) -> (f32, f32) {
        self.pane
    }

    /// The camera the last frame was drawn with.
    pub(super) fn camera(&self) -> isometer::SlabCamera {
        self.source
            .presented_camera()
            .expect("a drawn board has a camera")
    }
}

macro_rules! board_or_skip {
    ($what:literal) => {
        match crate::scene::harness::Board::new() {
            Some(board) => board,
            None => {
                eprintln!(
                    "SKIPPED: no wgpu adapter on this machine, so no frame is drawn and {} \
                     asserts nothing.",
                    $what
                );
                return;
            },
        }
    };
}

pub(super) use board_or_skip;

/// A flat tile the demo map carries, for an edit to land on.
pub(super) fn flat_tile(ui: &UiState) -> TileCoord {
    ui.map
        .ground
        .iter()
        .find(|(col, row, kind)| {
            kind.0 != 0 && *ui.map.elevation.get(*col, *row).unwrap_or(&0) == 0
        })
        .map(|(col, row, _)| (col as i32, row as i32))
        .expect("the demo map has flat ground")
}
