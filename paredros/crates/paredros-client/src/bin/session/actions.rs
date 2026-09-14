// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Every world change this host makes, on the one `Session` in `SceneModel`.
//!
//! Each one is the same intent path the timed-action host takes: motion through
//! `AdvanceMotion` with the profile's rules revision, injury as `Fall` plus
//! `ReconcileAnatomy` in one accepted cut, pickup as `Take`, rest and the
//! charged volley through the timed-action grammar that owns the session.

use std::time::{Duration, Instant};

use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use paredros_world::timed_action::Direction;
use paredros_world::{GameEvent, GameIntent, ItemId, ItemKind, MotionInput, StrikeOutcome};

use super::{CHARGE_INTERVAL, MOTION_INTERVAL, SessionApp};

impl SessionApp {
    /// One fixed 60 Hz motion step in the latched direction.
    pub(super) fn move_played(&mut self, toward: [i32; 3]) {
        let intent = {
            let model = self.model.borrow();
            let game = model.game();
            let played = model.played();
            let Some(pose) = game.movement().pose(played) else {
                return;
            };
            let Some(body) = game.bodies().get(played) else {
                return;
            };
            GameIntent::AdvanceMotion {
                tick: game.next_tick(),
                subject: played,
                revision: body.revision,
                step: pose.step + 1,
                input: MotionInput {
                    move_x: (toward[0] * 32767) as i16,
                    move_z: (toward[2] * 32767) as i16,
                },
                rules: paredros_client::session_fixture::motion_rules(game, played),
            }
        };
        let step = match &intent {
            GameIntent::AdvanceMotion { step, .. } => *step,
            _ => 0,
        };
        match self.model.borrow_mut().apply(intent) {
            Ok(events) => {
                self.status = vec![format!("Motion step {step}: {} event(s)", events.len())]
            },
            Err(error) => {
                self.latched = [None; 4];
                self.status = vec![format!("Motion failed: {error:?}")];
            },
        }
    }

    /// True while the played subject is walking or still falling.
    pub(super) fn motion_running(&self) -> bool {
        let model = self.model.borrow();
        let played = model.played();
        let game = model.game();
        game.bodies().get(played).is_some_and(|body| body.alive())
            && (self.steering() != [0, 0, 0]
                || game
                    .movement()
                    .pose(played)
                    .is_some_and(|pose| !pose.grounded))
    }

    /// The live direction from the movement latches. See `model::LATCH_FIRST`
    /// for why a latch stands in for a held key here.
    pub(super) fn steering(&self) -> [i32; 3] {
        let now = Instant::now();
        let live = |index: usize| {
            i32::from(self.latched[index].is_some_and(|until| until > now))
        };
        [live(3) - live(2), 0, live(0) - live(1)]
    }

    /// Advances motion at a fixed 60 Hz, catching up at most eight steps.
    pub(super) fn tick_motion(&mut self) {
        if !self.motion_running() {
            self.last_motion = Instant::now();
            return;
        }
        let mut steps = 0;
        while self.last_motion.elapsed() >= MOTION_INTERVAL && steps < 8 && self.motion_running() {
            let toward = self.steering();
            self.move_played(toward);
            self.last_motion += MOTION_INTERVAL;
            steps += 1;
        }
        if steps == 8 {
            self.last_motion = Instant::now();
        }
    }

    /// Accumulates charge while the strike is held.
    pub(super) fn tick_charge(&mut self) {
        if !self.charging || self.last_charge.elapsed() < CHARGE_INTERVAL {
            return;
        }
        let tick = {
            let model = self.model.borrow();
            let Some(action) = model.action() else { return };
            action
                .action()
                .map(|open| Tick(open.last_tick.0 + 1))
                .unwrap_or_else(|| model.game().next_tick())
        };
        let mut model = self.model.borrow_mut();
        let Some(action) = model.action_mut() else {
            return;
        };
        self.status = match action.charge(tick) {
            Ok(outcomes) => vec![format!("Charging tick {}: {outcomes:?}", tick.0)],
            Err(error) => {
                self.charging = false;
                vec![format!("Charge paused: {error:?}")]
            },
        };
        drop(model);
        self.last_charge = Instant::now();
    }

    pub(super) fn prepare(&mut self, direction: Direction) {
        self.direction = direction;
        let mut model = self.model.borrow_mut();
        let Some(action) = model.action_mut() else {
            return;
        };
        self.status = match action.prepare(direction) {
            Ok(open) => vec![
                format!(
                    "Prepared {:?} at tick {}",
                    open.direction, open.started_at.0
                ),
                format!("Joined limbs: {}", open.contributors.len()),
            ],
            Err(error) => vec![format!("Prepare failed: {error:?}")],
        };
    }

