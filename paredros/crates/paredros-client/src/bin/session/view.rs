// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The document: one viewport leaf and three panels over the same session.

use cambium::{
    AnyView, GenetCtx, GenetElement, PointerPhase, clickable, custom_leaf, el, focusable,
    on_pointer, text,
};
use mesocosm_core::PartId;
use paredros_world::ItemKind;

use super::{LEAF_KEY, SessionApp};

pub(super) type Child = Box<dyn AnyView<SessionApp, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&SessionApp) -> Child;

fn button(label: String, class: &'static str, action: impl Fn(&mut SessionApp) + 'static) -> Child {
    let aria = label.clone();
    Box::new(focusable(clickable(
        el("button", text(label))
            .attr("class", class)
            .attr("aria-label", aria),
        move |state: &mut SessionApp, _| action(state),
    )))
}

fn field(name: &str, value: String) -> Child {
    Box::new(
        el(
            "div",
            (
                el("div", text(name.to_owned())).attr("class", "field-name"),
                el("div", text(value)).attr("class", "field-value"),
            ),
        )
        .attr("class", "field"),
    )
}

fn viewport(state: &SessionApp) -> Child {
    Box::new(on_pointer(
        custom_leaf::<SessionApp, ()>(LEAF_KEY, 720, 440)
            .attr("id", "scene-viewport")
            .attr("class", "viewport")
            .attr("role", "img")
            .attr("aria-label", "Scene")
            .attr("aria-description", state.selection_line()),
        |state: &mut SessionApp, event: cambium::PointerEvent| match event.phase {
            // A press both names what was clicked and begins the charge, so a
            // mouse-only player has the same verb the keyboard has.
            PointerPhase::Down => {
                state.pick(event.local, event.size);
                state.begin_charge();
            },
            PointerPhase::Up => state.release(),
            PointerPhase::Move => {},
        },
    ))
}

fn sheet_panel(state: &SessionApp) -> Child {
    let Some(sheet) = state.sheet() else {
        return Box::new(
            el("aside", el("p", text("No admitted anatomy for the played subject.")))
                .attr("class", "panel"),
        );
    };
    let played = state.played();
    let parts: Vec<Child> = sheet
        .parts
        .iter()
        .map(|part| {
            let id = part.id;
            let selected = state.selected == Some((played, id));
            let label = format!(
                "{} ({}){}",
                part.name,
                id.0,
                if part.severed { " severed" } else { "" }
            );
            let class = match (part.severed, selected) {
                (true, _) => "part severed",
                (false, true) => "part selected",
                (false, false) => "part",
            };
            button(label, class, move |state: &mut SessionApp| {
                state.select(played, id)
            })
        })
        .collect();
    let mut rows: Vec<Child> = vec![
        field("Subject", format!("{}", sheet.subject.0)),
        field("Anatomy revision", format!("{}", sheet.revision.0)),
        field("Learned techniques", format!("{}", sheet.learned.len())),
        field(
            "Intact parts",
            format!(
                "{} of {}",
                sheet.parts.iter().filter(|part| !part.severed).count(),
                sheet.parts.len()
            ),
        ),
    ];
    for blocker in &sheet.global_blockers {
        rows.push(field("Blocker", blocker.clone()));
    }
    if let Some((subject, part)) = state.selected
        && let Some(row) = sheet.parts.iter().find(|row| row.id == part)
        && subject == played
    {
        rows.push(field("Selected part", row.name.clone()));
        rows.push(field(
            "Attached to",
            row.parent
                .map(|parent| format!("part {}", parent.0))
                .unwrap_or_else(|| "root".into()),
        ));
        rows.push(field(
            "Bounds",
            row.bounds
                .map(|bounds| format!("{:?} .. {:?}", bounds.min, bounds.max))
                .unwrap_or_else(|| "not placed".into()),
        ));
    }
    Box::new(
        el(
            "aside",
            (
                el("h2", text("Subject sheet")),
                el("div", parts).attr("class", "parts"),
                el("div", rows).attr("class", "reading"),
            ),
        )
        .attr("class", "panel"),
    )
}

fn equipment_panel(state: &SessionApp) -> Child {
    let attachable = state.attachable_parts();
    let carried = state.carried();
    let rows: Vec<Child> = if carried.is_empty() {
        vec![Box::new(el("p", text("Nothing carried. Press E over a dressing.")))]
    } else {
        carried
            .iter()
            .map(|item| {
                let id = item.item;
                let mut controls: Vec<Child> = Vec::new();
                match item.attached {
                    Some(part) => controls.push(button(
                        format!("Detach from part {}", part.0),
                        "equip",
                        move |state: &mut SessionApp| state.detach(id),
                    )),
                    None => {
                        for part in &attachable {
                            let part = *part;
                            controls.push(button(
                                format!("Attach to part {}", part.0),
                                "equip",
                                move |state: &mut SessionApp| state.attach(id, part),
                            ));
                        }
                    },
                }
                Box::new(
                    el(
                        "div",
                        (
                            el(
                                "div",
                                text(format!(
                                    "Item {} · {}",
                                    id.0,
                                    match item.kind {
                                        ItemKind::Dressing => "dressing",
                                        ItemKind::Food => "food",
                                        ItemKind::Scrap => "scrap",
                                    }
                                )),
                            )
                            .attr("class", "field-name"),
                            el("div", controls).attr("class", "toolbar"),
                        ),
                    )
                    .attr("class", "item"),
                ) as Child
            })
            .collect()
    };
    Box::new(
        el(
            "aside",
            (el("h2", text("Equipment")), el("div", rows).attr("class", "items")),
        )
        .attr("class", "panel"),
    )
}

