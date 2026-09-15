// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The document: one viewport leaf and three panels over the same session.

use std::sync::LazyLock;

use cambium::{AnyView, GenetCtx, GenetElement, PointerPhase, clickable, el, focusable, text};
use isomere::{
    ExaminerModel, ExaminerRow, JournalModel, JournalRow, Picked, Seeds, Sizes, StatusPanel,
    ViewportCard,
};
use isometer::core::PartId;
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

/// The scene card: isomere's shared container and leaf, with the session's own
/// key, label and selection description (M1 of the isomere plan). The card
/// carries no overlay; the session's controls live under it, not over it.
fn viewport(state: &SessionApp) -> Child {
    let description = state.selection_line();
    let card = ViewportCard {
        id: Some("scene-viewport"),
        description: Some(&description),
        ..ViewportCard::new(LEAF_KEY, (720, 440), "Scene")
    };
    isomere::viewport_card(
        &card,
        None,
        Some(
            |state: &mut SessionApp, event: cambium::PointerEvent| match event.phase {
                // A press both names what was clicked and begins the charge, so
                // a mouse-only player has the same verb the keyboard has.
                PointerPhase::Down => {
                    state.pick(event.local, event.size);
                    state.begin_charge();
                },
                PointerPhase::Up => state.release(),
                PointerPhase::Move => {},
            },
        ),
    )
}

fn sheet_panel(state: &SessionApp) -> Child {
    let Some(sheet) = state.sheet() else {
        return Box::new(
            el(
                "aside",
                el("p", text("No admitted anatomy for the played subject.")),
            )
            .attr("class", "panel"),
        );
    };
    let played = state.played();
    let parts: Vec<ExaminerRow<'_>> = sheet
        .parts
        .iter()
        .map(|part| ExaminerRow {
            // A severed part still reads as severed while it is the selected
            // one, which is what this panel has always done; isomere's chip
            // keeps that and publishes the selection as `aria-pressed`.
            state_class: part.severed.then_some("severed"),
            selected: state.selected == Some((played, part.id)),
            ..ExaminerRow::new(
                u64::from(part.id.0),
                format!(
                    "{} ({}){}",
                    part.name,
                    part.id.0,
                    if part.severed { " severed" } else { "" }
                ),
            )
        })
        .collect();
    let mut rows: Vec<(String, String)> = vec![
        ("Subject".into(), format!("{}", sheet.subject.0)),
        ("Anatomy revision".into(), format!("{}", sheet.revision.0)),
        (
            "Learned techniques".into(),
            format!("{}", sheet.learned.len()),
        ),
        (
            "Intact parts".into(),
            format!(
                "{} of {}",
                sheet.parts.iter().filter(|part| !part.severed).count(),
                sheet.parts.len()
            ),
        ),
    ];
    for blocker in &sheet.global_blockers {
        rows.push(("Blocker".into(), blocker.clone()));
    }
    if let Some((subject, part)) = state.selected
        && let Some(row) = sheet.parts.iter().find(|row| row.id == part)
        && subject == played
    {
        rows.push(("Selected part".into(), row.name.clone()));
        rows.push((
            "Attached to".into(),
            row.parent
                .map(|parent| format!("part {}", parent.0))
                .unwrap_or_else(|| "root".into()),
        ));
        rows.push((
            "Bounds".into(),
            row.bounds
                .map(|bounds| format!("{:?} .. {:?}", bounds.min, bounds.max))
                .unwrap_or_else(|| "not placed".into()),
        ));
    }
    // isomere's shared examiner (M2 of the isomere plan). What crosses is the
    // number this session's `PartId` wraps; the selection stays the session's.
    isomere::examiner(
        ExaminerModel {
            rows: parts,
            readings: rows,
            class: Some("panel"),
            ..ExaminerModel::new("Subject sheet")
        },
        move |state: &mut SessionApp, id| state.select(played, PartId(id as u32)),
    )
}

/// The acquisition journal, beside the subject sheet: what the played
/// subject's accepted history has been read as, in first-acquisition order.
///
/// isomere's shared journal (M3 of the isomere plan). What crosses is the four
/// pieces of text this panel already had in words; the canon, the reading and
/// the revision stay the session's.
fn journal_panel(state: &SessionApp) -> Child {
    let rows: Vec<JournalRow> = state
        .journal()
        .iter()
        .map(|row| JournalRow {
            // The founding line always shows; the live line is always emitted
            // and carries words only when the published revision moved this
            // base, so a row cannot change height as one arrives.
            live: Some(match (&row.live_effect, &row.cause) {
                (Some(live), Some(cause)) => format!("now {live} · {cause}"),
                _ => String::new(),
            }),
            ..JournalRow::new(
                row.display.clone(),
                format!("{} · {} · tick {}", row.effect, row.kind, row.tick),
            )
            .marked(row.glyph.clone())
        })
        .collect();
    isomere::journal(&JournalModel {
        rows,
        // The summary leads: in a window this size the rows run past the fold,
        // and the count and eligibility are the reading.
        summary: Some(("journal-summary", state.journal_summary())),
        note: Some("Nothing acquired yet. Move, strike, take, rest."),
        class: Some("panel"),
        ..JournalModel::new("Acquisition journal")
    })
}

fn equipment_panel(state: &SessionApp) -> Child {
    let attachable = state.attachable_parts();
    let carried = state.carried();
    let rows: Vec<Child> = if carried.is_empty() {
        vec![Box::new(el(
            "p",
            text("Nothing carried. Press E over a dressing."),
        ))]
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
            (
                el("h2", text("Equipment")),
                el("div", rows).attr("class", "items"),
            ),
        )
        .attr("class", "panel"),
    )
}