    pub(super) fn join_limb(&mut self, part: PartId) {
        let mut model = self.model.borrow_mut();
        let Some(action) = model.action_mut() else {
            return;
        };
        self.status = match action.join(part) {
            Ok(()) => vec![format!("Joined limb part {} into the action.", part.0)],
            Err(error) => vec![format!("Limb join failed: {error:?}")],
        };
    }

    /// Begins a charge, preparing the standing direction when none is open.
    pub(super) fn begin_charge(&mut self) {
        if self.model.borrow().action().and_then(|a| a.action()).is_none() {
            self.prepare(self.direction);
            let _ = self.join_limb(PartId(2));
        }
        self.charging = true;
        self.last_charge = Instant::now();
    }

    pub(super) fn release(&mut self) {
        self.charging = false;
        let (target, rules) = (self.target, self.combat_rules);
        let tick = {
            let model = self.model.borrow();
            match model.action().and_then(|action| action.action()) {
                Some(open) => open.last_tick,
                None => return,
            }
        };
        let mut model = self.model.borrow_mut();
        let Some(action) = model.action_mut() else {
            return;
        };
        self.status = match action.release_against(target, tick, rules) {
            Ok(events) => {
                let mut lines = vec![format!("Released volley: {} event(s)", events.len())];
                lines.extend(strike_lines(&events));
                lines
            },
            Err(error) => vec![format!("Release failed: {error:?}")],
        };
    }

    /// The injury debug cut: a fall and the anatomy reconciliation that removes
    /// part 1, applied as one batch so the open action repairs with it.
    pub(super) fn injure(&mut self) {
        let mut model = self.model.borrow_mut();
        let played = model.played();
        let Some(old) = model.game().bodies().get(played).map(|body| body.revision) else {
            return;
        };
        let first = model.game().next_tick();
        let second = Tick(first.0 + 1);
        let result = model.apply_batch(&[
            GameIntent::Fall {
                tick: first,
                subject: played,
                distance: 5,
            },
            GameIntent::ReconcileAnatomy {
                tick: second,
                subject: played,
                from_revision: old,
                revision: BodyRevisionId(old.0 + 1),
                severed_parts: vec![PartId(1)],
            },
        ]);
        self.status = vec![match result {
            Ok(events) => format!(
                "Injury cut: {} events; part 1 removed, surviving contributors retained",
                events.len()
            ),
            Err(error) => format!("Injury cut: {error:?}"),
        }];
    }

    pub(super) fn rest(&mut self) {
        let mut model = self.model.borrow_mut();
        let Some(action) = model.action_mut() else {
            return;
        };
        self.status = match action.rest() {
            Ok(events) => events
                .iter()
                .filter_map(|event| match event {
                    GameEvent::Rested {
                        recovered, fatigue, ..
                    } => Some(format!(
                        "Rested: recovered {recovered} wound; fatigue={fatigue}"
                    )),
                    _ => None,
                })
                .collect(),
            Err(error) => vec![format!("Rest failed: {error:?}")],
        };
    }

    pub(super) fn take_dressing(&mut self) {
        let intent = {
            let model = self.model.borrow();
            let game = model.game();
            let played = model.played();
            let Some(at) = game.movement().position(played) else {
                return;
            };
            game.items()
                .at(at)
                .find(|item| item.kind == ItemKind::Dressing)
                .map(|item| GameIntent::Take {
                    tick: game.next_tick(),
                    subject: played,
                    item: item.id,
                })
        };
        let Some(intent) = intent else {
            self.status = vec!["No dressing at this position".into()];
            return;
        };
        let item = match &intent {
            GameIntent::Take { item, .. } => *item,
            _ => ItemId(0),
        };
        self.status = vec![match self.model.borrow_mut().apply(intent) {
            Ok(_) => format!("Picked up dressing {}", item.0),
            Err(error) => format!("Pickup failed: {error:?}"),
        }];
    }

    pub(super) fn attach(&mut self, item: ItemId, part: PartId) {
        let intent = {
            let model = self.model.borrow();
            let played = model.played();
            let Some(revision) = model.game().bodies().get(played).map(|body| body.revision) else {
                return;
            };
            GameIntent::AttachItem {
                tick: model.game().next_tick(),
                subject: played,
                item,
                part,
                revision,
            }
        };
        self.status = vec![match self.model.borrow_mut().apply(intent) {
            Ok(_) => format!("Attached item {} to part {}", item.0, part.0),
            Err(error) => format!("Attach rejected: {error:?}"),
        }];
    }

