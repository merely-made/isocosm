// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which camera the played body is seen through (cases 1131, 1140, 1143):
//! the overview, over the shoulder, or first person. Host state like the
//! camera mode and the pan, never a world fact, so it reaches no queue and
//! no hash.

use isometer::core::PartId;
use isometer::View;

use super::Host;
use crate::section::perspective::{self, Looking};

impl Host {
    /// What the picture is asked to look through this frame.
    pub(super) fn looking(&self) -> Looking {
        if self.view == View::Overview {
            return Looking::overview();
        }
        Looking { view: self.view, sensing: self.sensing(), heading: self.heading(), height: self.height() }
    }

    /// How tall the played body stands, in cells.
    fn height(&self) -> f32 {
        self.scene.as_ref()
            .and_then(|scene| scene.body(scene.played?))
            .map_or(isometer::REFERENCE_BODY_HEIGHT, |body| {
                let aabb = body.document.aabb();
                (aabb.max[1] - aabb.min[1]) as f32
            })
    }

    /// The played body's most sensing part, by the sim.
    fn sensing(&self) -> Option<PartId> {
        let sim = self.runtime.sim();
        let played = self.runtime.critter()?;
        cosmograph::anatomy::sensing_part(sim.state().population.get(played)?)
    }

    /// What the perspective views face. A body with a saved physical root
    /// faces its yaw; the coarse picture gives a body no facing, so it takes
    /// the way the overview looks, which Z and V turn.
    fn heading(&self) -> [f32; 3] {
        let scene = self.scene.as_ref();
        if let Some(physical) = scene
            .and_then(|scene| scene.body(scene.played?))
            .and_then(|body| body.physical.as_ref())
        {
            return perspective::facing(physical.yaw);
        }
        let [x, _, z] = self.config.camera.forward();
        let length = x.hypot(z);
        if length > 1e-6 { [x / length, 0.0, z / length] } else { [0.0, 0.0, 1.0] }
    }
}

#[cfg(test)]
mod tests {
    use isometer::View;
    use winit::keyboard::Key;

    use super::super::{Host, HostConfig};
    use crate::section::perspective::{self, Looking};

    fn bench_host() -> Host {
        let mut host = Host::new(HostConfig { bench_world: true, seed: 9, population: 256, ..HostConfig::default() });
        host.refresh_scene();
        host
    }

    /// The key cycles the three views and back, and the world does not
    /// notice: the state hash, tick and revision are unchanged through a
    /// full cycle, and the overview it returns to asks for what it left.
    #[test]
    fn the_view_key_cycles_three_views_and_writes_no_world_state() {
        let mut host = bench_host();
        let (hash, tick, revision) = (host.runtime.state_hash(), host.runtime.tick(), host.runtime.revision().unwrap());
        let overview = format!("{:?}", host.looking());
        assert_eq!(host.view, View::Overview);
        let mut seen = Vec::new();
        for _ in 0..3 {
            assert!(host.try_camera_key(&Key::Character(crate::input::VIEW_KEY.into())));
            seen.push(host.view);
            assert_eq!(host.runtime.state_hash(), hash, "{:?}", host.view);
            assert_eq!(host.runtime.tick(), tick);
            assert_eq!(host.runtime.revision().unwrap(), revision);
        }
        assert_eq!(seen, [View::OverTheShoulder, View::FirstPerson, View::Overview]);
        assert_eq!(format!("{:?}", host.looking()), overview, "the overview is the one left");
    }

    /// First person stands the eye on the played body and draws a copy of it
    /// with parts at or above the eye severed; the world's body stays whole.
    #[test]
    fn first_person_draws_a_copy_of_the_body_below_its_eye() {
        let mut host = bench_host();
        host.view = View::FirstPerson;
        let looking = host.looking();
        let scene = host.scene.as_ref().unwrap();
        let played = scene.played.expect("a played body");
        let whole = scene.body(played).unwrap();
        assert!(whole.document.living().count() > 1, "a body with parts to hide");
        let (seen, eye) = perspective::seen(scene, &looking);
        let eye = eye.expect("the played body has an eye");
        let drawn = &seen.bodies.iter().find(|b| b.id == played).unwrap().document;
        let top = |id| whole.document.world_pivot(id).unwrap()[1] + whole.document.part(id).unwrap().half_extent[1];
        let eye_part = looking.sensing.filter(|p| whole.document.part(*p).is_some()).unwrap_or(whole.document.root);
        let line = top(eye_part);
        for part in &whole.document.parts {
            assert_eq!(drawn.part(part.id).unwrap().severed, part.severed || top(part.id) >= line, "{:?}", part.id);
        }
        assert!(drawn.living().count() < whole.document.living().count(), "something is hidden");
        assert_eq!(scene.body(played).unwrap().document.living().count(), whole.document.living().count());
        assert!(eye[1].is_finite());
        let (_, shoulder) = perspective::seen(scene, &Looking { view: View::OverTheShoulder, ..looking.clone() });
        assert!(shoulder.is_some());
        let (overview, none) = perspective::seen(scene, &Looking::overview());
        assert!(none.is_none());
        assert_eq!(overview.bodies.iter().map(|b| b.document.living().count()).sum::<usize>(),
            scene.bodies.iter().map(|b| b.document.living().count()).sum::<usize>());
    }
}
