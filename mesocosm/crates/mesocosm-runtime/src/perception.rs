// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! In-site sight over accepted native placement and edited terrain (740, 741).
//! Hosts supply actual cells and range. A patch's identity cell and a render
//! spot are never substituted for an eye or body position.

use std::collections::{BTreeMap, BTreeSet};
use isocosm::{Session, schema::Id, simulation::Simulation,
    directing::readings::{Mode, View}};
use isometer_space::{places::{Places, Rules}, volume::Volume};
use std::sync::Arc;
use isocosm::{operations::Window, spatial::{SiteGeometry, embodiment}};

/// One current native site's derived geometry. No second edit or placement log.
pub struct InSite {
    state_hash: u64,
    geometry: Arc<SiteGeometry>,
}

/// Exact caller geometry for a single observation, in base cells of its site.
/// This is a picture/query input, never new simulation position authority.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Sight {
    pub observer: Id,
    pub eye: [i64; 3],
    pub range: u64,
    pub targets: BTreeMap<Id, [i64; 3]>,
}

impl InSite {
    /// Rebuild from admitted edits and native facts after any accepted command.
    pub fn read(session: &Session, site: Id, min: [i64; 2], max: [i64; 2],
        depth: i64, rules: Rules) -> Result<Self, String> {
        Self::cached(&session.sim, site, &Window { min, max, depth }, rules)
    }

    pub(crate) fn cached(sim: &Simulation, site: Id, window: &Window, rules: Rules) -> Result<Self, String> {
        Ok(Self { state_hash:sim.state_hash(), geometry:sim.site_geometry(site, window, rules)? })
    }

    pub fn volume(&self) -> &Volume { &self.geometry.volume }
    pub fn places(&self) -> &Places { &self.geometry.places }
    pub fn site(&self) -> Id { self.volume().site }

    fn current(&self, sim: &Simulation) -> Result<(), String> {
        if self.state_hash != sim.state_hash() {
            return Err("in-site geometry is stale after native state changed".into());
        }
        Ok(())
    }

    pub(crate) fn cell(&self, sim: &Simulation, id: Id, at: [i64; 3]) -> Result<(), String> {
        if !self.volume().holds(at[0], at[2]) || at[1] < self.volume().datum
            || (!embodiment::enabled(sim) && at[1] >= self.volume().ceiling())
        { return Err("sight cell is outside the realized window".into()); }
        let e = sim.state().population.get(id).ok_or("sight names an absent member")?;
        if e.resident_site() != Some(self.site()) {
            return Err("sight member has no native residence at this site".into());
        }
        let patch = e.patch.ok_or("sight needs an admitted native patch")?;
        // Whole-site patch ids are local to the member's canonical window.
        // Validate there before casting through the observer's geometry.
        let owned = sim.genesis().rules.spatial_templates.as_ref().filter(|t| t.version >= 3);
        let admitted = if let Some(t) = owned {
            let resident = sim.state().spatial_poses.get(&id).ok_or("sight member has no native pose")?;
            if resident.site != self.site() || resident.arrived != e.arrived {
                return Err("sight pose belongs to another native visit".into());
            }
            let w = embodiment::site_window(t, resident.pose.at);
            let geometry = sim.site_geometry(self.site(), &w, t.places)?;
            geometry.places.place_at(resident.pose.at) == Some(patch)
                && geometry.places.place_at(at) == Some(patch)
        } else {
            self.places().places.contains_key(&patch) && self.places().place_at(at) == Some(patch)
        };
        if patch.site != self.site() || !admitted {
            return Err("sight cell is outside the member's current native patch".into());
        }
        let m = self.volume().materials;
        if !self.volume().material(at).is_some_and(|moved| moved == m.air || moved == m.water) {
            return Err("sight cell is outside the realized clear volume".into());
        }
        Ok(())
    }

    /// Place-scoped rays for the supplied geometry. All identities/cells are
    /// checked, even hidden candidates; malformed geometry grants no access.
    pub fn seen(&self, sim: &Simulation, sight: &Sight) -> Result<BTreeSet<Id>, String> {
        self.current(sim)?;
        if sight.range > i64::MAX as u64 { return Err("sight range exceeds base-cell bounds".into()); }
        let range = sight.range as i64;
        if self.places().places.values().any(|p| (0..3).any(|k|
            p.bounds[0][k].checked_sub(range).is_none()
                || p.bounds[1][k].checked_add(range).is_none())) {
            return Err("sight candidate bounds overflow".into());
        }
        self.cell(sim, sight.observer, sight.eye)?;
        if !sim.state().population.get(sight.observer).is_some_and(|e| e.alive) {
            return Err("sight observer is not living".into());
        }
        let mut seen = BTreeSet::from([sight.observer]);
        for (&id, &at) in &sight.targets {
            self.cell(sim, id, at)?;
            let span = (0..3).map(|k| sight.eye[k].abs_diff(at[k])).max().unwrap_or(0);
            if span.checked_mul(span).and_then(|n| n.checked_mul(2))
                .is_none_or(|n| n > i64::MAX as u64) {
                return Err("sight ray exceeds integer base-cell bounds".into());
            }
            if self.places().sees(self.volume(), sight.eye, at, sight.range) { seen.insert(id); }
        }
        Ok(seen)
    }

    /// Refine same-site bodies only. Events, visited sites and remembered
    /// remote subjects remain the native knowledge reading; no fact is added.
    /// Creative keeps native truth. Survival cannot invent missing body cells.
    pub fn refine(&self, sim: &Simulation, mode: Mode, sight: &Sight) -> Result<View, String> {
        let seen = self.seen(sim, sight)?;
        let mut view = isocosm::directing::readings::view(sim, sight.observer, mode);
        if mode == Mode::Creative { return Ok(view); }
        let mut things = BTreeMap::new();
        for (&first, &count) in &view.things {
            let e = sim.state().population.get(first).ok_or("view names an absent member")?;
            if e.resident_site() != Some(self.site()) { things.insert(first, count); continue; }
            let end = first.checked_add(count).ok_or("view identity interval overflow")?;
            // Geometry is per actual identity, not automatically the whole cohort.
            for id in seen.range(first..end) { things.insert(*id, 1); }
        }
        view.things = things;
        Ok(view)
    }
}
