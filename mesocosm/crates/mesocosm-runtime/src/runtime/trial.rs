// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A bounded, disposable run of an exact specimen world. The baseline world
//! plus the applied trace is its replay source; seed/count alone cannot replay
//! a proportion edit or resized content palette. This exposes no shipping
//! Runtime receipt and never commits its result back to the specimen.

use super::Runtime;
#[cfg(test)]
use super::deep_time_world;
use crate::Checkpoint;
use mesocosm_core::flow::{Account, Process, RecordedFlow};
use mesocosm_core::{History, Intent, OrganismId, Outcome, World, history::Event, state_hash};
use serde::Serialize;
use std::collections::BTreeMap;

/// How long a trial may run.
///
/// **One starter's lifespan** — the 3,000 ticks `rules::DEFAULT_EPOCH_TICKS`
/// measures itself against, so a trial is bounded by one body's life rather
/// than by a round number. Raised from 128 by Mark on 2026-09-16.
///
/// 128 was under the epoch's own 1,000-tick budget, so a trial stopped 872
/// ticks short of its first reckoning, every time. That put every **feat**
/// out of reach: a deed is significant, and the significant half of the deed
/// vocabulary is read from the reckoning, which only exists once an epoch has
/// closed. A trial now reaches that boundary and runs past it; room for three
/// is the point, since a fresh world's first reckoning sets marks on an empty
/// record rather than beating them. A birth or death under the hand still
/// stops a trial, because a trial never answers a checkpoint.
pub const MAX_TRIAL_STEPS: u32 = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UptakePosition {
    AfterTick,
    BeforeTick,
    Unavailable,
}
impl UptakePosition {
    pub fn label(self) -> &'static str {
        match self {
            Self::AfterTick => "recipient position after tick",
            Self::BeforeTick => "recipient position before tick",
            Self::Unavailable => "recipient position unavailable",
        }
    }
}

/// Actual positive soil uptake, kept separate from biographical events.
/// Source soil-cell and responsible organ coordinates are not in this record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialUptake {
    pub tick: u64,
    /// Ordinal in the complete accepted flow batch for this tick. Identity is
    /// (tick, sequence), not the unrelated history sequence in TrialActivity.
    pub sequence: u64,
    pub record: RecordedFlow,
    pub organism: OrganismId,
    pub at: Option<[i32; 3]>,
    pub position_basis: UptakePosition,
}

/// Positive accepted terrain removal. `at` is the recorded command centre,
/// not a reconstructed surface contact or normal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialCarve {
    pub tick: u64,
    /// Ordinal in this trial's complete history, shared with TrialActivity.
    pub sequence: u64,
    pub organism: OrganismId,
    pub at: [i32; 3],
    pub removed: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialActivity {
    /// Zero-based ordinal in this disposable trial's accepted history.
    pub sequence: u64,
    /// Core event tick, not a wall clock or frame number.
    pub tick: u64,
    pub event: Event,
    /// Mover or eater. This record does not identify a responsible organ.
    pub source: OrganismId,
    /// The donor for feeding; absent for movement.
    pub target: Option<OrganismId>,
    /// Movement's recorded departure, or the donor's pre-step position.
    pub from: [i32; 3],
    /// Movement's recorded arrival, or the eater's post-step position.
    pub to: [i32; 3],
}

pub struct Trial {
    baseline: World,
    /// What `baseline` carries in from before this trial: empty for a fresh
    /// admitted world, deep time's own log for a world built by
    /// [`Trial::with_past`]. Kept beside `baseline` so `reset` rebuilds the
    /// runtime with both, not just the world.
    baseline_history: History,
    runtime: Runtime,
    steps: u32,
    /// `MAX_TRIAL_STEPS` for every trial a product builds. Only a test sets
    /// it lower, through `with_ceiling`, so it survives `reset`.
    ceiling: u32,
    activities: Vec<TrialActivity>,
    uptakes: Vec<TrialUptake>,
    carves: Vec<TrialCarve>,
    glyphs: Option<crate::glyphs::GlyphReading>,
}

impl Trial {
    /// The bench currently supplies fresh admitted worlds. A later snapshot
    /// needs its runtime checkpoint/history too, so refuse it rather than
    /// silently reconstructing a missing decision or past. A world with a
    /// real past goes through [`Trial::with_past`] instead.
    pub fn new(source: &World) -> Result<Self, String> {
        if source.tick != 0 || source.epoch != 0 || source.at_boundary() {
            return Err("specimen trial requires a fresh tick-zero world".into());
        }
        Self::from_baseline(source, History::new())
    }

