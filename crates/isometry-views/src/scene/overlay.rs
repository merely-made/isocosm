//! The board's overlays as terrain materials (B4).
//!
//! The DOM board says what a tile is *doing* by swapping its diamond's
//! background — reach, path, selection, template, door, encounter site — and
//! by laying a translucent diamond over remembered ground. The scene board has
//! no diamonds, so each of those becomes a **material**: the tile's top voxel
//! takes an overlay material instead of its kind's, and the palette I1 gave
//! the tracer carries the colour. Nothing new is drawn, and the ruled
//! precedence stays the stylesheet's own.
//!
//! **The palette's shape.** With `K` tile kinds and `T` = [`Tint::ALL`] tints,
//!
//! ```text
//! 0                         the unknown colour, black
//! 1      ..= K              the tile kinds        (a kind's id plus one)
//! K+1    ..= K+T            the tints             (tint index plus K+1)
//! K+T+1  ..= 2(K+T)         every one of the above again, under the shroud
//! ```
//!
//! so `shrouded(m) = m + K + T` and the whole table is `2(K+T)+1` entries. The
//! shrouded half exists because the DOM's shroud is *translucent*: remembered
//! terrain is still legible under it, and a single flat shroud colour would
//! throw that away. Past [`MAX_TERRAIN_MATERIALS`] there is no room for it, so
//! a map with more than [`SHROUDED_KIND_LIMIT`] kinds loses the legibility and
//! takes one flat shroud entry instead.
//!
//! **What is not here.** Markers and the context menu stay DOM, as §3 says;
//! range templates, facing arcs and labels wait for isomere, so nothing here
//! invents a second overlay vocabulary for them.

use std::collections::BTreeSet;

use isometer_lens::{MAX_TERRAIN_MATERIALS, TerrainPalette};
use isometry_core::{MapDocument, TileCoord, Token, path_to};

use crate::state::{EditMode, FogLevel, UiState};
#[cfg(test)]
use crate::theme::SHROUD;
use crate::theme::{TILE_TINTS, hex_rgb, shrouded, tile_kind_colour};

/// One state tint a ground tile can wear, weakest first: the order
/// [`TILE_TINTS`] declares, which is the stylesheet's own source order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    Selected,
    Reach,
    Path,
    Template,
    Door,
    Encounter,
}

impl Tint {
    /// Every tint, weakest first. The index into this is the index into
    /// [`TILE_TINTS`], so the colours cannot drift from the names.
    pub const ALL: [Tint; 6] = [
        Tint::Selected,
        Tint::Reach,
        Tint::Path,
        Tint::Template,
        Tint::Door,
        Tint::Encounter,
    ];

    fn slot(self) -> usize {
        Tint::ALL
            .iter()
            .position(|tint| *tint == self)
            .expect("every tint is in ALL")
    }

    /// The stylesheet's colour for this tint.
    pub fn colour(self) -> [f32; 3] {
        hex_rgb(TILE_TINTS[self.slot()].1)
    }

    /// The class the DOM board spells it with.
    pub fn class(self) -> &'static str {
        TILE_TINTS[self.slot()].0
    }
}

/// Kinds a map may carry and still get a legible shroud. Past it the table
/// cannot hold the doubled half: `2(K + 6) + 1 <= 64`.
pub const SHROUDED_KIND_LIMIT: usize = (MAX_TERRAIN_MATERIALS - 1) / 2 - Tint::ALL.len();

/// What the board's state says about every tile, as one snapshot.
///
/// Sets rather than per-cell grids: an overlay covers a handful of tiles out
/// of a board, and equality is what decides whether the ground is regrown, so
/// the cheapest thing to compare is what is actually on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overlays {
    pub selected: Option<TileCoord>,
    pub reach: BTreeSet<TileCoord>,
    pub path: BTreeSet<TileCoord>,
    pub template: BTreeSet<TileCoord>,
    pub doors: BTreeSet<TileCoord>,
    pub encounters: BTreeSet<TileCoord>,
    /// Explored but out of sight: remembered terrain under the shroud.
    pub dim: BTreeSet<TileCoord>,
    /// Never seen: the DOM board draws no element at all, and the scene lays
    /// no voxel, so both show the pane behind.
    pub hidden: BTreeSet<TileCoord>,
    /// The highest elevation drawn; everything above it is cut away.
    pub focus: Option<i32>,
}

