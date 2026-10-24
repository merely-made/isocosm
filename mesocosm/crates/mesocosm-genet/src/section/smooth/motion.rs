// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Interpolate only adjacent admitted roots with a native clearance witness.
use super::super::{PlacedBody, SiteScene, Walk};
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
    pending: VecDeque<Walk>,
    stride: Option<isometer::core::legs::Stride>,
    walking: Option<embodiment::Walking>,
    shifting: bool,
    from: [f32; 3],
    target: [f32; 3],
    age: f32,
    sweep_from: Option<Pose>,
    started: bool,
    checked_revision: u64,
    pub gait: Gait,
    contacts: BTreeMap<isometer::core::PartId, [f32; 3]>,
}
impl Motion {
    pub fn new(body: &PlacedBody, scene: &SiteScene) -> Self {
        let pose = body.physical.as_ref().map(|p| p.clone());
        Self {
            site: scene.site,
            frame: (scene.window.min, scene.window.max, scene.window.datum),
            document: body.document.clone(),
            observed: pose.clone(),
            pending: VecDeque::new(),
            stride: None,
            walking: scene.walking,
            shifting: false,
            pose,
            from: body.at.map(|v| v as f32),
            target: body.at.map(|v| v as f32),
            age: 100.,
            sweep_from: None,
            started: false,
            checked_revision: 0,
            gait: Gait::default(),
            contacts: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    pub fn sample(&mut self, body: &PlacedBody, scene: &SiteScene, elapsed: f32) -> [f32; 3] {
        self.sample_revision(body, scene, elapsed, self.checked_revision.wrapping_add(1))
    }
    pub fn sample_revision(
        &mut self,
        body: &PlacedBody,
        scene: &SiteScene,
        elapsed: f32,
        revision: u64,
    ) -> [f32; 3] {
        if self.site != scene.site
            || self.frame != (scene.window.min, scene.window.max, scene.window.datum)
            || self.document != body.document
            || !body.alive
            || self.walking != scene.walking
        {
            if std::env::var_os("MESOCOSM_DEBUG_MOTION").is_some() {
                eprintln!(
                    "motion {} reset: site={}, frame={}, anatomy={}, alive={}",
                    body.id,
                    self.site != scene.site,
                    self.frame != (scene.window.min, scene.window.max, scene.window.datum),
                    self.document != body.document,
                    body.alive
                );
            }
            *self = Self::new(body, scene);
        }
        let next = body.physical.as_ref().map(|p| p.clone());
        let target = body.at.map(|v| v as f32);
        self.started = false;
        if next != self.observed {
            let path = self
                .observed
                .as_ref()
                .zip(next.as_ref())
                .and_then(|(from, to)| trail(&body.walks, from, to));
            // Keep the admitted route, but clear each segment at presentation
            // time. A later blocked climb must not erase an earlier clear step.
            if path
                .as_ref()
                .is_some_and(|p| self.pending.len() + p.len() <= 16)
            {
                self.pending.extend(path.unwrap());
            } else {
                self.pending.clear();
                self.pose = next.clone();
                self.from = target;
                self.target = target;
                self.age = 100.;
                self.sweep_from = None;
                self.stride = None;
                self.gait = Gait::default();
            }
            self.observed = next;
        }
        if self.age >= self.duration() && !self.pending.is_empty() {
            let walk = self.pending.pop_front().unwrap();
            let to = walk.poses[1].clone();
            if self.pose.as_ref().is_some_and(|from| {
                from == &walk.poses[0]
                    && from.yaw == to.yaw
                    && cleared(scene, body, from, &to, walk.stride.as_ref())
            }) {
                self.checked_revision = revision;
                self.from = self.target;
                self.target = scene.window.to_ground(to.at).map(|v| v as f32);
                self.sweep_from = self.pose.replace(to);
                self.stride = walk.stride;
                self.age = 0.;
                self.started = true;
                self.gait.start_step();
            } else {
                self.pending.clear();
                self.pose = self.observed.clone();
                self.from = target;
                self.target = target;
                self.age = 100.;
                self.sweep_from = None;
                self.stride = None;
                self.gait = Gait::default();
            }
        }
        if self.age < self.duration() && !self.started && self.checked_revision != revision {
            self.checked_revision = revision;
            if let (Some(from), Some(to)) = (&self.sweep_from, &self.pose) {
                if !cleared(scene, body, from, to, self.stride.as_ref()) {
                    self.from = target;
                    self.target = target;
                    self.age = 100.;
                    self.pending.clear();
                    self.pose = self.observed.clone();
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
        let t = (self.age / self.duration()).clamp(0., 1.);
        let articulated = self.pose.as_ref().is_some_and(|p| !p.feet.is_empty());
        let samples = self
            .stride
            .as_ref()
            .map_or(embodiment::SWING_SAMPLES, |s| s.samples());
        let phase = (t * samples as f32).floor() as i64 * isometer::core::legs::UNIT / samples;
        let t = if articulated {
            phase as f32 / isometer::core::legs::UNIT as f32
        } else {
            t * t * (3. - 2. * t)
        };
        let planned = self
            .sweep_from
            .as_ref()
            .zip(self.pose.as_ref())
            .zip(self.stride.as_ref())
            .filter(|_| self.age < self.duration())
            .and_then(|((from, to), stride)| {
                embodiment::stride_sample(from, to, stride, phase).ok()
            });
        let feet = self
            .pose
            .as_ref()
            .filter(|p| !p.feet.is_empty())
            .map(|pose| {
                if let Some(frame) = &planned {
                    return embodiment::stride_feet(frame, pose.yaw)
                        .unwrap_or_else(|_| pose.feet.clone());
                }
                self.sweep_from
                    .as_ref()
                    .filter(|_| self.age < self.duration() && self.stride.is_none())
                    .and_then(|from| embodiment::swing_feet(&body.document, from, pose, phase).ok())
                    .unwrap_or_else(|| pose.feet.clone())
            });
        self.gait.set_physical_feet(feet);
        let linear = [0, 1, 2].map(|k| self.from[k] + (self.target[k] - self.from[k]) * t);
        let origin = if let Some(frame) = &planned {
            let zero = scene.window.to_ground([0; 3]);
            [0, 1, 2]
                .map(|k| zero[k] as f32 + frame.root[k] as f32 / isometer::core::legs::UNIT as f32)
        } else if self.age < self.duration() && self.stride.is_some() {
            self.target
        } else {
            linear
        };
        self.shifting = planned.is_some() && (0..3).any(|k| (origin[k] - linear[k]).abs() > 0.002);
        let mut low = [0, 1, 2].map(|k| self.from[k].min(self.target[k]));
        let mut high = [0, 1, 2].map(|k| self.from[k].max(self.target[k]));
        if let Some(stride) = &self.stride {
            let zero = scene.window.to_ground([0; 3]);
            for s in &stride.stages {
                for k in 0..3 {
                    let at = zero[k] as f32 + s.root[k] as f32 / isometer::core::legs::UNIT as f32;
                    low[k] = low[k].min(at);
                    high[k] = high[k].max(at);
                }
            }
        }
        let min = if self.age < self.duration() {
            [0, 1, 2].map(|k| low[k] - origin[k])
        } else {
            [0.; 3]
        };
        let max = if self.age < self.duration() {
            [0, 1, 2].map(|k| high[k] - origin[k])
        } else {
            [0.; 3]
        };
        self.gait.set_envelope_offset(min, max);
        self.gait.set_native_origin(self.target);
        origin
    }

    fn duration(&self) -> f32 {
        self.pose
            .as_ref()
            .filter(|p| !p.feet.is_empty())
            .map_or(0.24, |p| (p.feet.len() as f32 * 0.18).max(0.36))
    }
    pub fn yaw_radians(&self) -> f32 {
        use isometer::core::Yaw;
        let turns = match self.pose.as_ref().map(|p| p.yaw).unwrap_or_default() {
            Yaw::Zero => 0.,
            Yaw::Quarter => 1.,
            Yaw::Half => 2.,
            Yaw::ThreeQuarter => 3.,
        };
        turns * std::f32::consts::FRAC_PI_2
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
        !self.pending.is_empty() || self.age < self.duration() || self.gait.swinging > 0
    }
    pub fn shifting(&self) -> bool {
        self.shifting
    }
}

fn cleared(
    scene: &SiteScene,
    body: &PlacedBody,
    from: &Pose,
    to: &Pose,
    stride: Option<&isometer::core::legs::Stride>,
) -> bool {
    let blocked: Vec<_> = scene
        .bodies
        .iter()
        .filter(|other| other.id != body.id)
        .filter_map(|other| {
            other
                .physical
                .as_ref()
                .and_then(|pose| embodiment::boxes(&other.document, &pose.clone()).ok())
        })
        .flatten()
        .collect();
    let clear = embodiment::check_walk(
        &scene.window,
        &body.document,
        from,
        to,
        scene.walking,
        stride,
        &blocked,
    );
    if let Err(why) = &clear {
        if std::env::var_os("MESOCOSM_DEBUG_MOTION").is_some() {
            eprintln!("motion {} {:?} -> {:?}: {why}", body.id, from.at, to.at);
        }
    }
    clear.is_ok()
}

// Only stitch explicit admitted steps. A direct adjacent observation retains
// the older bridge's conservative sweep; a distant observation needs history.
fn trail(steps: &[Walk], from: &Pose, to: &Pose) -> Option<Vec<Walk>> {
    if from.feet.is_empty()
        && to.feet.is_empty()
        && from.yaw == to.yaw
        && (0..3).all(|k| from.at[k].abs_diff(to.at[k]) <= 1)
    {
        return Some(vec![[from.clone(), to.clone()].into()]);
    }
    let mut at = from;
    let mut out = Vec::new();
    for step in steps {
        let [a, b] = &step.poses;
        if a == at {
            out.push(step.clone());
            at = b;
            if at == to {
                return Some(out);
            }
        }
    }
    None
}
