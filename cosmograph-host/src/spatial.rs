// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product spatial snapshots use the same current seat cut as ordinary rows.

use cosmograph::{Session, attention::spatial::{Crowd, InspectedBody, RegionReading}, directing::readings::Mode};
use cosmograph_overlay::{Seat, WindowHandle};
use crate::window::Projection;

fn current(session: &Session, cut: &Projection, window: WindowHandle) -> Result<Mode, String> {
    let access = crate::access::resolve(session, cut.access.participant, cut.access.seat, cut.access.subject)?;
    if access != cut.access || crate::access::permitted(session, access) != cut.visible {
        return Err("spatial snapshot uses a stale native access cut".into());
    }
    session.sim.attention_window(access.participant.0, window.0)?;
    Ok(if access.seat == Seat::Player { Mode::Survival } else { Mode::Creative })
}

pub fn far(session: &Session, cut: &Projection, window: WindowHandle) -> Result<Vec<Crowd>, String> {
    let mode = current(session, cut, window)?;
    session.sim.far_crowds_as(cut.access.participant.0, window.0, mode, cut.access.subject.map(|s| s.0))
}

pub fn region(session: &Session, cut: &Projection, window: WindowHandle, site: u64)
    -> Result<RegionReading, String> {
    let mode = current(session, cut, window)?;
    session.sim.region_reading_as(cut.access.participant.0, window.0, mode, site, cut.access.subject.map(|s| s.0))
}

pub fn inspect(session: &Session, cut: &Projection, window: WindowHandle, site: u64, body: u64)
    -> Result<InspectedBody, String> {
    let mode = current(session, cut, window)?;
    session.sim.inspected_body_as(cut.access.participant.0, window.0, mode, site, body, cut.access.subject.map(|s| s.0))
}
