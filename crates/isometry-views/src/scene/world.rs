//! Where the board sits in the scene's world, and the camera that frames it.
//!
//! B1 fixed the ground; the subdivision ruling of 2026-09-15 rescaled it. A
//! tile is [`VOXELS_PER_TILE`] world units square and an elevation step is
//! [`VOXELS_PER_STEP`] of them tall. This file is the rest of that convention —
//! the camera that reproduces the DOM board's own pixels from it, and the two
//! conversions the parity gate turns on.
//!
//! **Scale.** [`SlabCamera::dimetric_2_1`]'s doc derives the projection: with
//! `k` pixels per world unit, a world `+x` unit moves `(k / sqrt2, k / 2 sqrt2)`
//! pixels and a world `+z` unit `(-k / sqrt2, k / 2 sqrt2)`. The DOM board
//! moves `(tile_w / 2, tile_h / 2)` per column and `(-tile_w / 2, tile_h / 2)`
//! per row, so a tile of `t` world units lands right exactly when
//!
//! ```text
//! k = tile_w * sqrt(2) / 2 / t = 16 * sqrt(2) / 5   (the shipped 32-wide tile)
//! ```
//!
//! and then the row axis and the tile height fall out with it. [`WORLD_PX`] is
//! that `k`. An orthographic frame of height `h` pixels shows `h / k` world
//! units, which is [`half_height`](BoardWorld::camera)'s whole content.
//!
//! **The elevation step.** The same derivation puts a world `+y` unit at
//! `k cos(30)` pixels up the screen, so an elevation step of
//! [`VOXELS_PER_STEP`] rises [`ELEVATION_PX`] = 7.84 px against the DOM
//! board's `elev_step` of 8 — the ruled ratio, pinned by a receipt in
//! `parity_tests`. Before the subdivision a step was one cubic voxel and stood
//! 19.6 px, 2.45 times the DOM's.

use isometer::{Cutaway, SlabCamera};
use isometry_core::{IsoGeometry, MapDocument, TileCoord};

use super::MapTerrain;
use super::terrain::{VOXELS_PER_STEP, VOXELS_PER_TILE};

/// Pixels per world unit — per *voxel* — at the board's shipped 32 by 16 tile.
pub const WORLD_PX: f32 = std::f32::consts::SQRT_2 * 16.0 / VOXELS_PER_TILE as f32;

/// Screen pixels one elevation step rises at the shipped geometry:
/// `WORLD_PX * VOXELS_PER_STEP * cos(30)`. This is the number the DOM board
/// spells `elev_step` and sets to 8; the ruling accepts the 0.16 px it misses
/// by.
pub const ELEVATION_PX: f32 = WORLD_PX * VOXELS_PER_STEP as f32 * COS_30;

/// `cos(30)`, the factor the projection divides a world vertical by.
const COS_30: f32 = 0.866_025_4;

/// World y of the top face of an elevation-0 tile. `grow` lays the surface
/// voxel *at* `surface_of(0)`, so a flat cell occupies `[0, 2)` and the
/// surface a token stands on — and a pick lands on — is one step up.
const GROUND_TOP: f32 = VOXELS_PER_STEP as f32;

/// Half a tile: the offset from a tile's low corner to the point the board
/// calls that tile's centre.
const CELL_CENTRE: f32 = VOXELS_PER_TILE as f32 / 2.0;

/// Pixels per world unit under `geo`, which carries the DOM board's own scale.
/// [`WORLD_PX`] is this at the shipped geometry.
pub fn world_px(geo: &IsoGeometry) -> f32 {
    geo.tile_w * std::f32::consts::SQRT_2 / 2.0 / VOXELS_PER_TILE as f32
}

/// Screen pixels one elevation step rises under `geo`. [`ELEVATION_PX`] is
/// this at the shipped geometry, and `geo.elev_step` is what it answers to.
pub fn elevation_px(geo: &IsoGeometry) -> f32 {
    world_px(geo) * VOXELS_PER_STEP as f32 * COS_30
}

/// The board's placement in the scene's world: the map-to-terrain offset B1
/// fixed, plus the projection constants the camera and the picks share.
#[derive(Clone, Copy, Debug)]
pub struct BoardWorld {
    /// Map cell of the tile whose low corner is terrain column `(0, 0)`,
    /// exactly [`MapTerrain`]'s.
    origin: (i32, i32),
    /// The resident bound, in voxels.
    extent: i32,
    /// World y of the top face of the tallest tile, for the slab's depth and
    /// its box.
    ceiling: f32,
}

