// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl App {
    pub(super) fn move_player(&mut self, toward: [i32; 3]) {
        let subject = self.action.session().control().played();
        let game = self.action.session().game();
        let pose = game.movement().pose(subject).expect("played pose");
        let revision = game.bodies().get(subject).unwrap().revision;
        self.status = match self.action.apply_game_batch(&[GameIntent::AdvanceMotion {
            tick: game.next_tick(),
            subject,
            revision,
            step: pose.step + 1,
            input: paredros_world::MotionInput {
                move_x: (toward[0] * 32767) as i16,
                move_z: (toward[2] * 32767) as i16,
            },
            rules: support::rules(game, subject),
        }]) {
            Ok(events) => vec![format!(
                "Motion step {}: {} event(s)",
                pose.step + 1,
                events.len()
            )],
            Err(e) => {
                self.movement_keys = [false; 4];
                vec![format!("Motion failed: {e:?}")]
            },
        };
    }
    pub(super) fn motion_running(&self) -> bool {
        let subject = self.action.session().control().played();
        self.action
            .session()
            .game()
            .bodies()
            .get(subject)
            .is_some_and(|body| body.alive())
            && (self.movement_keys.iter().any(|key| *key)
                || self
                    .action
                    .session()
                    .game()
                    .movement()
                    .pose(subject)
                    .is_some_and(|pose| !pose.grounded))
    }
    pub(super) fn tick_motion(&mut self) {
        if !self.motion_running() {
            self.last_motion = Instant::now();
            return;
        }
        let interval = Duration::from_secs_f64(1.0 / 60.0);
        let mut steps = 0;
        while self.last_motion.elapsed() >= interval && steps < 8 && self.motion_running() {
            let input = [
                i32::from(self.movement_keys[3]) - i32::from(self.movement_keys[2]),
                0,
                i32::from(self.movement_keys[0]) - i32::from(self.movement_keys[1]),
            ];
            self.move_player(input);
            self.last_motion += interval;
            steps += 1;
        }
        if steps == 8 {
            self.last_motion = Instant::now();
        }
        if steps > 0 {
            self.redraw();
        }
    }
    pub(super) fn injure(&mut self) {
        let subject = self.action.session().control().played();
        let old = self
            .action
            .session()
            .game()
            .bodies()
            .get(subject)
            .unwrap()
            .revision;
        let first = self.action.session().game().next_tick();
        let second = Tick(first.0 + 1);
        let next = paredros_identity::BodyRevisionId(old.0 + 1);
        let result = self.action.apply_game_batch(&[
            GameIntent::Fall {
                tick: first,
                subject,
                distance: 5,
            },
            GameIntent::ReconcileAnatomy {
                tick: second,
                subject,
                from_revision: old,
                revision: next,
                severed_parts: vec![isometer_core::PartId(1)],
            },
        ]);
        self.status = vec![format!(
            "Injury cut: {}",
            result
                .map(|events| format!(
                    "{} events; part 1 removed, surviving contributors retained",
                    events.len()
                ))
                .unwrap_or_else(|e| format!("{e:?}"))
        )];
        self.redraw();
    }

    pub(super) fn rest(&mut self) {
        self.status = match self.action.rest() {
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
        self.redraw();
    }

    pub(super) fn take_dressing(&mut self) {
        let game = self.action.session().game();
        let subject = self.action.session().control().played();
        let at = game.movement().position(subject).unwrap();
        let item = game
            .items()
            .at(at)
            .find(|item| item.kind == ItemKind::Dressing)
            .map(|item| item.id);
        self.status = if let Some(item) = item {
            match self.action.apply_game_batch(&[GameIntent::Take {
                tick: game.next_tick(),
                subject,
                item,
            }]) {
                Ok(_) => vec![format!("Picked up dressing {}", item.0)],
                Err(error) => vec![format!("Pickup failed: {error:?}")],
            }
        } else {
            vec!["No dressing at this position".into()]
        };
        self.redraw();
    }
}
