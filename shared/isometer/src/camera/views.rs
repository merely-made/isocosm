// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The three views every product offers (cases 1131, 1140, 1143): the
//! dimetric overview its host frames, over the shoulder and first person.
//! The numbers a perspective view takes are one policy here, so the
//! products cannot drift apart on field of view or reach.

use super::SlabCamera;

/// Which camera a frame is seen through. Switching writes no world state and
/// leaves the overview as it was.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Overview,
    OverTheShoulder,
    FirstPerson,
}

impl View {
    /// The next view in the switch's cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::OverTheShoulder,
            Self::OverTheShoulder => Self::FirstPerson,
            Self::FirstPerson => Self::Overview,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::OverTheShoulder => "over the shoulder",
            Self::FirstPerson => "first person",
        }
    }

    /// The name a scenario or a receipt reads back: one word, hyphenated.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::OverTheShoulder => "over-the-shoulder",
            Self::FirstPerson => "first-person",
        }
    }

    pub fn parse(slug: &str) -> Option<Self> {
        [Self::Overview, Self::OverTheShoulder, Self::FirstPerson]
            .into_iter()
            .find(|view| view.slug() == slug)
    }
}

/// The perspective presets, in world cells, facing where the body aims. The
/// far plane keeps the view's ground about the overview's, within the
/// terrain residency budget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerspectivePolicy {
    pub vertical_fov: f32,
    pub near: f32,
    pub far: f32,
    /// First person's look below level, in radians.
    pub pitch: f32,
    pub shoulder_distance: f32,
    pub shoulder_rise: f32,
}

impl Default for PerspectivePolicy {
    fn default() -> Self {
        Self { vertical_fov: 1.2, near: 0.02, far: 28.0, pitch: 0.12, shoulder_distance: 3.0, shoulder_rise: 1.5 }
    }
}

/// The height in cells of the body the default policy's distances were set
/// for: Eponym's, standing about two cells.
pub const REFERENCE_BODY_HEIGHT: f32 = 2.0;

impl PerspectivePolicy {
    /// The policy with its distances (near, far and the shoulder's) scaled to
    /// a body `height` cells tall, so a creature of any size is framed alike.
    /// Field of view and pitch are angles and stay.
    pub fn for_body(self, height: f32) -> Self {
        let k = if height.is_finite() && height > 0.0 { height / REFERENCE_BODY_HEIGHT } else { 1.0 };
        Self { near: self.near * k, far: self.far * k, shoulder_distance: self.shoulder_distance * k,
            shoulder_rise: self.shoulder_rise * k, ..self }
    }

    /// The camera for a perspective `view` from `eye` toward `heading`;
    /// `None` for the overview, which its host frames.
    pub fn camera(self, view: View, eye: [f32; 3], heading: [f32; 3], aspect: f32) -> Option<SlabCamera> {
        match view {
            View::Overview => None,
            View::FirstPerson => SlabCamera::first_person(eye, heading, self.pitch, self.vertical_fov,
                aspect, self.near, self.far),
            View::OverTheShoulder => SlabCamera::over_the_shoulder(eye, heading, self.shoulder_distance,
                self.shoulder_rise, self.vertical_fov, aspect, self.near, self.far),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switch_cycles_through_all_three_and_back() {
        let mut view = View::default();
        let seen: Vec<_> = (0..3).map(|_| { let was = view; view = view.next(); was }).collect();
        assert_eq!(seen, [View::Overview, View::OverTheShoulder, View::FirstPerson]);
        assert_eq!(view, View::Overview);
        for v in seen {
            assert_eq!(View::parse(v.slug()), Some(v));
        }
    }

    #[test]
    fn a_larger_body_scales_the_distances_and_not_the_angles() {
        let base = PerspectivePolicy::default();
        let big = base.for_body(REFERENCE_BODY_HEIGHT * 12.0);
        assert_eq!((big.far, big.shoulder_distance), (base.far * 12.0, base.shoulder_distance * 12.0));
        assert_eq!((big.vertical_fov, big.pitch), (base.vertical_fov, base.pitch));
        assert_eq!(base.for_body(REFERENCE_BODY_HEIGHT), base);
        assert_eq!(base.for_body(f32::NAN), base);
    }

    #[test]
    fn only_the_perspective_views_have_a_policy_camera() {
        let policy = PerspectivePolicy::default();
        let (eye, heading) = ([4.0, 3.0, 2.0], [0.0, 0.0, 1.0]);
        assert!(policy.camera(View::Overview, eye, heading, 1.5).is_none());
        for view in [View::OverTheShoulder, View::FirstPerson] {
            let camera = policy.camera(view, eye, heading, 1.5).unwrap();
            assert!(camera.frustum().is_some(), "{view:?}");
        }
    }
}
