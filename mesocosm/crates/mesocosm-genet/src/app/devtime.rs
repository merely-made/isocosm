// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-only time control (DT1): pause, speed, and the manual step keys.
//!
//! Everything here is pacing over [`mesocosm_runtime::Runtime::advance`] and
//! `::step`: nothing reaches `Runtime::queue`, so nothing here enters the
//! session's log (the dev tools plan's second principle). DT3's four keys are
//! the exception, and queue ordinary dev intents.

use super::Host;
use crate::input;

/// The speed ladder the `[`/`]` dev keys move through, paired with the word
/// the dev lane shows for each rung. A multiplier over the elapsed
/// microseconds the host was already going to pass to `Runtime::advance` —
/// it never reaches the runtime itself, so it cannot move a replay's hash.
const DEV_SPEED_LADDER: [(f64, &str); 5] = [
    (0.25, "1/4"),
    (0.5, "1/2"),
    (1.0, "1"),
    (2.0, "2"),
    (4.0, "4"),
];

/// Where the ladder starts: ordinary speed.
pub(super) const DEV_SPEED_DEFAULT_IDX: usize = 2;

impl Host {
    /// Presentation-only terrarium quarter turns. They are handled before
    /// inspection and dev/world keys so Z/V always remain camera controls.
    pub(super) fn try_camera_key(&mut self, key: &winit::keyboard::Key) -> bool {
        let backwards = match key {
            winit::keyboard::Key::Character(c) if matches!(c.as_str(), "z" | "Z") => true,
            winit::keyboard::Key::Character(c) if matches!(c.as_str(), "v" | "V") => false,
            _ => return false,
        };
        if !self.config.camera.is_terrarium() {
            return true;
        }
        self.config.camera = self.config.camera.quarter_turn(backwards);
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.section.set_mode(self.config.camera);
        }
        true
    }

    /// Applies pause and speed to a played frame's elapsed time. `advance`
    /// calls this every frame, dev build or not: outside `--dev` it is the
    /// identity, so an ordinary build pays one branch and nothing else.
    ///
    /// **Pause drops the elapsed time rather than banking it** — the same
    /// "do not bank it" rule a checkpoint hold already uses inside
    /// `Runtime::advance` — and speed scales it before the clock ever sees
    /// it. `Runtime::advance` takes a `u64` of microseconds and does not know
    /// or care why this frame's was zero, or four times the wall clock's.
    pub(super) fn dev_paced_elapsed(&self, elapsed_us: u64) -> u64 {
        if !self.config.dev {
            elapsed_us
        } else if self.dev_paused {
            0
        } else {
            let (multiplier, _) = DEV_SPEED_LADDER[self.dev_speed_idx];
            ((elapsed_us as f64) * multiplier) as u64
        }
    }

    /// The whole of the key handler's dev interception: off outside
    /// `--dev`, and `true` (having already applied the action) for one of
    /// the twelve keys `--dev` makes live. Kept off unless the flag is set, so
    /// an ordinary build's keyboard is exactly what it was before DT1.
    ///
    /// The three follow keys are handed to [`super::follow`]; DT3's four queue
    /// dev intents and nothing else does.
    pub(super) fn try_dev_key(&mut self, key: &winit::keyboard::Key) -> bool {
        if !self.config.dev {
            return false;
        }
        let Some(action) = input::dev_key(key) else {
            return false;
        };
        match action {
            input::DevKey::TogglePause => self.dev_paused = !self.dev_paused,
            input::DevKey::Step => self.dev_step(1),
            input::DevKey::StepN => self.dev_step(super::DEV_STEP_N),
            input::DevKey::SlowDown => self.dev_speed_idx = self.dev_speed_idx.saturating_sub(1),
            input::DevKey::SpeedUp => {
                self.dev_speed_idx = (self.dev_speed_idx + 1).min(DEV_SPEED_LADDER.len() - 1);
            },
            // DT2: the camera's centre, and nothing else.
            input::DevKey::FollowNext | input::DevKey::FollowBack | input::DevKey::FollowSelf => {
                self.follow_key(action)
            },
            // DT3: an ordinary intent, queued. The only arm here that can
            // reach the world at all.
            action if action.changes_the_world() => self.dev_world_key(action),
            _ => {},
        }
        true
    }

    /// The step and step-N dev keys both land here: off the clock entirely,
    /// exactly [`mesocosm_runtime::Runtime::step`]'s own contract — `n`
    /// unless a checkpoint holds it, then fewer.
    fn dev_step(&mut self, n: u64) {
        let taken = self.runtime.step(n);
        self.steps += taken;
        self.dev_manual_steps += taken;
    }

    /// A dev intent for one of DT3's four keys, about the followed critter.
    fn dev_world_key(&mut self, action: input::DevKey) {
        use isocosm_overlay::mesocosm::{DevIntent, MesocosmIntent};
        use isocosm_overlay::{EntityHandle, WorldPoint};
        let followed = self.followed();
        let dev = match action {
            input::DevKey::EndEpoch => DevIntent::EndEpoch,
            input::DevKey::ForceBirth => DevIntent::ForceBirth {
                organism: EntityHandle(followed.unwrap_or_default()),
            },
            input::DevKey::Kill => DevIntent::Kill {
                organism: EntityHandle(followed.unwrap_or_default()),
            },
            input::DevKey::PlaceMatter => DevIntent::PlaceMatter {
                at: WorldPoint(self.follow_at()),
                mass_mg: super::DEV_PLACE_MG,
            },
            _ => return,
        };
        if input::admits(self.runtime.queued_len()) {
            let envelope = input::envelope(&self.runtime, MesocosmIntent::Dev(dev));
            self.runtime.queue(envelope);
        }
    }

    /// The dev lane's reading, taken fresh each frame; `None` outside `--dev`.
    pub(super) fn dev_reading(&self) -> Option<mesocosm_views::Dev> {
        if !self.config.dev {
            return None;
        }
        let (follow, lost) = self.follow_reading();
        Some(mesocosm_views::Dev {
            running: !self.dev_paused,
            speed: DEV_SPEED_LADDER[self.dev_speed_idx].1,
            tick: self.runtime.tick(),
            manual_steps: self.dev_manual_steps,
            follow,
            lost,
            inspection: self.inspection.open.then(|| {
                self.inspection.selected.map_or_else(
                    || mesocosm_views::PartInspection {
                        reading: None,
                        notice: Some(self.inspection.notice.into()),
                    },
                    |selected| {
                        mesocosm_views::part_of(
                            self.runtime.sim(),
                            selected.organism,
                            selected.part,
                        )
                    },
                )
            }),
        })
    }
}
