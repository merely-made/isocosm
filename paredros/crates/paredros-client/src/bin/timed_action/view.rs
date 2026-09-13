// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Text projection for the native timed-action surface.

use super::App;

pub(crate) trait AppView {
    fn target_lines(&self) -> Vec<String>;
    fn display_lines(&self) -> Vec<String>;
}

impl AppView for App {
    fn target_lines(&self) -> Vec<String> {
        let game = self.action.session().game();
        let Some(body) = game.bodies().get(self.target) else {
            return vec!["Target unavailable".into()];
        };
        let anatomy = game
            .current_anatomy(self.target)
            .ok()
            .or_else(|| game.anatomies().get(self.target));
        let parts = anatomy
            .map(|a| {
                a.document
                    .parts
                    .iter()
                    .map(|p| {
                        format!(
                            "{}:{}",
                            p.id.0,
                            if p.severed { "severed" } else { "intact" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "stale anatomy".into());
        let player = self.action.session().control().played();
        let items = game
            .items()
            .get(self.target_item)
            .map(|item| format!("{:?}", item.location))
            .unwrap_or_else(|| "missing".into());
        vec![
            format!("Player at {:?}", game.movement().position(player)),
            format!(
                "Target {} at {:?}; vitality={} {}",
                body.name.as_ref().map_or("unnamed", |n| n.as_str()),
                game.movement().position(self.target),
                body.vitality,
                body.died_at.map_or("alive", |_| "dead")
            ),
            format!("Target parts: {parts}"),
            format!("Target item {}: {items}", self.target_item.0),
        ]
    }

    fn display_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Arrows: prepare direction | WASD: move | Hold Space or left mouse: charge | Release: strike".into(),
            "J: join second limb | I: self-injury debug | F5: save | F9: load | Esc: close".into(),
        ];
        lines.extend(self.status.iter().cloned());
        if let Some(action) = self.action.action() {
            lines.push(format!(
                "Action: {:?}  started={}  last_tick={}",
                action.direction, action.started_at.0, action.last_tick.0
            ));
            for (part, contribution) in &action.contributors {
                lines.push(format!(
                    "Part {}: {:?}, charge={}, node={}",
                    part.0, contribution.state, contribution.charge, contribution.node.0
                ));
            }
        } else {
            lines.push("Action: idle".into());
        }
        lines.extend(self.target_lines());
        lines
    }
}
