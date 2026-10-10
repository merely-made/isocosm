// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! In-site space through isometer (rulings 670 and 696): the sim keeps each
//! edit as an asserted fact, and isometer lifts a site, applies the edits
//! and derives its places. A member's edit moves matter through its own
//! ledger by each material's density and account (rulings 412 and 739);
//! the dev source edits outside the conserved total and labels the run
//! assisted, and is the only editor of a material whose density is unset.
//! A member may also name the patch or room it stands in (740).

use crate::{
    Result,
    meaning::{credit, debit},
    schema::*,
    simulation::Simulation,
    terrain::View,
};
use isometer_space::{Atlas, Chunk, Edit, Op, edit::Moved, places::PlaceId, volume::Volume};

impl Simulation {
    /// The world's sites as isometer reads them, with conditions as they
    /// stand.
    pub fn atlas(&self) -> Result<View<'_>> {
        View::over(&self.genesis, &self.state.sites)
    }

    fn admit(&self, site: Id, edit: &Edit) -> Result<()> {
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
        Ok(())
    }

    /// Asserts an edit to `site`'s volume, in its frame, by the dev source.
    pub fn edit(&mut self, site: Id, edit: Edit) -> Result<()> {
        self.admit(site, &edit)?;
        let tick = self.state.tick;
        let by = None;
        self.state.edits.push(Edited { tick, site, edit, by });
        Ok(())
    }

    /// Asserts an edit by `actor`, standing in `site`: what it carves is
    /// credited to its ledger and what it fills is debited, each material
    /// by its density into its account. Refused whole if any material
    /// moved has no density, or the actor cannot pay for a fill. The
    /// actor is split out of its cohort, since its ledger now differs.
    pub fn edit_by(&mut self, actor: Id, site: Id, edit: Edit) -> Result<()> {
        self.admit(site, &edit)?;
        let member = self.state.population.get(actor).ok_or("unknown actor")?;
        if !member.alive || member.place != site {
            return Err("an actor not standing in the site".into());
        }
        let mut ledger = member.accounts.clone();
        let (mut gained, mut paid) = (Vec::new(), Vec::new());
        for (id, cells) in self.moved(site, &edit)? {
            let m = &self.genesis.world.materials[usize::from(id)];
            let (Some(density), Some(account)) = (m.density, &m.account) else {
                return Err(format!("{} has no density: the dev source only", m.key));
            };
            let amount = cells.unsigned_abs().checked_mul(density).ok_or("matter overflow")?;
            match cells > 0 {
                true => gained.push((account.clone(), amount)),
                false => paid.push((account.clone(), amount)),
            }
        }
        for (account, amount) in &gained {
            credit(&mut ledger, account, *amount)?;
        }
        for (account, amount) in &paid {
            debit(&mut ledger, account, *amount)?;
        }
        let sum = |legs: &[(Key, u64)]| legs.iter().map(|l| u128::from(l.1)).sum::<u128>();
        let conserved = (self.conserved + sum(&gained))
            .checked_sub(sum(&paid))
            .ok_or("matter underflow")?;
        self.state.population.lift(actor)?.accounts = ledger;
        self.conserved = conserved;
        let tick = self.state.tick;
        let by = Some(actor);
        self.state.edits.push(Edited { tick, site, edit, by });
        Ok(())
    }

    /// Names the patch or room `entity` stands in, or none. The member is
    /// split out of its cohort first, and the place must be in its site.
    /// The sim never sees a member move within its site, so its game
    /// names the new place as it moves it, and `None` returns it to the
    /// site; a move to another site clears it (ruling 740).
    pub fn set_patch(&mut self, entity: Id, patch: Option<PlaceId>) -> Result<()> {
        let member = self.state.population.get(entity).ok_or("unknown entity")?;
        if patch.is_some_and(|p| p.site != member.place) {
            return Err("a patch outside the entity's site".into());
        }
        self.state.population.lift(entity)?.patch = patch;
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
