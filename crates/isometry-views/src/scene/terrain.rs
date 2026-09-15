//! A map document read as terrain: the board's elevation grid grown into a
//! brick `Ground`, its tile kinds carried as materials.
//!
//! **Coordinates.** The map's `(col, row)` is the terrain's `(x, z)`, centred
//! on the origin, because `Ground::grow` lays bricks from `-extent` to
//! `+extent` on both axes: `x = col - width / 2`, `z = row - height / 2`.
//! Column and row order are preserved, so the board's axes stay the terrain's
//! `+x` and `+z` and no handedness flips on the way in. `extent` is the
//! largest of the four half-spans, so the whole map fits inside the square and
//! the padding around it is the void described below.
//!
//! **Height.** One voxel per height unit (the DOM board's `elev_step`): a cell
//! of elevation `e` has its surface voxel at world y `e`, with `grow` filling
//! y `0..=e` beneath it.
//!
//! **Sea level and the void.** Sea level is [`SEA_LEVEL`] = -1, one voxel under
//! the lowest ground, so every painted cell is dry land. A column outside the
//! map, and a column whose ground kind is 0 (empty), takes [`VOID_SURFACE`] =
//! -2: below sea level *and* below y 0, so `grow` lays no voxel there at all
//! and the column reads as water or void rather than as a black tile.
//!
//! **Materials.** A cell's material is its ground [`TileKindId`] plus one,
//! since material 0 is air and palette entry 0 is the unknown colour. Props
//! are not terrain: voxel-recipe props become bodies in a later lane, and
//! [`MapTerrain::prop_count`] is all this adapter says about them.

use isometer_core::ground::{Ground, MAX_MATERIAL, Terrain};
use isometer_lens::{MAX_TERRAIN_MATERIALS, TerrainPalette};
use isometry_core::{MapDocument, TileKindId};

use crate::theme::tile_kind_colour;

/// World y of the water line: one voxel under an elevation-0 cell.
pub const SEA_LEVEL: i32 = -1;

/// The surface an empty or off-map column reports: below [`SEA_LEVEL`], and
/// below the bedrock line, so nothing is laid there.
pub const VOID_SURFACE: i32 = -2;

/// A map document seen as terrain, with the map-to-world offset it was built
/// with. Borrowed, not owned: the document stays the board's.
pub struct MapTerrain<'a> {
    map: &'a MapDocument,
    extent: i32,
    /// Map cell of terrain column `(0, 0)`.
    origin: (i32, i32),
}

impl<'a> MapTerrain<'a> {
    pub fn new(map: &'a MapDocument) -> Self {
        let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
        let origin = (w / 2, h / 2);
        // The bound must reach both ends of both axes, not just the wider one.
        let span = |n: i32, c: i32| c.max(n - 1 - c).max(0);
        let extent = span(w, origin.0).max(span(h, origin.1));
        Self {
            map,
            extent,
            origin,
        }
    }

    /// The resident bound to hand `Ground::grow_with`.
    pub fn extent(&self) -> i32 {
        self.extent
    }

    /// The terrain column a map cell occupies.
    pub fn column(&self, col: u32, row: u32) -> (i32, i32) {
        (col as i32 - self.origin.0, row as i32 - self.origin.1)
    }

    /// The map cell a terrain column covers, `None` outside the map.
    pub fn cell(&self, x: i32, z: i32) -> Option<(u32, u32)> {
        let (col, row) = (x + self.origin.0, z + self.origin.1);
        self.map
            .ground
            .in_bounds(col, row)
            .then_some((col as u32, row as u32))
    }

    /// The material for a column: its ground kind plus one, or air where the
    /// column is empty or off the map.
    pub fn material(&self, x: i32, z: i32) -> u8 {
        self.cell(x, z)
            .and_then(|(col, row)| self.map.ground.get(col, row).copied())
            .map(material_of)
            .unwrap_or(0)
    }

    /// Raises the map's ground, materials and all.
    pub fn grow(&self) -> Ground {
        Ground::grow_with(self, self.extent, |x, z, _depth| self.material(x, z))
    }

    /// How many cells carry a prop. Props themselves are a later lane's.
    pub fn prop_count(&self) -> usize {
        self.map
            .props
            .iter()
            .filter(|(_, _, kind)| kind.0 != 0)
            .count()
    }
}

impl Terrain for MapTerrain<'_> {
    fn sea_level(&self, _extent: i32) -> i32 {
        SEA_LEVEL
    }

    fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
        match self.cell(x, z) {
            Some((col, row)) if self.map.ground.get(col, row).is_some_and(|k| k.0 != 0) => {
                self.map.elevation.get(col, row).copied().unwrap_or(0) as i32
            },
            _ => VOID_SURFACE,
        }
    }
}

/// The material a ground tile kind is laid as. Kind 0 is empty and never
/// reaches a voxel; everything else is shifted by one past air.
pub fn material_of(kind: TileKindId) -> u8 {
    let material = kind.0 as usize + 1;
    debug_assert!(
        material <= MAX_MATERIAL as usize,
        "tile kind {} is past the palette bound",
        kind.0
    );
    material.min(MAX_MATERIAL as usize) as u8
}

/// The map's material palette, built from the tileset stylesheet's own colours.
///
/// Entry 0 is the unknown colour, which is black: a kind the stylesheet does
/// not colour reads as the same nothing an empty column does, rather than as a
/// plausible material. Kinds past the table's 64 entries are dropped, which is
/// the same bound [`material_of`] clamps at.
pub fn terrain_palette(map: &MapDocument) -> TerrainPalette {
    let mut colours = vec![[0.0, 0.0, 0.0]];
    for kind in &map.tile_kinds {
        colours.push(tile_kind_colour(kind).unwrap_or([0.0, 0.0, 0.0]));
    }
    colours.truncate(MAX_TERRAIN_MATERIALS);
    TerrainPalette::new(colours)
}
