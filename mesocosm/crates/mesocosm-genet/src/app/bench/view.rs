// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::LazyLock;

use cambium::{AnyView, GenetCtx, GenetElement, PointerPhase, clickable, el, focusable, text};
use isomere::{ExaminerModel, ExaminerRow, Picked, Seeds, Sizes, ViewportCard};
use mesocosm_core::PartId;

use super::{LEAF_KEY, state::Bench};

pub(super) type Child = Box<dyn AnyView<Bench, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&Bench) -> Child;

fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |state: &mut Bench, _| action(state),
    )))
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
    let parts: Vec<ExaminerRow<'_>> = subject
        .map(|organism| {
            organism
                .body()
                .parts
                .iter()
                .filter(|p| !p.severed)
                .map(|part| ExaminerRow {
                    selected: model
                        .selected
                        .is_some_and(|s| s.organism == organism.id && s.part == part.id),
                    ..ExaminerRow::new(u64::from(part.id.0), format!("Part {}", part.id.0))
                })
                .collect()
        })
        .unwrap_or_default();
    let reading = model.reading();
    let mut detail: Vec<(String, String)> = reading
        .reading
        .as_ref()
        .map(|r| {
            [
                ("Part", &r.id),
                ("Role", &r.role),
                ("Condition", &r.condition),
                ("Processes", &r.process),
                ("Intake", &r.intake),
                ("Feeding", &r.feeding),
                ("Lineage", &r.lineage),
                ("Source", &r.donor),
                ("History", &r.history_event),
            ]
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.clone()))
            .collect()
        })
        .unwrap_or_default();
    // The prompt stands in for the reading, and rides *under* a generation
    // change the bench still has to say, which is why it is the product's call
    // and not "no rows".
    let note = reading
        .reading
        .is_none()
        .then_some("Choose a visible part, or use the part buttons.");
    if let Some(change) = model.selected_change() {
        detail.insert(0, ("Generation change".to_owned(), change));
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
    let overlay: Child = Box::new(focusable(clickable(
        el("button", text("Clear selection"))
            .attr("class", "overlay")
            .attr("aria-label", "Clear selection"),
        |state: &mut Bench, _| {
            state.overlay_clicks += 1;
            state.clear();
        },
    )));
    // isomere's shared card (M1 of the isomere plan), with the bench's own
    // tint class, its transform demonstration and its decorated frame. Hidden,
    // the same container holds the placeholder, so the overlay stays over it.
    let card_class = if state.decorated {
        "scene-card decorated"
    } else {
        "scene-card"
    };
    let scene: Child = if state.visible {
        let card = ViewportCard {
            id: Some("specimen-viewport"),
            class: Some(match state.tint {
                1 => "viewport warm",
                2 => "viewport cool",
                _ => "viewport",
            }),
            style: Some(if state.transformed {
                "transform:rotate(7deg) scale(0.9); transform-origin:25% 75%;"
            } else {
                ""
            }),
            card_class: Some(card_class),
            ..ViewportCard::new(LEAF_KEY, (640, 400), "Specimen")
        };
        isomere::viewport_card(
            &card,
            Some(overlay),
            Some(|state: &mut Bench, event: cambium::PointerEvent| {
                if event.phase == PointerPhase::Down {
                    state.pick(event.local, event.size);
                }
            }),
        )
    } else {
        isomere::scene_card(
            Some(card_class),
            vec![
                Box::new(el("div", text("Specimen hidden")).attr("class", "hidden-preview")),
                overlay,
            ],
        )
    };
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
                        // isomere's shared status line (M3 of the isomere
                        // plan): the bench's own id, the shared class, and the
                        // live-region role the line always deserved.
                        isomere::status_line("specimen-status", status),
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
                                scene,
                                el("div", controls).attr("class", "toolbar"),
                                el("div", appearance).attr("class", "toolbar appearance"),
                                if model.trial.is_none() {
                                    super::spatial::view(state)
                                } else {
                                    Box::new(el("div", ())) as Child
                                },
                                isomere::error_line("notice", notice),
                            ),
                        )
                        .attr("class", "preview-column"),
                        // isomere's shared examiner (M2 of the isomere plan).
                        // What crosses is the number this bench's `PartId`
                        // wraps; the selection stays the bench's.
                        isomere::examiner(
                            ExaminerModel {
                                rows: parts,
                                readings: detail,
                                note,
                                ..ExaminerModel::new("Parts examiner")
                            },
                            |state: &mut Bench, id| state.select(PartId(id as u32)),
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

/// The hex the bench sheet has carried since it was written, handed to isomere
/// as seeds so the derived palette is this palette and the bench capture sets
/// hold byte for byte (M0 of the isomere plan, Mark's ruling of 2026-09-15).
/// Anything left `None` in `picked` is isomere's to fill.
fn seeds() -> Seeds {
    let hex = |s: &str| isomere::color_from_hex(s).expect("bench seed hex");
    Seeds {
        background: hex("#eeeae1"),
        ink: hex("#27332e"),
        muted: hex("#65736a"),
        accent: hex("#315c3e"),
        warning: None,
        answer: None,
        picked: Picked {
            panel: Some(hex("#faf8f2")),
            card: Some(hex("#dce3d7")),
            border: Some(hex("#c3cabc")),
            card_border: Some(hex("#b7c1b3")),
            button_border: Some(hex("#a6b3a5")),
            viewport_border: Some(hex("#6f8470")),
            button_bg: Some(hex("#faf8f2")),
            button_ink: Some(hex("#263d2e")),
            hover: Some(hex("#dfebda")),
            focus: Some(hex("#367f56")),
            accent_ink: Some(hex("#ffffff")),
            selected_ink: Some(hex("#ffffff")),
            // The bench never restyled a selected chip's outline, so it keeps
            // the button's own.
            selected_border: Some(hex("#a6b3a5")),
            // `#notice` reads in the body ink.
            error: Some(hex("#27332e")),
            ..Picked::NONE
        },
    }
}

/// The lengths the bench reads at. Only the viewport's height is unusual: it
/// is measured off the window so the specimen grows with it.
const SIZES: Sizes = Sizes {
    viewport_height: "calc(100vh - 335px)",
    help_font: "15px",
    ..Sizes::DEFAULT
};

/// What is left once the shared sheet is subtracted: the bench's own columns,
/// its decorated card and tinted viewports, the comparison strip, the effect
/// and generation panels and the world trial. Rule order is the order the
/// sheet was written in, and these land after the shared rules so a product
/// override still wins.
const MESOCOSM_RULES: &str = r#"
.bench { padding:24px; min-height:100vh; background:var(--isomere-background); color:var(--isomere-ink); font:var(--isomere-font-size) sans-serif; }
.preview-column { flex:1; min-width:200px; }
.scene-card.decorated { border-color:#846b49; background:#c8baa0; }
.viewport.warm { color:rgb(255,176,112); }
.viewport.cool { color:rgb(125,190,255); }
.hidden-preview { height:422px; padding:24px; }
.overlay { position:absolute; right:30px; top:30px; z-index:5; }
aside { width:300px; padding:20px; background:var(--isomere-panel); border:1px solid var(--isomere-border); }
.comparison { margin-bottom:16px; }
.comparison-toolbar { display:flex; gap:10px; align-items:center; margin-bottom:8px; }
.comparison-toolbar p { flex:1; font-size:13px; }
.comparison-cards { display:flex; gap:10px; }
.proportion { flex:1; min-width:0; padding:6px; border:2px solid #bcc7b9; background:var(--isomere-panel); }
.proportion.chosen { border-color:var(--isomere-accent); }
.proportion-preview { display:block; width:100%; height:110px; color:rgb(255,255,255); background:#000000; }
.proportion button { margin-top:6px; padding:4px 8px; font-size:12px; }
.proportion p { font-size:11px; margin:4px 0; line-height:1.25; }
.change-summary { min-height:42px; }
.comparing .viewport { height:220px; min-height:220px; }
.effect-panel { padding:16px; margin-bottom:16px; background:var(--isomere-panel); border:1px solid var(--isomere-button-border); }
.effect-panel h2 { margin:0; }
.effect-panel p { font-size:12px; }
.effect-panel button { padding:5px 8px; font-size:12px; }
.effect-viewport { display:block; width:100%; height:240px; margin-top:12px; }
.generation-controls { padding:12px; margin-bottom:16px; border:1px solid var(--isomere-button-border); background:var(--isomere-panel); }
.generation-controls p { font-size:12px; }
.generation-controls button { padding:5px 8px; font-size:12px; }
.generation-controls .toolbar { margin-top:6px; align-items:center; }
.generation-options { display:flex; flex-wrap:wrap; gap:20px; }
.generation-choice.selected { background:var(--isomere-selected); color:white; }
.generation-input { width:145px; padding:6px; border:1px solid var(--isomere-button-border); background:white; }
.generation-input input { display:block; width:100%; min-height:20px; color:var(--isomere-ink); font:14px monospace; }
.world-trial { margin:0 0 12px; padding:8px; background:var(--isomere-panel); border:1px solid var(--isomere-button-border); }
.world-trial p { margin:4px 0; font-size:12px; }
.world-trial .toolbar { margin:0; }
.trial .viewport { height:calc(100vh - 680px); min-height:180px; }
.generating .viewport { height:180px; min-height:180px; }
"#;

/// The bench's sheet: isomere's shared rules under this product's palette and
/// lengths, then the rules above. Built once, and `&'static str` because that
/// is what the scenario lane's `Product::sheet` asks for.
pub(super) static SHEET: LazyLock<String> =
    LazyLock::new(|| isomere::sheet_with(&seeds(), &SIZES, MESOCOSM_RULES));
