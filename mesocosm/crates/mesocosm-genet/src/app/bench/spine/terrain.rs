// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP3's bounded presentation adapter. The founded world remains unchanged;
//! only the selected window is lowered into Isometer's palette and Ground.

use isocosm::{
    Founding,
    map::{Grid, Layout},
    simulation::Genesis,
    terrain::View,
};
use isometer::core::ground::{Ground, Terrain};
use isometer::space::{Atlas, CHUNK, Chunk};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Receipt {
    pub seed: u64,
    pub grid: Grid,
    pub sites: [u64; 2],
    pub level: u8,
    pub origin: [i64; 2],
    pub columns: [u32; 2],
    pub datum: i64,
    pub border_digests: [String; 2],
    pub border_equal: bool,
    pub perturbed: bool,
    pub material_keys: Vec<String>,
    pub material_samples: u64,
    pub material_sample_digests: [String; 2],
    pub filled_voxels: u64,
    pub occupied_bricks: u64,
    pub brick_bound: u64,
    /// Conservative 8 KiB per brick allowance for Ground, map, upload and GPU copies.
    pub estimated_resident_bytes: u64,
    pub world_before: String,
    pub world_after: String,
}

pub struct Window {
    columns: [u32; 2],
    cells: Vec<Column>,
    pub datum: i64,
    pub extent: i32,
    pub height: i32,
}

#[derive(Clone, Copy)]
struct Column {
    top: i64,
    water: i64,
    soil: i64,
}

impl Column {
    fn material_at(self, y: i64) -> u8 {
        if y > self.top {
            if y <= self.water { 1 } else { 0 }
        } else if y > self.top - self.soil {
            2
        } else {
            3
        }
    }
}

pub struct Landscape {
    pub ground: Ground,
    pub window: Window,
    pub receipt: Receipt,
}

pub fn draw(seed: u64, overview: bool, perturbed: bool) -> Result<Landscape, String> {
    let grid = Grid::drawn(seed);
    let founding = Founding {
        seed,
        sites: grid.width * grid.height,
        population: 32,
        map: Some(Layout::Grid(grid.clone())),
        ..Default::default()
    };
    let world = founding.generate()?;
    build(&world, grid, overview, perturbed)
}

fn digest(bytes: &[u8]) -> String {
    // A stable receipt checksum; equality is also checked on the exact samples.
    let hash = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    format!("{hash:016x}")
}

