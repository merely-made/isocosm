// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{MOTION_SCALE, walking};
use isometer_space::{places::Places, volume::Volume};

impl PoseBinding {
    pub fn current(&self, session: &Session) -> Result<MotionPose, String> {
        self.pose.validate().map_err(|e| format!("Eponym pose: {e:?}"))?;
        let body = session.sim.state().population.get(self.entity).ok_or("bound Eponym entity is absent")?;
        if body.resident_site() != Some(self.site) || body.arrived != self.resident_since
            || body.body.is_none() || body.body_revision != self.body_revision
            || body.patch != Some(self.patch) || self.patch.site != self.site
            || session.sim.terrain_revision() != self.terrain_revision {
            return Err("Eponym physical binding is stale, departed or lacks current native geometry".into());
        }
        let volume = session.sim.volume(self.site,self.window.min,self.window.max,self.window.depth)?;
        let cell = walking::to_site(&volume,self.pose.cell().map_err(|e| format!("Eponym pose: {e:?}"))?);
        let places = Places::derive(&volume,Default::default())?;
        if !volume.holds(cell[0],cell[2]) || cell[1] < volume.datum || cell[1] >= volume.ceiling()
            || places.place_at(cell) != Some(self.patch)
            || !volume.material(cell).is_some_and(|m| m == volume.materials.air || m == volume.materials.water) {
            return Err("Eponym feet cell does not match its realized native clear patch".into());
        }
        Ok(self.pose)
    }

    /// Translate full accepted quanta between two explicit Ground windows.
    /// Integer cells are used only for the native patch check, never rendering.
    pub(super) fn in_frame(&self, session: &Session, frame: &Volume) -> Result<MotionPose, String> {
        let mut pose = self.current(session)?;
        if frame.site != self.site { return Err("Eponym picture frame belongs to another native site".into()); }
        let source = session.sim.volume(self.site,self.window.min,self.window.max,self.window.depth)?;
        let from = walking::to_site(&source,[0;3]);
        let to = walking::to_site(frame,[0;3]);
        for axis in 0..3 {
            let shift = from[axis].checked_sub(to[axis]).and_then(|n| n.checked_mul(MOTION_SCALE))
                .ok_or("Eponym picture frame translation overflows quanta")?;
            pose.position[axis] = pose.position[axis].checked_add(shift)
                .ok_or("Eponym picture position overflows quanta")?;
            if axis == 1 {
                pose.fall_origin_y = pose.fall_origin_y.map(|y|y.checked_add(shift)
                    .ok_or("Eponym fall origin overflows framed quanta")).transpose()?;
            }
        }
        pose.validate().map_err(|e| format!("Eponym framed pose: {e:?}"))?;
        Ok(pose)
    }
}
