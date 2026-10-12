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
use crate::review::{Authored, Proposed, Review};
use crate::succession::{self, Checkpoint};

pub mod placing;
pub(crate) mod translate;
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
    /// A generated world with bodies, a map layout whose sites have
    /// coordinates (783) and a played native lineage (682), one tick a
    /// round, so a host's fixed step is a tick.
    pub fn generated(seed: u64, population: u64) -> Result<Self, String> {
        let grid = isocosm::map::Grid {
            width: 3,
            height: 2,
            ..isocosm::map::Grid::drawn(seed)
        };
        let genesis = isocosm::Founding {
            seed,
            population,
            sites: grid.width * grid.height,
            map: Some(isocosm::map::Layout::Grid(grid)),
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
    /// A pack's expression scripts, and what they made of the review (781).
    authored: Option<Authored>,
    proposed: Vec<Proposed>,
    /// What the last round brought.
    happenings: Vec<Happening>,
    windows: FlowWindows,
    /// An opt-in glyph reading of the played critter's acts (774).
    glyphs: Option<GlyphReading>,
    dev_intents: u64,
    /// The sim's refusal of a round, which stops the run until replaced.
    fault: Option<String>,
    max_steps: u64,
    spatial: crate::spatial::Game,
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
        let mut runtime = Self::over(interim, founded, ticks_per_second);
        runtime.place_members();
        Ok(runtime)
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
            authored: None,
            proposed: Vec::new(),
            happenings: Vec::new(),
            windows: FlowWindows::new(),
            glyphs: None,
            dev_intents: 0,
            fault: None,
            max_steps: DEFAULT_MAX_STEPS_PER_ADVANCE,
            spatial: crate::spatial::Game::default(),
        }
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        assert!(max_steps > 0, "at least one step per advance");
        self.max_steps = max_steps;
        self
    }

    /// Authored physical input; no display position supplies a missing value.
    pub fn set_spatial(&mut self, placements: &[crate::spatial::Placement]) -> Result<(), String> {
        self.spatial.replace(&self.interim.session.sim, placements)
    }

    pub fn setup_spatial(&mut self, setup: &crate::spatial::Setup) -> Result<(), String> {
        self.spatial.setup(&self.interim.session, setup)
    }

    pub fn spatial(&self) -> &crate::spatial::Game { &self.spatial }

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
        let spatial = &mut self.spatial;
        // This is the runtime's existing native clock, including periods
        // without a played member. It is not a player command or a new grant.
        let round = self.interim.round_with_flows_using(&revisions, &mut |session| {
            spatial.advance_clock(session).map(|out| out.flows)
        });
        match round {
            Ok((happenings, flows)) => {
                self.absorb(happenings, &flows);
                self.place_members();
                // Refresh valid same-cell physical bindings after native
                // placement/geometry changes; dormant history stays saved.
                if let Err(why) = self.spatial.refresh(&mut self.interim.session) {
                    self.fault = Some(why);
                    return false;
                }
            },
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
        self.proposed = self.authored_proposals();
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

    /// Gives the review a pack's expression scripts as a second source.
    pub fn with_authored(mut self, authored: Authored) -> Self {
        self.authored = Some(authored);
        self
    }

    /// What the scripts made of the open review's declared-tract offers.
    pub fn proposed(&self) -> &[Proposed] {
        &self.proposed
    }

    fn authored_proposals(&self) -> Vec<Proposed> {
        let (Some(authored), Some(review), Some(body)) =
            (&self.authored, &self.review, self.played())
        else {
            return Vec::new();
        };
        let lines = &self.sim().state().lineages;
        let held: Vec<String> = lines
            .get(&body.lineage)
            .and_then(|l| l.development.as_ref())
            .map(|d| d.tracts.iter().map(|t| crate::review::qualified(&t.function)).collect())
            .unwrap_or_default();
        authored.propose(review, body, &held, self.sim().genesis().seed)
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

    /// Native awareness refined by explicit in-site geometry, when the host
    /// has actual eye/body cells. It never uses the scene's fallback spots.
    pub fn sighted_view(&self, site: &crate::perception::InSite,
        sight: &crate::perception::Sight) -> Result<View, String> {
        if self.critter() != Some(sight.observer) {
            return Err("sight observer is not the currently played critter".into());
        }
        site.refine(self.sim(), self.interim.mode, sight)
    }

    /// Current edited geometry at the played site. Call again after admitted
    /// Edit/Patch commands or a round; departed critters have no site to lift.
    pub fn in_site(&self) -> Result<crate::perception::InSite, String> {
        let site = self.played().and_then(|e| e.resident_site())
            .ok_or("played critter has no resident site")?;
        let volume = placing::lift(self.sim(), site)?;
        crate::perception::InSite::of(self.sim(), volume, Default::default())
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
pub use replay::{GameSave, Receipt, Replayed};
