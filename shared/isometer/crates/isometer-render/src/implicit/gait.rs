// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Contact-driven stance and swing. Movement comes from admitted poses; an
//! unchanged root never starts another step. State is disposable presentation.
use glam::Vec3;
use isometer_core::PartId;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
mod anatomy_tests;

#[derive(Clone, Copy)]
struct Foot {
    at: Vec3,
    start: Vec3,
    goal: Vec3,
    phase: f32,
}

#[derive(Default)]
pub struct Gait {
    feet: BTreeMap<PartId, Foot>,
    seen: BTreeSet<PartId>,
    root: Option<Vec3>,
    native_origin: Option<Vec3>,
    moving: bool,
    elapsed: f32,
    pending_step: bool,
    envelope: [Vec3; 2],
    pub planted: usize,
    pub swinging: usize,
}

impl Gait {
    pub fn set_native_origin(&mut self, origin: [f32; 3]) {
        self.native_origin = Some(origin.into());
    }
    pub(super) fn native_offset(&self, origin: Vec3) -> Vec3 {
        self.native_origin
            .map_or(Vec3::ZERO, |native| native - origin)
    }
    pub(super) fn reject(&mut self, part: PartId, planted: bool) {
        self.feet.remove(&part);
        if planted {
            self.planted = self.planted.saturating_sub(1);
        } else {
            self.swinging = self.swinging.saturating_sub(1);
        }
    }
    pub fn start_step(&mut self) {
        self.pending_step = true;
    }
    /// Extra room may only come from the caller's cleared physical sweep.
    pub fn set_envelope_offset(&mut self, min: [f32; 3], max: [f32; 3]) {
        let min = Vec3::from_array(min);
        let max = Vec3::from_array(max);
        self.envelope = if min.is_finite()
            && max.is_finite()
            && min.cmple(Vec3::ZERO).all()
            && max.cmpge(Vec3::ZERO).all()
        {
            [min, max]
        } else {
            [Vec3::ZERO; 2]
        };
    }
    pub(super) fn envelope(&self, bounds: [Vec3; 2]) -> [Vec3; 2] {
        [bounds[0] + self.envelope[0], bounds[1] + self.envelope[1]]
    }
    pub fn begin(&mut self, root: Vec3, elapsed: f32) {
        self.moving = self.pending_step
            || self
                .root
                .is_some_and(|old| old.distance_squared(root) > 1e-8);
        self.pending_step = false;
        self.root = Some(root);
        self.elapsed = if elapsed.is_finite() {
            elapsed.clamp(0., 0.1)
        } else {
            0.
        };
        self.seen.clear();
        self.planted = 0;
        self.swinging = 0;
    }

    /// The caller provides a real terrain witness and its admitted envelope.
    /// Stance is stationary in world space. Swing follows a fixed endpoint
    /// with a short lift arc; it cannot claim support or chase a moving goal.
    pub fn foot(
        &mut self,
        part: PartId,
        witness: Vec3,
        bounds: [Vec3; 2],
        radius: f32,
        ground: &mut impl FnMut([f32; 3]) -> Option<f32>,
    ) -> (Vec3, bool) {
        self.seen.insert(part);
        let margin = Vec3::new(radius, 0., radius);
        let valid = |p: Vec3| {
            p.cmpge(bounds[0] + margin - 0.025).all()
                && p.cmple(bounds[1] - margin - Vec3::Y * (radius * 2.) + 0.025)
                    .all()
        };
        if !ground(witness.to_array()).is_some_and(|h| (h - witness.y).abs() < 0.025) {
            self.feet.remove(&part);
            return (witness, false);
        }
        let another_swing = self.feet.iter().any(|(id, f)| *id != part && f.phase < 1.);
        let foot = self.feet.entry(part).or_insert(Foot {
            at: witness,
            start: witness,
            goal: witness,
            phase: 1.,
        });
        let on_ground = ground(foot.at.to_array()).is_some_and(|h| (h - foot.at.y).abs() < 0.025);
        // An edited column, changed envelope or unreachable previous contact
        // withdraws the old witness. It cannot become an invisible support.
        if !valid(foot.at) || (foot.phase >= 1. && !on_ground) {
            *foot = Foot {
                at: witness,
                start: witness,
                goal: witness,
                phase: 1.,
            };
        }
        let width =
            ((bounds[1] - bounds[0]).x.min((bounds[1] - bounds[0]).z) - 2. * radius).max(radius);
        let stride = (width * 0.2).max(radius * 0.5);
        if self.moving && !another_swing && foot.phase >= 1. && foot.at.distance(witness) > stride {
            foot.start = foot.at;
            foot.goal = witness;
            foot.phase = 0.;
        }
        if foot.phase < 1. {
            foot.phase = (foot.phase + self.elapsed / 0.18).min(1.);
            let t = foot.phase * foot.phase * (3. - 2. * foot.phase);
            foot.at = foot.start.lerp(foot.goal, t);
            let lift = (width * 0.15).min((bounds[1].y - foot.at.y - 2. * radius).max(0.));
            foot.at.y += (std::f32::consts::PI * foot.phase).sin() * lift;
            if !valid(foot.at) {
                foot.at = witness;
                foot.phase = 1.;
            }
        }
        let planted = foot.phase >= 1.
            && ground(foot.at.to_array()).is_some_and(|h| (h - foot.at.y).abs() < 0.025);
        self.planted += usize::from(planted);
        self.swinging += usize::from(!planted);
        (foot.at, planted)
    }

