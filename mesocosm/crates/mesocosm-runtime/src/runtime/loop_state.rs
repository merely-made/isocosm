// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The owned driver uses the same saved loop as the borrowed product door.

use super::{Runtime, Refusal};
use crate::{spatial::Game, continuation::{self, Continuation, LoopInput, LoopReading,
    LoopSettings, LoopPhase, LoopAction, LoopAnswer, SavedProposal, Origin}, succession::Occasion};
use isocosm_overlay::{OrdinaryAccess, ParticipantHandle, EntityHandle, Seat, Tick};
use isocosm::directing::interim::Happening;

impl Runtime {
    pub fn loop_settings(&self) -> LoopSettings {
        LoopSettings { mode: self.interim.mode, pace: self.interim.pace,
            on_collapse: self.interim.on_collapse }
    }

    pub(super) fn loop_game(&self) -> Game {
        let mut game = self.spatial.clone();
        let prior = game.continuation.as_ref();
        let phase = match prior.map(|p| p.phase) {
            Some(LoopPhase::Watching) => LoopPhase::Watching,
            Some(LoopPhase::NewWorldRequested) => LoopPhase::NewWorldRequested,
            _ if self.checkpoint.as_ref().is_some_and(|c|
                matches!(&c.occasion, Occasion::Loss(loss) if loss.next.is_none()) && c.heirs.is_empty()) =>
                LoopPhase::LineageEnded,
            _ if self.checkpoint.is_some() => LoopPhase::Held,
            _ => LoopPhase::Running,
        };
        let played = self.critter();
        let proposed = self.proposed.iter().map(|p| SavedProposal { offer: p.offer,
            script: p.script.clone(), cells: p.cells.clone(),
            commands: played.map(|id| p.commands(id)).unwrap_or_default() }).collect();
        let mut saved = Continuation { version: continuation::VERSION,
            participant: self.interim.participant, settings: self.loop_settings(),
            native: String::new(), native_entries: 0, native_tick: 0, physical: String::new(), phase,
            checkpoint: self.checkpoint.clone(), review: self.review.clone(), proposed,
            happenings: self.happenings.clone(), windows: self.windows.clone(),
            expression: self.authored.as_ref().map(|authored| authored.saved()),
            ended_lineage: self.happenings.iter().find_map(|h| match h {
                Happening::LineageEnded { lineage } => Some(lineage.clone()), _ => None,
            }).or_else(|| prior.and_then(|p| p.ended_lineage.clone())),
            queued: self.queued.iter().cloned().collect(),
            round: prior.and_then(|p| p.round.clone()),
            origin: prior.and_then(|p| p.origin.clone()), fault: self.fault.clone() };
        saved.seal(self.session(), &game);
        game.continuation = Some(saved);
        game.version = game.current_version();
        game
    }

    pub(super) fn sync_loop(&mut self) { self.spatial = self.loop_game(); }

    pub(super) fn retain_origin(&mut self) {
        self.sync_loop();
        if let Some(saved) = &mut self.spatial.continuation {
            saved.origin = Some(Origin { start: self.founded.start,
                handover: self.interim.handover, habitable_at: self.interim.habitable_at });
        }
    }

    pub(super) fn restore_loop(&mut self) -> Result<(), String> {
        let Some(saved) = &self.spatial.continuation else { return Ok(()); };
        saved.validate_session(self.session(), &self.spatial)?;
        self.interim.participant = saved.participant;
        self.interim.mode = saved.settings.mode;
        self.interim.pace = saved.settings.pace;
        self.interim.on_collapse = saved.settings.on_collapse;
        self.founded.mode = saved.settings.mode;
        self.founded.pace = saved.settings.pace;
        self.founded.on_collapse = saved.settings.on_collapse;
        if let Some(origin) = &saved.origin {
            self.founded.start = origin.start;
            self.interim.handover = origin.handover;
            self.interim.habitable_at = origin.habitable_at;
        }
        self.checkpoint = saved.checkpoint.clone();
        self.review = saved.review.clone();
        self.happenings = saved.happenings.clone();
        self.windows = saved.windows.clone();
        self.queued = saved.queued.iter().cloned().collect();
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

    fn loop_access(&self) -> OrdinaryAccess {
        OrdinaryAccess { participant: ParticipantHandle(self.interim.participant),
            seat: Seat::Player, subject: self.critter().map(EntityHandle) }
    }

    pub fn loop_reading(&self) -> Result<LoopReading, String> {
        let game = self.loop_game();
        let access = self.loop_access();
        let visible = continuation::admission::visible(self.session(), &game, access)
            .map_err(|why| format!("{why:?}"))?;
        continuation::read_loop(self.session(), &game, access, &visible)
    }

    pub fn submit_loop_input(&mut self, input: &LoopInput) -> Result<String, Refusal> {
        if !self.queued.is_empty() && !matches!(&input.action, LoopAction::Answer(_)) {
            return Err(Refusal::Sim("queued input precedes this loop step or configuration".into()));
        }
        self.sync_loop();
        let access = self.loop_access();
        let out = continuation::submit_loop(&mut self.interim.session, &mut self.spatial, access, input)?;
        self.restore_loop().map_err(Refusal::Sim)?;
        Ok(out)
    }

    pub(super) fn answer_saved(&mut self, answer: isocosm_overlay::mesocosm::CheckpointAnswer)
        -> Result<String, Refusal> {
        self.sync_loop();
        let access = self.loop_access();
        let input = LoopInput { kind: continuation::KIND.into(), version: continuation::VERSION,
            participant: access.participant, tick: Tick(self.tick()),
            expected: continuation::cut(self.session(), &self.spatial),
            action: LoopAction::Answer(LoopAnswer::Checkpoint(answer)) };
        let out = continuation::answer_loop(&mut self.interim.session, &mut self.spatial, access, &input)?;
        self.restore_loop().map_err(Refusal::Sim)?;
        Ok(out)
    }
}
