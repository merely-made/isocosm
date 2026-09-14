// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The `act` vocabulary, and the two verbs this product adds to the grammar.
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

use std::collections::BTreeMap;
use std::time::Instant;

use mesocosm_core::PartId;
use paredros_world::timed_action::Direction;

use super::super::{CHARGE_INTERVAL, SessionApp};
use super::{Context, state};

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
        "save" => state.save(),
        "load" => state.load(),
        _ => return false,
    }
    true
}

/// `mark <name>`, `differs <name> <field>...` and `dropped <name> <field>...`.
///
/// The shared grammar has `remember`/`same`/`more`: equality against a
/// checkpoint, and a counter that grew. It has no "this changed" and no "this
/// fell", and `Product::app_step` is handed no access to `Lane`'s checkpoints,
/// so a product that wants them keeps its own. Used here for the readings whose
/// *movement* is the claim: the played position and the `GameState` hash, which
/// are not counters, and the struck target's vitality, which only ever falls.
pub(super) fn app_step(
    marks: &mut Vec<(String, BTreeMap<String, String>)>,
    ctx: &mut Context<'_>,
    line: &str,
) -> Result<(), String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice() {
        ["mark", name] => {
            let fields = state::snapshot(ctx, 0, 1.0).fields;
            marks.retain(|(held, _)| held != name);
            marks.push(((*name).to_owned(), fields));
            Ok(())
        },
        [verb @ ("differs" | "dropped"), name, fields @ ..] if !fields.is_empty() => {
            let before = marks
                .iter()
                .find(|(held, _)| held == name)
                .map(|(_, fields)| fields.clone())
                .ok_or_else(|| format!("unknown mark {name}"))?;
            let now = state::snapshot(ctx, 0, 1.0);
            for field in fields {
                let first = before
                    .get(*field)
                    .ok_or_else(|| format!("mark {name} has no {field}"))?;
                let current = now
                    .field(field)
                    .ok_or_else(|| format!("no snapshot field {field}"))?;
                let held = if *verb == "differs" {
                    first != current
                } else {
                    let number = |value: &str| {
                        value
                            .parse::<f64>()
                            .map_err(|_| format!("{field} is not a number"))
                    };
                    number(current)? < number(first)?
                };
                if !held {
                    return Err(format!("{verb} {name} {field}: {first} -> {current}"));
                }
            }
            Ok(())
        },
        _ => Err(format!("unknown session step: {line}")),
    }
}
