// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{LEAF_KEY, state::Bench, view::Child};
use cambium::{clickable, custom_leaf, el, focusable, text};

pub(super) fn strip(state: &Bench) -> Child {
    let model = state.model.borrow();
    let Some(comparison) = &model.comparison else {
        return Box::new(el("div", ()));
    };
    let cards: Vec<Child> = comparison
        .cards
        .iter()
        .enumerate()
        .map(|(index, card)| {
            let label = if index == 0 {
                "Original".into()
            } else {
                format!("Alternative {index}")
            };
            let preview: Child = if state.visible && card.world.is_some() {
                Box::new(
                    custom_leaf::<Bench, ()>(LEAF_KEY + 1 + index as u64, 220, 110)
                        .attr("class", "proportion-preview")
                        .attr("role", "img")
                        .attr("aria-label", format!("{label} preview")),
                )
            } else {
                Box::new(
                    el(
                        "div",
                        text(if card.world.is_none() {
                            "Refused"
                        } else {
                            "Hidden"
                        }),
                    )
                    .attr("class", "proportion-preview"),
                )
            };
            Box::new(
                el(
                    "div",
                    (
                        preview,
                        focusable(clickable(
                            el("button", text(label.clone()))
                                .attr("aria-label", label)
                                .attr("aria-pressed", (comparison.selected == index).to_string())
                                .attr("aria-disabled", card.world.is_none().to_string()),
                            move |state: &mut Bench, _| state.choose_proportion(index),
                        )),
                        el("p", text(card.counts())),
                        el("p", text(card.description.clone())).attr("class", "change-summary"),
                    ),
                )
                .attr(
                    "class",
                    if comparison.selected == index {
                        "proportion chosen"
                    } else {
                        "proportion"
                    },
                ),
            ) as Child
        })
        .collect();
    Box::new(
        el(
            "section",
            (
                el(
                    "div",
                    (
                        el(
                            "p",
                            text("Proportions · topology, segment count, appendages and mass held"),
                        ),
                        focusable(clickable(
                            el("button", text("More proportions"))
                                .attr("aria-label", "More proportions"),
                            |s: &mut Bench, _| s.more_proportions(),
                        )),
                        focusable(clickable(
                            el(
                                "button",
                                text(if comparison.shared_scale {
                                    "Shared scale"
                                } else {
                                    "Fit each"
                                }),
                            )
                            .attr("aria-label", "Shared scale / fit each"),
                            |s: &mut Bench, _| s.shared_scale(),
                        )),
                        focusable(clickable(
                            el("button", text("Save comparison"))
                                .attr("aria-label", "Save comparison"),
                            |s: &mut Bench, _| s.save_comparison(),
                        )),
                    ),
                )
                .attr("class", "comparison-toolbar"),
                el("div", cards).attr("class", "comparison-cards"),
            ),
        )
        .attr("class", "comparison"),
    )
}
