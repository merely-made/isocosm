// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! In-site space through isometer (rulings 670 and 696): the sim keeps each
//! edit as an asserted fact, and isometer lifts a site, applies the edits
//! and derives its places. Edits enter through the dev source for now,
//! outside the conserved total, and label the run assisted (ruling 412's
//! reading, as ruling 271 has it for placed matter): carving into a
//! carver's ledger waits on the world's densities.

use crate::{Result, schema::*, simulation::Simulation, terrain::View};
use isometer_space::{Atlas, Chunk, Edit, Op, edit::Moved, volume::Volume};

impl Simulation {
    /// The world's sites as isometer reads them, with conditions as they
    /// stand.
    pub fn atlas(&self) -> Result<View<'_>> {
        View::over(&self.genesis, &self.state.sites)
    }

    /// Asserts an edit to `site`'s volume, in its frame.
    pub fn edit(&mut self, site: Id, edit: Edit) -> Result<()> {
        let atlas = self.atlas()?;
        if !self.state.sites.contains_key(&site) {
            return Err("unknown site".into());
        }
        edit.check(atlas.footprint())?;
        if let Op::Fill(id) = edit.op
            && (usize::from(id) >= self.genesis.world.materials.len() || id == atlas.materials()?.air)
        {
            return Err("a fill of no material".into());
        }
        let tick = self.state.tick;
        self.state.edits.push(Edited { tick, site, edit });
        Ok(())
    }

    /// Every edit asserted, as isometer replays them.
    fn replayed(&self) -> Vec<(Id, Edit)> {
        self.state.edits.iter().map(|e| (e.site, e.edit.clone())).collect()
    }

    /// Chunk `at` of `site` at `level`, with every edit reaching it.
    pub fn lift(&self, site: Id, level: u8, at: [u32; 2]) -> Result<Chunk> {
        self.atlas()?.lift_edited(site, level, at, &self.replayed())
    }

    /// Base columns `min..max` of `site` realized with every edit, `depth`
    /// cells of rock held under the window's lowest soil.
    pub fn volume(&self, site: Id, min: [i64; 2], max: [i64; 2], depth: i64) -> Result<Volume> {
        Volume::lift(&self.atlas()?, site, min, max, depth, &self.replayed())
    }

    /// The cells of each material `edit` would move if asserted now.
    pub fn moved(&self, site: Id, edit: &Edit) -> Result<Moved> {
        isometer_space::edit::tally(&self.atlas()?, site, &self.replayed(), edit)
    }
}