    /// A trial over a baseline that already has a past: deep time's
    /// handover, not a fresh admitted world (D4). Accepts a world standing
    /// anywhere past tick zero, including exactly on an epoch boundary,
    /// provided `history` is the record that got it there. An empty history
    /// paired with a world already in motion is refused, the same way
    /// [`Trial::new`] refuses a bare snapshot: replay would have nothing to
    /// reckon the world's own epoch against.
    pub fn with_past(source: &World, history: &History) -> Result<Self, String> {
        let has_past = source.tick != 0 || source.epoch != 0 || source.at_boundary();
        if has_past && history.is_empty() {
            return Err(
                "specimen trial requires the baseline's history for a world past tick zero".into(),
            );
        }
        Self::from_baseline(source, history.clone())
    }

    /// What both entry points share: a living controlled body, and the
    /// exact baseline — world and history both — that `reset` rebuilds from.
    fn from_baseline(source: &World, baseline_history: History) -> Result<Self, String> {
        if source.controlled().is_none_or(|body| !body.is_alive()) {
            return Err("specimen trial requires a living controlled body".into());
        }
        // Pending founding events are serialized world state. Preserve them
        // exactly; the ordinary runtime absorbs them on its first step, while
        // the activity projection below excludes Born and other founding facts.
        let baseline = source.clone();
        let runtime = driver(&baseline, &baseline_history);
        Ok(Self {
            baseline,
            baseline_history,
            runtime,
            steps: 0,
            ceiling: MAX_TRIAL_STEPS,
            activities: Vec::new(),
            uptakes: Vec::new(),
            carves: Vec::new(),
            glyphs: None,
        })
    }

    /// A trial that refuses past `ceiling` steps rather than the shipped one.
    ///
    /// **For tests that run a trial to its end.** They were written against a
    /// ceiling of 128: they step until refused, assert the refusal changes
    /// nothing, reset and replay to the end again, and some compare the whole
    /// history on every step. Against the 3,000-step ceiling that is 23 times
    /// the ticks, quadratic where history is compared, in a debug build. A
    /// short ceiling keeps what each of them proves and costs what it did.
    /// The ceiling every run-to-the-end test was written against.
    #[cfg(test)]
    pub(crate) const SHORT: u32 = 128;

    #[cfg(test)]
    pub(crate) fn with_ceiling(source: &World, ceiling: u32) -> Result<Self, String> {
        let mut trial = Self::new(source)?;
        trial.ceiling = ceiling.min(MAX_TRIAL_STEPS);
        Ok(trial)
    }

    /// Opt into the gameplay-data experiment before any tick. Invalid rules
    /// leave an existing configuration intact. No rendering action grants it.
    pub fn enable_glyphs(&mut self, rules: crate::glyphs::GlyphRules) -> Result<(), String> {
        if self.steps != 0 {
            return Err("enable glyph rules before the first trial step".into());
        }
        let reading = crate::glyphs::GlyphReading::new(rules, &self.baseline)?;
        self.glyphs = Some(reading);
        Ok(())
    }

    pub fn glyphs(&self) -> Option<&crate::glyphs::GlyphReading> {
        self.glyphs.as_ref()
    }

