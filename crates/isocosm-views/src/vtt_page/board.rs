//! Present the seat's disclosed native landings and actions on the table page.

use std::collections::{BTreeMap, BTreeSet};

use cambium::{clickable, el, text};
use isocosm_core::{IsoGeometry, encounter_board::EncounterBoard};

use crate::{
    board::UiChild,
    state::{UiState, VttCardView, VttProjectionView},
};

pub(super) use isocosm_core::encounter_board::{BADGE, VALUE};

pub(crate) const CSS: &str = r#"
.vtt-family .vtt-page.vtt-with-board { width: 85%; max-width: 720px; }
.vtt-board-viewport { overflow: auto; max-height: 360px; background-color: #101820; border: 1px solid #384750; margin: 4px 0 8px; }
.vtt-board-canvas { position: relative; margin: 0 auto; }
.tile.vtt-board-cell, .tile.vtt-board-cell:hover { background-color: #4b5b64; }
.vtt-board-cell.vtt-board-stride { background-color: #418984; border: 0; padding: 0; cursor: pointer; }
.vtt-board-cell.vtt-board-stride:hover, .vtt-board-cell.vtt-board-stride:focus { background-color: #8ce4cf; }
.vtt-board-stride-mark { position: absolute; pointer-events: none; left: 50%; top: 50%; width: 4px; height: 4px; margin-left: -2px; margin-top: -2px; border-radius: 2px; background-color: #d1fff2; }
.vtt-board-token { position: absolute; pointer-events: none; width: 16px; height: 16px; line-height: 14px; text-align: center; font-size: 11px; font-weight: bold; color: #fff; background-color: #3b5079; border: 1px solid #c3d5ed; border-radius: 10px; }
.vtt-board-leader { position: absolute; pointer-events: none; height: 2px; background-color: #d4dfee; }
.vtt-board-token.vtt-board-turn { z-index: 1; color: #292015; background-color: #e7ba70; border-color: #fff0d0; }
.vtt-board-legend { color: #cfd3dd; font-size: 12px; margin: 3px 0; }
"#;

pub(super) fn is_board(view: &VttProjectionView, item: &VttCardView) -> bool {
    view.id == "vtt:table" && item.badges.iter().any(|badge| badge == BADGE)
}

/// Canvas coordinates are local to the site. Offset before projecting so a
/// small encounter at a large native coordinate keeps its precision.
struct Frame {
    origin: [i64; 3],
    geometry: IsoGeometry,
    min: (f32, f32),
    max: (f32, f32),
}

const TILE: (f32, f32) = (40., 20.);
const PAD: (f32, f32) = (23., 25.);

impl Frame {
    fn for_board(board: &EncounterBoard) -> Option<Self> {
        let points = board
            .cells
            .iter()
            .map(|c| c.at)
            .chain(board.tokens.iter().map(|t| t.at));
        let origin = board
            .tokens
            .iter()
            .find(|token| token.active)
            .or_else(|| board.tokens.first())
            .map(|token| token.at)
            .or_else(|| points.clone().next())?;
        let scale = board.cells_per_square.max(1) as f32;
        let mut frame = Self {
            origin,
            geometry: IsoGeometry {
                tile_w: TILE.0 / scale,
                tile_h: TILE.1 / scale,
                elev_step: TILE.1 / (2. * scale),
            },
            min: (0., 0.),
            max: (0., 0.),
        };
        for point in points {
            let Some((x, y)) = frame.project(point) else {
                continue;
            };
            frame.min = (frame.min.0.min(x), frame.min.1.min(y));
            frame.max = (frame.max.0.max(x), frame.max.1.max(y));
        }
        Some(frame)
    }

    fn project(&self, at: [i64; 3]) -> Option<(f32, f32)> {
        let delta = |axis: usize| -> Option<i32> {
            i32::try_from(at[axis].checked_sub(self.origin[axis])?).ok()
        };
        let (col, row) = (delta(0)?, delta(2)?);
        col.checked_add(row)?;
        col.checked_sub(row)?;
        // One viewport on each side of the encounter. A far-away token must
        // not turn the local terrain into a site-sized scroll surface.
        let point = self.geometry.tile_to_screen((col, row), delta(1)?);
        (point.0.abs() <= 360. && point.1.abs() <= 165.).then_some(point)
    }

    fn center(&self, at: [i64; 3]) -> Option<(f32, f32)> {
        let (x, y) = self.project(at)?;
        Some((x - self.min.0 + PAD.0, y - self.min.1 + PAD.1))
    }
}

pub(super) fn render(view: &VttProjectionView, item: &VttCardView) -> (UiChild, BTreeSet<String>) {
    let board = item
        .values
        .iter()
        .find(|value| value.label == VALUE)
        .and_then(|v| serde_json::from_str::<EncounterBoard>(&v.value).ok());
    let Some(board) = board else {
        return (
            Box::new(el("div", text("Board data is unavailable.")).attr("class", "vtt-value")),
            BTreeSet::new(),
        );
    };
    let Some(frame) = Frame::for_board(&board) else {
        return (
            Box::new(
                el("div", text("No board can be displayed for this site."))
                    .attr("class", "vtt-value"),
            ),
            BTreeSet::new(),
        );
    };

    let mut nodes: Vec<UiChild> = Vec::new();
    let mut strides = BTreeSet::new();
    let actions: BTreeMap<_, _> = item
        .actions
        .iter()
        .filter(|action| !action.requires_payload())
        .map(|action| (action.id.as_str(), action))
        .collect();
    let mut cells = board.cells.iter().collect::<Vec<_>>();
    cells.sort_by_key(|cell| (i128::from(cell.at[0]) + i128::from(cell.at[2]), cell.at[1]));

    for cell in cells {
        let Some((x, y)) = frame.center(cell.at) else {
            continue;
        };
        // Leave a narrow gutter between neighbouring top faces.
        let style = format!(
            "left:{}px;top:{}px;width:{}px;height:{}px;",
            x - TILE.0 / 2. + 1.,
            y - TILE.1 / 2. + 1.,
            TILE.0 - 2.,
            TILE.1 - 2.
        );
        if let Some(action) = cell.stride.as_deref().and_then(|id| actions.get(id)) {
            let projection = view.id.clone();
            let card = item.id.clone();
            let id = action.id.clone();
            let cut = (view.epoch, view.revision);
            strides.insert(id.clone());
            nodes.push(Box::new(clickable(
                el(
                    "button",
                    el("span", ())
                        .attr("class", "vtt-board-stride-mark")
                        .attr("aria-hidden", "true"),
                )
                .attr("class", "tile vtt-board-cell vtt-board-stride")
                .attr("type", "button")
                .attr("aria-label", action.label.clone())
                .attr("title", action.label.clone())
                .attr("data-native-at", format!("{:?}", cell.at))
                .attr("style", style),
                move |ui: &mut UiState, _| ui.request_vtt_action(&projection, &card, &id, cut),
            )));
        } else {
            nodes.push(Box::new(
                el("div", ())
                    .attr("class", "tile vtt-board-cell")
                    .attr("style", style),
            ));
        }
    }

    let mut legend: Vec<UiChild> = Vec::new();
    let mut markers = Vec::<(f32, f32)>::new();
    for (i, token) in board.tokens.iter().enumerate() {
        let number = (i + 1).to_string();
        let who = if token.active {
            format!("{} (current turn)", token.name)
        } else {
            token.name.clone()
        };
        let class = if token.active {
            "vtt-board-token vtt-board-turn"
        } else {
            "vtt-board-token"
        };
        let Some((x, y)) = frame.center(token.at) else {
            legend.push(Box::new(
                el(
                    "div",
                    text(format!(
                        "{number}  {who} · outside this view at {:?}",
                        token.at
                    )),
                )
                .attr("class", "vtt-board-legend"),
            ));
            continue;
        };
        // Nearby landings can project less than one marker apart. Keep the
        // numeral readable and leave a short leader at its actual landing.
        let label_x = [0., -14., 14., -20., 20.]
            .into_iter()
            .map(|dx| x + dx)
            .find(|cx| {
                markers
                    .iter()
                    .all(|(px, py)| (cx - px).hypot(y - py) >= 18.)
            })
            .unwrap_or(x);
        markers.push((label_x, y));
        if label_x != x {
            nodes.push(Box::new(
                el("div", ()).attr("class", "vtt-board-leader").attr(
                    "style",
                    format!(
                        "left:{}px;top:{}px;width:{}px;",
                        x.min(label_x),
                        y,
                        (label_x - x).abs()
                    ),
                ),
            ));
        }
        nodes.push(Box::new(
            el("div", text(number.clone()))
                .attr("class", class)
                .attr("title", who.clone())
                .attr("aria-label", who.clone())
                .attr("data-native-at", format!("{:?}", token.at))
                .attr("data-body", token.body.to_string())
                .attr(
                    "style",
                    format!("left:{}px;top:{}px;", label_x - 8., y - 10.),
                ),
        ));
        legend.push(Box::new(
            el("div", text(format!("{number}  {who}"))).attr("class", "vtt-board-legend"),
        ));
    }

    let size = format!(
        "width:{}px;height:{}px;",
        frame.max.0 - frame.min.0 + PAD.0 * 2.,
        frame.max.1 - frame.min.1 + PAD.1 * 2.
    );
    let canvas = el("div", nodes)
        .attr("class", "vtt-board-canvas")
        .attr("style", size);
    let viewport = el("div", canvas)
        .attr("class", "vtt-board-viewport")
        .attr("role", "group")
        .attr("aria-label", item.title.clone());
    let hint = (!strides.is_empty())
        .then(|| el("div", text("Select a marked square to Stride.")).attr("class", "vtt-value"));
    (Box::new(el("div", (viewport, hint, legend))), strides)
}
