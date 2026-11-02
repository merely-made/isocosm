// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's walking speed (doctrine Bodies 16; cases 1085, 1087, 1095).
//! Animals of every size walk near one Froude number, so speed goes as the
//! square root of leg length: a world's baseline is the speed of a body whose
//! leg is the world's reference, and any other leg scales it by the root of
//! their ratio. Bodies without legs move a world rate of body lengths. The
//! cap joint swing puts on stride waits for joint ranges (case 1088).

use isometer_core::{BodyDocument, legs};
use serde::{Deserialize, Serialize};

/// A world's speeds, in thousandths of a cell per world second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Speed {
    /// The speed of a body whose leg is the reference, in milli-cells per
    /// world second.
    pub baseline: u64,
    /// The reference leg, hip to sole, in base cells.
    pub leg: u64,
    /// Body lengths a legless body moves per world second, in thousandths.
    pub legless: u64,
}

impl Speed {
    /// A world's speeds where a cell is `micrometres` across: a one-cell
    /// leg walking at Froude 0.25 under Earth's gravity, v = √(Fr·g·L), and
    /// a legless body a length a second.
    pub fn froude(micrometres: u64) -> Self {
        // (v / L)² = Fr·g / L, in (milli-cells per second)²: 0.25 × 9.81 m/s²
        // is 2.4525e6 µm/s², over L µm, times 1000².
        let baseline = (2_452_500_000_000u128 / u128::from(micrometres.max(1))).isqrt();
        Self { baseline: baseline as u64, leg: 1, legless: 1000 }
    }

    /// `doc`'s speed in milli-cells per world second: by its mean leg under
    /// the Froude rule, or by its length if it has no legs. Zero for a body
    /// with neither.
    pub fn of(&self, doc: &BodyDocument) -> u64 {
        let chains = legs::chains(doc);
        if chains.is_empty() {
            let [x, _, z] = doc.aabb().extent();
            let length = x.max(z).max(0) as u64;
            return length.saturating_mul(self.legless);
        }
        let total: u128 = chains.iter().map(|l| (l.lengths[0] + l.lengths[1]).max(0) as u128).sum();
        let leg = total / chains.len() as u128;
        let reference = u128::from(self.leg.max(1)) * legs::UNIT as u128;
        // The factor √(leg / reference), in millionths before its root.
        let factor = (leg * 1_000_000 / reference).isqrt();
        (u128::from(self.baseline) * factor / 1000).min(u128::from(u64::MAX)) as u64
    }

    /// Whole ticks of `tick_microseconds` a body at `speed` milli-cells per
    /// world second takes over `cells`, at least one.
    pub fn ticks(cells: u64, speed: u64, tick_microseconds: u64) -> Option<u64> {
        if speed == 0 {
            return None;
        }
        // Microseconds: cells / (speed / 1000) seconds.
        let micros = u128::from(cells) * 1_000_000_000 / u128::from(speed);
        let ticks = micros.div_ceil(u128::from(tick_microseconds.max(1))).max(1);
        u64::try_from(ticks).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_goes_as_the_root_of_leg_and_time_as_distance() {
        let s = Speed { baseline: 1000, leg: 4, legless: 500 };
        // A leg four times the reference walks twice as fast.
        let factor = |leg: u128| (leg * 1_000_000 / (4 * legs::UNIT as u128)).isqrt();
        assert_eq!(factor(16 * legs::UNIT as u128), 2000);
        assert_eq!(factor(4 * legs::UNIT as u128), 1000);
        // 120 cells at 2 cells a second is a minute: one tick, and 121 two.
        assert_eq!(Speed::ticks(120, 2000, 60_000_000), Some(1));
        assert_eq!(Speed::ticks(121, 2000, 60_000_000), Some(2));
        assert_eq!(Speed::ticks(0, 2000, 60_000_000), Some(1));
        assert_eq!(Speed::ticks(5, 0, 60_000_000), None);
        let _ = s;
        // A millimetre leg walks about 5 cm a second; a 0.9 m one 1.5 m/s.
        assert_eq!(Speed::froude(1000).baseline, 49_522);
        assert_eq!(Speed::froude(900_000).baseline / 1000, 1);
    }
}