fn status_panel(state: &SessionApp) -> Child {
    let lines: Vec<Child> = state
        .status_lines()
        .into_iter()
        .map(|line| Box::new(el("p", text(line))) as Child)
        .collect();
    Box::new(
        el(
            "section",
            (
                el("h2", text("Status")),
                el("div", lines).attr("class", "status-lines"),
            ),
        )
        .attr("class", "panel status"),
    )
}

pub(super) fn root(state: &SessionApp) -> Child {
    let controls = vec![
        button("Strike".into(), "control", |state: &mut SessionApp| {
            if state.charging {
                state.release()
            } else {
                state.begin_charge()
            }
        }),
        button("Join part 2".into(), "control", |state: &mut SessionApp| {
            state.join_limb(PartId(2))
        }),
        button("Take dressing".into(), "control", SessionApp::take_dressing),
        button("Rest".into(), "control", SessionApp::rest),
        button("Injury debug".into(), "control", SessionApp::injure),
        button("Save".into(), "control", SessionApp::save),
        button("Load".into(), "control", SessionApp::load),
    ];
    let error = state
        .published_error
        .clone()
        .unwrap_or_else(|| "No viewport error.".to_owned());
    Box::new(
        el(
            "div",
            (
                el(
                    "header",
                    (
                        el("h1", text("Paredros · Session")),
                        el(
                            "p",
                            text(
                                "WASD move · arrows aim · Space charge, Space again to strike · \
                                 left mouse hold on the scene to charge · E take · R rest · \
                                 I injury · J join part 2 · Ctrl+S save · Ctrl+L load · Esc close",
                            ),
                        )
                        .attr("id", "controls-help"),
                    ),
                ),
                el(
                    "main",
                    (
                        el(
                            "section",
                            (
                                el("div", viewport(state)).attr("class", "scene-card"),
                                el("p", text(state.selection_line()))
                                    .attr("id", "selection")
                                    .attr("role", "status"),
                                el("div", controls).attr("class", "toolbar"),
                                el("p", text(error))
                                    .attr("id", "viewport-error")
                                    .attr("role", "status"),
                            ),
                        )
                        .attr("class", "scene-column"),
                        el("div", (sheet_panel(state), equipment_panel(state)))
                            .attr("class", "panel-column"),
                    ),
                ),
                status_panel(state),
            ),
        )
        .attr("class", "session"),
    )
}

pub(super) const SHEET: &str = r#"
html, body { margin:0; padding:0; background:#1b1d21; color:#dfe3e6; font:14px sans-serif; }
* { box-sizing:border-box; }
.session { padding:16px; min-height:100vh; background:#1b1d21; }
header { margin-bottom:12px; }
h1 { margin:0; font-size:22px; font-weight:700; }
h2 { margin:0 0 10px; font-size:16px; }
p { margin:5px 0; line-height:1.4; }
#controls-help { color:#93a0a8; font-size:12px; }
main { display:flex; gap:16px; align-items:flex-start; }
.scene-column { flex:0 0 auto; min-width:240px; }
.panel-column { display:flex; flex-direction:column; gap:12px; width:330px; }
.scene-card { padding:8px; border:2px solid #39424a; background:#000000; }
.viewport { display:block; width:100%; height:440px; min-height:240px; color:rgb(255,255,255); }
.panel { padding:12px; background:#23272c; border:1px solid #39424a; }
.status { margin-top:14px; }
.status-lines p { font:12px monospace; margin:2px 0; color:#c6d0d6; }
.toolbar { display:flex; flex-wrap:wrap; gap:6px; margin-top:8px; }
button { padding:5px 9px; border:1px solid #4a555e; border-radius:4px; background:#2e343a; color:#e4e9ec; font:12px sans-serif; cursor:pointer; }
button:hover { background:#3a434b; }
button:focus { outline:2px solid #6fa8dc; outline-offset:2px; }
.parts { display:flex; flex-wrap:wrap; gap:5px; max-height:150px; overflow:auto; margin-bottom:12px; }
.part.selected { background:#2f6b46; border-color:#49a06a; }
.part.severed { color:#8b7076; text-decoration:line-through; }
.reading { max-height:260px; overflow:auto; }
.field { margin-bottom:8px; }
.field-name { font-size:11px; color:#8e9aa2; margin-bottom:2px; }
.field-value { font-size:13px; line-height:1.35; }
.items { max-height:260px; overflow:auto; }
.item { padding:6px; margin-bottom:6px; border:1px solid #39424a; }
#selection { font-size:12px; color:#9fb0ba; }
#viewport-error { min-height:18px; font-size:12px; color:#e0a0a0; }
"#;