impl Overlays {
    /// What the board is currently showing, read off the state exactly as
    /// `board::tiles::ground_tiles` reads it.
    ///
    /// Cheap enough for every frame: the sets are small, the fog walk is one
    /// pass over the ground layer, and the campaign lookups are the same two
    /// the DOM board already does per frame.
    pub fn of(ui: &UiState) -> Self {
        let playing = ui.mode == EditMode::Play;
        let path: BTreeSet<TileCoord> = if playing {
            ui.hover_tile
                .filter(|at| ui.reach.contains_key(at))
                .map(|at| path_to(&ui.reach, at).into_iter().collect())
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        let reach = if playing {
            ui.reach.keys().copied().collect()
        } else {
            BTreeSet::new()
        };
        let (mut dim, mut hidden) = (BTreeSet::new(), BTreeSet::new());
        if ui.fog_active() {
            for (col, row, kind) in ui.map.ground.iter() {
                if kind.0 == 0 {
                    continue;
                }
                let at = (col as i32, row as i32);
                match ui.fog_level(at) {
                    FogLevel::Clear => {},
                    FogLevel::Dim => {
                        dim.insert(at);
                    },
                    FogLevel::Hidden => {
                        hidden.insert(at);
                    },
                }
            }
        }
        Self {
            selected: ui.selected,
            reach,
            path,
            template: ui.template_preview().into_iter().collect(),
            doors: ui.door_tiles().into_iter().collect(),
            encounters: encounter_tiles(ui),
            dim,
            hidden,
            focus: ui.focus_elevation,
        }
    }

    /// The tint a tile wears, in the stylesheet's own precedence.
    ///
    /// Read off the two encounter rules directly: `.tile.tile-door
    /// .tile-encounter` carries the door's colour, and `.tile.tile-encounter`
    /// outranks every single-class rule, so a door wins outright and an
    /// encounter site wins over everything below it.
    pub fn tint(&self, at: TileCoord) -> Option<Tint> {
        if self.doors.contains(&at) {
            Some(Tint::Door)
        } else if self.encounters.contains(&at) {
            Some(Tint::Encounter)
        } else if self.template.contains(&at) {
            Some(Tint::Template)
        } else if self.path.contains(&at) {
            Some(Tint::Path)
        } else if self.reach.contains(&at) {
            Some(Tint::Reach)
        } else if self.selected == Some(at) {
            Some(Tint::Selected)
        } else {
            None
        }
    }
}

impl Overlays {
    /// Whether a piece is not on the board at all: it stands on ground the
    /// viewer has never seen, or above the focus elevation.
    ///
    /// Cut rather than clipped. A focus is a plane laid on a top face, and a
    /// body standing on that face straddles it, so a geometric cut would take
    /// every token on the focus layer off at the ankles. What is above the
    /// focus is simply not drawn.
    pub fn cuts(&self, map: &MapDocument, token: &Token) -> bool {
        if self.hidden.contains(&token.at) {
            return true;
        }
        let Some(focus) = self.focus else {
            return false;
        };
        let elevation = i32::from(
            *map.elevation
                .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
                .unwrap_or(&0),
        );
        elevation > focus
    }
}

/// Authored encounter sites on the active campaign map.
///
/// `UiState::authored_site_labels_at` answers this per tile and builds two
/// display strings doing it, which is the DOM board's shape rather than a
/// layer's; here the anchors are read once.
fn encounter_tiles(ui: &UiState) -> BTreeSet<TileCoord> {
    let Some(active) = ui.active_map.as_ref() else {
        return BTreeSet::new();
    };
    ui.campaign_maps
        .get(active)
        .map(|map| {
            map.encounter_anchors
                .iter()
                .map(|anchor| (anchor.at.col as i32, anchor.at.row as i32))
                .collect()
        })
        .unwrap_or_default()
}

/// The material layout one map's palette is built on: how many kinds it has,
/// and whether the shrouded half fits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardPalette {
    kinds: usize,
    shrouded_half: bool,
}

impl BoardPalette {
    pub fn of(map: &MapDocument) -> Self {
        let kinds = map.tile_kinds.len();
        Self {
            kinds,
            shrouded_half: kinds <= SHROUDED_KIND_LIMIT,
        }
    }

    /// The material a tint is drawn as.
    pub fn tint_material(&self, tint: Tint) -> u8 {
        (self.kinds + 1 + tint.slot()) as u8
    }

    /// The same material under the fog shroud. Where the doubled half does not
    /// fit, every shrouded tile takes one flat entry instead.
    pub fn shrouded_material(&self, material: u8) -> u8 {
        if !self.shrouded_half {
            return self.flat_shroud();
        }
        material + (self.kinds + Tint::ALL.len()) as u8
    }

    /// The one entry a board with too many kinds shrouds everything with.
    fn flat_shroud(&self) -> u8 {
        (self.kinds + 1 + Tint::ALL.len()) as u8
    }

    /// How many entries the table holds.
    pub fn len(&self) -> usize {
        let plain = 1 + self.kinds + Tint::ALL.len();
        if self.shrouded_half {
            2 * (self.kinds + Tint::ALL.len()) + 1
        } else {
            plain + 1
        }
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

/// The map's material palette: the tileset stylesheet's kind colours, the
/// state tints beside them, and the shrouded twin of both.
///
/// Entry 0 is the unknown colour, which is black: a kind the stylesheet does
/// not colour reads as the same nothing an empty column does, rather than as a
/// plausible material.
pub fn terrain_palette(map: &MapDocument) -> TerrainPalette {
    let layout = BoardPalette::of(map);
    let mut colours = vec![[0.0, 0.0, 0.0]];
    for kind in &map.tile_kinds {
        colours.push(tile_kind_colour(kind).unwrap_or([0.0, 0.0, 0.0]));
    }
    colours.truncate(layout.kinds + 1);
    for tint in Tint::ALL {
        colours.push(tint.colour());
    }
    if layout.shrouded_half {
        let plain: Vec<[f32; 3]> = colours[1..].to_vec();
        colours.extend(plain.into_iter().map(shrouded));
    } else {
        colours.push(shrouded([0.0, 0.0, 0.0]));
    }
    colours.truncate(MAX_TERRAIN_MATERIALS);
    TerrainPalette::new(colours)
}

#[cfg(test)]
mod tests;
