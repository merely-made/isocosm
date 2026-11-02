// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The session host in rounds (cases 1096, 1118, 1121, 1122): each frame
//! steps the played body locally and draws it there, minutes go to the
//! model's worker, and a world act waits for the minute in flight. An act
//! first hands in the drawn motion, so it reads the body where it is drawn.

use std::time::Instant;

use eponym_play::{MotionInput, MotionRules};

use super::SessionApp;

/// A world act waiting for the minute in flight.
pub(super) type Queued = Box<dyn FnOnce(&mut SessionApp)>;

impl SessionApp {
    pub(super) fn in_rounds(&self) -> bool {
        self.model.borrow().round().is_some()
    }

    /// The rules the played body steps by: the world's authored ones, or the
    /// demo profile's.
    pub(super) fn played_rules(&self) -> Option<MotionRules> {
        let model = self.model.borrow();
        let game = model.game();
        match model.motion_rules {
            Some(rules) => Some(rules),
            None if game.world().material_profile().is_none() => {
                Some(eponym_play::fixtures::session::motion_rules(game, model.played()))
            },
            None => None,
        }
    }

    /// Runs a world act now, or after the minute in flight (case 1121).
    pub(super) fn world(&mut self, act: impl FnOnce(&mut SessionApp) + 'static) {
        let handed = self.model.borrow_mut().hand_in_motion();
        if let Err(why) = handed {
            self.status = vec![format!("Motion refused: {why}")];
        }
        if self.model.borrow().minute_pending() {
            let polled = self.model.borrow_mut().poll_round();
            if let Ok(adopted) = polled { self.report(adopted); }
        }
        // Earlier acts still waiting keep their order.
        let waiting = self.model.borrow().minute_pending() || !self.queued.is_empty();
        if waiting {
            self.queued.push(Box::new(act));
            self.status = vec!["Waiting for the minute".into()];
        } else {
            act(self);
        }
    }

    /// One frame of a round: the latched direction stepped locally while the
    /// body walks or falls, due minutes sent, answers taken.
    pub(super) fn tick_round(&mut self) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_round).as_micros() as u64;
        self.last_round = now;
        let Some(rules) = self.played_rules() else {
            self.status = vec!["Motion needs the world's authored rules".into()];
            return;
        };
        let toward = self.steering();
        let input = MotionInput { move_x: (toward[0] * 32767) as i16, move_z: (toward[2] * 32767) as i16 };
        let result = {
            let mut model = self.model.borrow_mut();
            let played = model.played();
            let alive = model.game().bodies().get(played).is_some_and(|b| b.alive());
            model.advance_round(elapsed, rules, |m| (alive && (toward != [0, 0, 0]
                || m.drawn_pose(played).is_some_and(|p| !p.grounded))).then_some(input))
        };
        if let Err(why) = result {
            self.latched = [None; 4];
            self.status = vec![format!("Motion failed: {why}")];
        }
        self.take_answers();
    }

    /// One fixed step in `toward` outside the pacer.
    pub(super) fn step_round(&mut self, toward: [i32; 3]) {
        let Some(rules) = self.played_rules() else { return };
        let input = MotionInput { move_x: (toward[0] * 32767) as i16, move_z: (toward[2] * 32767) as i16 };
        let stepped = self.model.borrow_mut().step_played(input, rules);
        self.status = vec![match stepped {
            Ok(()) => "Stepped; the minute takes it to the world".into(),
            Err(why) => {
                self.latched = [None; 4];
                format!("Motion failed: {why}")
            },
        }];
        self.take_answers();
    }

    /// Sets the round's speed from now on (case 1097).
    pub(super) fn set_speed(&mut self, speed: cosmograph_host::drive::Speed) {
        let set = self.model.borrow_mut().round_mut().map(|round| round.set_speed(speed)).is_some();
        self.status = vec![if set { format!("Round speed {}x", speed.factor()) } else { "Not in a round".into() }];
    }

    /// The panel's round line: speed, the world's minute, a minute in flight.
    pub(super) fn round_line(&self) -> Option<String> {
        let model = self.model.borrow();
        let round = model.round()?;
        Some(format!("Round {}x; minute {}{}", round.speed().factor(), model.game().world().sim().state().tick,
            if round.in_flight() { "; next minute in flight" } else { "" }))
    }

    /// The minute now, waited for: a scenario's fixture control.
    pub(super) fn minute_now(&mut self) {
        let sent = self.model.borrow_mut().minute_now();
        if let Err(why) = sent {
            self.status = vec![format!("Minute refused: {why}")];
            return;
        }
        loop {
            let drained = self.model.borrow_mut().drain_round();
            match drained {
                Ok(adopted) => self.report(adopted),
                Err(why) => self.status = vec![format!("Minute failed: {why}")],
            }
            if !self.model.borrow().minute_pending() { break; }
        }
        self.take_answers();
    }

    /// The worker's answers, then the acts that waited for them.
    pub(super) fn take_answers(&mut self) {
        let polled = self.model.borrow_mut().poll_round();
        match polled {
            Ok(adopted) => self.report(adopted),
            Err(why) => self.status = vec![format!("Minute failed: {why}")],
        }
        if self.model.borrow().minute_pending() {
            return;
        }
        for act in std::mem::take(&mut self.queued) {
            act(self);
        }
    }

    fn report(&mut self, adopted: eponym_client::round::Adopted) {
        if !adopted.refusals.is_empty() {
            self.status = adopted.refusals.iter().map(|why| format!("Minute refused: {why}")).collect();
        } else if adopted.any() {
            self.status = vec![format!("Minute taken: {} event(s)", adopted.events.len())];
        }
    }
}
