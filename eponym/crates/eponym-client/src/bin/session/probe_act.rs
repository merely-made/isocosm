// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The `act` vocabulary: every intent this host can only be told with a key.
//!
//! Everything the session host can only be told with a key — movement, aim,
//! charge, strike, and the four world verbs that have no pointer target of
//! their own — is reachable here by name. The DOM controls are *not*: a button
//! is exercised as a button, through the host's own pointer routing, exactly as
//! Mesocosm's bench exercises its own.
//!
//! Two deliberate departures from the keyboard, so a scenario is a receipt
//! rather than a stopwatch: a movement act runs [`MOVE_STEPS`] fixed 60 Hz
//! steps outright instead of arming the auto-repeat latch (see the declared
//! limits in `model.rs`), and `charge` accumulates [`CHARGE_TICKS`] charge
//! ticks outright instead of waiting out `CHARGE_INTERVAL` wall time.

use std::time::Instant;

use isometer::core::PartId;
use eponym_world::timed_action::Direction;

use super::super::{CHARGE_INTERVAL, SessionApp};

/// Fixed motion steps one movement act runs. Enough to move the drawn body.
const MOVE_STEPS: usize = 8;
/// Charge ticks `charge` accumulates before the strike is released.
const CHARGE_TICKS: usize = 4;

/// Run one named command. `false` when no such command exists, so the driver
/// fails loudly on a typo rather than passing a step that did nothing.
pub(super) fn act(state: &mut SessionApp, label: &str) -> bool {
    let toward = match label {
        "move-forward" => Some([0, 0, 1]),
        "move-back" => Some([0, 0, -1]),
        "move-left" => Some([-1, 0, 0]),
        "move-right" => Some([1, 0, 0]),
        _ => None,
    };
    if let Some(toward) = toward {
        for _ in 0..MOVE_STEPS {
            state.move_played(toward);
        }
        return true;
    }
    let aim = match label {
        "aim-forward" => Some(Direction::Forward),
        "aim-back" => Some(Direction::Backward),
        "aim-left" => Some(Direction::Left),
        "aim-right" => Some(Direction::Right),
        _ => None,
    };
    if let Some(direction) = aim {
        state.prepare(direction);
        return true;
    }
    match label {
        "join-2" => state.join_limb(PartId(2)),
        "charge" => {
            state.begin_charge();
            for _ in 0..CHARGE_TICKS {
                state.last_charge = Instant::now() - CHARGE_INTERVAL;
                state.tick_charge();
            }
            // The open action stays; only the wall-clock accumulator stops, so
            // the host does not keep charging between steps.
            state.charging = false;
        },
        "strike" => state.release(),
        "take" => state.take_dressing(),
        "rest" => state.rest(),
        "injure" => state.injure(),
        "revise-canon" => state.revise_canon(),
        "save" => state.save(),
        "load" => state.load(),
        _ => return false,
    }
    true
}