    pub fn world(&self) -> &World {
        self.runtime.world()
    }
    pub fn history(&self) -> &History {
        self.runtime.history()
    }
    pub fn steps(&self) -> u32 {
        self.steps
    }
    pub fn state_hash(&self) -> u64 {
        self.runtime.state_hash()
    }
    pub fn baseline_hash(&self) -> u64 {
        state_hash(&self.baseline)
    }
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.runtime.checkpoint()
    }
    pub fn trace(&self) -> &[Intent] {
        self.runtime.trace()
    }
    /// Latest successful step's batch. Reading or redrawing never drains it;
    /// process it only when `step()` returns true, keyed by tick/sequence.
    pub fn activities(&self) -> &[TrialActivity] {
        &self.activities
    }
    /// Latest successful step's accepted uptake reading. Repeated reads do
    /// not consume it; process only after a successful step, by tick/ordinal.
    pub fn uptakes(&self) -> &[TrialUptake] {
        &self.uptakes
    }
    /// Latest applied step's positive terrain changes. Reads never consume it.
    pub fn carves(&self) -> &[TrialCarve] {
        &self.carves
    }
    /// Actual core outcomes of the last applied step, including rejections.
    /// A refused trial action (false) preserves this previous result.
    pub fn last_outcomes(&self) -> &[Outcome] {
        self.runtime.last_outcomes()
    }
    pub fn drain_ground_dirty(&mut self) -> Vec<[i16; 3]> {
        self.runtime.drain_ground_dirty()
    }

    pub fn reset(&mut self) {
        self.runtime = driver(&self.baseline, &self.baseline_history);
        self.steps = 0;
        self.activities.clear();
        self.uptakes.clear();
        self.carves.clear();
        if let Some(reading) = &mut self.glyphs {
            reading.reset(&self.baseline);
        }
    }

    /// One ordinary idle step. Checkpoints retain their existing runtime
    /// semantics; the trial never answers a question on the user's behalf.
    pub fn step(&mut self) -> bool {
        self.apply_one(Intent::Idle)
    }

    /// Queue one ordinary carve intent and apply exactly one tick. True means
    /// a tick ran, including a core rejection; inspect `last_outcomes` for it.
    /// Checkpoints, the step bound and unrepresentable coordinate arithmetic
    /// refuse before queuing. Radius and anatomical reach remain core rules.
    pub fn carve(&mut self, at: [i32; 3], radius: i32) -> bool {
        if let Some(body) = self.world().controlled() {
            if (0..3).any(|axis| {
                at[axis]
                    .checked_sub(body.position[axis])
                    .and_then(i32::checked_abs)
                    .is_none()
            }) {
                return false;
            }
        }
        if (1..=2).contains(&radius)
            && at
                .iter()
                .any(|v| v.checked_sub(radius).is_none() || v.checked_add(radius).is_none())
        {
            return false;
        }
        self.apply_one(Intent::Carve { at, radius })
    }

    fn apply_one(&mut self, intent: Intent) -> bool {
        if self.steps >= self.ceiling || self.checkpoint().is_some() {
            return false;
        }
        let before: BTreeMap<_, _> = self
            .world()
            .organisms
            .iter()
            .map(|o| (o.id, o.position))
            .collect();
        let start = self.history().len();
        debug_assert_eq!(self.runtime.queued_len(), 0);
        self.runtime.queue(intent);
        if self.runtime.step(1) == 0 {
            return false;
        }
        self.steps += 1;
        self.activities.clear();
        self.carves.clear();
        self.uptakes = self
            .runtime
            .trial_flows
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                uptake(*record, index as u64, &before, self.runtime.world())
            })
            .collect();
        for (offset, recorded) in self.runtime.history().log().entries()[start..]
            .iter()
            .enumerate()
        {
            if let Event::Carved {
                organism,
                at,
                removed,
            } = recorded.record
            {
                if removed > 0 {
                    self.carves.push(TrialCarve {
                        tick: recorded.tick,
                        sequence: (start + offset) as u64,
                        organism,
                        at,
                        removed,
                    });
                }
            }
            let endpoints = match recorded.record {
                Event::Moved { organism, from, to } if from != to => {
                    Some((organism, None, from, to))
                },
                Event::Fed {
                    eater,
                    from: donor,
                    mass_mg,
                    ..
                } if mass_mg > 0 => {
                    let old = |id| before.get(&id).copied();
                    let now = |id| {
                        self.runtime
                            .world()
                            .organisms
                            .iter()
                            .find(|o| o.id == id)
                            .map(|o| o.position)
                    };
                    old(donor)
                        .or_else(|| now(donor))
                        .zip(now(eater).or_else(|| old(eater)))
                        .map(|(from, to)| (eater, Some(donor), from, to))
                },
                _ => None,
            };
            if let Some((source, target, from, to)) = endpoints {
                self.activities.push(TrialActivity {
                    sequence: (start + offset) as u64,
                    tick: recorded.tick,
                    event: recorded.record,
                    source,
                    target,
                    from,
                    to,
                });
            }
        }
        // Both accepted sources reach the adapter, history first: events in
        // log order, then this tick's positive soil uptake. Only the adapter
        // grants; nothing drawn ever gets here.
        let post_hash = self.runtime.state_hash();
        if let Some(reading) = &mut self.glyphs {
            reading.absorb(self.runtime.history(), start, post_hash);
            reading.absorb_uptake(&self.uptakes, post_hash);
        }
        true
    }
}

fn driver(baseline: &World, history: &History) -> Runtime {
    // Runtime's legacy seed/count fields are deliberately unexposed here.
    // The owned baseline is this trial's exact and only reconstruction source,
    // and its history rides beside it so a trial with a past reckons and
    // replays the same way a fresh one does.
    let mut runtime = Runtime::from_world_with_history(baseline.clone(), history.clone(), 0, 0, 1);
    runtime.trial_flows = Some(Vec::new());
    runtime
}

fn uptake(
    record: RecordedFlow,
    sequence: u64,
    before: &BTreeMap<OrganismId, [i32; 3]>,
    world: &World,
) -> Option<TrialUptake> {
    let flow = record.record;
    // Internal Substance→Reserve synthesis bookkeeping also uses Uptake.
    // Only the actual soil transfer is an intake, so never count both legs.
    if flow.process != Process::Uptake || flow.source != Account::Soil || flow.amount_mg == 0 {
        return None;
    }
    let organism = flow.to?.organism;
    let (at, position_basis) = if let Some(body) = world.organisms.iter().find(|o| o.id == organism)
    {
        (Some(body.position), UptakePosition::AfterTick)
    } else if let Some(at) = before.get(&organism) {
        (Some(*at), UptakePosition::BeforeTick)
    } else {
        (None, UptakePosition::Unavailable)
    };
    Some(TrialUptake {
        tick: record.tick,
        sequence,
        record,
        organism,
        at,
        position_basis,
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod carve_tests;
