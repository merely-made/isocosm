//! Where the board sits in the scene's world, and the camera that frames it.
//!
//! B1 fixed the ground: map `(col, row)` is terrain `(x, z)` centred on the
//! origin, one voxel per height unit. This file is the rest of that
//! convention — the camera that reproduces the DOM board's own pixels from it,
//! and the two conversions the parity gate turns on.
//!
//! **Scale.** [`SlabCamera::dimetric_2_1`]'s doc derives the projection: with
//! `k` pixels per world unit, a world `+x` unit moves `(k / sqrt2, k / 2 sqrt2)`
//! pixels and a world `+z` unit `(-k / sqrt2, k / 2 sqrt2)`. The DOM board
//! moves `(tile_w / 2, tile_h / 2)` per column and `(-tile_w / 2, tile_h / 2)`
//! per row, so one tile is one world unit exactly when
//!
//! ```text
//! k = tile_w * sqrt(2) / 2 = 16 * sqrt(2)      (the shipped 32-wide tile)
//! ```
//!
//! and then the row axis and the tile height fall out with it. [`WORLD_PX`] is
//! that `k`. An orthographic frame of height `h` pixels shows `h / k` world
//! units, which is [`half_height`](BoardWorld::camera)'s whole content.
//!
//! **The elevation deviation.** The same derivation puts a world `+y` unit at
//! `k * cos(30) = 19.6` pixels up the screen, while the DOM board's
//! `elev_step` is 8. A voxel is a cube and B1 laid one voxel per height unit,
//! so the scene board's cliffs stand about 2.45 times taller than the DOM
//! board's. The preset's doc names the world height that *would* match
//! (`tile_len * (elev_step / tile_h) * sqrt(2/3)`, about 0.41 voxels), which a
//! cubic ground cannot express. Recorded for the plan rather than fudged here:
//! the horizontal projection, which is what a tile pick reads, agrees exactly.

use isometer::{Cutaway, SlabCamera};
use isometry_core::{IsoGeometry, MapDocument, TileCoord};

use super::MapTerrain;

/// Pixels per world unit at the board's shipped 32 by 16 tile: the scale that
/// makes one tile one world unit under the dimetric preset.
pub const WORLD_PX: f32 = std::f32::consts::SQRT_2 * 16.0;

/// Screen pixels one world unit of height rises: `WORLD_PX * cos(30)`. This is
/// the number the DOM board spells `elev_step` and sets to 8; see the module
/// note on the deviation.
pub const ELEVATION_PX: f32 = WORLD_PX * COS_30;

/// `cos(30)`, the factor the projection divides a world vertical by.
const COS_30: f32 = 0.866_025_4;

/// World y of the top face of an elevation-0 tile. `grow` lays the surface
/// voxel *at* the elevation, so a flat cell occupies `[0, 1)` and the surface a
/// token stands on — and a pick lands on — is one voxel up.
const GROUND_TOP: f32 = 1.0;

/// Half a voxel: the offset from a terrain column's own index to the point the
/// board calls that tile's centre.
const CELL_CENTRE: f32 = 0.5;

/// Pixels per world unit under `geo`, which carries the DOM board's own scale.
/// [`WORLD_PX`] is this at the shipped geometry.
pub fn world_px(geo: &IsoGeometry) -> f32 {
    geo.tile_w * std::f32::consts::SQRT_2 / 2.0
}

/// The board's placement in the scene's world: the map-to-terrain offset B1
/// fixed, plus the projection constants the camera and the picks share.
#[derive(Clone, Copy, Debug)]
pub struct BoardWorld {
    /// Map cell of terrain column `(0, 0)`, exactly [`MapTerrain`]'s.
    origin: (i32, i32),
    extent: i32,
    /// The tallest elevation on the map, for the slab's depth and its box.
    ceiling: f32,
}

impl BoardWorld {
    pub fn new(map: &MapDocument) -> Self {
        let terrain = MapTerrain::new(map);
        let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
        let ceiling = map
            .elevation
            .iter()
            .map(|(_, _, e)| *e as f32)
            .fold(0.0, f32::max);
        Self {
            origin: (w / 2, h / 2),
            extent: terrain.extent(),
            ceiling,
        }
    }