fn build(
    world: &Genesis,
    grid: Grid,
    overview: bool,
    perturbed: bool,
) -> Result<Landscape, String> {
    let before = serde_json::to_vec(world).map_err(|e| e.to_string())?;
    let near = View::of(world)?;
    let (right, border) = near
        .border(0, 1)
        .ok_or("first site has no east neighbour")?;
    let mut changed = world.clone();
    if perturbed {
        // One side is lifted from a changed skeleton while its neighbour still
        // reads the original. This is deliberately inconsistent presentation.
        let key = &world
            .rules
            .skeleton
            .as_ref()
            .ok_or("no skeleton")?
            .elevation;
        *changed
            .sites
            .get_mut(&right)
            .ok_or("missing neighbour")?
            .conditions
            .get_mut(key)
            .ok_or("no elevation")? += 64;
    }
    let far = View::of(&changed)?;
    let a = near.lattice(0)?;
    let b = far.lattice(right)?;
    let samples = |l: &isometer::space::Lattice, side: u8, reverse: bool| {
        let mut bytes = l.denominator().to_le_bytes().to_vec();
        for t in 0..=grid.side {
            bytes.extend_from_slice(
                &l.on_side(side, if reverse { grid.side - t } else { t })
                    .to_le_bytes(),
            );
        }
        bytes
    };
    let borders = [
        samples(&a, 1, false),
        samples(&b, border.enters, !border.flipped),
    ];
    let mut level = 0u8;
    if overview {
        while grid.side.div_ceil(1u64 << level) > 64 {
            level += 1;
        }
    }
    let side = grid.side.div_ceil(1u64 << level) as u32;
    let columns = if overview { [side * 2, side] } else { [32, 32] };
    let origin = if overview {
        [0, 0]
    } else {
        [side as i64 - 16, side as i64 / 2 - 16]
    };
    let mut chunks: BTreeMap<(u64, [u32; 2]), Chunk> = BTreeMap::new();
    let mut cells = Vec::new();
    let mut expected_materials = Vec::new();
    let mut actual_materials = Vec::new();
    for z in 0..columns[1] {
        for x in 0..columns[0] {
            let gx = origin[0] + i64::from(x);
            let gz = origin[1] + i64::from(z);
            let (site, sx, view) = if gx < i64::from(side) {
                (0, gx as u32, near)
            } else {
                (right, gx as u32 - side, far)
            };
            let sz = gz as u32;
            let at = [sx / CHUNK, sz / CHUNK];
            if !chunks.contains_key(&(site, at)) {
                chunks.insert((site, at), view.lift(site, level, at)?);
            }
            let c = &chunks[&(site, at)];
            // Resolve by the world's keys, never assume its numeric material ids.
            for (id, key) in [
                (c.materials.air, "world:air"),
                (c.materials.water, "world:water"),
                (c.materials.soil, "world:soil"),
                (c.materials.rock, "world:rock"),
            ] {
                if world
                    .world
                    .materials
                    .get(id as usize)
                    .is_none_or(|m| m.key != key)
                {
                    return Err("lift material does not resolve in its world".into());
                }
            }
            let top = c.surface[((sz % CHUNK) * c.columns[0] + sx % CHUNK) as usize];
            let column = Column {
                top,
                water: c.water,
                soil: c.soil,
            };
            // Check each column at the water, soil and rock transitions against
            // the actual lift, including air. Soil thickness may differ across
            // the border; palette meaning must not.
            for y in [
                top + 1,
                c.water + 1,
                c.water,
                top,
                top - c.soil + 1,
                top - c.soil,
            ] {
                let source = c.material(sx % CHUNK, sz % CHUNK, y);
                let expected = match world.world.materials[source as usize].key.as_str() {
                    "world:air" => 0,
                    "world:water" => 1,
                    "world:soil" => 2,
                    "world:rock" => 3,
                    _ => return Err("unmapped lift material".into()),
                };
                let actual = column.material_at(y);
                if expected != actual {
                    return Err("lowered material differs from its source lift".into());
                }
                expected_materials.push(expected);
                actual_materials.push(actual);
            }
            cells.push(column);
        }
    }
    let datum = cells
        .iter()
        .map(|c| c.top - c.soil)
        .min()
        .ok_or("empty terrain window")?;
    let highest = cells.iter().map(|c| c.top.max(c.water)).max().unwrap();
    let height = i32::try_from(highest - datum)
        .map_err(|_| "window height exceeds the presentation range")?;
    if height > 8192 {
        return Err("terrain window exceeds the bench's vertical allocation bound".into());
    }
    let filled_voxels: u64 = cells
        .iter()
        .map(|c| (c.top.max(c.water) - datum + 1) as u64)
        .sum();
    // Ground fills from zero. Count each x/z brick's tallest occupied column
    // before allocation, using the same centered Euclidean addressing.
    let mut brick_heights = BTreeMap::<[i32; 2], i64>::new();
    for (index, column) in cells.iter().enumerate() {
        let x = index as i32 % columns[0] as i32 - columns[0] as i32 / 2;
        let z = index as i32 / columns[0] as i32 - columns[1] as i32 / 2;
        let height = column.top.max(column.water) - datum;
        brick_heights
            .entry([x.div_euclid(8), z.div_euclid(8)])
            .and_modify(|h| *h = (*h).max(height))
            .or_insert(height);
    }
    let occupied_bricks: u64 = brick_heights.values().map(|h| (h / 8 + 1) as u64).sum();
    let capacity = isometer::lens::AtlasLimits::DEFAULT.max_bricks() as u64;
    if occupied_bricks > capacity {
        return Err(format!(
            "terrain window needs {occupied_bricks} bricks; scene capacity is {capacity}"
        ));
    }
    let brick_bound = (u64::from(columns[0]).div_ceil(8) + 1)
        * (u64::from(columns[1]).div_ceil(8) + 1)
        * (height as u64 + 1).div_ceil(8);
    let estimated_resident_bytes = brick_bound * 8192;
    if filled_voxels > 8 * 1024 * 1024 || estimated_resident_bytes > 128 * 1024 * 1024 {
        return Err(format!(
            "terrain window exceeds bench budget: {filled_voxels} filled voxels, {estimated_resident_bytes} estimated resident bytes"
        ));
    }
    let extent = columns[0].max(columns[1]).div_ceil(2) as i32;
    let window = Window {
        columns,
        cells,
        datum,
        extent,
        height,
    };
    let ground = Ground::grow_with(&window, extent, |x, z, depth| window.material(x, z, depth));
    debug_assert_eq!(ground.brick_count() as u64, occupied_bricks);
    let after = serde_json::to_vec(world).map_err(|e| e.to_string())?;
    Ok(Landscape {
        ground,
        window,
        receipt: Receipt {
            seed: world.seed,
            grid,
            sites: [0, right],
            level,
            origin,
            columns,
            datum,
            border_digests: [digest(&borders[0]), digest(&borders[1])],
            border_equal: borders[0] == borders[1],
            perturbed,
            material_keys: world
                .world
                .materials
                .iter()
                .map(|m| m.key.clone())
                .collect(),
            material_samples: expected_materials.len() as u64,
            material_sample_digests: [digest(&expected_materials), digest(&actual_materials)],
            filled_voxels,
            occupied_bricks,
            brick_bound,
            estimated_resident_bytes,
            world_before: digest(&before),
            world_after: digest(&after),
        },
    })
}

impl Window {
    fn column(&self, x: i32, z: i32) -> Option<Column> {
        let x = x + self.columns[0] as i32 / 2;
        let z = z + self.columns[1] as i32 / 2;
        if x < 0 || z < 0 || x >= self.columns[0] as i32 || z >= self.columns[1] as i32 {
            return None;
        }
        Some(self.cells[(z as u32 * self.columns[0] + x as u32) as usize])
    }
    fn material(&self, x: i32, z: i32, depth: i32) -> u8 {
        let Some(c) = self.column(x, z) else {
            return 0;
        };
        let y = c.top.max(c.water) - i64::from(depth);
        c.material_at(y)
    }
}

impl Terrain for Window {
    fn sea_level(&self, _: i32) -> i32 {
        0
    }
    fn surface(&self, _: i32, x: i32, z: i32) -> i32 {
        self.column(x, z)
            .map_or(-1, |c| (c.top.max(c.water) - self.datum) as i32)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
