// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A window of a site realized as voxels in `Ground`, where isometer applies
//! edits (rulings 696 and 733). `Ground` is the store until nisus sits under
//! this API; its revision and dirty bricks are what places re-derive from.
//!
//! Ground's columns run `-extent..=extent` from `y = 0`, so the window is
//! offset into them and lifted by a datum: everything below the datum reads
//! as rock, the bedrock `Ground` already assumes there.

use crate::edit::{Edit, SiteRule, reaching, runs, touched};
use crate::lift::Materials;
use crate::{Atlas, Result, SiteId};
use isometer_core::ground::{BRICK, Ground, Terrain};

/// A realized window of one site's base cells.
#[derive(Clone, Debug)]
pub struct Volume {
    pub site: SiteId,
    /// Base columns from `min` up to but excluding `max`, in `(x, z)`.
    pub min: [i64; 2],
    pub max: [i64; 2],
    /// The height, in base units, that `Ground` holds at `y = 0`.
    pub datum: i64,
    pub materials: Materials,
    extent: i32,
    ground: Ground,
}

/// The window's columns as `Ground::grow_with` reads them.
struct Columns<'a> {
    rule: &'a SiteRule,
    tops: Vec<i64>,
    min: [i64; 2],
    wide: i64,
    extent: i64,
    datum: i64,
}

impl Columns<'_> {
    fn top(&self, gx: i32, gz: i32) -> Option<i64> {
        let x = i64::from(gx) + self.extent;
        let z = i64::from(gz) + self.extent;
        let deep = self.tops.len() as i64 / self.wide;
        (0..self.wide).contains(&x).then_some(())?;
        (0..deep).contains(&z).then(|| self.tops[(z * self.wide + x) as usize])
    }
}

impl Terrain for Columns<'_> {
    fn sea_level(&self, _extent: i32) -> i32 {
        (self.rule.water - self.datum) as i32
    }

    fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
        self.top(x, z)
            .map_or(-1, |top| (top.max(self.rule.water) - self.datum) as i32)
    }
}

impl Volume {
    /// Base columns `min..max` of `site` with `edits` replayed, holding
    /// `depth` cells of rock under the window's lowest soil.
    pub fn lift<A: Atlas + ?Sized>(
        a: &A,
        site: SiteId,
        min: [i64; 2],
        max: [i64; 2],
        depth: i64,
        edits: &[(SiteId, Edit)],
    ) -> Result<Self> {
        let side = a.footprint().side as i64;
        if (0..2).any(|k| min[k] < 0 || max[k] > side || min[k] >= max[k]) {
            return Err("a window outside its site".into());
        }
        let rule = SiteRule::of(a, site)?;
        let m = rule.m;
        if [m.air, m.water, m.soil, m.rock].iter().any(|&id| id > isometer_core::ground::MAX_MATERIAL) {
            return Err("a material id past what Ground holds".into());
        }
        if m.air != isometer_core::ground::AIR {
            return Err("air that Ground would not read as empty".into());
        }
        let wide = max[0] - min[0];
        let mut tops = Vec::new();
        for z in min[1]..max[1] {
            for x in min[0]..max[0] {
                tops.push(rule.top(x, z));
            }
        }
        let lowest = tops.iter().min().copied().unwrap_or(0);
        let extent = (wide.max(max[1] - min[1]) / 2) as i32;
        let columns = Columns {
            rule: &rule,
            tops,
            min,
            wide,
            extent: i64::from(extent),
            datum: lowest - rule.soil - depth.max(0),
        };
        let ground = Ground::grow_with(&columns, extent, |gx, gz, depth| {
            let top = columns.top(gx, gz).unwrap_or(0);
            rule.rule(top, top.max(rule.water) - i64::from(depth))
        });
        let mut volume = Self {
            site,
            min: columns.min,
            max,
            datum: columns.datum,
            materials: m,
            extent,
            ground,
        };
        let cells = touched(&reaching(a, site, edits), m, min, max, 1);
        volume.write(cells.into_iter().map(|(([x, z], y), m)| ([x, y, z], m)))?;
        volume.ground.drain_dirty();
        Ok(volume)
    }