/// The status panel: isomere's shared stack of published lines (M3), over the
/// same readings the timed-action host's text view prints.
fn status_panel(state: &SessionApp) -> Child {
    isomere::status_panel(&StatusPanel {
        lines: state.status_lines(),
        class: Some("panel status"),
        ..StatusPanel::new("Status")
    })
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
                        // The control-help line, read off the one declared
                        // keymap the key handler dispatches through (M3), so
                        // the sentence cannot outlive a binding.
                        isomere::help_line("controls-help", super::KEYMAP.help()),
                    ),
                ),
                el(
                    "main",
                    (
                        el(
                            "section",
                            (
                                viewport(state),
                                isomere::status_line("selection", state.selection_line()),
                                el("div", controls).attr("class", "toolbar"),
                                isomere::error_line("viewport-error", error),
                            ),
                        )
                        .attr("class", "scene-column"),
                        el(
                            "div",
                            (
                                sheet_panel(state),
                                journal_panel(state),
                                equipment_panel(state),
                            ),
                        )
                        .attr("class", "panel-column"),
                    ),
                ),
                status_panel(state),
            ),
        )
        .attr("class", "session"),
    )
}

/// The hex the session sheet has carried since it was written, handed to
/// isomere as seeds so the derived palette is this palette and the P12
/// capture set holds byte for byte (M0 of the isomere plan, Mark's ruling of
/// 2026-09-15). Anything left `None` in `picked` is isomere's to fill.
fn seeds() -> Seeds {
    let hex = |s: &str| isomere::color_from_hex(s).expect("session seed hex");
    Seeds {
        background: hex("#1b1d21"),
        ink: hex("#dfe3e6"),
        muted: hex("#8e9aa2"),
        accent: hex("#2f6b46"),
        warning: None,
        answer: None,
        picked: Picked {
            panel: Some(hex("#23272c")),
            card: Some(hex("#000000")),
            help: Some(hex("#93a0a8")),
            border: Some(hex("#39424a")),
            card_border: Some(hex("#39424a")),
            button_border: Some(hex("#4a555e")),
            // The session draws no frame on the viewport; the width below is
            // zero, so this colour never reaches a pixel.
            viewport_border: Some(hex("#39424a")),
            button_bg: Some(hex("#2e343a")),
            button_ink: Some(hex("#e4e9ec")),
            hover: Some(hex("#3a434b")),
            focus: Some(hex("#6fa8dc")),
            accent_ink: Some(hex("#e4e9ec")),
            selected_ink: Some(hex("#e4e9ec")),
            selected_border: Some(hex("#49a06a")),
            error: Some(hex("#e0a0a0")),
            ..Picked::NONE
        },
    }
}

/// The lengths the session reads at. Every one of them differed from the
/// bench's, which is why they are properties rather than constants.
const SIZES: Sizes = Sizes {
    font: "14px",
    line: "1.4",
    header_gap: "12px",
    h1: "22px",
    h2: "16px",
    h2_gap: "10px",
    p_gap: "5px",
    main_gap: "16px",
    card_pad: "8px",
    viewport_height: "440px",
    viewport_pad: "0",
    viewport_border_width: "0",
    toolbar_gap: "6px",
    toolbar_top: "8px",
    button_pad: "5px 9px",
    button_radius: "4px",
    button_font: "12px",
    parts_gap: "5px",
    parts_height: "150px",
    parts_gap_bottom: "12px",
    reading_height: "260px",
    field_gap: "8px",
    field_name_font: "11px",
    field_name_gap: "2px",
    field_value_font: "13px",
    field_value_line: "1.35",
    error_height: "18px",
    error_font: "12px",
    help_font: "12px",
};

/// What is left once the shared sheet is subtracted: the session's own
/// columns, its three panels, the glyph journal and the equipment list. Rule
/// order is the order the sheet was written in, and these land after the
/// shared rules so a product override still wins.
const PAREDROS_RULES: &str = r#"
.session { padding:16px; min-height:100vh; background:var(--isomere-background); }
.scene-column { flex:0 0 auto; min-width:240px; }
.panel-column { display:flex; flex-direction:column; gap:12px; width:330px; }
.panel { padding:12px; background:var(--isomere-panel); border:1px solid var(--isomere-border); }
.status { margin-top:14px; }
.status-lines p { font:12px monospace; margin:2px 0; color:#c6d0d6; }
.part.severed { color:#8b7076; text-decoration:line-through; }
.items { max-height:260px; overflow:auto; }
.journal { max-height:220px; overflow:auto; }
.journal-row { padding:5px 6px; margin-bottom:5px; border:1px solid var(--isomere-border); }
.journal-headline { font:13px monospace; color:var(--isomere-button-ink); }
#journal-summary { font-size:12px; color:#9fb0ba; }
.item { padding:6px; margin-bottom:6px; border:1px solid var(--isomere-border); }
#selection { font-size:12px; color:#9fb0ba; }
"#;

/// The document's sheet: isomere's shared rules under this product's palette
/// and lengths, then the rules above. Built once, and `&'static str` because
/// that is what the scenario lane's `Product::sheet` asks for.
pub(super) static SHEET: LazyLock<String> =
    LazyLock::new(|| isomere::sheet_with(&seeds(), &SIZES, PAREDROS_RULES));
