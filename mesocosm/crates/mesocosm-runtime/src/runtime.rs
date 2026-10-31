// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host-neutral driver over a native `Session` (the switch, ruling 681):
//! the interim loop at site grain, stepped a round at a time. A host queues
//! contract envelopes, which this driver translates into native commands
//! (686); it never touches world state, and the session's log is the record
//! a replay reads back. Since H3 (rulings 925 to 927) the session lives in
//! the sim-host core's `World` and the core's `Driver` queues, traces, caps
//! and paces; Mesocosm is the `Product` it drives, and the game sits in the
//! world's `mesocosm` section, the door its endpoint uses too.

use std::cell::Ref;

use cosmograph::directing::interim::{Handover, Happening, Interim, OnCollapse, Pace, Start};
use cosmograph::directing::readings::{Mode, View};
use cosmograph::schema::{Entity, Id, Tick};
use cosmograph::simulation::{Genesis, Simulation};
use cosmograph::{Execution, Session};
use cosmograph_host::drive::{Button, Context, Driver, InputRefusal, Pacing, Pick, Pointed, Press, Product};
use cosmograph_host::world::World;
use cosmograph_overlay::mesocosm::{MesocosmIntent, MesocosmIntentEnvelope};
use serde::{Deserialize, Serialize};

use crate::glyphs::{GlyphReading, GlyphRules};
use crate::readings::{FlowWindows, Trend};
use crate::review::{Authored, Proposed, Review};
use crate::succession::Checkpoint;

pub mod placing;
pub mod author_spread;
pub mod graft;
pub mod ecology;
mod founding;
pub(crate) mod translate;
pub mod keymap;
mod loop_state;
mod background;
mod frame;
pub use translate::Refusal;

/// Default ceiling on rounds one `advance` call runs.
pub const DEFAULT_MAX_STEPS_PER_ADVANCE: u64 = cosmograph_host::drive::DEFAULT_MAX_STEPS;

