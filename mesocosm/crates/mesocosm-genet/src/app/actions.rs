// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a scenario's `act` can ask for, and the one route a key takes (DT4).
//!
//! `act` carries the same key names `--help` documents, and goes through
//! [`Host::press_key`], the function the window's keyboard handler calls, so
//! a script cannot drift from a person's keypress. Three host actions cover
//! what no key says:
//!
//! | Name | What it does |
//! | --- | --- |
//! | `follow <id>` | camera onto one body |
//! | `follow-nearest` | camera onto the first other living body in the played site |
//! | `demo <steps>` | N rounds of the demo script, off the clock |

use winit::keyboard::{Key, NamedKey};

use super::Host;
use crate::input;

/// Scripted rounds one frame takes. Each is one round whatever pumps it, so
/// the session's log is the headless recording's either way.
pub const PUMP_STEPS_PER_FRAME: u64 = 25;

/// A stretch of the demo script, mid-pump.
#[derive(Clone, Copy, Debug)]
pub enum Pump {
    Demo { step: u64, until: u64 },
}

impl Host {
    /// **The one route a key takes into the world**, pressed or scripted:
    /// camera, inspection, dev, then the board, a standing question's
    /// answers, and the play keys.
    pub(super) fn press_key(&mut self, key: &Key) {
        if self.try_camera_key(key) || self.try_inspection_key(key) || self.try_dev_key(key) {
            return;
        }
        let intent = match self.board_key(key) {
            Some(taken) => taken,
            None => match self.runtime.checkpoint() {
                Some(checkpoint) => input::answer_for(checkpoint, self.runtime.critter(), key),
                None => input::intent_for(&self.runtime, self.follow, key),
            },
        };
        if let Some(intent) = intent
            && input::admits(self.runtime.queued_len())
        {
            let envelope = input::envelope(&self.runtime, intent);
            self.runtime.queue(envelope);
        }
    }

    /// The trait board's own keys while a review stands. `Some(None)` means
    /// the board took the key and sends nothing: a cursor move is host state.
    fn board_key(
        &mut self,
        key: &Key,
    ) -> Option<Option<isocosm_overlay::mesocosm::MesocosmIntent>> {
        let action = input::board_key(key)?;
        let offers = &self.runtime.review()?.offers;
        match action {
            input::BoardKey::Commit => Some(
                offers
                    .get(self.board_row)
                    .filter(|offer| offer.takeable())
                    .map(|_| input::revision(self.runtime.critter(), self.board_row)),
            ),
            input::BoardKey::Next => {
                self.board_row = (self.board_row + 1) % offers.len().max(1);
                Some(None)
            },
        }
    }

    /// Runs one named action. `false` when there is no such name, which the
    /// scenario driver reports as a failed step.
    pub(super) fn run_action(&mut self, label: &str) -> bool {
        let (name, rest) = match label.trim().split_once(char::is_whitespace) {
            Some((name, rest)) => (name, rest.trim()),
            None => (label.trim(), ""),
        };
        if let Some(key) = key_named(name) {
            self.press_key(&key);
            // A dev step key takes rounds here, not in a frame.
            self.note_outcomes();
            return true;
        }
        match name {
            "follow" => match rest.parse::<u64>() {
                Ok(id) => {
                    self.follow = Some(id);
                    self.follow_lost = None;
                    true
                },
                Err(_) => false,
            },
            "follow-nearest" => match self.nearest_neighbour() {
                Some(id) => {
                    self.follow = Some(id);
                    self.follow_lost = None;
                    self.events.push(format!("followed-nearest {id}"));
                    true
                },
                None => {
                    self.events
                        .push("follow-nearest: nobody else is alive here".to_string());
                    false
                },
            },
            "demo" => match rest.parse::<u64>() {
                Ok(until) => {
                    self.pump = (until > 0).then_some(Pump::Demo { step: 0, until });
                    true
                },
                Err(_) => false,
            },
            _ => false,
        }
    }

    /// The first other living body in the played site, by id. A native body
    /// has no position inside its site, so there is no nearer one.
    fn nearest_neighbour(&self) -> Option<u64> {
        let scene = self.scene.as_ref()?;
        scene
            .bodies
            .iter()
            .find(|body| body.alive && Some(body.id) != scene.played)
            .map(|body| body.id)
    }

    /// One frame of a scripted stretch: up to [`PUMP_STEPS_PER_FRAME`] rounds.
    pub(super) fn pump_frame(&mut self) {
        let Some(Pump::Demo { mut step, until }) = self.pump.take() else {
            return;
        };
        let end = (step + PUMP_STEPS_PER_FRAME).min(until);
        while step < end {
            crate::played::demo_step(&mut self.runtime, step);
            step += 1;
            self.steps += 1;
        }
        self.pump = if step >= until {
            self.events.push(format!("demo-finished {step} steps"));
            None
        } else {
            Some(Pump::Demo { step, until })
        };
        self.note_outcomes();
    }
}

/// The key a documented name stands for. `space`, `enter` and `tab` are
/// spelled out because a scenario line is whitespace-delimited.
fn key_named(name: &str) -> Option<Key> {
    match name {
        "space" => Some(Key::Named(NamedKey::Space)),
        "enter" => Some(Key::Named(NamedKey::Enter)),
        "tab" => Some(Key::Named(NamedKey::Tab)),
        // Play: E, Q, S; T at a checkpoint; R at the board; Z/V turn.
        // Dev: P . , [ ] N B M X F K G, and I J L U for inspection.
        "e" | "q" | "s" | "t" | "r" | "z" | "v" | "p" | "." | "," | "[" | "]" | "n" | "b" | "m"
        | "x" | "f" | "k" | "g" | "i" | "j" | "l" | "u" => Some(Key::Character(name.into())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_documented_dev_key_is_an_act_name() {
        for name in ["p", ".", ",", "[", "]", "n", "b", "m", "x", "f", "k", "g"] {
            let key = key_named(name).expect("a documented dev key");
            assert!(input::dev_key(&key).is_some(), "{name}");
        }
        assert!(key_named("follow").is_none(), "a host action, not a key");
    }
}
