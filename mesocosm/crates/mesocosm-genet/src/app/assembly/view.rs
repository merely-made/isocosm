// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{LEAF, MAP, State};
use cambium::{AnyView, GenetCtx, GenetElement, Keyed, clickable, el, focusable, lens, text};

#[cfg(test)]
mod tests;

pub(super) type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&State) -> Child;

/// Position and display values do not name a retained panel. A part revision
/// validates its reading; changing that token alone does not name a new panel.
/// A fresh world source always begins a new inspector/route lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PanelKey {
    Map,
    Vitals,
    Board,
    NextOffer,
    CommitOffer,
    KeepLine,
    Question,
    NextChoice,
    Answer,
    CloseHints,
    TakeBody,
    Inspector(u64, Option<(u64, u32)>),
}

impl PanelKey {
    fn name(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Vitals => "vitals",
            Self::Board => "board",
            Self::NextOffer => "next-offer",
            Self::CommitOffer => "commit-offer",
            Self::KeepLine => "keep-line",
            Self::Question => "question",
            Self::NextChoice => "next-choice",
            Self::Answer => "answer",
            Self::CloseHints => "close-hints",
            Self::TakeBody => "take-body",
            Self::Inspector(..) => "inspector",
        }
    }
}

fn inspector_key(state: &State) -> PanelKey {
    PanelKey::Inspector(
        state.source_lifetime,
        state
            .host
            .inspection
            .selected
            .map(|part| (part.organism, part.part.0)),
    )
}

fn inspector(state: &State) -> (PanelKey, Child) {
    (
        inspector_key(state),
        Box::new(lens(
            |v: &mut mesocosm_views::Dev| mesocosm_views::dev_root(v),
            |s: &mut State| &mut s.dev,
        )),
    )
}

fn aside(panels: Vec<(PanelKey, Child)>) -> Child {
    let panels = panels
        .into_iter()
        .map(|(key, panel)| {
            // Keyed single-element children move their existing DOM subtree.
            // display:contents keeps the previous panel/button layout.
            let root: Child = Box::new(
                el("div", panel)
                    .attr("class", "aside-panel")
                    .attr("data-panel", key.name()),
            );
            (key, root)
        })
        .collect();
    Box::new(el("aside", Keyed::new(panels)))
}

fn button(label: &str, key: winit::keyboard::Key) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label.to_owned())).attr("aria-label", label.to_owned()),
        move |s: &mut State, _| s.press(&key),
    )))
}

