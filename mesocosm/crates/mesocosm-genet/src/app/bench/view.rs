// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cambium::{
    AnyView, GenetCtx, GenetElement, PointerPhase, clickable, custom_leaf, el, focusable,
    on_pointer, text,
};

use super::{LEAF_KEY, state::Bench};

pub(super) type Child = Box<dyn AnyView<Bench, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&Bench) -> Child;

fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |state: &mut Bench, _| action(state),
    )))
}

fn row(label: &str, value: String) -> Child {
    Box::new(
        el(
            "div",
            (
                el("div", text(label.to_owned())).attr("class", "field-name"),
                el("div", text(value)).attr("class", "field-value"),
            ),
        )
        .attr("class", "field"),
    )
}

pub(super) fn root(state: &Bench) -> Child {
    if state.model.borrow().population.is_some() {
        return super::population::view(state);
    }
    if state.effects.open {
        return Box::new(
            el(
                "div",
                (
                    el("h1", text("Specimen bench / Effects")),
                    button("Back to specimen", |s| {
                        s.effects.open = false;
                        s.effects.playing = false;
                    }),
                    super::effects::view(state),
                ),
            )
            .attr("class", "bench"),
        );
    }

    let model = state.model.borrow();
    let creator = &model.creator;
    let subject = model
        .subject()
        .and_then(|id| model.world().organisms.iter().find(|o| o.id == id));
    let parts: Vec<Child> = subject
        .map(|organism| {
            organism
                .body()
                .parts
                .iter()
                .filter(|p| !p.severed)
                .map(|part| {
                    let id = part.id;
                    let selected = model
                        .selected
                        .is_some_and(|s| s.organism == organism.id && s.part == id);
                    Box::new(focusable(clickable(
                        el("button", text(format!("Part {}", id.0)))
                            .attr("aria-label", format!("Part {}", id.0))
                            .attr("aria-pressed", selected.to_string())
                            .attr("class", if selected { "part selected" } else { "part" }),
                        move |state: &mut Bench, _| state.select(id),
                    ))) as Child
                })
                .collect()
        })
        .unwrap_or_default();
    let reading = model.reading();
    let mut detail: Vec<Child> = reading
        .reading
        .map(|r| {
            vec![
                row("Part", r.id),
                row("Role", r.role),
                row("Condition", r.condition),
                row("Processes", r.process),
                row("Intake", r.intake),
                row("Feeding", r.feeding),
                row("Lineage", r.lineage),
                row("Source", r.donor),
                row("History", r.history_event),
            ]
        })
        .unwrap_or_else(|| {
            vec![Box::new(el(
                "p",
                text("Choose a visible part, or use the part buttons."),
            ))]
        });
    if let Some(change) = model.selected_change() {
        detail.insert(0, row("Generation change", change));
    }
    let status = if creator.pending {
        "Generating specimens…".to_owned()
    } else {
        format!(
            "Seed {} · Candidate {} of {} · {} parts",
            creator.request.seed,
            if creator.count() > 0 {
                creator.selected + 1
            } else {
                0
            },
            creator.count(),
            parts.len()
        )
    };
    let viewport: Child = if state.visible {
        Box::new(on_pointer(
            custom_leaf::<Bench, ()>(LEAF_KEY, 640, 400)
                .attr("id", "specimen-viewport")
                .attr(
                    "class",
                    match state.tint {
                        1 => "viewport warm",
                        2 => "viewport cool",
                        _ => "viewport",
                    },
                )
                .attr("role", "img")
                .attr("aria-label", "Specimen")
                .attr(
                    "style",
                    if state.transformed {
                        "transform:rotate(7deg) scale(0.9); transform-origin:25% 75%;"
                    } else {
                        ""
                    },
                ),
            |state: &mut Bench, event: cambium::PointerEvent| {
                if event.phase == PointerPhase::Down {
                    state.pick(event.local, event.size);
                }
            },
        ))
    } else {
        Box::new(el("div", text("Specimen hidden")).attr("class", "hidden-preview"))
    };
    let overlay: Child = Box::new(focusable(clickable(
        el("button", text("Clear selection"))
            .attr("class", "overlay")
            .attr("aria-label", "Clear selection"),
        |state: &mut Bench, _| {
            state.overlay_clicks += 1;
            state.clear();
        },
    )));
    let mut controls = vec![
        button("World trial", Bench::start_trial),
        button("Effects experiment", |s| {
            s.effects.open = !s.effects.open;
            s.model.borrow_mut().spatial.playing = false;
            s.model.borrow_mut().pause_trial();
            s.effects.playing = false;
        }),
        button("Generation controls", |s| {
            s.generation.open = !s.generation.open
        }),
        button("Previous", |s| s.candidate(true)),
        button("Next", |s| s.candidate(false)),
        button("Reroll", Bench::reroll),
        button("Compare proportions", Bench::compare),
        button("Save criteria", Bench::save),
        button("Turn left", |s| s.turn(-0.2617994)),
        button("Turn right", |s| s.turn(0.2617994)),
    ];
    if state.model.borrow().trial.is_none() {
        controls.push(button("Body / habitat", Bench::toggle_habitat));
    }
    let appearance = vec![
        button("Natural", |s| s.tint = 0),
        button("Warm", |s| s.tint = 1),
        button("Cool", |s| s.tint = 2),
        button("Frame", |s| s.decorated = !s.decorated),
        button("Hide / show", |s| {
            s.visible = !s.visible;
            s.model.borrow_mut().spatial.playing = false;
            s.model.borrow_mut().pause_trial();
            s.clear();
        }),
        button("Transform", |s| s.transformed = !s.transformed),
    ];
    let mut notice = state.notice.clone();
    if let Some(error) = &state.published_error {
        notice = error.clone();
    }
    Box::new(
        el(
            "div",
            (
                el(
                    "header",
                    (
                        el("h1", text("Specimen bench")),
                        el("p", text(status)).attr("id", "specimen-status"),
                    ),
                ),
                super::comparison_view::strip(state),
                super::generation_controls::view(state),
                super::effects::view(state),
                super::trial::view(state),
                el(
                    "main",
                    (
                        el(
                            "section",
                            (
                                el("div", (viewport, overlay)).attr(
                                    "class",
                                    if state.decorated {
                                        "scene-card decorated"
                                    } else {
                                        "scene-card"
                                    },
                                ),
                                el("div", controls).attr("class", "toolbar"),
                                el("div", appearance).attr("class", "toolbar appearance"),
                                if model.trial.is_none() {
                                    super::spatial::view(state)
                                } else {
                                    Box::new(el("div", ())) as Child
                                },
                                el("p", text(notice))
                                    .attr("role", "status")
                                    .attr("id", "notice"),
                            ),
                        )
                        .attr("class", "preview-column"),
                        el(
                            "aside",
                            (
                                el("h2", text("Parts examiner")),
                                el("div", parts).attr("class", "parts"),
                                el("div", detail).attr("class", "reading"),
                            ),
                        ),
                    ),
                ),
            ),
        )
        .attr(
            "class",
            if model.trial.is_some() {
                "bench trial"
            } else if state.generation.open {
                "bench generating"
            } else if model.comparison.is_some() {
                "bench comparing"
            } else {
                "bench"
            },
        ),
    )
}

