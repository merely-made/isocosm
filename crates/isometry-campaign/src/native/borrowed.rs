// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A seat-cut campaign projection and trusted assertion door over borrowed history.

use isocosm::{Session, Command, directing::readings::View, schema::Id};
use isocosm_overlay::vtt::Assertion;
use super::ContractError;

pub struct WindowReading<'a> { session: &'a Session, visible: &'a View }

impl<'a> WindowReading<'a> {
    pub fn new(session: &'a Session, visible: &'a View) -> Self { Self { session, visible } }
    pub fn characters(&self) -> Vec<(String, Id)> {
        self.session.sim.state().population.groups.iter().filter_map(|(id, group)| {
            let authored = group.entity.authored.as_ref()?;
            self.visible.shows(*id).then(|| (authored.key.clone(), *id))
        }).collect()
    }
    pub fn sites(&self) -> Vec<(String, Id)> {
        self.session.sim.state().sites.iter().filter_map(|(id, site)| {
            let authored = site.authored.as_ref()?;
            self.visible.sites.contains(id).then(|| (authored.key.clone(), *id))
        }).collect()
    }
    pub fn events(&self) -> Vec<String> { self.visible.events.iter().cloned().collect() }
}

/// Trusted table-author admission; the host checks DM access before this door.
pub fn submit_contract_assertions(session: &mut Session,
    assertions: &[Assertion]) -> Result<(), ContractError> {
    let mut candidate = session.clone();
    for (index, assertion) in assertions.iter().enumerate() {
        let entry = crate::contract::lower(assertion, &candidate.sim)
            .map_err(|refusal| ContractError::Lowering { index, refusal })?;
        candidate.command(Command::Assert(entry))
            .map_err(|reason| ContractError::Native { index, reason })?;
    }
    *session = candidate;
    Ok(())
}
