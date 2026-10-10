// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host-neutral driver over a native `Session` (the switch, ruling 681):
//! the interim loop at site grain, stepped a round at a time. A host queues
//! contract envelopes, which this driver translates into native commands
//! (686); it never touches world state, and the session's log is the record
//! a replay reads back.

use std::collections::VecDeque;

use isocosm::directing::interim::{Happening, Interim, OnCollapse, Pace, Start};
use isocosm::directing::readings::{Mode, View};
use isocosm::flows::Flow;
use isocosm::schema::{Entity, Id};
use isocosm::simulation::{Genesis, Simulation};
use isocosm::{Execution, Session};
use isocosm_overlay::mesocosm::MesocosmIntentEnvelope;
use serde::{Deserialize, Serialize};

use crate::clock::Clock;
use crate::glyphs::{GlyphReading, GlyphRules};
use crate::readings::{FlowWindows, Trend};
use crate::review::Review;
use crate::succession::{self, Checkpoint};

mod translate;
pub use translate::Refusal;

/// Default ceiling on rounds one `advance` call runs.
pub const DEFAULT_MAX_STEPS_PER_ADVANCE: u64 = 8;

/// One contract envelope, as a host queues it.
pub type Envelope = MesocosmIntentEnvelope;

/// How a run was founded: what a replay needs besides the session's log.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Founded {
    pub genesis: Genesis,
    pub start: Start,
    pub mode: Mode,
    pub pace: Pace,
    pub on_collapse: OnCollapse,
}

impl Founded {
    /// A generated world with bodies and a played native lineage (682),
    /// one tick a round, so a host's fixed step is a tick.
    pub fn generated(seed: u64, population: u64) -> Result<Self, String> {
        let genesis = isocosm::Founding {
            seed,
            population,
            ecology: true,
            bodies: Some(isocosm::bodied::Bodies::default()),
            played: Some(isocosm::directing::Played {
                lineage: 1,
                region_sites: 2,
            }),
            ..Default::default()
        }
        .generate()?;
        Ok(Self {
            genesis,
            start: Start::default(),
            mode: Mode::Creative,
            pace: Pace {
                round: 1,
                scoring: 6,
            },
            on_collapse: OnCollapse::Stay,
        })
    }
}

/// Drives the interim loop at a fixed rate from a host's uneven frames.
pub struct Runtime {
    interim: Interim,
    founded: Founded,
    clock: Clock,
    queued: VecDeque<Envelope>,
    /// Every envelope applied, in order, with what it came to.
    trace: Vec<(Envelope, Result<String, Refusal>)>,
    /// The question the run is holding at, if any.
    checkpoint: Option<Checkpoint>,
    /// The played line's turn, while holding at a boundary.
    review: Option<Review>,
    /// What the last round brought.
    happenings: Vec<Happening>,
    windows: FlowWindows,
    /// An opt-in glyph reading of the played critter's acts (774).
    glyphs: Option<GlyphReading>,
    dev_intents: u64,
    /// The sim's refusal of a round, which stops the run until replaced.
    fault: Option<String>,
    max_steps: u64,
}

impl Runtime {
    pub fn new(founded: Founded, ticks_per_second: u32) -> Result<Self, String> {
        let mut interim = Interim::found(
            founded.genesis.clone(),
            founded.start,
            founded.mode,
            founded.pace,
            Execution::Grouped,
        )?;
        interim.on_collapse = founded.on_collapse;
        Ok(Self::over(interim, founded, ticks_per_second))
    }

    /// A generated world from `seed` (682).
    pub fn generated(seed: u64, population: u64, ticks_per_second: u32) -> Result<Self, String> {
        Self::new(Founded::generated(seed, population)?, ticks_per_second)
    }

    fn over(interim: Interim, founded: Founded, ticks_per_second: u32) -> Self {
        Self {
            interim,
            founded,
            clock: Clock::new(ticks_per_second),
            queued: VecDeque::new(),
            trace: Vec::new(),
            checkpoint: None,
            review: None,
            happenings: Vec::new(),
            windows: FlowWindows::new(),
            glyphs: None,
            dev_intents: 0,
            fault: None,
            max_steps: DEFAULT_MAX_STEPS_PER_ADVANCE,
        }
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        assert!(max_steps > 0, "at least one step per advance");
        self.max_steps = max_steps;
        self
    }

    /// Queues an envelope for the next step.
    pub fn queue(&mut self, envelope: Envelope) {
        self.queued.push_back(envelope);
    }

    pub fn queued_len(&self) -> usize {
        self.queued.len()
    }

