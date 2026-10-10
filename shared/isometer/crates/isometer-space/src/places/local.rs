// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Local re-derivation (ruling 14): after an edit, only the places its
//! dirty region reaches are derived again. A cell's own column decides
//! whether it is a stance, its headroom and its roof, so the region is the
//! dirty columns and their neighbours. Membership is not local: a flood
//! from the region can run into places it never touched, a filled cave
//! mouth can seal a room far away, and a patch's cut depends on its whole
//! component. So the affected set grows until no flood meets an untouched
//! place, and every patch cut from an affected component comes with it.

use super::cells::Cells;
use super::{Kind, PlaceId, Places, flood};
use crate::Result;
use crate::volume::Volume;
use std::collections::BTreeSet;

impl Places {
    /// Re-derives the places `dirty` reaches, each box `[lo, hi)` of base
    /// cells as `Volume::drain_dirty` gives them.
    pub fn rederive(&mut self, volume: &Volume, dirty: &[[[i64; 3]; 2]]) -> Result<()> {
        if volume.site != self.site {
            return Err("a volume of another site".into());
        }
        let cells = Cells::new(volume, self.rules);
        let mut columns = BTreeSet::new();
        for [lo, hi] in dirty {
            for x in lo[0] - 1..hi[0] + 1 {
                for z in lo[2] - 1..hi[2] + 1 {
                    columns.insert([x, z]);
                }
            }
        }
        let region = cells.columns(columns.iter().copied());
        let mut affected: BTreeSet<PlaceId> =
            region.iter().filter_map(|&a| self.label(a)).collect();
        let found = loop {
            let components: BTreeSet<[i64; 3]> = affected
                .iter()
                .filter_map(|id| self.places.get(id))
                .filter(|p| p.kind == Kind::Patch)
                .map(|p| p.component)
                .collect();
            affected.extend(
                self.places
                    .values()
                    .filter(|p| p.kind == Kind::Patch && components.contains(&p.component))
                    .map(|p| p.id),
            );
            let mut seeds = region.clone();
            seeds.extend(
                self.labels
                    .iter()
                    .filter(|(_, id)| affected.contains(id))
                    .map(|(k, _)| [k[0], k[2], k[1]]),
            );
            let found = flood::components(&cells, seeds);
            let before = affected.len();
            for f in &found {
                affected.extend(f.cells.iter().filter_map(|&a| self.label(a)));
            }
            if affected.len() == before {
                break found;
            }
        };
        self.labels.retain(|_, id| !affected.contains(id));
        self.places.retain(|id, _| !affected.contains(id));
        self.passages
            .retain(|between, _| !between.iter().any(|id| affected.contains(id)));
        self.insert(&cells, found);
        Ok(())
    }
}