impl BoardWorld {
    pub fn new(map: &MapDocument) -> Self {
        let terrain = MapTerrain::new(map);
        let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
        let tallest = map
            .elevation
            .iter()
            .map(|(_, _, e)| *e as f32)
            .fold(0.0, f32::max);
        Self {
            origin: (w / 2, h / 2),
            extent: terrain.extent(),
            ceiling: tallest * VOXELS_PER_STEP as f32 + GROUND_TOP,
        }
    }

    /// The world point at a tile's diamond centre, standing on its surface:
    /// the top face of the voxels `grow` laid for elevation `elevation`.
    ///
    /// [`CELL_CENTRE`] is the half the DOM board has no name for: a tile
    /// *occupies* a [`VOXELS_PER_TILE`] square of columns from its low corner,
    /// while a tile is a diamond about a point. Drop it and the whole board
    /// draws half a tile up the screen.
    pub fn stand(&self, (col, row): TileCoord, elevation: i32) -> [f32; 3] {
        [
            ((col - self.origin.0) * VOXELS_PER_TILE) as f32 + CELL_CENTRE,
            (elevation * VOXELS_PER_STEP) as f32 + GROUND_TOP,
            ((row - self.origin.1) * VOXELS_PER_TILE) as f32 + CELL_CENTRE,
        ]
    }

    /// The map cell a terrain voxel belongs to, whatever its height. This is
    /// the pick's half of the convention, and it is integer arithmetic on the
    /// hit's own `voxel` rather than a round of its float point, so a hit on a
    /// cell boundary cannot land in the neighbour.
    pub fn tile_of_voxel(&self, voxel: [i32; 3]) -> TileCoord {
        (
            voxel[0].div_euclid(VOXELS_PER_TILE) + self.origin.0,
            voxel[2].div_euclid(VOXELS_PER_TILE) + self.origin.1,
        )
    }

    /// The height unit a terrain voxel belongs to. Both voxels of a step
    /// answer the same elevation, so a hit on a cliff face names the step it
    /// cuts through.
    pub fn elevation_of_voxel(&self, voxel: [i32; 3]) -> i32 {
        voxel[1].div_euclid(VOXELS_PER_STEP)
    }

    /// The world point a board-space screen point names, on the plane flat
    /// ground presents: the top faces of the elevation-0 tiles.
    ///
    /// Board space is the DOM board's own: tile `(0, 0)` at elevation 0 sits at
    /// `(0, 0)` and `geo` is the projection. The inverse is
    /// [`IsoGeometry::screen_to_tile`]'s arithmetic without the rounding, so
    /// the camera lands between tiles as readily as on one, and the tile-unit
    /// answer is scaled into voxels last.
    ///
    /// The point is taken at [`GROUND_TOP`] rather than at world zero, and
    /// that is the whole of what lines the two boards up: the centre and a
    /// flat tile's top face are then on one plane, the projection's height
    /// term cancels between them, and a flat tile lands on exactly the pixels
    /// the DOM board draws it on. An elevated tile misses by the ruling's
    /// `elev_step - ELEVATION_PX` per step, under a fifth of a pixel.
    fn ground_point(&self, geo: &IsoGeometry, (x, y): (f32, f32)) -> [f32; 3] {
        let a = x / (geo.tile_w / 2.0);
        let b = y / (geo.tile_h / 2.0);
        let tile = VOXELS_PER_TILE as f32;
        [
            ((a + b) / 2.0 - self.origin.0 as f32) * tile + CELL_CENTRE,
            GROUND_TOP,
            ((b - a) / 2.0 - self.origin.1 as f32) * tile + CELL_CENTRE,
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
        let tile = VOXELS_PER_TILE as f32;
        // Deep enough that the slab reaches the whole map from any centre
        // inside it, with the ceiling and a few tiles of margin on top.
        let depth = 4.0 * (self.extent as f32 + self.ceiling + 8.0 * tile);
        let reach = self.extent as f32 + tile;
        Some(
            SlabCamera::dimetric_2_1(centre, half_height, aspect, depth)?.with_cutaway(Some(
                Cutaway::Bounds {
                    min: [-reach, -2.0 * tile, -reach],
                    max: [reach, self.ceiling + 4.0 * tile, reach],
                },
            )),
        )
    }
}
