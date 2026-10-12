// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rebind the SAME saved actual cells after native geometry changes.
//! Invalid or departed positions remain historical, never relocated. Their
//! stale binding refuses physical operations/rendering, not ordinary clocks.

use super::{Game, Placement};
use isocosm::{Command, Session, schema::Id, Simulation};
use isometer_space::places::{Places, Rules};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Refresh {
    pub current: Vec<Id>,
    pub inactive: Vec<Id>,
    pub stale: BTreeMap<Id, String>,
}

impl Placement {
    /// Read current native body/terrain at this unchanged pose and eye.
    /// Both cells must remain clear and belong to the same realized patch.
    pub(super) fn rebound(&self, sim: &Simulation) -> Result<Self, String> {
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if member.body.is_none() || member.resident_site() != Some(self.site.0) {
            return Err("saved pose has no realized native body at its recorded site".into());
        }
        if self.resident_since != Some(member.arrived) {
            return Err("saved pose is not bound to this native visit".into());
        }
        let volume = sim.volume(self.site.0, self.window.min, self.window.max, self.window.depth)?;
        let places = Places::derive(&volume, self.places.unwrap_or_else(Rules::default))?;
        let mut next = self.clone();
        let patch = places.place_at(self.pose.at).ok_or("saved pose has no current realized patch")?;
        for cell in [self.pose.at, self.eye] {
            if !volume.holds(cell[0], cell[2]) || cell[1] < volume.datum || cell[1] >= volume.ceiling()
                || places.place_at(cell) != Some(patch)
                || !volume.material(cell).is_some_and(|m| m == volume.materials.air || m == volume.materials.water)
            { return Err("saved pose or eye no longer admits in the current realized clear patch".into()); }
        }
        next.patch = patch;
        next.body_revision = member.body_revision;
        next.terrain_revision = sim.terrain_revision();
        Ok(next)
    }

    /// The exact physical binding used for rendering, including a native
    /// corpse if that native seat cut supplies it. This grants no awareness.
    pub fn realized(&self, sim: &Simulation) -> Result<(), String> {
        let next = self.rebound(sim)?;
        let member = sim.state().population.get(self.actor.0).ok_or("spatial actor is absent")?;
        if next != *self || member.patch != Some(self.patch) {
            return Err("saved physical body, patch or terrain binding is stale".into());
        }
        Ok(())
    }
}

impl Game {
    /// Update valid same-site bindings and log only a changed realized patch.
    /// Dormant or obstructed history is retained for an explicitly selected
    /// movement/arrival solver to resolve later, without invented coordinates.
    pub fn refresh(&mut self, session: &mut Session) -> Result<Refresh, String> {
        self.version()?;
        let mut game = self.clone();
        let mut native = session.clone();
        let mut report = Refresh::default();
        for (&id, held) in &self.members {
            let Some(member) = native.sim.state().population.get(id) else {
                report.inactive.push(id); continue;
            };
            if member.body.is_none() || member.resident_site() != Some(held.site.0) {
                report.inactive.push(id); continue;
            }
            let result = (|| {
                let next = held.rebound(&native.sim)?;
                let mut candidate = native.clone();
                if member.patch != Some(next.patch) {
                    candidate.command(Command::Patch { entity:id, patch:Some(next.patch) })?;
                }
                next.realized(&candidate.sim)?;
                Ok::<_, String>((candidate, next))
            })();
            match result {
                Ok((candidate, next)) => {
                    native = candidate; game.members.insert(id, next); report.current.push(id);
                },
                Err(why) => { report.stale.insert(id, why); },
            }
        }
        *session = native;
        *self = game;
        Ok(report)
    }
}
