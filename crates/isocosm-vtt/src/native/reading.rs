// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{
    Execution, Session,
    history::Entry,
    schema::{Entity, Id, Polity, Site},
    simulation::{Genesis, State},
};

/// Borrowed native nouns and history; reading grants no command or control authority.
#[derive(Clone, Copy, Debug)]
pub struct Reading<'a> {
    session: &'a Session,
}

mod geography;
mod disclosure;
pub use disclosure::Geography;
mod space;
mod storylets;
mod polities;
pub use polities::polity_readings;
pub mod places;
pub use places::place_readings;

impl<'a> Reading<'a> {
    pub(super) fn new(session: &'a Session) -> Self {
        Self { session }
    }

    pub fn state(&self) -> &'a State {
        self.session.sim.state()
    }

    pub fn genesis(&self) -> &'a Genesis {
        self.session.sim.genesis()
    }

    pub fn entries(&self) -> &'a [Entry] {
        &self.session.entries
    }

    pub fn execution(&self) -> Execution {
        self.session.sim.mode()
    }

    pub fn state_hash(&self) -> u64 {
        self.session.sim.state_hash()
    }

    pub fn authored_site(&self, key: &str) -> Option<(Id, &'a Site)> {
        self.session.sim.authored_site(key)
    }

    pub fn authored_entity(&self, key: &str) -> Option<(Id, &'a Entity)> {
        self.session.sim.authored_entity(key)
    }

    pub fn authored_polity(&self, key: &str) -> Option<(Id, &'a Polity)> {
        self.session.sim.authored_polity(key)
    }
    /// Native event knowing, including its entity/event refusals; no place disclosure inferred.
    pub fn knows(&self, entity: Id, event: &str) -> isocosm::Result<bool> {
        self.session.sim.knows(entity, event)
    }
}
