// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A local planar development of the world's square-site topology. A sphere
//! needs cuts when unfolded. Never place two unrelated edges against one
//! another. Only the lightweight site index is planet-wide; a renderer asks
//! for column recipes for the tiles its camera reaches.
use super::*;
use crate::{Border, Footprint, border::across_cell};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, PartialEq, Eq)]
struct Skeleton {
    seed: u64,
    elevation: i64,
    relief: i64,
    water: i64,
    borders: [Option<(SiteId, Border)>; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub site: SiteId,
    origin: [i64; 2],
    axes: [[i64; 2]; 2],
}
impl Tile {
    /// Tile-local chart coordinates to native site cells.
    pub fn local(self, [x, z]: [i64; 2]) -> [i64; 2] {
        [0, 1].map(|i| self.origin[i] + x * self.axes[0][i] + z * self.axes[1][i])
    }
    fn adjacent(self, chart: &Chart, delta: [i64; 2]) -> Option<Self> {
        let n = chart.side();
        let p = self.local(delta.map(|d| d * n + n / 2));
        let side = if p[0] < 0 {
            3
        } else if p[0] >= n {
            1
        } else if p[1] < 0 {
            0
        } else {
            2
        };
        let (site, border) = chart.border(self.site, side)?;
        let cross = |x, z| {
            let [x, z] = self.local([delta[0] * n + x, delta[1] * n + z]);
            let [x, _, z] = across_cell([x, 0, z], border, n);
            [x, z]
        };
        let origin = cross(0, 0);
        let axes = [cross(1, 0), cross(0, 1)].map(|p| [p[0] - origin[0], p[1] - origin[1]]);
        Some(Self { site, origin, axes })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Chart {
    seed: u64,
    footprint: Footprint,
    materials: Materials,
    skeleton: BTreeMap<SiteId, Skeleton>,
    tiles: BTreeMap<[i64; 2], Tile>,
    edits: Vec<(SiteId, Edit)>,
    depth: i64,
}
impl Chart {
    fn new(a: &impl Atlas, root: SiteId, edits: &[(SiteId, Edit)], depth: i64) -> Result<Self> {
        let mut skeleton = BTreeMap::new();
        for id in a.sites() {
            skeleton.insert(
                id,
                Skeleton {
                    seed: a.terrain_seed(id)?,
                    elevation: a.elevation(id)?,
                    relief: a.relief(id)?,
                    water: a.water(id)?,
                    borders: std::array::from_fn(|i| a.border(id, i as u8)),
                },
            );
        }
        let mut chart = Self {
            seed: a.seed(),
            footprint: a.footprint(),
            materials: a.materials()?,
            skeleton,
            tiles: BTreeMap::new(),
            edits: edits.to_vec(),
            depth,
        };
        let first = Tile {
            site: root,
            origin: [0; 2],
            axes: [[1, 0], [0, 1]],
        };
        chart.tiles.insert([0; 2], first);
        let mut queue = VecDeque::from([([0; 2], first)]);
        let mut placed = BTreeSet::from([root]);
        const STEPS: [[i64; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
        while let Some(([x, z], tile)) = queue.pop_front() {
            for [dx, dz] in STEPS {
                let pos = [x + dx, z + dz];
                if chart.tiles.contains_key(&pos) {
                    continue;
                }
                let Some(next) = tile.adjacent(&chart, [dx, dz]) else {
                    continue;
                };
                if placed.contains(&next.site) {
                    continue;
                }
                if STEPS.iter().any(|&[dx, dz]| {
                    chart
                        .tiles
                        .get(&[pos[0] + dx, pos[1] + dz])
                        .is_some_and(|other| next.adjacent(&chart, [dx, dz]) != Some(*other))
                }) {
                    continue;
                }
                placed.insert(next.site);
                chart.tiles.insert(pos, next);
                queue.push_back((pos, next));
            }
        }
        Ok(chart)
    }
    pub fn side(&self) -> i64 {
        self.footprint.side as i64
    }
    /// The site under a chart cell measured from the root site's centre.
    pub fn ground_site(&self, cell: [i64; 2]) -> Option<SiteId> {
        let side = self.side();
        self.tile(cell.map(|v| (v + side / 2).div_euclid(side)))
            .map(|tile| tile.site)
    }
    pub fn tile(&self, pos: [i64; 2]) -> Option<Tile> {
        self.tiles.get(&pos).copied()
    }
    pub fn visible(
        &self,
        lo: [i64; 2],
        hi: [i64; 2],
    ) -> impl Iterator<Item = ([i64; 2], Tile)> + '_ {
        self.tiles
            .range([lo[0], i64::MIN]..=[hi[0], i64::MAX])
            .filter(move |(p, _)| (lo[1]..=hi[1]).contains(&p[1]))
            .map(|(&p, &v)| (p, v))
    }
    pub fn volume(&self, site: SiteId) -> Result<Volume> {
        Volume::paged_with(
            self,
            site,
            [0; 2],
            [self.side(); 2],
            self.depth,
            &self.edits,
            false,
        )
    }
}
impl Atlas for Chart {
    fn seed(&self) -> u64 {
        self.seed
    }
    fn footprint(&self) -> Footprint {
        self.footprint
    }
    fn sites(&self) -> Vec<SiteId> {
        self.skeleton.keys().copied().collect()
    }
    fn terrain_seed(&self, s: SiteId) -> Result<u64> {
        Ok(self.skeleton[&s].seed)
    }
    fn elevation(&self, s: SiteId) -> Result<i64> {
        Ok(self.skeleton[&s].elevation)
    }
    fn relief(&self, s: SiteId) -> Result<i64> {
        Ok(self.skeleton[&s].relief)
    }
    fn water(&self, s: SiteId) -> Result<i64> {
        Ok(self.skeleton[&s].water)
    }
    fn materials(&self) -> Result<Materials> {
        Ok(self.materials)
    }
    fn border(&self, s: SiteId, b: u8) -> Option<(SiteId, Border)> {
        *self.skeleton.get(&s)?.borders.get(b as usize)?
    }
}
impl Volume {
    pub fn charted(
        a: &impl Atlas,
        site: SiteId,
        depth: i64,
        edits: &[(SiteId, Edit)],
    ) -> Result<Self> {
        let side = a.footprint().side as i64;
        let mut volume = Self::paged(a, site, [0; 2], [side; 2], depth, edits)?;
        if a.footprint().sides == 4 {
            volume.chart = Some(Arc::new(Chart::new(a, site, edits, depth)?));
        }
        Ok(volume)
    }
}

#[cfg(test)]
#[path = "chart_tests.rs"]
mod tests;
