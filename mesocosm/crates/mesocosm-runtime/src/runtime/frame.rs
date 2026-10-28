// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The camera's frame as attention (the execution plan's D32): the terrain
//! window around the played critter, held as the participant's one window,
//! attached once and replaced as the critter moves, so members outside it
//! leave the physical path.

use isocosm::attention::{Region, WindowAttention};
use isocosm_host::world::World;
use super::{Mesocosm, commit, session};
use crate::runtime::translate::Refusal;

const PROFILE: &str = "mesocosm";

/// Off on main until the tier line's fixes land together (the execution
/// plan's D33): with it off, no window is attached and every member keeps
/// its pose, as before.
pub(super) const FRAME_TIER: bool = false;

impl Mesocosm {
    /// Brings the participant's window to the played critter's frame.
    pub(super) fn frame_attention(&self, world: &World) {
        let participant = self.lp.participant;
        let attention = {
            let s = session(world);
            let sim = &s.sim;
            let Some(critter) = sim.plays(participant) else { return };
            let Some(t) = sim.genesis().rules.spatial_templates.as_ref() else { return };
            let Some(at) = sim.state().spatial_poses.get(&critter) else { return };
            let w = isocosm::spatial::embodiment::site_window(t, at.pose.at);
            let attention = WindowAttention { examined: Some(Region::Frame { site: at.site, min: w.min, max: w.max }),
                ..Default::default() };
            let held = sim.window_for(participant, PROFILE)
                .and_then(|w| sim.attention_window(participant, w.id()).ok());
            if held.is_some_and(|(_, _, value)| value == attention) { return; }
            attention
        };
        let _ = commit(world, |session| {
            let changed = match session.sim.window_for(participant, PROFILE) {
                Some(w) => session.replace_attention(participant, w, PROFILE.into(), attention),
                None => session.attach_window(participant, PROFILE.into(), attention).map(|_| String::new()),
            };
            changed.map(|_| ()).map_err(Refusal::Sim)
        });
    }
}
