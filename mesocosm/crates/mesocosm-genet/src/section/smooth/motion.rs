// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Interpolate only adjacent admitted roots with a native clearance witness.
use super::super::{PlacedBody, SiteScene};
use isocosm::spatial::{Pose, embodiment};
use isometer::{core::BodyDocument, render::implicit::Gait};
use std::collections::{BTreeMap, VecDeque};
#[cfg(test)]
mod tests;

pub(super) struct Motion {
    site: u64,
    frame: ([i64; 2], [i64; 2], i64),
    document: BodyDocument,
    pose: Option<Pose>,
    observed: Option<Pose>,
    pending: VecDeque<Pose>,
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
            observed: pose.clone(),
            pending: VecDeque::new(),
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
            if std::env::var_os("MESOCOSM_DEBUG_MOTION").is_some() {
                eprintln!("motion {} reset: site={}, frame={}, anatomy={}, alive={}", body.id,
                    self.site != scene.site,
                    self.frame != (scene.window.min, scene.window.max, scene.window.datum),
                    self.document != body.document, body.alive);
            }
            *self = Self::new(body, scene);
        }
        let next = body.physical.as_ref().map(|p| Pose {
            at: p.at,
            yaw: p.yaw,
        });
        let target = body.at.map(|v| v as f32);
        self.started = false;
        if next != self.observed {
            let path = self.observed.as_ref().zip(next.as_ref())
                .and_then(|(from,to)| trail(&body.walks, from, to));
            // Keep the admitted route, but clear each segment at presentation
            // time. A later blocked climb must not erase an earlier clear step.
            if path.as_ref().is_some_and(|p| self.pending.len() + p.len() <= 16) {
                self.pending.extend(path.unwrap());
            } else {
                self.pending.clear(); self.pose = next.clone();
                self.from = target; self.target = target; self.age = 1.;
                self.sweep_from = None; self.gait = Gait::default();
            }
            self.observed = next;
        }
        if self.age >= 0.24 && !self.pending.is_empty() {
            let to = self.pending.pop_front().unwrap();
            if self.pose.as_ref().is_some_and(|from| cleared(scene, body, from, &to)) {
                self.from = self.target;
                self.target = scene.window.to_ground(to.at).map(|v| v as f32);
                self.sweep_from = self.pose.replace(to);
                self.age = 0.; self.started = true; self.gait.start_step();
            } else {
                self.pending.clear(); self.pose = self.observed.clone();
                self.from = target; self.target = target; self.age = 1.;
                self.sweep_from = None; self.gait = Gait::default();
            }
        }
        if self.age < 0.24 && !self.started {
            if let (Some(from), Some(to)) = (&self.sweep_from, &self.pose) {
                if !cleared(scene, body, from, to) {
                    self.from = target;
                    self.target = target;
                    self.age = 1.;
                    self.pending.clear(); self.pose = self.observed.clone();
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
        self.gait.set_native_origin(self.target);
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
        !self.pending.is_empty() || self.age < 0.24 || self.gait.swinging > 0
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
    let clear = embodiment::sweep_clear(&scene.window, &body.document, from, to, &blocked);
    if let Err(why) = &clear {
        if std::env::var_os("MESOCOSM_DEBUG_MOTION").is_some() {
            eprintln!("motion {} {:?} -> {:?}: {why}", body.id, from.at, to.at);
        }
    }
    clear.is_ok()
}

// Only stitch explicit admitted steps. A direct adjacent observation retains
// the older bridge's conservative sweep; a distant observation needs history.
fn trail(steps: &[[Pose;2]], from: &Pose, to: &Pose) -> Option<Vec<Pose>> {
    if from.yaw == to.yaw && (0..3).all(|k| from.at[k].abs_diff(to.at[k]) <= 1) {
        return Some(vec![to.clone()]);
    }
    let mut at = from; let mut out = Vec::new();
    for [a,b] in steps {
        if a == at {
            out.push(b.clone()); at = b;
            if at == to { return Some(out); }
        }
    }
    None
}
