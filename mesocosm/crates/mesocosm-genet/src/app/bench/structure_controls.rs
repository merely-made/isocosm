// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Independent recipe choices, saved in the ordinary generation request.
use super::{generation_controls::choice, state::Bench, view::Child};
use cambium::{el, text};
use mesocosm_core::world::generation::{BodyPlan, Structure, StructureLayout, StructureOrgans};

impl Bench {
    fn structure(&mut self, change: impl FnOnce(&mut Structure)) {
        self.commit_generation(|request| {
            request.criteria.body_plan = BodyPlan::Generated;
            request.criteria.archetype = None;
            change(
                request
                    .criteria
                    .structure
                    .get_or_insert_with(Structure::default),
            );
            if request
                .criteria
                .structure
                .as_ref()
                .is_some_and(|s| s.organs == StructureOrgans::Leaves)
            {
                request.criteria.role = Some(mesocosm_core::Kingdom::Producer);
            }
            request.criteria.movement_organs = None;
            request.variation = 0;
        });
    }
}

pub(super) fn view(state: &Bench) -> Child {
    let model = state.model.borrow();
    let Some(structure) = model.creator.request.criteria.structure.as_ref() else {
        return choice("Compose anatomy", false, |s| s.structure(|_| {}));
    };
    let layouts: Vec<Child> = StructureLayout::ALL
        .into_iter()
        .map(|layout| {
            choice(
                &format!("Layout: {}", layout.label()),
                structure.layout == layout,
                move |s| s.structure(|v| v.layout = layout),
            )
        })
        .collect();
    let organs: Vec<Child> = StructureOrgans::ALL
        .into_iter()
        .map(|organs| {
            choice(
                &format!("Organs: {}", organs.label()),
                structure.organs == organs,
                move |s| s.structure(|v| v.organs = organs),
            )
        })
        .collect();
    let counts: Vec<Child> = (2..=6)
        .map(|count| {
            choice(
                &format!("Stretches {count}"),
                structure.branch_count == count,
                move |s| s.structure(|v| v.branch_count = count),
            )
        })
        .collect();
    let lengths: Vec<Child> = (1..=4)
        .map(|length| {
            choice(
                &format!("Length ≤{length}"),
                structure.segment_length == length,
                move |s| s.structure(|v| v.segment_length = length),
            )
        })
        .collect();
    Box::new(el("div", (
        el("div", layouts).attr("class", "toolbar"),
        el("div", organs).attr("class", "toolbar"),
        el("div", (el("div", counts).attr("class", "toolbar"), el("div", lengths).attr("class", "toolbar"))).attr("class", "generation-options"),
        el("p", text("Structural stretches; feeding supports and paired organs add parts. Length bounds segments per stretch. Bare situs retain feeding organs. Radial branches use cardinal directions. Leaves selects Producer.")),
    )).attr("class", "structure-controls"))
}
