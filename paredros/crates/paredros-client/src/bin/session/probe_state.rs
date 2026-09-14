// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The typed reading a Paredros scenario asserts against, and the event stream
//! it matches. Projections only: nothing here applies an intent.

use genet_probe::ProbeSnapshot;
use paredros_world::{GameEvent, ItemKind, MOTION_SCALE};

use super::Context;

/// Every field the acceptance scenario may name. `error` is also read by the
/// shared lane's own final check, so it is "none" or a real producer message.
pub(super) fn snapshot(ctx: &Context<'_>, captures: usize, opacity: f32) -> ProbeSnapshot {
    let app = ctx.runner.state();
    let scene = app.scene.borrow();
    let model = app.model.borrow();
    let game = model.game();
    let played = model.played();
    let yes = |value: bool| if value { "yes" } else { "no" };
    let body = game.bodies().get(played);

    let position = match game.movement().pose(played) {
        Some(pose) => {
            let at = pose.position.map(|v| v as f64 / MOTION_SCALE as f64);
            format!("{:.3},{:.3},{:.3}", at[0], at[1], at[2])
        },
        None => game
            .movement()
            .position(played)
            .map(|at| format!("{},{},{}", at[0], at[1], at[2]))
            .unwrap_or_else(|| "none".into()),
    };
    let supports = match game.movement_projection(played) {
        Ok(Some(projection)) => format!(
            "{}/{}",
            projection.active_supports.len(),
            projection.declared_supports
        ),
        Ok(None) => "legacy".into(),
        Err(_) => "none".into(),
    };
    let target_parts = game
        .current_anatomy(app.target)
        .ok()
        .or_else(|| game.anatomies().get(app.target))
        .map(|record| {
            record
                .document
                .parts
                .iter()
                .map(|part| {
                    format!(
                        "{}:{}",
                        part.id.0,
                        if part.severed { "severed" } else { "intact" }
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| "stale".into());

    ProbeSnapshot::default()
        .with_field(
            "ready",
            yes(scene.renders() > 0
                && scene.last_error().is_none()
                && body.is_some_and(|body| body.alive())),
        )
        .with_field(
            "error",
            scene.last_error().map(str::to_owned).unwrap_or("none".into()),
        )
        .with_field(
            "viewport-error",
            app.published_error.clone().unwrap_or("none".into()),
        )
        .with_field(
            "action",
            model
                .action()
                .and_then(|action| action.action())
                .map(|open| format!("{:?}", open.direction))
                .unwrap_or("idle".into()),
        )
        .with_field("aim", format!("{:?}", app.direction))
        .with_field("charging", yes(app.charging))
        .with_field("position", position)
        .with_field(
            "step",
            game.movement()
                .pose(played)
                .map_or(0, |pose| pose.step)
                .to_string(),
        )
        .with_field(
            "grounded",
            yes(game
                .movement()
                .pose(played)
                .is_some_and(|pose| pose.grounded)),
        )
        .with_field("supports", supports)
        .with_field("vitality", body.map_or(0, |body| body.vitality).to_string())
        .with_field("wound", body.map_or(0, |body| body.wound).to_string())
        .with_field(
            "dressings",
            game.items()
                .carried_by(played)
                .filter(|item| item.kind == ItemKind::Dressing)
                .count()
                .to_string(),
        )
        .with_field(
            "lost-parts",
            game.anatomies()
                .get(played)
                .map_or(0, |record| {
                    record.document.parts.iter().filter(|p| p.severed).count()
                })
                .to_string(),
        )
        .with_field(
            "target-vitality",
            game.bodies()
                .get(app.target)
                .map_or(0, |body| body.vitality)
                .to_string(),
        )
        .with_field("target-parts", target_parts)
        .with_field(
            "selected-subject",
            app.selected
                .map(|(subject, _)| subject.0.to_string())
                .unwrap_or("none".into()),
        )
        .with_field(
            "selected-part",
            app.selected
                .map(|(_, part)| part.0.to_string())
                .unwrap_or("none".into()),
        )
        .with_field(
            "glyphs",
            app.glyphs
                .as_ref()
                .map_or(0, |reading| reading.journey().acquisitions().len())
                .to_string(),
        )
        .with_field("glyph-last", app.last_glyph().unwrap_or("none".into()))
        .with_field("saves", app.saves.to_string())
        .with_field("loads", app.loads.to_string())
        .with_field("save-loaded", yes(app.loads > 0))
        .with_field(
            "hash",
            game.state_hash()
                .map(|hash| format!("{hash:016x}"))
                .unwrap_or("none".into()),
        )
        .with_field("renders", scene.renders().to_string())
        .with_field("captures", captures.to_string())
        .with_field("opacity", format!("{opacity}"))
}

/// Accepted `GameEvent`s since `drained`, as text, plus the new cursor.
///
/// A load replaces the session wholesale, so the cursor is clamped to the
/// restored log rather than trusted across the round trip.
pub(super) fn events_since(ctx: &Context<'_>, drained: usize) -> (Vec<String>, usize) {
    let model = ctx.runner.state().model.borrow();
    let events = model.game().events();
    let from = drained.min(events.len());
    (events[from..].iter().map(describe).collect(), events.len())
}

/// One accepted event as a matchable line: the kebab-cased variant name — the
/// same words the timed-action view uses for them — plus its bounded fields.
fn describe(event: &GameEvent) -> String {
    let debug = format!("{event:?}");
    let variant: String = debug
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
    let mut name = String::new();
    for (index, character) in variant.char_indices() {
        if character.is_ascii_uppercase() && index > 0 {
            name.push('-');
        }
        name.push(character.to_ascii_lowercase());
    }
    let detail: String = debug[variant.len()..]
        .chars()
        .take(180)
        .collect::<String>()
        .replace('\n', " ");
    format!("{name} {}", detail.trim())
}