pub(super) const SHEET: &str = r#"
html, body { margin:0; padding:0; background:#eeeae1; color:#27332e; font:15px sans-serif; }
* { box-sizing:border-box; }
.bench { padding:24px; min-height:100vh; background:#eeeae1; color:#27332e; font:15px sans-serif; }
header { margin-bottom:20px; }
h1 { margin:0; font-size:28px; font-weight:700; }
h2 { margin:0 0 16px; font-size:20px; }
p { margin:8px 0; line-height:1.5; }
header p, .field-name { color:#65736a; }
main { display:flex; gap:24px; align-items:flex-start; }
.preview-column { flex:1; min-width:200px; }
.scene-card { position:relative; overflow:hidden; padding:16px; border:2px solid #b7c1b3; background:#dce3d7; }
.scene-card.decorated { border-color:#846b49; background:#c8baa0; }
.viewport { display:block; width:100%; height:calc(100vh - 335px); min-height:240px; padding:8px; border:3px solid #6f8470; color:rgb(255,255,255); }
.viewport.warm { color:rgb(255,176,112); }
.viewport.cool { color:rgb(125,190,255); }
.hidden-preview { height:422px; padding:24px; }
.overlay { position:absolute; right:30px; top:30px; z-index:5; }
.toolbar { display:flex; flex-wrap:wrap; gap:8px; margin-top:14px; }
button { padding:8px 12px; border:1px solid #a6b3a5; border-radius:5px; background:#faf8f2; color:#263d2e; font:14px sans-serif; cursor:pointer; }
button:hover { background:#dfebda; }
button:focus { outline:2px solid #367f56; outline-offset:2px; }
aside { width:300px; padding:20px; background:#faf8f2; border:1px solid #c3cabc; }
.parts { display:flex; flex-wrap:wrap; gap:6px; max-height:150px; overflow:auto; margin-bottom:18px; }
.part.selected { background:#315c3e; color:#ffffff; }
.reading { max-height:430px; overflow:auto; }
.field { margin-bottom:12px; }
.field-name { font-size:12px; margin-bottom:3px; }
.field-value { font-size:14px; line-height:1.4; }
#notice { min-height:24px; }
.comparison { margin-bottom:16px; }
.comparison-toolbar { display:flex; gap:10px; align-items:center; margin-bottom:8px; }
.comparison-toolbar p { flex:1; font-size:13px; }
.comparison-cards { display:flex; gap:10px; }
.proportion { flex:1; min-width:0; padding:6px; border:2px solid #bcc7b9; background:#faf8f2; }
.proportion.chosen { border-color:#315c3e; }
.proportion-preview { display:block; width:100%; height:110px; color:rgb(255,255,255); background:#000000; }
.proportion button { margin-top:6px; padding:4px 8px; font-size:12px; }
.proportion p { font-size:11px; margin:4px 0; line-height:1.25; }
.change-summary { min-height:42px; }
.comparing .viewport { height:220px; min-height:220px; }
.effect-panel { padding:16px; margin-bottom:16px; background:#faf8f2; border:1px solid #a6b3a5; }
.effect-panel h2 { margin:0; }
.effect-panel p { font-size:12px; }
.effect-panel button { padding:5px 8px; font-size:12px; }
.effect-viewport { display:block; width:100%; height:240px; margin-top:12px; }
.generation-controls { padding:12px; margin-bottom:16px; border:1px solid #a6b3a5; background:#faf8f2; }
.generation-controls p { font-size:12px; }
.generation-controls button { padding:5px 8px; font-size:12px; }
.generation-controls .toolbar { margin-top:6px; align-items:center; }
.generation-options { display:flex; flex-wrap:wrap; gap:20px; }
.generation-choice.selected { background:#315c3e; color:white; }
.generation-input { width:145px; padding:6px; border:1px solid #a6b3a5; background:white; }
.generation-input input { display:block; width:100%; min-height:20px; color:#27332e; font:14px monospace; }
.world-trial { margin:0 0 12px; padding:8px; background:#faf8f2; border:1px solid #a6b3a5; }
.world-trial p { margin:4px 0; font-size:12px; }
.world-trial .toolbar { margin:0; }
.trial .viewport { height:calc(100vh - 680px); min-height:180px; }
.generating .viewport { height:180px; min-height:180px; }
"#;
