// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The graph drawn on the stack's canvas (case 1048): Graphshell's remote
//! board over the mounted scene, painted with its titles, edges and folds
//! and rasterized by the host's own render core.

use cambium_genet_winit_host::SceneProducer;
use cambium_rootstock::ProducerSemantics;
use graphshell_host::board::RemoteBoard;
use pictograph::canvas::{BoardFit, BoardText, Speed};

use crate::client::Client;

/// The leaf the board paints into.
pub const LEAF: u64 = 1;
const FIT: f32 = 24.0;

#[derive(Default)]
pub struct Board {
    remote: RemoteBoard,
    text: BoardText,
    /// Advances whenever the drawn board can change.
    revision: u64,
}

impl Board {
    /// Reconcile with the client's acknowledged scene.
    pub fn sync(&mut self, client: &Client) {
        self.remote.sync(client.mounted(), client.revision(), None, Speed::REAL_TIME);
        self.revision += 1;
    }

    /// One physics step; whether the board is still moving.
    pub fn step(&mut self) -> bool {
        let moving = self.remote.is_settling();
        if moving { self.revision += 1; }
        moving
    }

    pub fn cards(&self) -> usize { self.remote.scene().cards.len() }
    pub fn edges(&self) -> usize { self.remote.scene().drawn_relations() }

    fn paint(&mut self, width: u32, height: u32) -> netrender::Scene {
        self.remote.tick();
        let fit = BoardFit::new(FIT, FIT, FIT, FIT);
        let scene = self.remote.scene().clone();
        scene.paint_titled(self.remote.board(), width, height, fit, &mut self.text)
            .unwrap_or_else(|_| scene.paint(self.remote.board(), width, height, fit))
    }
}

fn frame(board: &mut Board, width: u32, height: u32) -> netrender::Scene { board.paint(width, height) }
fn revision(board: &Board) -> u64 { board.revision }
fn semantics(_: &Board) -> Option<ProducerSemantics> { None }

pub fn producer(board: std::rc::Rc<std::cell::RefCell<Board>>) -> SceneProducer<Board> {
    SceneProducer::new(board, LEAF, frame, revision, semantics)
}
