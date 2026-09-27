//! The board's ground made on demand: one column per tile, and any brick built
//! from the columns it covers when the frame asks for it.
//!
//! B4 raised a whole `Ground` on every change, which at 256 by 256 is 1.6
//! million voxel columns: 0.4 to 1.2 s in release and 13 to 15 s in the debug
//! build for a one-tile tint, before a single byte reached the tracer. A tile
//! is uniform, though — one surface, one top material, one body material under
//! it — so the board keeps one record per tile and makes a brick from the
//! records under its footprint.
//!
//! **Parity by construction.** A record is [`MapTerrain`]'s own `surface` and
//! `material` read at the tile's low corner, and [`BoardBricks`] lays
//! `0..=surface` from it exactly as `Ground::grow_with` does, so a made brick
//! is the grown one to the byte. `columns_tests` holds that over the demo, the
//! stress board and a board of relief, with overlays, fog and a focus.
//!
//! **Edits, per tile.** [`TileColumns::changed`] compares the new records with
//! the held ones and names every brick a changed tile's footprint reaches, up
//! to the higher of its two surfaces. A held brick in that set is remade and
//! refreshed; one the frame does not hold is remade from the records as they
//! stand when it next comes into view.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::ops::Range;
use std::time::{Duration, Instant};

use isometer::BrickSource;
use isometer::core::ground::{BRICK, MAX_MATERIAL, Terrain};
use isometry_core::MapDocument;

use super::overlay::Overlays;
use super::terrain::{MapTerrain, VOID_SURFACE, VOXELS_PER_TILE};

/// One tile's column of voxels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Column {
    /// World y of the topmost voxel. Below zero lays nothing.
    surface: i32,
    /// The top voxel's material, with its overlay tint and shroud.
    top: u8,
    /// The material of every voxel under the top one.
    body: u8,
}

/// A column that lays no voxel: off the map, empty, or never seen.
const VOID: Column = Column {
    surface: VOID_SURFACE,
    top: 0,
    body: 0,
};

/// The board's ground as one column per tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileColumns {
    width: i32,
    height: i32,
    /// Map cell whose low corner is terrain column `(0, 0)`, as
    /// [`MapTerrain`] places it.
    origin: (i32, i32),
    cells: Vec<Column>,
    /// The highest surface on the board, below zero when nothing stands.
    tallest: i32,
}

/// What one change did to the columns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColumnChange {
    /// Tiles whose column moved.
    pub tiles: usize,
    /// Every brick those tiles reach, sorted.
    pub bricks: Vec<[i16; 3]>,
}

impl TileColumns {
    /// Reads every tile of `map`, with the board's overlays, as a column.
    pub fn of(map: &MapDocument, overlays: &Overlays) -> Self {
        let terrain = MapTerrain::with_overlays(map, Some(overlays));
        let (width, height) = terrain.size();
        let extent = terrain.extent();
        let mut cells = Vec::with_capacity(width as usize * height as usize);
        let mut tallest = VOID_SURFACE;
        for row in 0..height {
            for col in 0..width {
                let (x, z) = terrain.column(col, row);
                let surface = terrain.surface(extent, x, z);
                let column = if surface < 0 {
                    VOID
                } else {
                    Column {
                        surface,
                        top: terrain.material(x, z, 0).min(MAX_MATERIAL),
                        body: terrain.material(x, z, 1).min(MAX_MATERIAL),
                    }
                };
                tallest = tallest.max(column.surface);
                cells.push(column);
            }
        }
        let (x, z) = terrain.column(0, 0);
        Self {
            width: width as i32,
            height: height as i32,
            origin: (-x / VOXELS_PER_TILE, -z / VOXELS_PER_TILE),
            cells,
            tallest,
        }
    }

    /// The column of map cell `(col, row)`, `None` off the map.
    fn cell(&self, col: i32, row: i32) -> Option<&Column> {
        let inside = (0..self.width).contains(&col) && (0..self.height).contains(&row);
        inside.then(|| &self.cells[(row * self.width + col) as usize])
    }

    /// The column a terrain voxel column `(x, z)` stands in.
    fn at(&self, x: i32, z: i32) -> Option<&Column> {
        self.cell(
            x.div_euclid(VOXELS_PER_TILE) + self.origin.0,
            z.div_euclid(VOXELS_PER_TILE) + self.origin.1,
        )
    }

