// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl App {
    pub(super) fn move_player(&mut self, toward: [i32; 3]) {
        let tick = self.action.session().game().next_tick();
        let subject = self.action.session().control().played();
        let at = self
            .action
            .session()
            .game()
            .movement()
            .position(subject)
            .expect("played body has a position");
        let goal = [at[0] + toward[0], at[1] + toward[1], at[2] + toward[2]];
        self.status = match self.action.apply_game_batch(&[GameIntent::Move {
            tick,
            subject,
            toward: goal,
        }]) {
            Ok(events) => vec![format!("Moved: {} event(s)", events.len())],
            Err(e) => vec![format!("Move failed: {e:?}")],
        };
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
                severed_parts: vec![mesocosm_core::PartId(1)],
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
