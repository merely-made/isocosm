// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Two disjoint bands of terrain bricks under one camera. Their boundary
//! snaps to an outer brick edge so promotion never leaves overlapping cells.
use super::{BrickSource, FramedBricks, coordinates, framed_bricks};
use crate::SlabCamera;
use isometer_space::volume::{Volume, chart::Tile};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ops::Range,
};

pub struct TerrainBands {
    root: Volume,
    tiles: BTreeMap<[i64; 2], (Tile, Volume)>,
    window: Option<[[i64; 2]; 2]>,
    layers: RefCell<BTreeMap<(i64, [i16; 2]), Range<i16>>>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BandStats {
    pub cell: [i64; 2],
    pub bricks: [usize; 2],
    pub sites: usize,
    pub centre_site: Option<u64>,
}
pub struct Band<'a> {
    world: &'a TerrainBands,
    cell: i64,
    // x/z bounds of the fine square, half open, relative to the root centre.
    inner: [[i64; 2]; 2],
    fine: bool,
    bounds: Option<[[i16; 3]; 2]>,
}
impl TerrainBands {
    pub fn new(root: Volume) -> Self {
        Self {
            root,
            tiles: BTreeMap::new(),
            window: None,
            layers: Default::default(),
        }
    }
    fn locate(&self, x: i64, z: i64) -> Option<(&Volume, [i64; 2])> {
        let side = self.root.max[0];
        let p = [x, z].map(|v| v + side / 2);
        let (tile, volume) = self.tiles.get(&p.map(|v| v.div_euclid(side)))?;
        Some((volume, tile.local(p.map(|v| v.rem_euclid(side)))))
    }
    fn prepare(&mut self, camera: SlabCamera, grain: i64) -> Result<(), String> {
        let chart = self
            .root
            .chart
            .as_ref()
            .ok_or("terrain bands need a site chart")?;
        let side = chart.side();
        let w = camera.window();
        let bounds = [-1., 1.].map(|sign| {
            [0, 2].map(|k| {
                let reach = (0..3).map(|a| w.axes[a][k].abs() * w.half[a]).sum::<f32>();
                ((w.centre[k] + sign * (reach + 16. * grain as f32 + 16.)) as i64 + side / 2)
                    .div_euclid(side)
            })
        });
        if self.window == Some(bounds) {
            return Ok(());
        }
        let [lo, hi] = bounds;
        self.tiles
            .retain(|p, _| (0..2).all(|i| (lo[i]..=hi[i]).contains(&p[i])));
        for (p, tile) in chart.visible(lo, hi) {
            if let std::collections::btree_map::Entry::Vacant(slot) = self.tiles.entry(p) {
                let volume = if tile.site == self.root.site {
                    self.root.clone()
                } else {
                    chart.volume(tile.site)?
                };
                slot.insert((tile, volume));
            }
        }
        self.window = Some(bounds);
        self.layers.get_mut().clear();
        Ok(())
    }
    /// Finest useful grain and two independent slot budgets. Only increase
    /// both grains when one of the two would omit visible bricks.
    pub fn frame(
        &mut self,
        camera: SlabCamera,
        initial: i64,
        capacity: [usize; 2],
    ) -> Result<([Band<'_>; 2], [FramedBricks; 2], BandStats), String> {
        self.prepare(camera, initial)?;
        // A long pan inside one enormous site must not retain every sampled
        // column. Keep at most a few frames of lookup work between evictions.
        if self.layers.get_mut().len() > 4 * (capacity[0] + capacity[1]).max(1) {
            self.layers.get_mut().clear();
        }
        let side = self.root.max[0];
        let mut low = [i64::MAX; 3];
        let mut high = [i64::MIN; 3];
        for (&[x, z], (_, v)) in &self.tiles {
            let a = [
                x * side - side / 2,
                v.datum - self.root.datum,
                z * side - side / 2,
            ];
            let b = [
                a[0] + side - 1,
                v.ceiling() - self.root.datum - 1,
                a[2] + side - 1,
            ];
            for i in 0..3 {
                low[i] = low[i].min(a[i]);
                high[i] = high[i].max(b[i]);
            }
        }
        let mut cell = initial.max(1);
        loop {
            let outer = 2 * cell;
            let edge = 8 * outer;
            // Snap the centre to the nearest outer-brick boundary. A square
            // at least one brick wide follows even a very close camera.
            let centre = [camera.centre[0], camera.centre[2]]
                .map(|v| (v / edge as f32).round() as i64 * edge);
            let radius = ((camera.half_height * 0.6 / edge as f32).ceil().max(1.) as i64) * edge;
            let inner = [centre.map(|v| v - radius), centre.map(|v| v + radius)];
            let bounds =
                |cell: i64| [low, high].map(|p| p.map(|v| i16::try_from(v.div_euclid(8 * cell))));
            if !self.tiles.is_empty()
                && bounds(cell)
                    .into_iter()
                    .flatten()
                    .any(|k| k.is_err() || k == Ok(i16::MAX))
            {
                cell *= 2;
                continue;
            }
            let bands = [true, false].map(|fine| {
                let cell = if fine { cell } else { outer };
                let mut b = bounds(cell).map(|p| p.map(|k| k.unwrap_or(0)));
                if fine {
                    for (j, k) in [(0, 0), (1, 2)] {
                        b[0][k] = b[0][k]
                            .max(inner[0][j].div_euclid(8 * cell).clamp(-32768, 32767) as i16);
                        b[1][k] =
                            b[1][k]
                                .min((inner[1][j] - 1).div_euclid(8 * cell).clamp(-32768, 32767)
                                    as i16);
                    }
                }
                Band {
                    world: self,
                    cell,
                    inner,
                    fine,
                    bounds: (!self.tiles.is_empty() && (0..3).all(|i| b[0][i] <= b[1][i]))
                        .then_some(b),
                }
            });
            let frames = std::array::from_fn(|i| {
                framed_bricks(
                    coordinates::camera(camera, bands[i].cell as f32),
                    8.0 / bands[i].cell as f32,
                    &bands[i],
                    capacity[i],
                )
            });
            if frames.iter().any(|f| f.overflow > 0) && cell < side.max(1) {
                cell *= 2;
                continue;
            }
            let mut sites = BTreeSet::new();
            for (band, frame) in bands.iter().zip(&frames) {
                let e = 8 * band.cell;
                for key in &frame.keys {
                    // Site edges need not coincide with brick edges.
                    for x in [0, e - band.cell] {
                        for z in [0, e - band.cell] {
                            if let Some((v, _)) =
                                self.locate(i64::from(key[0]) * e + x, i64::from(key[2]) * e + z)
                            {
                                sites.insert(v.site);
                            }
                        }
                    }
                }
            }
            let stats = BandStats {
                cell: [cell, outer],
                bricks: frames.each_ref().map(|f| f.keys.len()),
                sites: sites.len(),
                centre_site: self
                    .locate(
                        camera.centre[0].floor() as i64,
                        camera.centre[2].floor() as i64,
                    )
                    .map(|(v, _)| v.site),
            };
            return Ok((bands, frames, stats));
        }
    }
}
impl BrickSource for Band<'_> {
    fn cell_size(&self) -> f32 {
        self.cell as f32
    }
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        self.bounds
    }
    fn layers(&self, key: [i16; 2]) -> Range<i16> {
        let base = key.map(|v| i64::from(v) * 8 * self.cell);
        if (0..2).all(|i| (self.inner[0][i]..self.inner[1][i]).contains(&base[i])) != self.fine {
            return 0..0;
        }
        if let Some(r) = self.world.layers.borrow().get(&(self.cell, key)) {
            return r.clone();
        }
        let mut low = i64::MAX;
        let mut high = i64::MIN;
        for z in 0..8 {
            for x in 0..8 {
                if let Some((v, [x, z])) = self
                    .world
                    .locate(base[0] + x * self.cell, base[1] + z * self.cell)
                    && let Some(top) = v.surface(x, z)
                {
                    low = low.min(v.datum);
                    high = high.max(top);
                }
            }
        }
        let layer = |y: i64| (y - self.world.root.datum).div_euclid(8 * self.cell) as i16;
        let range = if low <= high {
            layer(low)..layer(high) + 1
        } else {
            0..0
        };
        self.world
            .layers
            .borrow_mut()
            .insert((self.cell, key), range.clone());
        range
    }
    fn fill(&self, key: [i16; 3], out: &mut [u8]) {
        out.fill(0);
        let [bx, by, bz] = key.map(|v| i64::from(v) * 8 * self.cell);
        let bottom = by + self.world.root.datum;
        for z in 0..8 {
            for x in 0..8 {
                let Some((v, [sx, sz])) = self.world.locate(bx + x * self.cell, bz + z * self.cell)
                else {
                    continue;
                };
                for (y, material) in v
                    .brick_column(sx, sz, bottom, self.cell)
                    .into_iter()
                    .enumerate()
                {
                    out[y * 64 + (z * 8 + x) as usize] = material;
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "bands_tests.rs"]
mod tests;
