// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cambium::{AnyView, GenetCtx, GenetElement, clickable, el, focusable, text};
use super::model::App;
use crate::products::Mode;

pub(crate) type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
pub(crate) type Logic = Box<dyn FnMut(&App) -> Child>;

pub(crate) fn lens(key: u64) -> Logic { Box::new(move |state| root(state, key)) }

fn button(id: String, label: String, key: u64,
    action: impl Fn(&mut App) -> Result<(), String> + 'static) -> Child {
    Box::new(focusable(clickable(el("button", text(label.clone())).attr("id", id)
        .attr("aria-label", label), move |state: &mut App, _| {
            let result = action(state); state.record(key, result);
        })))
}

fn list(heading: &str, lines: &[String]) -> Child {
    let mut children: Vec<Child> = vec![Box::new(el("h2", text(heading.to_owned())))];
    children.extend(lines.iter().map(|line| Box::new(el("p", text(line.clone()))) as Child));
    Box::new(el("section", children).attr("class", "panel"))
}

pub(crate) fn root(state: &App, key: u64) -> Child {
    let Some(local) = state.windows.get(&key) else { return Box::new(el("p", text("Closed"))); };
    let mode = local.native.binding().mode.clone();
    let mut controls: Vec<Child> = vec![Box::new(el("span", text("Mode")))];
    for candidate in Mode::enabled() {
        let label = candidate.key().to_owned();
        controls.push(button(format!("mode-{}", candidate.key()), label, key,
            move |app| app.switch(key, candidate)));
    }
    controls.push(button("new-window".into(), "New window".into(), key,
        move |app| app.duplicate(key).map(|_| ())));
    controls.push(button("close-window".into(), "Close window".into(), key, move |app| {
        app.windows.get_mut(&key).ok_or("window is closed")?.close_requested = true; Ok(())
    }));
    let mut actions: Vec<Child> = vec![button("step".into(), "Step world".into(), key,
        |app| app.step()), button("turn".into(), "Quarter turn".into(), key,
        move |app| app.turn(key))];
    if local.input.as_ref().is_some_and(|i| i.mode.key() == mode) {
        actions.push(button("send-intent".into(), "Send chosen intent".into(), key,
            move |app| app.send(key).map(|_| ())));
    }
    if state.save_path.is_some() {
        actions.push(button("save".into(), "Save world".into(), key, |app| app.save()));
    }
    // Selecting a close-up region logs its watch; its camera remains local.
    if let Ok(cut) = local.native.projection() {
        for site in cut.visible.sites {
            actions.push(button(format!("examine-{site}"), format!("Examine site {site}"), key,
                move |app| app.examine(key, Some(site))));
        }
    }
    actions.push(button("stop-examining".into(), "Stop examining".into(), key,
        move |app| app.examine(key, None)));
    let mut pins: Vec<Child> = vec![Box::new(el("h2", text("Pins in this window")))];
    match state.pin_choices(key) {
        Ok(choices) => for choice in choices {
            let action = if choice.pinned { "Unpin" } else { "Pin" };
            pins.push(button(choice.id, format!("{action} {}", choice.label), key,
                move |app| app.pin(key, choice.point)));
        },
        Err(why) => pins.push(Box::new(el("p", text(why)).attr("role", "status"))),
    }
    let controls_line = local.native.binding().controls.join(", ");
    let error = local.error.clone().unwrap_or_default();
    let mut children: Vec<Child> = vec![
        Box::new(el("h1", text(format!("Isocosm: {mode}")))) as Child,
        Box::new(el("nav", controls)) as Child,
        Box::new(el("p", text(controls_line))) as Child,
        Box::new(el("nav", actions)) as Child,
        list("Things", &local.rows.things), list("Sites", &local.rows.sites),
        list("Attended events", &local.rows.events),
        Box::new(el("section", pins).attr("class", "panel")) as Child,
        Box::new(el("p", text(error)).attr("id", "error").attr("role", "status")) as Child,
    ];
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    if local.scene.snapshot().is_some() {
        let card = isomere::ViewportCard::new(crate::viewport::key(key).expect("admitted local slot"),
            (720, 440), "Site view");
        children.insert(4, isomere::viewport_card(&card, None, None::<fn(&mut App, cambium::PointerEvent)>));
    }
    Box::new(el("main", children).attr("data-local-window", key.to_string()))
}

pub(crate) fn sheet() -> String {
    let hex = |s| isomere::color_from_hex(s).expect("the existing Eponym sheet seeds are valid");
    let seeds = isomere::Seeds::anchors(hex("#1b1d21"), hex("#dfe3e6"), hex("#8e9aa2"), hex("#2f6b46"));
    isomere::sheet(&seeds, EXTRA)
}

const EXTRA: &str = "
main { padding: 20px; display: flex; flex-direction: column; gap: 12px; }
nav { display: flex; flex-direction: row; gap: 8px; flex-wrap: wrap; }
.panel { padding: 12px; background: var(--isomere-panel); }
";