    pub fn finish(&mut self) {
        self.feet.retain(|part, _| self.seen.contains(part));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiple_contacts_keep_a_support_while_another_swings() {
        let mut gait = Gait::default();
        let bounds = [Vec3::new(-2., 0., -2.), Vec3::splat(2.)];
        gait.begin(Vec3::ZERO, 0.02);
        for (id, x) in [(1, -0.5), (2, 0.5)] {
            gait.foot(PartId(id), Vec3::X * x, bounds, 0.2, &mut |_| Some(0.));
        }
        gait.finish();
        gait.begin(Vec3::X, 0.02);
        for (id, x) in [(1, 0.5), (2, 1.5)] {
            gait.foot(PartId(id), Vec3::X * x, bounds, 0.2, &mut |_| Some(0.));
        }
        assert_eq!(gait.swinging, 1);
        assert_eq!(gait.planted, 1);
        gait.finish();
    }
    #[test]
    fn stance_stays_in_world_and_idle_does_not_march() {
        let mut gait = Gait::default();
        let bounds = [Vec3::new(-2., 0., -2.), Vec3::splat(2.)];
        gait.begin(Vec3::ZERO, 0.02);
        gait.foot(PartId(1), Vec3::ZERO, bounds, 0.2, &mut |_| Some(0.));
        gait.finish();
        for x in [0.1, 0.2, 0.3] {
            gait.begin(Vec3::X * x, 0.02);
            let (foot, planted) = gait.foot(PartId(1), Vec3::X * x, bounds, 0.2, &mut |_| Some(0.));
            assert!(planted);
            assert_eq!(foot, Vec3::ZERO);
            gait.finish();
        }
        for _ in 0..100 {
            gait.begin(Vec3::X * 0.3, 0.02);
            assert_eq!(
                gait.foot(PartId(1), Vec3::X * 0.3, bounds, 0.2, &mut |_| Some(0.)),
                (Vec3::ZERO, true)
            );
            gait.finish();
        }
    }
    #[test]
    fn movement_starts_swing_and_edits_withdraw_contacts() {
        let mut gait = Gait::default();
        let bounds = [Vec3::new(-2., 0., -2.), Vec3::splat(2.)];
        gait.begin(Vec3::ZERO, 0.02);
        gait.foot(PartId(1), Vec3::ZERO, bounds, 0.2, &mut |_| Some(0.));
        gait.finish();
        gait.begin(Vec3::X, 0.02);
        let (foot, planted) = gait.foot(PartId(1), Vec3::X, bounds, 0.2, &mut |_| Some(0.));
        assert!(!planted);
        assert!(foot.y > 0.);
        assert_eq!(gait.swinging, 1);
        gait.finish();
        for _ in 0..15 {
            gait.begin(Vec3::X, 0.02);
            gait.foot(PartId(1), Vec3::X, bounds, 0.2, &mut |_| Some(0.));
            gait.finish();
        }
        assert_eq!(gait.planted, 1);
        gait.begin(Vec3::X, 0.02);
        let changed = Vec3::new(1., 1., 0.);
        assert_eq!(
            gait.foot(PartId(1), changed, bounds, 0.2, &mut |_| Some(1.)),
            (changed, true)
        );
        gait.finish();
        gait.begin(Vec3::X, 0.02);
        gait.finish();
        assert!(gait.feet.is_empty());
    }
}
