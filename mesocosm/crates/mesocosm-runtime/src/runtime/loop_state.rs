// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The owned driver uses the same saved loop as the borrowed product door.
//! The core driver holds the queue, so the loop seals it only when the
//! runtime publishes or saves.

use super::{Envelope, Mesocosm, Refusal, Runtime, commit, session};
use crate::{spatial::Game, continuation::{self, Continuation, LoopInput, LoopReading,
    LoopSettings, LoopPhase, LoopAction, LoopAnswer, SavedProposal, Origin}, succession::Occasion};
use isocosm_host::world::World;
use isocosm_overlay::{OrdinaryAccess, ParticipantHandle, EntityHandle, Seat, Tick};
use isocosm::directing::interim::Happening;

impl Mesocosm {
    pub(crate) fn loop_settings(&self) -> LoopSettings {
        LoopSettings { mode: self.lp.mode, pace: self.lp.pace, on_collapse: self.lp.on_collapse }
    }

    /// The game with its saved loop sealed against the world's session.
    pub(crate) fn loop_game(&self, world: &World, queued: Vec<Envelope>) -> Game {
        let _span = crate::profile::Span::new("loop_game.total");
        let mut game = self.spatial.clone();
        let prior = game.continuation.as_ref();
        let phase = match prior.map(|p| p.phase) {
            Some(LoopPhase::Watching) => LoopPhase::Watching,
            Some(LoopPhase::NewWorldRequested) => LoopPhase::NewWorldRequested,
            Some(LoopPhase::WorldEnded) => LoopPhase::WorldEnded,
            _ if prior.is_some_and(|saved| saved.world_end.is_some()) => LoopPhase::WorldEnded,
            _ if self.happenings.iter().any(|h| matches!(h, Happening::WorldEnded { .. })) => LoopPhase::WorldEnded,
            _ if self.checkpoint.as_ref().is_some_and(|c|
                matches!(&c.occasion, Occasion::Loss(loss) if loss.next.is_none()) && c.heirs.is_empty()) =>
                LoopPhase::LineageEnded,
            _ if self.checkpoint.is_some() => LoopPhase::Held,
            _ => LoopPhase::Running,
        };
        let session = session(world);
        let played = session.sim.plays(self.lp.participant);
        let proposed = self.proposed.iter().map(|p| SavedProposal { offer: p.offer,
            script: p.script.clone(), cells: p.cells.clone(),
            commands: played.map(|id| p.commands(id)).unwrap_or_default() }).collect();
        let mut saved = Continuation { version: continuation::VERSION,
            participant: self.lp.participant, settings: self.loop_settings(),
            native: String::new(), native_entries: 0, native_tick: 0, physical: String::new(), phase,
            checkpoint: self.checkpoint.clone(), review: self.review.clone(), proposed,
            happenings: self.happenings.clone(), windows: self.windows.clone(),
            expression: self.authored.as_ref().map(|authored| authored.saved()),
            ended_lineage: self.happenings.iter().find_map(|h| match h {
                Happening::LineageEnded { lineage } => Some(lineage.clone()), _ => None,
            }).or_else(|| prior.and_then(|p| p.ended_lineage.clone())),
            world_end: prior.and_then(|p| p.world_end.clone()),
            queued,
            round: prior.and_then(|p| p.round.clone()),
            origin: prior.and_then(|p| p.origin.clone()), fault: self.fault.clone() };
        saved.seal(&session, &game);
        game.continuation = Some(saved);
        game.version = game.current_version();
        game
    }

    pub(crate) fn sync_loop(&mut self, world: &World) { self.spatial = self.loop_game(world, Vec::new()); }

    pub(crate) fn retain_origin(&mut self, world: &World) {
        self.sync_loop(world);
        if let Some(saved) = &mut self.spatial.continuation {
            saved.origin = Some(Origin { start: self.founded.start,
                handover: self.lp.handover, habitable_at: self.lp.habitable_at });
        }
    }

