// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation-only anchoring for the continuous uptake pulse. Under
//! continuous uptake a stationary recipient records a transfer every tick, so
//! the newest record is always one tick old and a pulse aged from it never
//! moves. The age here comes from the tick the recipient's current run of
//! consecutive uptake ticks began; a gap of a whole tick without uptake breaks
//! the run and re-anchors it. Retained flow facts are untouched by this.
use super::{GlyphOrientation, SpatialGlyph, Stroke};
use mesocosm_core::OrganismId;
use std::collections::{BTreeMap, btree_map::Entry};

/// Ticks a pulse lives, matching the retention of every other trial mark.
pub(super) const PULSE_TICKS: u64 = 8;

#[derive(Clone, Copy, Debug)]
struct Run {
    anchor: u64,
    last: u64,
}

/// Per-recipient run anchors. Observation is driven by accepted uptake
/// records, so it counts ticks of flow, not frames.
#[derive(Default)]
pub(super) struct Pulses {
    runs: BTreeMap<OrganismId, Run>,
}

impl Pulses {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn reset(&mut self) {
        self.runs.clear();
    }

    /// Note uptake by `organism` at `tick`. Repeating a tick is idempotent;
    /// anything other than the next consecutive tick starts a new run.
    pub(super) fn observe(&mut self, organism: OrganismId, tick: u64) {
        match self.runs.entry(organism) {
            Entry::Occupied(mut slot) => {
                let run = slot.get_mut();
                if tick == run.last || tick == run.last.saturating_add(1) {
                    run.last = tick;
                } else if tick > run.last {
                    *run = Run {
                        anchor: tick,
                        last: tick,
                    };
                }
            },
            Entry::Vacant(slot) => {
                slot.insert(Run {
                    anchor: tick,
                    last: tick,
                });
            },
        }
    }

    /// Ticks since the recipient's current run began, or None once nothing is
    /// anchored for it.
    pub(super) fn age(&self, organism: OrganismId, tick: u64) -> Option<u64> {
        let run = self.runs.get(&organism)?;
        Some(tick.saturating_sub(run.anchor))
    }

    /// Drop runs whose last uptake is older than a pulse's life; their records
    /// have expired too, so nothing can refer to them.
    pub(super) fn retain(&mut self, tick: u64) {
        self.runs
            .retain(|_, run| tick.saturating_sub(run.last) < PULSE_TICKS);
    }
}

/// Glyph for one recipient's pulse, `ticks` after its run began. None once the
/// pulse has expired. Recipient-level indicator: the flow has no soil cell or
/// root tip, so the mark rises out of the body rather than marking a contact.
pub(super) fn pulse(at: [i32; 3], ticks: u64, height: f32, size: f32) -> Option<SpatialGlyph> {
    // Unbroken flow keeps pulsing: the phase wraps every PULSE_TICKS while
    // the run lives, and `Pulses::retain` retires the run once flow stops.
    let age = (ticks % PULSE_TICKS) as f32 / PULSE_TICKS as f32;
    let mut centre = at.map(|v| v as f32);
    centre[1] += height + age * size * 2.;
    Some(SpatialGlyph {
        centre,
        size: size * (1. - age * 0.5),
        angle: 0.,
        glyph: Stroke::Backticks,
        orientation: GlyphOrientation::WorldPlane {
            right: [1., 0., 0.],
            up: [0., 1., 0.],
        },
        color: [0.6, 0.85, 0.3, 1.],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHO: OrganismId = OrganismId(1);
    const AT: [i32; 3] = [2, 3, 4];

    fn height(pulses: &Pulses, tick: u64) -> f32 {
        pulse(AT, pulses.age(WHO, tick).expect("anchored"), 4., 1.4)
            .expect("live pulse")
            .centre[1]
    }

    #[test]
    fn continuous_uptake_rises_across_consecutive_ticks() {
        let mut pulses = Pulses::new();
        let mut last = f32::MIN;
        for tick in 1..=5 {
            pulses.observe(WHO, tick);
            pulses.retain(tick);
            assert_eq!(pulses.age(WHO, tick), Some(tick - 1));
            let now = height(&pulses, tick);
            assert!(now > last, "tick {tick}: {now} did not rise above {last}");
            last = now;
        }
        // Eight ticks of unbroken flow start the next pulse rather than
        // retiring the recipient's indicator while uptake continues.
        for tick in 6..=9 {
            pulses.observe(WHO, tick);
        }
        assert_eq!(pulses.age(WHO, 9), Some(8));
        assert_eq!(height(&pulses, 9), height(&pulses, 1));
        assert!(height(&pulses, 8) > height(&pulses, 9));
    }

    #[test]
    fn a_gap_of_one_tick_re_anchors_the_run() {
        let mut pulses = Pulses::new();
        for tick in [1, 2, 3] {
            pulses.observe(WHO, tick);
        }
        assert_eq!(pulses.age(WHO, 3), Some(2));
        // Tick 4 has no uptake for this recipient; tick 5 starts a fresh run.
        pulses.observe(WHO, 5);
        assert_eq!(pulses.age(WHO, 5), Some(0));
        // A fresh run sits at the bare marker height above the recipient.
        assert_eq!(height(&pulses, 5), AT[1] as f32 + 4.);
        pulses.observe(WHO, 6);
        assert_eq!(pulses.age(WHO, 6), Some(1));
        assert!(height(&pulses, 6) > height(&pulses, 5));
    }

    #[test]
    fn repeated_and_stale_observations_are_handled() {
        let mut pulses = Pulses::new();
        pulses.observe(WHO, 4);
        pulses.observe(WHO, 4);
        pulses.observe(WHO, 3);
        assert_eq!(pulses.age(WHO, 4), Some(0));
        pulses.retain(4 + PULSE_TICKS - 1);
        assert!(pulses.age(WHO, 11).is_some());
        pulses.retain(4 + PULSE_TICKS);
        assert!(pulses.age(WHO, 12).is_none());
        pulses.observe(WHO, 12);
        assert_eq!(pulses.age(WHO, 12), Some(0));
        pulses.reset();
        assert!(pulses.age(WHO, 12).is_none());
    }
}
