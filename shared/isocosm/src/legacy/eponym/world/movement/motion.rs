// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The append-only continuous-pose receipt inside the legacy movement store.

use super::*;

impl Movement {
    pub(crate) fn record_contact_pose(
        &mut self,
        world: &World,
        subject: SubjectId,
        pose: MotionPose,
    ) -> Result<MovementEvent, MovementError> {
        self.apply(
            world,
            MovementIntent::ContactPose {
                tick: self.next_tick(),
                subject,
                pose,
            },
        )
    }

    pub(crate) fn has_contact_pose(&self, subject: SubjectId) -> bool {
        self.intents.iter().any(|intent| {
            matches!(intent, MovementIntent::ContactPose { subject: found, .. } if *found == subject)
        })
    }
}
