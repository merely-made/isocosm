// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world rule of members' learning (cases 1051, 1053 and 1058;
//! mesocosm-overlay M3.6). Unset, no member carries a delta, and the world
//! serializes, hashes and behaves as before.

use crate::{Result, schema::{Learned, Pending, Tick}};
use serde::{Deserialize, Serialize};

/// How a member's learned delta moves, fades and passes on (case 1051).
/// Weights are in the policy's 64ths, like the drives they are added to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Learning {
    /// How far one served or failed act moves a weight.
    pub step: i32,
    /// What the step is multiplied by (numerator, denominator) while the
    /// member is younger than its lineage's maturity.
    pub youth: [u32; 2],
    /// The largest absolute weight.
    pub bound: i32,
    /// The share (numerator, denominator) of each weight lost every `span`.
    pub fade: [u32; 2],
    /// Ticks in one span of fading; only whole spans count.
    pub span: Tick,
    /// The share (numerator, denominator) of a parent's delta a child takes
    /// at birth (case 1058: a third).
    pub pass: [u32; 2],
    /// Ticks after an act that did not serve at once in which a later act
    /// that serves still credits it; nought keeps no delayed credit.
    pub horizon: Tick,
}

impl Default for Learning {
    /// First cuts to tune: a step of a sixteenth of a unit drive, doubled in
    /// youth, bounded at half a unit, a quarter lost each span, a span a
    /// day of minute ticks, and a dozen ticks to wait on a payoff.
    fn default() -> Self {
        Self { step: 4, youth: [2, 1], bound: 32, fade: [1, 4], span: 1440, pass: [1, 3], horizon: 12 }
    }
}

impl Learning {
    pub fn validate(&self) -> Result<()> {
        if self.step <= 0 || self.bound < self.step || self.bound > i32::from(i16::MAX) {
            return Err("learning needs a step above nought and a bound from the step to 32767".into());
        }
        if self.youth[0] == 0 || self.youth[1] == 0 || self.span == 0 {
            return Err("learning needs a youth multiplier and a span above nought".into());
        }
        if self.fade[1] == 0 || self.fade[0] > self.fade[1] || self.pass[1] == 0 || self.pass[0] > self.pass[1] {
            return Err("learning's fade and pass are shares of at most a whole".into());
        }
        Ok(())
    }

    /// `learned` as it stands at `at`: each whole span since it was settled
    /// has taken the fade's share of every weight, toward nought, and the
    /// settling moved on by those spans. Pure integer arithmetic, so reading
    /// it at any tick says the same of the same history.
    pub fn faded(&self, learned: &Learned, at: Tick) -> Learned {
        let spans = at.saturating_sub(learned.settled) / self.span;
        let mut out = Learned {
            settled: learned.settled.saturating_add(spans.saturating_mul(self.span)),
            weights: learned.weights.clone(),
            acts: learned.acts.clone(),
            pending: learned.pending.clone(),
        };
        let [lost, whole] = self.fade;
        let mut left = spans;
        while left > 0 && lost > 0 && !(out.weights.is_empty() && out.acts.is_empty()) {
            let keep = |w: &mut i32| *w = (i64::from(*w) * i64::from(whole - lost) / i64::from(whole)) as i32;
            out.weights.values_mut().for_each(keep);
            out.acts.values_mut().for_each(keep);
            out.weights.retain(|_, w| *w != 0);
            out.acts.retain(|_, w| *w != 0);
            left -= 1;
        }
        out
    }

    /// Whether a pending act has waited out its horizon by `at`: an act
    /// that ended at `since` can still be credited through `since + horizon`.
    pub fn lapsed(&self, pending: &Pending, at: Tick) -> bool {
        at > pending.since.saturating_add(self.horizon)
    }

    /// `faded`, and a pending act whose horizon has passed settled as a
    /// failure: its weight steps down, by the step of a member `young` at
    /// the tick the horizon passed. Pure, so a read says what the next
    /// learning step will settle (it writes only then).
    pub fn settled(&self, learned: &Learned, at: Tick, young: impl Fn(Tick) -> bool) -> Learned {
        let mut out = self.faded(learned, at);
        if let Some(p) = out.pending.take_if(|p| self.lapsed(p, at)) {
            let by = self.moved(young(p.since.saturating_add(self.horizon).saturating_add(1)));
            let w = out.acts.entry(p.act).or_insert(0);
            *w = (*w - by).clamp(-self.bound, self.bound);
            out.acts.retain(|_, w| *w != 0);
        }
        out
    }

    /// The distance one act moves a weight: the step, times the youth
    /// multiplier while `young`.
    pub fn moved(&self, young: bool) -> i32 {
        match young {
            true => (i64::from(self.step) * i64::from(self.youth[0]) / i64::from(self.youth[1]))
                .min(i64::from(self.bound)) as i32,
            false => self.step,
        }
    }

    /// What a child takes of its parent's delta, born at `tick` (case 1058):
    /// the share of each weight as faded to then, integer division toward
    /// nought, settled at the birth. A pending act stays with the parent.
    pub fn handed(&self, parent: &Learned, tick: Tick) -> Option<Learned> {
        let faded = self.faded(parent, tick);
        let [n, d] = self.pass;
        let share = |w: i32| (i64::from(w) * i64::from(n) / i64::from(d)) as i32;
        let weights = faded.weights.into_iter().map(|(k, w)| (k, share(w))).filter(|(_, w)| *w != 0).collect();
        let acts = faded.acts.into_iter().map(|(k, w)| (k, share(w))).filter(|(_, w)| *w != 0).collect();
        Some(Learned { settled: tick, weights, acts, pending: None }).filter(|k| !k.is_empty())
    }
}