    /// Runs the rounds the elapsed time authorises; a held run banks none.
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        if self.held() {
            self.drain_answers();
            return 0;
        }
        let advance = self.clock.advance(elapsed_us, self.max_steps);
        self.step(advance.steps)
    }

    /// Runs up to `steps` rounds off the clock; fewer while a question holds.
    pub fn step(&mut self, steps: u64) -> u64 {
        let mut taken = 0;
        for _ in 0..steps {
            if !self.step_once() {
                break;
            }
            taken += 1;
        }
        taken
    }

    fn held(&self) -> bool {
        self.checkpoint.is_some()
    }

    /// Applies queued answers while a question holds; anything else waits.
    fn drain_answers(&mut self) {
        while let Some(front) = self.queued.front() {
            let answers = self.checkpoint.as_ref().is_some_and(|c| c.answers(front));
            if !answers {
                return;
            }
            let envelope = self.queued.pop_front().expect("a front");
            self.apply(envelope);
        }
    }

    /// One round, after the queued envelopes: `false` while a question
    /// holds that nothing queued answers.
    fn step_once(&mut self) -> bool {
        if self.fault.is_some() {
            return false;
        }
        if self.held() {
            self.drain_answers();
            if self.held() {
                return false;
            }
        }
        while let Some(envelope) = self.queued.pop_front() {
            self.apply(envelope);
            if self.held() {
                return true;
            }
        }
        let revisions = isocosm::lineage::revise::revisions;
        match self.interim.round_with_flows(&revisions) {
            Ok((happenings, flows)) => self.absorb(happenings, &flows),
            Err(why) => {
                self.happenings.clear();
                self.fault = Some(why);
                return false;
            },
        }
        true
    }

    fn absorb(&mut self, happenings: Vec<Happening>, flows: &[Flow]) {
        self.windows
            .absorb(&self.interim.session.sim, &happenings, flows);
        if let Some(glyphs) = &mut self.glyphs {
            glyphs.absorb(&self.interim.session, flows);
        }
        self.checkpoint = succession::opened(&self.interim, &happenings);
        self.review = match self.checkpoint.as_ref().map(|c| &c.occasion) {
            Some(succession::Occasion::Epoch(_)) => crate::review::of(&self.interim),
            _ => None,
        };
        self.happenings = happenings;
    }

    /// Starts reading glyphs from the bound critter's acts from here on.
    pub fn read_glyphs(&mut self, rules: GlyphRules) -> Result<(), String> {
        self.glyphs = Some(GlyphReading::new(rules, &self.interim.session)?);
        Ok(())
    }

    pub fn glyphs(&self) -> Option<&GlyphReading> {
        self.glyphs.as_ref()
    }

    /// The question the run is holding at, if any.
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoint.as_ref()
    }

    /// The played line's turn, while holding at a boundary.
    pub fn review(&self) -> Option<&Review> {
        self.review.as_ref()
    }

    /// What the last round brought.
    pub fn happenings(&self) -> &[Happening] {
        &self.happenings
    }

    pub fn interim(&self) -> &Interim {
        &self.interim
    }

    pub fn session(&self) -> &Session {
        &self.interim.session
    }

    pub fn sim(&self) -> &Simulation {
        &self.interim.session.sim
    }

    pub fn founded(&self) -> &Founded {
        &self.founded
    }

    /// The critter the participant plays, if any.
    pub fn critter(&self) -> Option<Id> {
        self.interim.critter()
    }

    /// The played critter as the world holds it.
    pub fn played(&self) -> Option<&Entity> {
        let critter = self.critter()?;
        self.sim().state().population.get(critter)
    }

    /// What the player sees now, by mode (180, 691).
    pub fn view(&self) -> View {
        self.interim.view()
    }

    pub fn windows(&self) -> &FlowWindows {
        &self.windows
    }

    pub fn trend(&self) -> Trend {
        self.windows.trend()
    }

    /// Every envelope applied, with what it came to.
    pub fn trace(&self) -> &[(Envelope, Result<String, Refusal>)] {
        &self.trace
    }

    /// Why the sim refused a round, if it did: the run has stopped there.
    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    /// Dev intents this run applied (DT3).
    pub fn dev_intents(&self) -> u64 {
        self.dev_intents
    }

    pub fn tick(&self) -> u64 {
        self.sim().state().tick
    }

    pub fn state_hash(&self) -> u64 {
        self.sim().state_hash()
    }
}

mod replay;
pub use replay::{Receipt, Replayed};