    /// The world point at a tile's diamond centre, standing on its surface:
    /// the top face of the voxel `grow` laid for elevation `elevation`.
    ///
    /// [`CELL_CENTRE`] is the half the DOM board has no name for: a terrain
    /// column indexed `(x, z)` *occupies* `[x, x+1) x [z, z+1)`, while a tile
    /// is a diamond about a point. Drop it and the whole board draws half a
    /// tile up the screen.
    pub fn stand(&self, (col, row): TileCoord, elevation: i32) -> [f32; 3] {
        [
            (col - self.origin.0) as f32 + CELL_CENTRE,
            elevation as f32 + GROUND_TOP,
            (row - self.origin.1) as f32 + CELL_CENTRE,
        ]
    }

    /// The map cell a terrain voxel belongs to, whatever its height. This is
    /// the pick's half of B1's convention, and it is integer arithmetic on the
    /// hit's own `voxel` rather than a round of its float point, so a hit on a
    /// cell boundary cannot land in the neighbour.
    pub fn tile_of_voxel(&self, voxel: [i32; 3]) -> TileCoord {
        (voxel[0] + self.origin.0, voxel[2] + self.origin.1)
    }

    /// The world point a board-space screen point names, on the plane flat
    /// ground presents: the top faces of the elevation-0 tiles.
    ///
    /// Board space is the DOM board's own: tile `(0, 0)` at elevation 0 sits at
    /// `(0, 0)` and `geo` is the projection. The inverse is
    /// [`IsoGeometry::screen_to_tile`]'s arithmetic without the rounding, so
    /// the camera lands between tiles as readily as on one.
    ///
    /// The point is taken at [`GROUND_TOP`] rather than at world zero, and
    /// that is the whole of what lines the two boards up: the centre and a
    /// flat tile's top face are then on one plane, the projection's height
    /// term cancels between them, and a flat tile lands on exactly the pixels
    /// the DOM board draws it on. An elevated tile does not, by
    /// [`ELEVATION_PX`] per step — the deviation the module header states.
    fn ground_point(&self, geo: &IsoGeometry, (x, y): (f32, f32)) -> [f32; 3] {
        let a = x / (geo.tile_w / 2.0);
        let b = y / (geo.tile_h / 2.0);
        [
            (a + b) / 2.0 - self.origin.0 as f32 + CELL_CENTRE,
            GROUND_TOP,
            (b - a) / 2.0 - self.origin.1 as f32 + CELL_CENTRE,
        ]
    }

    /// The camera for one board pane: the DOM board's pan and viewport, read
    /// as a dimetric slab over the same world.
    ///
    /// `size` is the texture the scene draws into and `pane` the pane's own
    /// logical box, which are the same numbers at render scale 1 and differ by
    /// exactly the scale otherwise. `camera` is the state's own board-origin
    /// offset inside the pane.
    ///
    /// [`Cutaway::Bounds`] is the map's own box, grown by a tile on every side
    /// so a token standing at the edge is not clipped by the ground it stands
    /// on. Nothing outside the box can reach a pixel anyway; the cut is what
    /// keeps a later focus elevation (B4) a change of one field.
    pub fn camera(
        &self,
        geo: &IsoGeometry,
        camera: (f32, f32),
        pane: (f32, f32),
        size: [u32; 2],
    ) -> Option<SlabCamera> {
        let centre = self.ground_point(geo, (pane.0 / 2.0 - camera.0, pane.1 / 2.0 - camera.1));
        // One scene pixel is one board pixel: the frame shows as many world
        // units as it has pixels, over the projection's own scale.
        let half_height = size[1].max(1) as f32 / (2.0 * world_px(geo));
        let aspect = size[0].max(1) as f32 / size[1].max(1) as f32;
        // Deep enough that the slab reaches the whole map from any centre
        // inside it, with the ceiling and a tile of margin on top.
        let depth = 4.0 * (self.extent as f32 + self.ceiling + 8.0);
        let reach = self.extent as f32 + 1.0;
        Some(
            SlabCamera::dimetric_2_1(centre, half_height, aspect, depth)?.with_cutaway(Some(
                Cutaway::Bounds {
                    min: [-reach, -2.0, -reach],
                    max: [reach, self.ceiling + 4.0, reach],
                },
            )),
        )
    }
}