pub(super) fn root(state: &State) -> Child {
    let mut card = isomere::ViewportCard::new(
        LEAF,
        (
            state.host.config.width.saturating_sub(290).max(240),
            state.host.config.height.saturating_sub(110).max(160),
        ),
        "Terrarium",
    );
    card.style = Some(
        "display:block;width:calc(100vw - 290px);height:calc(100vh - 110px);min-width:240px;min-height:160px",
    );
    let viewport = isomere::viewport_card(
        &card,
        None,
        Some(|s: &mut State, e: cambium::PointerEvent| s.point(e)),
    );
    let mut readings: Vec<(PanelKey, Child)> = vec![
        (
            PanelKey::Map,
            Box::new(
                cambium::custom_leaf::<State, ()>(MAP, 160, 160)
                    .attr("role", "img")
                    .attr("aria-label", "World map"),
            ),
        ),
        (
            PanelKey::Vitals,
            Box::new(lens(
                |v: &mut mesocosm_views::Vitals| mesocosm_views::vitals_root(v),
                |s: &mut State| &mut s.vitals,
            )),
        ),
    ];
    let mut panels: Vec<(PanelKey, Child)> = Vec::new();
    if state.review_open {
        panels.push((
            PanelKey::Board,
            Box::new(lens(
                |v: &mut mesocosm_views::Board| mesocosm_views::board_root(v),
                |s: &mut State| &mut s.board,
            )),
        ));
        panels.push((
            PanelKey::NextOffer,
            button(
                "Next offer",
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
            ),
        ));
        if state.board.commit.is_some() {
            panels.push((
                PanelKey::CommitOffer,
                button("Commit offer", winit::keyboard::Key::Character("r".into())),
            ));
        }
        panels.push((
            PanelKey::KeepLine,
            button(
                "Keep the line",
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ),
        ));
    }
    if state.question_open {
        let reading_root: fn(&mesocosm_views::Succession) -> mesocosm_views::SuccessionChild =
            if state.host.continuation_shown() {
                super::super::continuation::root
            } else if state.host.directing_open() {
                crate::directing::root
            } else {
                mesocosm_views::succession_root
            };
        panels.push((
            PanelKey::Question,
            Box::new(lens(
                move |v: &mut mesocosm_views::Succession| reading_root(v),
                |s: &mut State| &mut s.question,
            )),
        ));
        let choosing = state.host.continuation_shown() || state.host.directing_open();
        if choosing {
            panels.push((
                PanelKey::NextChoice,
                button(
                    "Next choice",
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown),
                ),
            ));
        }
        panels.push((
            PanelKey::Answer,
            button(
                if choosing {
                    "Use selected choice"
                } else {
                    "Carry on"
                },
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ),
        ));
        if state.host.directing_open() {
            panels.push((
                PanelKey::CloseHints,
                button(
                    "Close hints",
                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                ),
            ));
        }
        if state.question.take.is_some() {
            panels.push((
                PanelKey::TakeBody,
                button(
                    "Take offered body",
                    winit::keyboard::Key::Character("t".into()),
                ),
            ));
        }
    }
    if state.host.config.dev && state.host.inspection.open {
        panels.push(inspector(state));
    }
    panels.append(&mut readings);
    if state.host.config.dev && !state.host.inspection.open {
        panels.push(inspector(state));
    }
    Box::new(
        el(
            "main",
            vec![
                Box::new(el(
                    "h1",
                    text(if state.host.config.bench_world {
                        "Mesocosm · Bench world"
                    } else {
                        "Mesocosm"
                    }),
                )) as Child,
                Box::new(
                    el(
                        "div",
                        text(format!(
                            "tick {} · E / Space attend · S speciate · Z / V turn",
                            state.tick_reading
                        )),
                    )
                    .attr("class", "status"),
                ),
                Box::new(el("div", vec![viewport, aside(panels)]).attr("class", "play")),
                Box::new(
                    el(
                        "div",
                        text(
                            state
                                .notice
                                .clone()
                                .or_else(|| state.error.clone())
                                .unwrap_or_default(),
                        ),
                    )
                    .attr("id", "viewport-error"),
                ),
            ],
        )
        .attr("class", "mesocosm"),
    )
}

pub(super) fn sheet() -> String {
    let mut css = String::from(
        "body{margin:0;background:#17211d;color:#e6eee8;font:14px sans-serif} .mesocosm{padding:12px} h1{font-size:18px;margin:0 0 8px} .play{display:flex;gap:12px;align-items:flex-start} aside{width:250px;max-height:calc(100vh - 110px);overflow:auto} .viewport{display:block;background:#050a07} button{padding:8px;margin:4px;color:#e6eee8;background:#304d3a;border:1px solid #668875} .status{margin:8px 0} #viewport-error{color:#ffb080} .board{max-width:250px}",
    );
    css.push_str(mesocosm_views::vitals_css());
    css.push_str(mesocosm_views::board_css());
    css.push_str(mesocosm_views::succession_css());
    css.push_str(mesocosm_views::dev_css());
    css.push_str(".mesocosm{background:#17211d;color:#e6eee8;font:14px sans-serif;min-height:100vh} .play aside{flex-shrink:0;min-width:0;max-width:250px;max-height:calc(100vh - 86px);overflow:auto} .aside-panel{display:contents} aside .board,aside .vitals,aside .dev,aside .checkpoint{box-sizing:border-box;width:250px;max-width:250px} aside .detail-row{white-space:normal;overflow-wrap:anywhere} aside .vitals .detail-value{display:block;margin-left:0}");
    css
}
