// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A drawn grid of square sites, the tests' stand-in for a product's world.

use crate::{Atlas, Border, Footprint, Materials, Result, SiteId};

#[derive(Clone, Debug)]
pub struct Grid {
    pub seed: u64,
    pub width: u64,
    pub height: u64,
    pub side: u64,
    /// Opposite edges meet.
    pub torus: bool,
    /// Each site's elevation, relief and water, by id.
    pub skeleton: Vec<[i64; 3]>,
}

impl Grid {
    pub fn drawn(seed: u64, width: u64, height: u64, side: u64, torus: bool) -> Self {
        let r = |id: u64, k: u64, n: u64| (crate::draw(seed, "fixture", &[id, k]) % n) as i64;
        let skeleton = (0..width * height)
            .map(|id| {
                let elevation = side as i64 / 4 + r(id, 0, side / 8);
                let relief = 1 + r(id, 1, side / 8);
                [elevation, relief, elevation - 2 + r(id, 2, 4)]
            })
            .collect();
        Self {
            seed,
            width,
            height,
            side,
            torus,
            skeleton,
        }
    }

    fn get(&self, site: SiteId) -> Result<[i64; 3]> {
        self.skeleton.get(site as usize).copied().ok_or_else(|| "an absent site".into())
    }
}

impl Atlas for Grid {
    fn seed(&self) -> u64 {
        self.seed
    }

    fn footprint(&self) -> Footprint {
        Footprint {
            sides: 4,
            side: self.side,
        }
    }

    fn sites(&self) -> Vec<SiteId> {
        (0..self.width * self.height).collect()
    }

    fn terrain_seed(&self, site: SiteId) -> Result<u64> {
        self.get(site).map(|_| crate::draw(self.seed, "fixture-site", &[site]))
    }

    fn elevation(&self, site: SiteId) -> Result<i64> {
        Ok(self.get(site)?[0])
    }

    fn relief(&self, site: SiteId) -> Result<i64> {
        Ok(self.get(site)?[1])
    }

    fn water(&self, site: SiteId) -> Result<i64> {
        Ok(self.get(site)?[2])
    }

    fn border(&self, site: SiteId, side: u8) -> Option<(SiteId, Border)> {
        let (w, h) = (self.width as i64, self.height as i64);
        let (i, j) = ((site as i64) % w, (site as i64) / w);
        let (di, dj) = [(0, -1), (1, 0), (0, 1), (-1, 0)][side as usize % 4];
        let (mut ni, mut nj) = (i + di, j + dj);
        if self.torus {
            (ni, nj) = (ni.rem_euclid(w), nj.rem_euclid(h));
        }
        if !(0..w).contains(&ni) || !(0..h).contains(&nj) {
            return None;
        }
        let border = Border {
            side,
            enters: (side + 2) % 4,
            flipped: false,
        };
        Some(((nj * w + ni) as SiteId, border))
    }

    fn materials(&self) -> Result<Materials> {
        Ok(Materials {
            air: 0,
            water: 1,
            soil: 2,
            rock: 3,
        })
    }
}