/// The world section that holds Mesocosm's game.
pub const MODE: &str = "mesocosm";

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
        Self::generated_with_affinity(seed, population, None)
    }
    pub fn generated_with_affinity(seed: u64, population: u64,
        affinity: Option<cosmograph::rules::Affinity>) -> Result<Self, String> {
        let grid = cosmograph::map::Grid {
            width: 3,
            height: 2,
            ..cosmograph::map::Grid::drawn(seed)
        };
        let genesis = cosmograph::Founding {
            seed,
            affinity,
            population,
            sites: grid.width * grid.height,
            map: Some(cosmograph::map::Layout::Grid(grid)),
            ecology: true,
            // Mesocosm's worlds wound and rest at the probe's rates (704, the
            // 2026-10-18 note to ruling 227); the generator's default does not.
            bodies: Some(cosmograph::bodied::Bodies {
                hazard: cosmograph::harm::hazard::RATE,
                wound: cosmograph::harm::hazard::CELLS,
                ..Default::default()
            }),
            played: Some(cosmograph::directing::Played {
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

/// What `Interim` holds besides its session: who plays, how, and since when.
#[derive(Clone, Debug)]
pub struct Loop {
    pub participant: Id,
    pub mode: Mode,
    pub pace: Pace,
    pub on_collapse: OnCollapse,
    pub handover: Handover,
    /// The boundary at which the played lineage first had a habitable site.
    pub habitable_at: Tick,
}

impl Loop {
    fn split(interim: Interim) -> (Session, Self) {
        let Interim { session, participant, mode, pace, on_collapse, handover, habitable_at } = interim;
        (session, Self { participant, mode, pace, on_collapse, handover, habitable_at })
    }
}

/// Mesocosm as the core driver's product: the loop's settings and saved
/// state, with the game mirrored into the world's section at each publish.
pub(crate) struct Mesocosm {
    pub(crate) lp: Loop,
    pub(crate) founded: Founded,
    /// The question the run is holding at, if any.
    pub(crate) checkpoint: Option<Checkpoint>,
    /// The played line's turn, while holding at a boundary.
    pub(crate) review: Option<Review>,
    /// A pack's expression scripts, and what they made of the review (781).
    pub(crate) authored: Option<Authored>,
    /// Host display forms; absent from saved native and game state.
    pub(crate) terms: mesocosm_phenotype::TermTable,
    pub(crate) proposed: Vec<Proposed>,
    /// What the last round brought.
    pub(crate) happenings: Vec<Happening>,
    pub(crate) windows: FlowWindows,
    /// An opt-in glyph reading of the played critter's acts (774).
    pub(crate) glyphs: Option<GlyphReading>,
    pub(crate) dev_intents: u64,
    /// The sim's refusal of a round, sealed into the saved loop; the driver
    /// stops on the same refusal.
    pub(crate) fault: Option<String>,
    pub(crate) spatial: crate::spatial::Game,
    verified_question: Option<crate::continuation::VerifiedQuestion>,
    geometry: crate::spatial::GeometryCache,
    /// Rounds from the clock run on a worker thread (`with_background`).
    background: bool,
    pending: Option<background::Pending>,
}

/// The session lent for reading; a product never holds it across a commit.
pub(crate) fn session(world: &World) -> Ref<'_, Session> {
    world.session().expect("the world is borrowed only between transactions")
}

/// One native transaction, carrying a product refusal through unchanged.
pub(crate) fn commit<T>(world: &World, admit: impl FnOnce(&mut Session) -> Result<T, Refusal>)
    -> Result<T, Refusal> {
    let before = world.cut().map_err(Refusal::Sim)?;
    let mut refused = None;
    world.product(before, |session| admit(session).map_err(|refusal| { refused = Some(refusal); String::new() }))
        .map_err(|why| refused.unwrap_or(Refusal::Sim(why)))
}

impl Mesocosm {
    fn held(&self) -> bool {
        self.checkpoint.is_some() || self.spatial.continuation.as_ref()
            .is_some_and(|saved| matches!(saved.phase,
                crate::continuation::LoopPhase::NewWorldRequested | crate::continuation::LoopPhase::WorldEnded))
    }

    pub(crate) fn critter(&self, world: &World) -> Option<Id> { session(world).sim.plays(self.lp.participant) }

    /// One round after the queued envelopes have applied.
    fn round(&mut self, world: &World) -> Result<(), String> {
        if frame::FRAME_TIER { self.frame_attention(world); }
        if self.background { return self.begin_round(world); }
        let _span = crate::profile::Span::new("round.total");
        self.sync_loop(world);
        let participant = self.lp.participant;
        let mut game = self.spatial.clone();
        let round =
            crate::profile::measure("round.transaction", || world.product_current(|session|
                crate::continuation::round::advance_with_flows_cached(
                    session, &mut game, participant, &mut self.geometry)
                    .map_err(|why| format!("{why:?}"))));
        match round {
            Ok((_, flows, verified)) => self.finish_round(world, game, flows, verified),
            Err(why) => self.refused(world, why),
        }
    }

    /// A round's game adopted after its session committed.
    fn finish_round(&mut self, world: &World, game: crate::spatial::Game,
        flows: Vec<cosmograph::flows::Flow>, verified: Option<crate::continuation::VerifiedQuestion>) -> Result<(), String> {
        self.spatial = game;
        self.verified_question = verified;
        if let Err(why) = self.restore_loop(world) { self.fault = Some(why.clone()); return Err(why); }
        if let Some(glyphs) = &mut self.glyphs { glyphs.absorb(&session(world), &flows); }
        // Preserve the old wholly unmarked run's patch placement only.
        // Marked runs retain their actually admitted physical bindings.
        if self.spatial.driver.is_none() && self.spatial.members.is_empty()
            && self.spatial.sight.is_none() { self.place_members(world); }
        self.sync_loop(world);
        Ok(())
    }

    fn refused(&mut self, world: &World, why: String) -> Result<(), String> {
        self.fault = Some(why.clone());
        self.sync_loop(world);
        Err(why)
    }
}

impl Product for Mesocosm {
    type Envelope = Envelope;
    type Refusal = Refusal;
    fn apply(&mut self, world: &World, envelope: &Envelope) -> Result<String, Refusal> { self.apply_envelope(world, envelope) }
    fn held(&self, _: &World) -> bool { self.held() }
    fn answers(&self, _: &World, envelope: &Envelope) -> bool {
        self.checkpoint.as_ref().is_some_and(|c| c.answers(envelope))
    }
    fn round(&mut self, world: &World) -> Result<(), String> { self.round(world) }
    fn press(&self, world: &World, press: &Press, context: &Context) -> Option<Envelope> {
        self.keyed(world, press, context).map(|intent| self.envelope(world, intent))
    }
    fn point(&self, world: &World, pick: Pick, button: Button, _: &Context) -> Option<Envelope> {
        self.pointed(world, pick, button).map(|intent| self.envelope(world, intent))
    }
}

/// Drives the interim loop at a fixed rate from a host's uneven frames.
pub struct Runtime {
    driver: Driver<Mesocosm>,
    /// The world's revision when this runtime last published its game.
    /// `None` means no successful mode-section publication/restore is known.
    seen: Option<u64>,
    published_phase: Option<crate::continuation::LoopPhase>,
    /// Clock time that passed while a worker round was out.
    banked_us: u64,
}

impl Runtime {
    /// A generated world from `seed` (682).
    pub fn generated(seed: u64, population: u64, ticks_per_second: u32) -> Result<Self, String> {
        Self::new(Founded::generated(seed, population)?, ticks_per_second)
    }
    pub fn generated_with_affinity(seed: u64, population: u64, ticks_per_second: u32,
        affinity: Option<cosmograph::rules::Affinity>) -> Result<Self, String> {
        Self::new(Founded::generated_with_affinity(seed, population, affinity)?, ticks_per_second)
    }

    fn over(interim: Interim, founded: Founded, ticks_per_second: u32) -> Self {
        let (session, lp) = Loop::split(interim);
        let world = World::new(session);
        Self::over_world(world, lp, founded, ticks_per_second)
    }

    fn over_world(world: World, lp: Loop, founded: Founded, ticks_per_second: u32) -> Self {
        let product = Mesocosm {
            lp,
            founded,
            checkpoint: None,
            review: None,
            authored: None,
            terms: Default::default(),
            proposed: Vec::new(),
            happenings: Vec::new(),
            windows: FlowWindows::new(),
            glyphs: None,
            dev_intents: 0,
            fault: None,
            spatial: crate::spatial::Game::default(),
            verified_question: None,
            geometry: crate::spatial::GeometryCache::default(),
            background: false,
            pending: None,
        };
        let driver = Driver::new(&world, product, Pacing::Clock { ticks_per_second });
        let mut runtime = Self { driver, seen: None, published_phase: None, banked_us: 0 };
        runtime.publish();
        runtime
    }

    /// The core's world this runtime drives; an endpoint over it shares the game.
    pub fn world(&self) -> &World { self.driver.world() }

    fn product(&self) -> &Mesocosm { self.driver.product() }

    /// Runs `change` over the product and the world, adopting any game an
    /// endpoint published since, then publishes the game for the next reader.
    pub(crate) fn with_product<T>(&mut self, change: impl FnOnce(&mut Mesocosm, &World) -> T) -> T {
        self.refresh();
        let world = self.driver.world().share();
        let out = change(self.driver.product_mut(), &world);
        self.publish();
        out
    }

    /// Writes the sealed game, the driver's queue and fault included, to the
    /// world's section.
    fn publish(&mut self) {
        let _span = crate::profile::Span::new("publish.total");
        self.seen = None;
        self.published_phase = None;
        let world = self.driver.world().share();
        let queued = self.driver.queued().cloned().collect();
        let game = crate::profile::measure("publish.loop_game", || self.driver.product().loop_game(&world, queued));
        if let Ok(bytes) = crate::profile::measure("publish.json", || serde_json::to_vec(&game)) {
            if crate::profile::measure("publish.section", || world.put_section(MODE, bytes)).is_ok() {
                self.seen = world.revision().ok();
                self.published_phase = game.continuation.as_ref().map(|saved| saved.phase);
            }
        }
    }

    /// Adopts a game another door published since this runtime's last publish.
    fn refresh(&mut self) {
        let world = self.driver.world().share();
        if self.seen.is_some_and(|seen| world.revision().is_ok_and(|now| now == seen)) { return; }
        self.seen = None;
        self.published_phase = None;
        let Ok(Some(bytes)) = world.section(MODE) else { return };
        let Ok(game) = serde_json::from_slice::<crate::spatial::Game>(&bytes) else { return };
        let product = self.driver.product_mut();
        product.spatial = game;
        if let Err(why) = product.restore_loop(&world) {
            product.fault = Some(why.clone());
            self.driver.restore_fault(Some(why));
            return;
        }
        self.seen = world.revision().ok();
        self.published_phase = self.product().spatial.continuation.as_ref().map(|saved| saved.phase);
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        self.driver.set_max_steps(max_steps);
        self
    }

    /// Authored physical input; no display position supplies a missing value.
    pub fn set_spatial(&mut self, placements: &[crate::spatial::Placement]) -> Result<(), String> {
        self.with_product(|product, world| {
            product.spatial.replace(&session(world).sim, placements)?;
            product.sync_loop(world);
            Ok(())
        })
    }

    pub fn setup_spatial(&mut self, setup: &crate::spatial::Setup) -> Result<(), String> {
        self.with_product(|product, world| {
            product.spatial.setup(&session(world), setup)?;
            product.sync_loop(world);
            Ok(())
        })
    }

    pub fn spatial(&self) -> &crate::spatial::Game { &self.product().spatial }

    /// Queues an envelope for the next step.
    pub fn queue(&mut self, envelope: Envelope) { self.driver.queue(envelope); }

    pub fn queued_len(&self) -> usize { self.driver.queued_len() }

    /// Live input (H4): a press through Mesocosm's keymap, refused at the
    /// backlog cap. `Ok(false)` when the keymap does not claim it.
    pub fn press(&mut self, press: &Press, context: &Context) -> Result<bool, InputRefusal> {
        self.refresh();
        self.driver.press(press, context)
    }
    /// The keymap's intent for a press, for a script that queues directly.
    pub fn keyed(&self, press: &Press, context: &Context) -> Option<MesocosmIntent> {
        self.product().keyed(self.driver.world(), press, context)
    }
    /// A pick through the keymap, refused if its frame is stale.
    pub fn point(&mut self, pointed: &Pointed, context: &Context) -> Result<bool, InputRefusal> {
        self.refresh();
        self.driver.point(pointed, context)
    }
    /// The pick, if its frame is still current.
    pub fn current(&self, pointed: &Pointed) -> Result<Pick, InputRefusal> { self.driver.current(pointed) }
    /// Live input the window composed itself, such as a chosen hint.
    pub fn offer(&mut self, intent: MesocosmIntent) -> Result<(), InputRefusal> {
        self.refresh();
        let envelope = self.envelope(intent);
        self.driver.offer(envelope)
    }
    /// Stamps an intent for the current tick and participant.
    pub fn envelope(&self, intent: MesocosmIntent) -> Envelope { self.product().envelope(self.driver.world(), intent) }
    /// The world revision a presented frame is stamped with.
    pub fn revision(&self) -> Result<u64, String> { self.driver.world().revision() }
    pub fn set_max_backlog(&mut self, max_backlog: usize) { self.driver.set_max_backlog(max_backlog); }

    /// Runs the rounds the elapsed time authorises; a held run banks none.
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        let _span = crate::profile::Span::new("advance.total");
        if let Some(taken) = self.advance_pending(elapsed_us) { return taken; }
        let elapsed_us = elapsed_us.saturating_add(std::mem::take(&mut self.banked_us));
        // While held, elapsed time is discarded. With no queued answer and
        // no shared-world revision since the last successful publication,
        // the saved game is already authoritative and remains unchanged.
        if self.driver.product().held()
            && self.driver.queued_len() == 0
            && self.seen.is_some_and(|seen| self.driver.world().revision().is_ok_and(|now| now == seen))
        {
            return 0;
        }
        self.refresh();
        let taken = self.driver.advance(elapsed_us);
        self.publish();
        // A round out on the worker counts when it lands.
        if self.driver.product().pending.is_some() { taken.saturating_sub(1) } else { taken }
    }

    /// Runs up to `steps` rounds off the clock; fewer while a question holds.
    pub fn step(&mut self, steps: u64) -> u64 {
        self.refresh();
        let product = self.driver.product_mut();
        product.pending = None;
        let background = std::mem::replace(&mut product.background, false);
        let taken = self.driver.step(steps);
        self.driver.product_mut().background = background;
        self.publish();
        taken
    }

    /// Starts reading glyphs from the bound critter's acts from here on.
    pub fn read_glyphs(&mut self, rules: GlyphRules) -> Result<(), String> {
        self.with_product(|product, world| {
            product.glyphs = Some(GlyphReading::new(rules, &session(world))?);
            Ok(())
        })
    }

    pub fn glyphs(&self) -> Option<&GlyphReading> { self.product().glyphs.as_ref() }

    /// The question the run is holding at, if any.
    pub fn checkpoint(&self) -> Option<&Checkpoint> { self.product().checkpoint.as_ref() }

    /// Gives the review a pack's expression scripts as a second source.
    pub fn with_authored(mut self, authored: Authored) -> Result<Self, String> {
        self.with_product(|product, world| {
            authored.validate_world(&product.founded.genesis)?;
            product.terms = authored.terms().clone();
            product.authored = Some(authored);
            product.sync_loop(world);
            Ok::<_, String>(())
        })?;
        Ok(self)
    }

    pub fn terms(&self) -> &mesocosm_phenotype::TermTable { &self.product().terms }

    /// What the scripts made of the open review's declared-tract offers.
    pub fn proposed(&self) -> &[Proposed] { &self.product().proposed }

    /// The played line's turn, while holding at a boundary.
    pub fn review(&self) -> Option<&Review> { self.product().review.as_ref() }

    /// What the last round brought.
    pub fn happenings(&self) -> &[Happening] { &self.product().happenings }

    /// The loop's settings: who plays, how, and since when.
    pub fn interim(&self) -> &Loop { &self.product().lp }

    pub fn session(&self) -> Ref<'_, Session> { session(self.driver.world()) }

    pub fn sim(&self) -> Ref<'_, Simulation> { Ref::map(self.session(), |session| &session.sim) }

    pub fn founded(&self) -> &Founded { &self.product().founded }

    /// The critter the participant plays, if any.
    pub fn critter(&self) -> Option<Id> { self.product().critter(self.driver.world()) }

    /// The played critter as the world holds it.
    pub fn played(&self) -> Option<Ref<'_, Entity>> {
        let critter = self.critter()?;
        Ref::filter_map(self.sim(), |sim| sim.state().population.get(critter)).ok()
    }

    /// What the player sees now, by mode (180, 691).
    pub fn view(&self) -> View {
        let lp = &self.product().lp;
        let session = self.session();
        let critter = session.sim.plays(lp.participant).unwrap_or(cosmograph::directing::PLACELESS);
        cosmograph::directing::readings::view(&session.sim, critter, lp.mode)
    }

    /// Native awareness refined by explicit in-site geometry, when the host
    /// has actual eye/body cells. It never uses the scene's fallback spots.
    pub fn sighted_view(&self, site: &crate::perception::InSite,
        sight: &crate::perception::Sight) -> Result<View, String> {
        if self.critter() != Some(sight.observer) {
            return Err("sight observer is not the currently played critter".into());
        }
        site.refine(&self.sim(), self.product().lp.mode, sight)
    }

    /// Actual physical scene callbacks opt into the saved authored range.
    /// Creative remains native truth; old unmarked saves keep their reading.
    pub fn physical_view(&self) -> Result<View, String> {
        let spatial = &self.product().spatial;
        spatial.version()?;
        let visible = self.view();
        if self.product().lp.mode == Mode::Creative || !spatial.has_sight_source() { return Ok(visible); }
        let observer = self.critter().ok_or("marked sight requires the actual played observer")?;
        let sighted = spatial.sight(&self.sim(), observer, &visible)?;
        match sighted {
            Some((site, sight)) => self.sighted_view(&site, &sight),
            None => Err("marked sight source is missing".into()),
        }
    }

    /// Current edited geometry at the played site. Call again after admitted
    /// Edit/Patch commands or a round; departed critters have no site to lift.
    pub fn in_site(&self) -> Result<crate::perception::InSite, String> {
        let site = self.played().and_then(|e| e.resident_site())
            .ok_or("played critter has no resident site")?;
        let sim = self.sim();
        let at = self.critter().and_then(|id| self.spatial().members.get(&id));
        let window = at.map(|p| p.window.clone()).map(Ok).unwrap_or_else(|| placing::window(&sim))?;
        crate::perception::InSite::cached(&sim, site, &window, at.and_then(|p| p.places).unwrap_or_default())
    }

    pub fn windows(&self) -> &FlowWindows { &self.product().windows }

    pub fn trend(&self) -> Trend { self.product().windows.trend() }

    /// Every envelope applied, with what it came to.
    pub fn trace(&self) -> &[(Envelope, Result<String, Refusal>)] { self.driver.trace() }

    /// Why the sim refused a round, if it did: the run has stopped there.
    pub fn fault(&self) -> Option<&str> { self.driver.fault() }

    /// Dev intents this run applied (DT3).
    pub fn dev_intents(&self) -> u64 { self.product().dev_intents }

    pub fn tick(&self) -> u64 { self.sim().state().tick }

    pub fn state_hash(&self) -> u64 {
        self.world().cut().expect("the world is read between transactions").state_hash
    }
}

mod replay;
mod world_save;
pub use replay::{GameSave, Receipt, Replayed};

#[cfg(test)]
#[path = "runtime/held_advance_tests.rs"]
mod held_advance_tests;

#[cfg(test)]
#[path = "runtime/active_profile.rs"]
mod active_profile;

#[cfg(test)]
#[path = "runtime/verified_question_tests.rs"]
mod verified_question_tests;