    /// The saved loop's settings and state, adopted. Its queue is the
    /// driver's to restore; its fault is kept here and restored there.
    pub(crate) fn restore_loop(&mut self, world: &World) -> Result<(), String> {
        let _span = crate::profile::Span::new("restore_loop.total");
        let Some(saved) = &self.spatial.continuation else { return Ok(()); };
        saved.validate_session(&session(world), &self.spatial)?;
        self.lp.participant = saved.participant;
        self.lp.mode = saved.settings.mode;
        self.lp.pace = saved.settings.pace;
        self.lp.on_collapse = saved.settings.on_collapse;
        self.founded.mode = saved.settings.mode;
        self.founded.pace = saved.settings.pace;
        self.founded.on_collapse = saved.settings.on_collapse;
        if let Some(origin) = &saved.origin {
            self.founded.start = origin.start;
            self.lp.handover = origin.handover;
            self.lp.habitable_at = origin.habitable_at;
        }
        self.checkpoint = saved.checkpoint.clone();
        self.review = saved.review.clone();
        self.happenings = saved.happenings.clone();
        self.windows = saved.windows.clone();
        self.fault = saved.fault.clone();
        self.authored = saved.expression.as_ref().map(crate::review::Authored::from_saved).transpose()?;
        if let Some(authored) = &self.authored { self.terms = authored.terms().clone(); }
        self.proposed = saved.proposed.iter().map(|p| {
            let parts = p.commands.iter().filter_map(|c| match c {
                isocosm::Command::Express { part, tracts, .. } => Some((*part, tracts.clone())),
                _ => None,
            }).collect();
            crate::review::Proposed { offer: p.offer, script: p.script.clone(), cells: p.cells.clone(),
                allocation: p.cells.is_ok().then_some(mesocosm_phenotype::express::Allocation { parts }) }
        }).collect();
        Ok(())
    }

    pub(super) fn question_reading(&self, world: &World, queued: Vec<Envelope>) -> Result<LoopReading, String> {
        let game = self.loop_game(world, queued);
        let access = self.loop_access(world);
        let session = session(world);
        let visible = continuation::admission::visible(&session, &game, access)
            .map_err(|why| format!("{why:?}"))?;
        continuation::read_loop(&session, &game, access, &visible)
    }

    pub(crate) fn loop_access(&self, world: &World) -> OrdinaryAccess {
        OrdinaryAccess { participant: ParticipantHandle(self.lp.participant),
            seat: Seat::Player, subject: self.critter(world).map(EntityHandle) }
    }

    /// A loop input through the saved loop's door, then its state adopted.
    fn loop_input(&mut self, world: &World, input: &LoopInput,
        door: fn(&mut isocosm::Session, &mut Game, OrdinaryAccess, &LoopInput) -> Result<String, Refusal>)
        -> Result<String, Refusal> {
        self.sync_loop(world);
        let access = self.loop_access(world);
        let mut game = self.spatial.clone();
        let out = commit(world, |session| door(session, &mut game, access, input))?;
        self.spatial = game;
        self.restore_loop(world).map_err(Refusal::Sim)?;
        Ok(out)
    }

    pub(super) fn answer_saved(&mut self, world: &World, answer: isocosm_overlay::mesocosm::CheckpointAnswer)
        -> Result<String, Refusal> {
        self.sync_loop(world);
        let access = self.loop_access(world);
        let input = {
            let session = session(world);
            LoopInput { kind: continuation::KIND.into(), version: continuation::VERSION,
                participant: access.participant, tick: Tick(session.sim.state().tick),
                expected: continuation::cut(&session, &self.spatial),
                action: LoopAction::Answer(LoopAnswer::Checkpoint(answer)) }
        };
        let mut game = self.spatial.clone();
        let out = commit(world, |session| continuation::answer_verified(
            session, &mut game, access, &input, self.verified_question.as_ref()))?;
        self.spatial = game;
        self.verified_question = None;
        self.restore_loop(world).map_err(Refusal::Sim)?;
        Ok(out)
    }
}

impl Runtime {
    /// A presentation filter over this driver's last successful publication or
    /// validated restore. A changed shared World requires the full reading.
    pub fn current_loop_phase(&self) -> Option<LoopPhase> {
        (self.seen == Some(self.revision().ok()?)).then_some(self.published_phase).flatten()
    }

    pub fn loop_settings(&self) -> LoopSettings { self.driver.product().loop_settings() }

    pub fn loop_reading(&self) -> Result<LoopReading, String> {
        let _span = crate::profile::Span::new("loop_reading.total");
        self.driver.product().question_reading(self.driver.world(), self.driver.queued().cloned().collect())
    }

    pub fn submit_loop_input(&mut self, input: &LoopInput) -> Result<String, Refusal> {
        if self.driver.queued_len() > 0 && !matches!(&input.action, LoopAction::Answer(_)) {
            return Err(Refusal::Sim("queued input precedes this loop step or configuration".into()));
        }
        self.with_product(|product, world| product.loop_input(world, input, continuation::submit_loop))
    }
}