    /// Applies one edit made in `made_in`, read across a border when that is
    /// a neighbour. Returns how many cells changed.
    pub fn apply<A: Atlas + ?Sized>(&mut self, a: &A, made_in: SiteId, edit: &Edit) -> Result<u32> {
        let mut cells = Vec::new();
        for copy in reaching(a, self.site, &[(made_in, edit.clone())]) {
            let [lo, hi] = copy.shape.extent();
            let material = copy.op.material(self.materials);
            for x in lo[0].max(self.min[0])..hi[0].min(self.max[0]) {
                for z in lo[2].max(self.min[1])..hi[2].min(self.max[1]) {
                    for [y0, y1] in runs(&copy.shape, x, z) {
                        cells.extend((y0..y1).map(|y| ([x, y, z], material)));
                    }
                }
            }
        }
        self.write(cells)
    }

    fn write(&mut self, cells: impl IntoIterator<Item = ([i64; 3], u8)>) -> Result<u32> {
        let voxels: Vec<_> = cells.into_iter().map(|(at, m)| (self.to_ground(at), m)).collect();
        self.ground
            .write(voxels)
            .map_err(|_| "an edit reaching below the window's datum".to_string())
    }

    fn to_ground(&self, [x, y, z]: [i64; 3]) -> [i32; 3] {
        let e = i64::from(self.extent);
        [
            (x - self.min[0] - e) as i32,
            (y - self.datum) as i32,
            (z - self.min[1] - e) as i32,
        ]
    }

    /// Whether base column `(x, z)` lies in the window.
    pub fn holds(&self, x: i64, z: i64) -> bool {
        (self.min[0]..self.max[0]).contains(&x) && (self.min[1]..self.max[1]).contains(&z)
    }

    /// The material of a base cell, `None` outside the window's columns.
    /// Below the datum is rock.
    pub fn material(&self, at: [i64; 3]) -> Option<u8> {
        if !self.holds(at[0], at[2]) {
            return None;
        }
        Some(self.ground.material(self.to_ground(at)).unwrap_or(self.materials.rock))
    }

    /// The highest cell `Ground` may hold anything in, plus one.
    pub fn ceiling(&self) -> i64 {
        let top = self.ground.keys().map(|k| i64::from(k[1])).max().unwrap_or(0);
        self.datum + (top + 1) * i64::from(BRICK)
    }

    pub fn ground(&self) -> &Ground {
        &self.ground
    }

    pub fn revision(&self) -> u64 {
        self.ground.revision()
    }

    /// The boxes of base cells changed since the last drain, `[lo, hi)`.
    pub fn drain_dirty(&mut self) -> Vec<[[i64; 3]; 2]> {
        let e = i64::from(self.extent);
        let b = i64::from(BRICK);
        self.ground
            .drain_dirty()
            .into_iter()
            .map(|k| {
                let lo = [
                    i64::from(k[0]) * b + self.min[0] + e,
                    i64::from(k[1]) * b + self.datum,
                    i64::from(k[2]) * b + self.min[1] + e,
                ];
                [lo, lo.map(|v| v + b)]
            })
            .collect()
    }

    /// Whether two volumes hold the same cells, whatever revisions brought
    /// them there.
    pub fn same_cells(&self, other: &Volume) -> bool {
        let frame = |v: &Volume| (v.site, v.min, v.max, v.datum, v.extent);
        if frame(self) != frame(other) {
            return false;
        }
        let air = [isometer_core::ground::AIR; (BRICK * BRICK * BRICK) as usize];
        fn raw<'a>(v: &'a Volume, k: [i16; 3], air: &'a [u8]) -> &'a [u8] {
            v.ground.brick_materials(k).map_or(air, |(b, _)| b.raw())
        }
        let keys: std::collections::BTreeSet<_> =
            self.ground.keys().chain(other.ground.keys()).collect();
        keys.into_iter().all(|k| raw(self, k, &air) == raw(other, k, &air))
    }
}
