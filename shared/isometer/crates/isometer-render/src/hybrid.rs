// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Per-body R0/R1 selection for case 1161. The caller owns the quality preset
//! and decides which subjects to examine.

use std::num::NonZeroU64;

/// Finest representation the held body data can supply, in increasing order
/// of detail. Sprite availability here means the inputs needed to bake it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BodyDetail {
    /// Neither hybrid rung is available. A caller may use a coarser rung.
    Unavailable,
    Sprite,
    Live,
}

/// A sprite refresh class, independent of simulation time and the body rung.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshClass {
    Slow,
    Medium,
    Fast,
}

/// Pixel thresholds and sprite refresh periods, ordered from far to near.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridPolicy {
    /// `[leave, enter]` for Medium, Fast, and Live respectively.
    bands: [[f32; 2]; 3],
    periods: [NonZeroU64; 3],
}

/// Keep the returned choice per subject and view for the next selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyChoice {
    pub detail: BodyDetail,
    pub refresh: RefreshClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodySelection {
    pub choice: BodyChoice,
    /// Route through the existing examination path. The resulting detail must
    /// be supplied as `held` on a later selection before it can be drawn.
    pub request_live_detail: bool,
}

impl HybridPolicy {
    /// Pass `[leave, enter]` pixel bands for Medium, Fast and Live, in that
    /// order. They must be finite, nonnegative and ordered without overlap.
    /// Periods are `[slow, medium, fast]`, measured in displayed frames.
    pub fn new(bands: [[f32; 2]; 3], periods: [NonZeroU64; 3]) -> Option<Self> {
        let thresholds = bands.iter().flatten().copied();
        if bands[0][0] < 0.0
            || !thresholds.clone().all(f32::is_finite)
            || !thresholds.is_sorted()
            || !periods.iter().rev().is_sorted()
        {
            return None;
        }
        Some(Self { bands, periods })
    }

    pub fn refresh_period(self, class: RefreshClass) -> NonZeroU64 {
        self.periods[class as usize]
    }

    /// Select from the body's projected pixel extent. With no prior choice,
    /// the entry thresholds apply. Invalid sizes act as zero; callers can
    /// cull bodies with no projection before selecting.
    pub fn select(
        self,
        pixels: f32,
        previous: Option<BodyChoice>,
        examined: bool,
        held: BodyDetail,
    ) -> BodySelection {
        use BodyDetail::{Live, Sprite};
        use RefreshClass::{Fast, Medium, Slow};

        let pixels = if pixels.is_finite() {
            pixels.max(0.0)
        } else {
            0.0
        };
        let above =
            |band: usize, was_above: bool| pixels >= self.bands[band][usize::from(!was_above)];
        let wants_live = examined || above(2, previous.is_some_and(|p| p.detail == Live));
        let detail = (if wants_live { Live } else { Sprite }).min(held);
        let refresh = if above(1, previous.is_some_and(|p| p.refresh == Fast)) {
            Fast
        } else if above(0, previous.is_some_and(|p| p.refresh != Slow)) {
            Medium
        } else {
            Slow
        };

        BodySelection {
            choice: BodyChoice { detail, refresh },
            request_live_detail: examined && held < Live,
        }
    }
}
