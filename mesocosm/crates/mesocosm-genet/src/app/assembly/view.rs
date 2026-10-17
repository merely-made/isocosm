// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{LEAF, MAP, State};
use cambium::{AnyView, GenetCtx, GenetElement, clickable, el, focusable, lens, text};

pub(super) type Child = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&State) -> Child;

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
    let mut readings: Vec<Child> = vec![
        Box::new(
            cambium::custom_leaf::<State, ()>(MAP, 160, 160)
                .attr("role", "img")
                .attr("aria-label", "World map"),
        ),
        Box::new(lens(
            |v: &mut mesocosm_views::Vitals| mesocosm_views::vitals_root(v),
            |s: &mut State| &mut s.vitals,
        )),
    ];
    let mut panels: Vec<Child> = Vec::new();
    if state.host.runtime.review().is_some() {
        panels.push(Box::new(lens(
            |v: &mut mesocosm_views::Board| mesocosm_views::board_root(v),
            |s: &mut State| &mut s.board,
        )));
        panels.push(button(
            "Next offer",
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
        ));
        if state.board.commit.is_some() {
            panels.push(button(
                "Commit offer",
                winit::keyboard::Key::Character("r".into()),
            ));
        }
        panels.push(button(
            "Keep the line",
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
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
        panels.push(Box::new(lens(
            move |v: &mut mesocosm_views::Succession| reading_root(v),
            |s: &mut State| &mut s.question,
        )));
        let choosing = state.host.continuation_shown() || state.host.directing_open();
        if choosing {
            panels.push(button(
                "Next choice",
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown),
            ));
        }
        panels.push(button(
            if choosing {
                "Use selected choice"
            } else {
                "Carry on"
            },
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
        ));
        if state.host.directing_open() {
            panels.push(button(
                "Close hints",
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
            ));
        }
        if state.question.take.is_some() {
            panels.push(button(
                "Take offered body",
                winit::keyboard::Key::Character("t".into()),
            ));
        }
    }
    if state.host.config.dev && state.host.inspection.open {
        panels.push(Box::new(lens(
            |v: &mut mesocosm_views::Dev| mesocosm_views::dev_root(v),
            |s: &mut State| &mut s.dev,
        )));
    }
    panels.append(&mut readings);
    if state.host.config.dev && !state.host.inspection.open {
        panels.push(Box::new(lens(
            |v: &mut mesocosm_views::Dev| mesocosm_views::dev_root(v),
            |s: &mut State| &mut s.dev,
        )));
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
                            state.host.runtime.tick()
                        )),
                    )
                    .attr("class", "status"),
                ),
                Box::new(
                    el(
                        "div",
                        vec![viewport, Box::new(el("aside", panels)) as Child],
                    )
                    .attr("class", "play"),
                ),
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
        "body{margin:0;background:#17211d;color:#e6eee8;font:14px sans-serif} .mesocosm{padding:12px} h1{font-size:18px;margin:0 0 8px} .play{display:flex;gap:12px;align-items:flex-start} aside{width:250px;overflow:auto} .viewport{display:block;background:#050a07} button{padding:8px;margin:4px;color:#e6eee8;background:#304d3a;border:1px solid #668875} .status{margin:8px 0} #viewport-error{color:#ffb080} .board{max-width:250px}",
    );
    css.push_str(mesocosm_views::vitals_css());
    css.push_str(mesocosm_views::board_css());
    css.push_str(mesocosm_views::succession_css());
    css.push_str(mesocosm_views::dev_css());
    css.push_str(".mesocosm{background:#17211d;color:#e6eee8;font:14px sans-serif;min-height:100vh} .play aside{flex-shrink:0;min-width:0;max-width:250px;max-height:calc(100vh - 86px);overflow:auto} aside .board,aside .vitals,aside .dev,aside .checkpoint{box-sizing:border-box;width:250px;max-width:250px} aside .detail-row{white-space:normal;overflow-wrap:anywhere} aside .vitals .detail-value{display:block;margin-left:0}");
    css
}
