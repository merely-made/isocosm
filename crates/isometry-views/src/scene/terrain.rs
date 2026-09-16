//! A map document read as terrain: the board's elevation grid grown into a
//! brick `Ground`, its tile kinds carried as materials.
//!
//! **The subdivided grid.** A voxel is a cube, so the only way a cubic ground
//! can hold a cliff shallower than a tile is to make the tile several voxels
//! across. [`VOXELS_PER_TILE`] and [`VOXELS_PER_STEP`] are Mark's ruling of
//! 2026-09-15 and the whole of the scale: a tile is 5 voxels square and an
//! elevation step is 2 voxels tall.
//!
//! **Coordinates.** The map's `(col, row)` is the terrain's `(x, z)`, centred
//! on the origin, because `Ground::grow` lays bricks from `-extent` to
//! `+extent` on both axes. A cell's low corner is at
//! `x = (col - width / 2) * VOXELS_PER_TILE`, and the cell owns the
//! `VOXELS_PER_TILE` square from there, every voxel of it the cell's own
//! material. Column and row order are preserved, so the board's axes stay the
//! terrain's `+x` and `+z` and no handedness flips on the way in. `extent` is
//! the largest voxel index any of the four map edges reaches, so the whole map
//! fits inside the square and the padding around it is the void below.
//!
//! **Height.** [`VOXELS_PER_STEP`] voxels per height unit: a cell of elevation
//! `e` has its surface voxel at world y [`surface_of`]`(e)` = `2e + 1`, with
//! `grow` filling `0..=2e+1` beneath it, so a flat cell is two voxels deep and
//! its top face is at world y 2.
//!
//! **Sea level and the void.** Sea level is [`SEA_LEVEL`] = -2, one elevation
//! step under the lowest ground, so every painted cell is dry land. A column
//! outside the map, and a column whose ground kind is 0 (empty), takes
//! [`VOID_SURFACE`] = -4: below sea level *and* below y 0, so `grow` lays no
//! voxel there at all and the column reads as water or void rather than as a
//! black tile.
//!
//! **Materials.** A cell's material is its ground [`TileKindId`] plus one,
//! since material 0 is air and palette entry 0 is the unknown colour. Props
//! are not terrain: voxel-recipe props become bodies in a later lane, and
//! [`MapTerrain::prop_count`] is all this adapter says about them.

use isometer_core::ground::{Ground, MAX_MATERIAL, Terrain};
use isometer_lens::{MAX_TERRAIN_MATERIALS, TerrainPalette};
use isometry_core::{MapDocument, TileKindId};

use crate::theme::tile_kind_colour;

/// Voxels along one side of a tile. Ruled 2026-09-15 with [`VOXELS_PER_STEP`].
///
/// The pair exists to express a cliff shorter than a tile on a grid of cubes.
/// Under the 2:1 dimetric a world unit of height rises `k cos30` pixels while
/// a tile of ground is `k sqrt2` pixels wide, so with `t` voxels to a tile the
/// height that rises the DOM board's `elev_step` = 8 px against its
/// `tile_h` = 16 px is
///
/// ```text
/// step = t * (elev_step / tile_h) * sqrt(2 / 3) = t * 0.4082
/// ```
///
/// One voxel per tile wants 0.408 of a voxel and cannot have it: a step of 1
/// projects 19.6 px, 2.45 times the DOM's. `t = 5` wants 2.041 voxels, so
/// [`VOXELS_PER_STEP`] = 2 gives a ratio of 0.400 that projects 7.84 px —
/// within a fifth of a pixel of the DOM's step, and within two thirds of a
/// pixel over the demo map's four-step height range. The cost is 25 times the
/// columns; the gain is sub-tile terrain the voxel appearance can use later.
pub const VOXELS_PER_TILE: i32 = 5;

/// Voxels one elevation step rises. See [`VOXELS_PER_TILE`] for the ratio.
pub const VOXELS_PER_STEP: i32 = 2;

/// World y of the water line: one elevation step under an elevation-0 cell.
pub const SEA_LEVEL: i32 = -VOXELS_PER_STEP;

/// The surface an empty or off-map column reports: below [`SEA_LEVEL`], and
/// below the bedrock line, so nothing is laid there.
pub const VOID_SURFACE: i32 = SEA_LEVEL - VOXELS_PER_STEP;

/// The world y of the topmost voxel of a column at height unit `elevation`.
/// Its top *face* is one voxel higher, which is `world::GROUND_TOP` at
/// elevation 0.
pub fn surface_of(elevation: i32) -> i32 {
    elevation * VOXELS_PER_STEP + VOXELS_PER_STEP - 1
}

/// A map document seen as terrain, with the map-to-world offset it was built
/// with. Borrowed, not owned: the document stays the board's.
pub struct MapTerrain<'a> {
    map: &'a MapDocument,
    extent: i32,
    /// Map cell of the tile whose low corner is terrain column `(0, 0)`.
    origin: (i32, i32),
}

impl<'a> MapTerrain<'a> {
    pub fn new(map: &'a MapDocument) -> Self {
        let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
        let origin = (w / 2, h / 2);
        // The bound must reach both ends of both axes, not just the wider one,
        // and on the high side it must reach the far corner of the last tile's
        // footprint rather than its low corner.
        let span = |n: i32, c: i32| {
            (c * VOXELS_PER_TILE)
                .max((n - c) * VOXELS_PER_TILE - 1)
                .max(0)
        };
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

    /// The terrain column at a map cell's low corner. The cell owns the
    /// [`VOXELS_PER_TILE`] square running `+x` and `+z` from here.
    pub fn column(&self, col: u32, row: u32) -> (i32, i32) {
        (
            (col as i32 - self.origin.0) * VOXELS_PER_TILE,
            (row as i32 - self.origin.1) * VOXELS_PER_TILE,
        )
    }

    /// The map cell a terrain column falls in, `None` outside the map.
    pub fn cell(&self, x: i32, z: i32) -> Option<(u32, u32)> {
        let col = x.div_euclid(VOXELS_PER_TILE) + self.origin.0;
        let row = z.div_euclid(VOXELS_PER_TILE) + self.origin.1;
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
                surface_of(self.map.elevation.get(col, row).copied().unwrap_or(0) as i32)
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