    pub(super) fn detach(&mut self, item: ItemId) {
        let intent = {
            let model = self.model.borrow();
            GameIntent::DetachItem {
                tick: model.game().next_tick(),
                subject: model.played(),
                item,
            }
        };
        self.status = vec![match self.model.borrow_mut().apply(intent) {
            Ok(_) => format!("Detached item {}", item.0),
            Err(error) => format!("Detach rejected: {error:?}"),
        }];
    }

    pub(super) fn select(&mut self, subject: SubjectId, part: PartId) {
        self.selected = Some((subject, part));
    }

    /// A pointer press on the viewport leaf. `local`/`size` are the leaf's own
    /// content box, so this is the whole mapping to the producer's clip space.
    pub(super) fn pick(&mut self, local: (f32, f32), size: (f32, f32)) {
        if size.0 <= 0.0 || size.1 <= 0.0 || !local.0.is_finite() || !local.1.is_finite() {
            return;
        }
        let ndc = [
            (local.0 / size.0) * 2.0 - 1.0,
            1.0 - (local.1 / size.1) * 2.0,
        ];
        match self.scene.borrow().pick_body_ignoring_terrain(ndc) {
            Some((subject, part)) => {
                self.selected = Some((subject, part));
                self.status = vec![format!("Picked subject {} part {}", subject.0, part.0)];
            },
            None => {
                self.selected = None;
                self.status = vec!["No body under the pointer".into()];
            },
        }
    }

    pub(super) fn save(&mut self) {
        let bytes = match self.save_bytes() {
            Ok(bytes) => bytes,
            Err(why) => {
                self.status = vec![format!("Save failed: {why}")];
                return;
            },
        };
        let path = self.save_path.clone();
        self.status = vec![match write_atomically(&path, &bytes) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(why) => format!("Save failed: {why}"),
        }];
    }

    /// The save bytes: the timed-action record, which nests the same
    /// `SessionSave`/`GameSave` versions the timed-action bin writes today.
    pub(super) fn save_bytes(&self) -> Result<Vec<u8>, String> {
        let model = self.model.borrow();
        match model.action() {
            Some(action) => action.save().map_err(|error| format!("{error:?}")),
            None => model.session().save().map_err(|error| format!("{error:?}")),
        }
    }

    pub(super) fn load(&mut self) {
        let path = self.save_path.clone();
        let restored = std::fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                paredros_world::timed_action::TimedActionSession::restore(&bytes)
                    .map_err(|error| format!("{error:?}"))
            });
        self.status = vec![match restored {
            Ok(action) => {
                if action.session().game().bodies().get(self.target).is_none() {
                    "Load failed: saved session has no selected target".to_owned()
                } else {
                    let played = action.session().control().played();
                    self.model.borrow_mut().set_action(action);
                    self.model.borrow_mut().set_played(played);
                    self.charging = false;
                    self.latched = [None; 4];
                    self.last_motion = Instant::now();
                    self.last_charge = Instant::now();
                    format!("Loaded {}", path.display())
                }
            },
            Err(why) => format!("Load failed: {why}"),
        }];
    }
}

/// Latch a movement key. A fresh press holds longer than an auto-repeat so the
/// platform's repeat delay does not stutter a held key; see `model`.
pub(super) fn latch(slot: &mut Option<Instant>, hold: Duration) {
    *slot = Some(Instant::now() + hold);
}

fn strike_lines(events: &[GameEvent]) -> Vec<String> {
    events
        .iter()
        .find_map(|event| match event {
            GameEvent::VolleyResolved { strikes, .. } => Some(
                strikes
                    .iter()
                    .enumerate()
                    .map(|(index, strike)| match strike.outcome {
                        StrikeOutcome::Miss => {
                            format!("Strike {}: MISS (source {})", index + 1, strike.source.0)
                        },
                        StrikeOutcome::Hit {
                            part,
                            quality,
                            harm,
                        } => format!(
                            "Strike {}: HIT target part {} quality {quality} harm {harm}",
                            index + 1,
                            part.0
                        ),
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| vec!["No volley resolution recorded".into()])
}

fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let pending = path.with_extension("pending");
    std::fs::write(&pending, bytes).map_err(|error| error.to_string())?;
    std::fs::rename(&pending, path).map_err(|error| error.to_string())
}