    /// The low corner of map cell `(col, row)` in terrain voxels.
    fn corner(&self, col: i32, row: i32) -> (i32, i32) {
        (
            (col - self.origin.0) * VOXELS_PER_TILE,
            (row - self.origin.1) * VOXELS_PER_TILE,
        )
    }

    /// The bricks a change from `before` to these columns touched, or `None`
    /// when the two are different boards — another size or placement — and no
    /// tile can be matched with the one it replaced.
    pub fn changed(&self, before: &TileColumns) -> Option<ColumnChange> {
        if (self.width, self.height, self.origin) != (before.width, before.height, before.origin) {
            return None;
        }
        let mut change = ColumnChange::default();
        let mut bricks = BTreeSet::new();
        let span =
            |from: i32| from.div_euclid(BRICK)..=(from + VOXELS_PER_TILE - 1).div_euclid(BRICK);
        for (index, (now, was)) in self.cells.iter().zip(&before.cells).enumerate() {
            if now == was {
                continue;
            }
            change.tiles += 1;
            let top = now.surface.max(was.surface);
            if top < 0 {
                continue;
            }
            let index = index as i32;
            let (x, z) = self.corner(index % self.width, index / self.width);
            for kz in span(z) {
                for kx in span(x) {
                    for ky in 0..=top.div_euclid(BRICK) {
                        bricks.insert([kx as i16, ky as i16, kz as i16]);
                    }
                }
            }
        }
        change.bricks = bricks.into_iter().collect();
        Some(change)
    }
}

/// The columns as the scene's [`BrickSource`], cut at a focus elevation's
/// ceiling, keeping count of the time spent making bricks.
pub struct BoardBricks<'a> {
    columns: &'a TileColumns,
    /// The highest voxel a focus keeps; `None` keeps the whole board.
    ceiling: Option<i32>,
    made: Cell<Duration>,
}

impl<'a> BoardBricks<'a> {
    pub fn new(columns: &'a TileColumns, ceiling: Option<i32>) -> Self {
        Self {
            columns,
            ceiling,
            made: Cell::new(Duration::ZERO),
        }
    }

    /// Time spent in [`BrickSource::fill`] since this source was made.
    pub fn fill_time(&self) -> Duration {
        self.made.get()
    }

    /// A column's topmost kept voxel.
    fn top(&self, surface: i32) -> i32 {
        self.ceiling.map_or(surface, |ceiling| surface.min(ceiling))
    }
}

impl BrickSource for BoardBricks<'_> {
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        let top = self.top(self.columns.tallest);
        if top < 0 {
            return None;
        }
        let key = |voxel: i32| voxel.div_euclid(BRICK) as i16;
        let low = self.columns.corner(0, 0);
        let high = self.columns.corner(self.columns.width, self.columns.height);
        Some([
            [key(low.0), 0, key(low.1)],
            [key(high.0 - 1), key(top), key(high.1 - 1)],
        ])
    }

    fn layers(&self, column: [i16; 2]) -> Range<i16> {
        let [x, z] = column.map(|key| i32::from(key) * BRICK);
        let cells = |from: i32, origin: i32| {
            from.div_euclid(VOXELS_PER_TILE) + origin
                ..=(from + BRICK - 1).div_euclid(VOXELS_PER_TILE) + origin
        };
        let mut top = -1;
        for row in cells(z, self.columns.origin.1) {
            for col in cells(x, self.columns.origin.0) {
                if let Some(cell) = self.columns.cell(col, row) {
                    top = top.max(self.top(cell.surface));
                }
            }
        }
        if top < 0 {
            0..0
        } else {
            0..top.div_euclid(BRICK) as i16 + 1
        }
    }

    fn fill(&self, key: [i16; 3], out: &mut [u8]) {
        let started = Instant::now();
        out.fill(0);
        let [x0, y0, z0] = key.map(|k| i32::from(k) * BRICK);
        for z in 0..BRICK {
            for x in 0..BRICK {
                let Some(column) = self.columns.at(x0 + x, z0 + z) else {
                    continue;
                };
                let top = self.top(column.surface);
                for y in y0.max(0)..=top.min(y0 + BRICK - 1) {
                    let material = if y == column.surface {
                        column.top
                    } else {
                        column.body
                    };
                    out[(((y - y0) * BRICK + z) * BRICK + x) as usize] = material;
                }
            }
        }
        self.made.set(self.made.get() + started.elapsed());
    }
}
