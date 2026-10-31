// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Reading;
use cosmograph::{Session,polities::PolityReading};
use cosmograph_overlay::{OrdinaryAccess,Seat};

impl Reading<'_> {
    /// Trusted author read; no disclosed map site grants this live polity state.
    pub fn polities(&self) -> cosmograph::Result<Vec<PolityReading>> {
        self.state().polities.keys().map(|id| self.session.sim.polity_reading(*id)).collect()
    }
}

/// Actual DM-only knowledge boundary for the product's live polity panel.
pub fn polity_readings(session: &Session, access: OrdinaryAccess) -> Result<Vec<PolityReading>,String> {
    super::super::movement::current_access(session,access)?;
    if access.seat != Seat::Dm { return Err("live polity member readings require the actual DM seat".into()); }
    session.sim.state().polities.keys().map(|id| session.sim.polity_reading(*id)).collect()
}
