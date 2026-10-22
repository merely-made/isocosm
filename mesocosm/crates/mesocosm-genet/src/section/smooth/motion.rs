// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Interpolate only adjacent admitted roots with a native clearance witness.
use super::super::{PlacedBody, SiteScene};
use isocosm::spatial::{Pose, embodiment};
use isometer::{core::BodyDocument, render::implicit::Gait};
use std::collections::BTreeMap;
#[cfg(test)]
mod tests;

pub(super) struct Motion {
    site: u64,
    frame: ([i64; 2], [i64; 2], i64),
    document: BodyDocument,
    pose: Option<Pose>,
    from: [f32; 3],
    target: [f32; 3],
    age: f32,
    sweep_from: Option<Pose>,
    started: bool,
    pub gait: Gait,
    contacts: BTreeMap<isometer::core::PartId, [f32; 3]>,
}
impl Motion {
    pub fn new(body: &PlacedBody, scene: &SiteScene) -> Self {
        let pose = body.physical.as_ref().map(|p| Pose {
            at: p.at,
            yaw: p.yaw,
        });
        Self {
            site: scene.site,
            frame: (scene.window.min, scene.window.max, scene.window.datum),
            document: body.document.clone(),
            pose,
            from: body.at.map(|v| v as f32),
            target: body.at.map(|v| v as f32),
            age: 1.,
            sweep_from: None,
            started: false,
            gait: Gait::default(),
            contacts: BTreeMap::new(),
        }
    }

    pub fn sample(&mut self, body: &PlacedBody, scene: &SiteScene, elapsed: f32) -> [f32; 3] {
        if self.site != scene.site
            || self.frame != (scene.window.min, scene.window.max, scene.window.datum)
            || self.document != body.document
            || !body.alive
        {
            *self = Self::new(body, scene);
        }
        let next = body.physical.as_ref().map(|p| Pose {
            at: p.at,
            yaw: p.yaw,
        });
        let target = body.at.map(|v| v as f32);
        self.started = false;
        if next != self.pose || target != self.target {
            let clear = match (&self.pose, &next) {
                (Some(from), Some(to)) if self.age >= 0.24 && body.alive => {
                    cleared(scene, body, from, to)
                },
                _ => false,
            };
            self.from = if clear { self.target } else { target };
            self.sweep_from = clear.then(|| self.pose.clone()).flatten();
            self.target = target;
            self.age = if clear { 0. } else { 1. };
            if !clear {
                self.gait = Gait::default();
            } else {
                self.started = true;
                self.gait.start_step();
            }
            self.pose = next;
        }
        if self.age < 0.24 && !self.started {
            if let (Some(from), Some(to)) = (&self.sweep_from, &self.pose) {
                if !cleared(scene, body, from, to) {
                    self.from = target;
                    self.target = target;
                    self.age = 1.;
                    self.gait = Gait::default();
                }
            }
        }
        // Contacts refresh from current terrain even when the root did not
        // change. An edit can withdraw a stance without generating a walk.
        self.contacts.clear();
        if let Some(pose) = &self.pose {
            if let Ok(contacts) = embodiment::contacts(&scene.window, &body.document, pose) {
                for contact in contacts {
                    let at = scene.window.to_ground(contact.at);
                    self.contacts.insert(
                        contact.part,
                        [at[0] as f32 + 0.5, at[1] as f32, at[2] as f32 + 0.5],
                    );
                }
            }
        }
        self.age += self.elapsed(elapsed);
        let t = (self.age / 0.24).clamp(0., 1.);
        let t = t * t * (3. - 2. * t);
        let origin = [0, 1, 2].map(|k| self.from[k] + (self.target[k] - self.from[k]) * t);
        let min = if self.age < 0.24 {
            [0, 1, 2].map(|k| self.from[k].min(self.target[k]) - origin[k])
        } else {
            [0.; 3]
        };
        let max = if self.age < 0.24 {
            [0, 1, 2].map(|k| self.from[k].max(self.target[k]) - origin[k])
        } else {
            [0.; 3]
        };
        self.gait.set_envelope_offset(min, max);
        origin
    }

    pub fn elapsed(&self, elapsed: f32) -> f32 {
        if self.started {
            0.
        } else {
            elapsed.clamp(0., 0.1)
        }
    }

    pub fn contacts(&self) -> BTreeMap<isometer::core::PartId, [f32; 3]> {
        self.contacts.clone()
    }
    pub fn active(&self) -> bool {
        self.age < 0.24 || self.gait.swinging > 0
    }
}

fn cleared(scene: &SiteScene, body: &PlacedBody, from: &Pose, to: &Pose) -> bool {
    let blocked: Vec<_> = scene
        .bodies
        .iter()
        .filter(|other| other.id != body.id)
        .filter_map(|other| {
            other.physical.as_ref().and_then(|pose| {
                embodiment::boxes(
                    &other.document,
                    &Pose {
                        at: pose.at,
                        yaw: pose.yaw,
                    },
                )
                .ok()
            })
        })
        .flatten()
        .collect();
    embodiment::sweep_clear(&scene.window, &body.document, from, to, &blocked).is_ok()
}
