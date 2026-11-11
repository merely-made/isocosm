// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The graph drawn on the stack's canvas (case 1048): Graphshell's remote
//! board over the mounted scene, its law reading the scene's relations so
//! linked cards cluster, under a camera that zooms and pans over cards of a
//! fixed size (mere stack-lift D3). The law is the panel's to choose.

use cambium_genet_winit_host::SceneProducer;
use cambium_rootstock::ProducerSemantics;
use graphshell_host::board::RemoteBoard;
use cartography::canvas::{BoardFit, BoardText, PhysicsChoice, PhysicsLaw, Speed};

use crate::client::Client;

/// The leaf the board paints into.
pub const LEAF: u64 = 1;
const MARGIN: f32 = 24.0;
/// The law a viewer opens with. Energy gathers linked cards hardest (mere's
/// stack-lift findings, 2026-10-31: 116 within against 712 across), but at
/// that spacing a card of fixed size overlaps its neighbours; Springs rests
/// linked cards about 170 apart, wider than a node card (132).
pub const DEFAULT_LAW: PhysicsLaw = PhysicsLaw::Springs;
const ZOOMS: (f32, f32) = (0.1, 4.0);

pub struct Board {
    remote: RemoteBoard,
    text: BoardText,
    law: PhysicsLaw,
    /// The person's zoom, over the base the view mode sets.
    zoom: f32,
    pan: (f32, f32),
    /// Whether the base is one layout unit to one pixel, where cards keep
    /// their spacing; otherwise the whole graph is fitted.
    actual: bool,
    /// Advances whenever the drawn board can change.
    revision: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self { remote: RemoteBoard::default(), text: BoardText::default(), law: DEFAULT_LAW, zoom: 1.0,
            pan: (0.0, 0.0), actual: true, revision: 0 }
    }
}

impl Board {
    fn stage(&self) -> cartography::canvas::dynamics_spec::DynamicsSpec {
        let mut choice = PhysicsChoice::default();
        choice.law = self.law;
        choice.into_spec()
    }

    /// Reconcile with the client's acknowledged scene, under the chosen law.
    pub fn sync(&mut self, client: &Client) {
        let stage = self.stage();
        self.remote.sync(client.mounted(), client.revision(), Some(&stage), Speed::REAL_TIME);
        self.revision += 1;
    }

    pub fn law(&self) -> PhysicsLaw { self.law }

    /// Runs `law` from now on; the board's refusal, if any, is returned.
    pub fn set_law(&mut self, law: PhysicsLaw) -> Result<(), String> {
        let previous = self.law;
        self.law = law;
        let stage = self.stage();
        self.remote.board_mut().set_stage(&stage).map_err(|why| {
            self.law = previous;
            format!("{why:?}")
        })?;
        self.revision += 1;
        Ok(())
    }

    /// Zoom by `factor` about the viewport's centre.
    pub fn zoom_by(&mut self, factor: f32) {
        let zoom = (self.zoom * factor).clamp(ZOOMS.0, ZOOMS.1);
        let applied = zoom / self.zoom;
        self.pan = (self.pan.0 * applied, self.pan.1 * applied);
        self.zoom = zoom;
        self.revision += 1;
    }
    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.pan = (self.pan.0 + dx, self.pan.1 + dy);
        self.revision += 1;
    }
    /// The whole graph in view, or one layout unit to a pixel.
    pub fn view(&mut self, actual: bool) {
        (self.zoom, self.pan, self.actual) = (1.0, (0.0, 0.0), actual);
        self.revision += 1;
    }
    pub fn zoom(&self) -> f32 { self.zoom }

    /// One physics step; whether the board is still moving.
    pub fn step(&mut self) -> bool {
        let moving = self.remote.is_settling();
        if moving { self.revision += 1; }
        moving
    }

    pub fn cards(&self) -> usize { self.remote.scene().cards.len() }
    pub fn edges(&self) -> usize { self.remote.scene().drawn_relations() }

    /// The mean gap between linked cards against all cards' mean gap: below
    /// one when linked cards sit closer than cards in general.
    pub fn cluster_ratio(&self) -> Option<f32> {
        let scene = self.remote.scene();
        let board = self.remote.board();
        let mean = |pairs: &mut dyn Iterator<Item = (&str, &str)>| {
            let gaps: Vec<f32> = pairs.filter_map(|(a, b)| board.gap(a, b)).collect();
            (!gaps.is_empty()).then(|| gaps.iter().sum::<f32>() / gaps.len() as f32)
        };
        let links = scene.links();
        let linked = mean(&mut links.iter().map(|(a, b)| (a.as_str(), b.as_str())))?;
        let ids: Vec<&str> = scene.cards.iter().map(|c| c.id.as_str()).collect();
        let all = mean(&mut ids.iter().enumerate()
            .flat_map(|(i, a)| ids[i + 1..].iter().map(move |b| (*a, *b))))?;
        Some(linked / all)
    }

    /// The camera for a `width` × `height` viewport: the fit, at the base
    /// the view mode sets, times the person's zoom.
    fn fit_now(&self, width: f32, height: f32) -> BoardFit {
        let fit = BoardFit::new(MARGIN, MARGIN, MARGIN, MARGIN).fixed_cards(1.0);
        let base = if self.actual { 1.0 / self.fitted_scale(width, height) } else { 1.0 };
        fit.camera(base * self.zoom, self.pan)
    }

    /// The scale fitting the cards' held centres into the viewport.
    fn fitted_scale(&self, width: f32, height: f32) -> f32 {
        let scene = self.remote.scene();
        let held: Vec<(f32, f32)> = scene.cards.iter().map(|c| scene.position(c, self.remote.board())).collect();
        let span = |pick: fn(&(f32, f32)) -> f32| {
            let values = held.iter().map(pick);
            values.clone().fold(f32::MIN, f32::max) - values.fold(f32::MAX, f32::min)
        };
        if held.is_empty() { return 1.0; }
        ((width - 2.0 * MARGIN) / span(|p| p.0).max(1.0)).min((height - 2.0 * MARGIN) / span(|p| p.1).max(1.0))
            .min(1.0).max(1e-3)
    }

    fn paint(&mut self, width: u32, height: u32) -> netrender::Scene {
        self.remote.tick();
        let fit = self.fit_now(width as f32, height as f32);
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
