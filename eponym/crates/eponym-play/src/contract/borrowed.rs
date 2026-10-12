// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's existing contract receiver, borrowing the host's native session.

use isocosm::{Session, Command, Simulation, schema::Id, directing::readings::View};
use isometer_space::volume::Volume;
use crate::WorldError;

pub struct Borrowed<'a> {
    pub(super) session: &'a mut Session,
    pub(super) site: Id,
    pub(super) volume: Volume,
}

impl<'a> Borrowed<'a> {
    /// The named native region is re-lifted from this exact session.
    pub fn new(session: &'a mut Session, site: Id, min: [i64; 2],
        max: [i64; 2], depth: i64) -> Result<Self, WorldError> {
        let volume = session.sim.volume(site, min, max, depth).map_err(|_| WorldError::Refused)?;
        Ok(Self { session, site, volume })
    }
    /// Preserve the standalone owner's existing coordinate window.
    pub(crate) fn over_volume(session: &'a mut Session, site: Id, volume: &Volume) -> Self {
        Self { session, site, volume: volume.clone() }
    }
    pub(super) fn run(&mut self, command: Command) -> Result<String, WorldError> {
        self.session.command(command).map_err(|_| WorldError::Refused)
    }
    pub(super) fn sim(&self) -> &Simulation { &self.session.sim }
    pub(super) fn site(&self) -> Id { self.site }
    pub(super) fn volume(&self) -> &Volume { &self.volume }
}

pub struct Reading<'a> { session: &'a Session, visible: &'a View }

impl<'a> Reading<'a> {
    pub fn new(session: &'a Session, visible: &'a View) -> Self { Self { session, visible } }
    pub fn bodies(&self) -> Vec<(Id, String, usize)> {
        self.visible.things.keys().filter_map(|id| {
            let entity = self.session.sim.state().population.get(*id)?;
            let name = entity.authored.as_ref().map(|a| a.name.clone())
                .unwrap_or_else(|| entity.lineage.clone());
            Some((*id, name, entity.parts.len()))
        }).collect()
    }
    pub fn events(&self) -> Vec<String> { self.visible.events.iter().cloned().collect() }
}
