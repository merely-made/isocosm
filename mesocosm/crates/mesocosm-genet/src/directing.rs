// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-local suggestion words; choosing sends a hint, never an admitted act.

use cambium::{DetailRow, DetailSection, detail_panel, el, text};
use mesocosm_views::{Succession, SuccessionChild};

pub(crate) fn root(state: &Succession) -> SuccessionChild {
    let rows = state.facts.iter().map(|(key, value)| DetailRow::new(key.clone(), value.clone())).collect();
    let children: Vec<SuccessionChild> = vec![
        Box::new(el::<_, Succession, ()>("div", text(state.headline.clone())).attr("class", "checkpoint-headline")),
        Box::new(detail_panel::<Succession, ()>(&[DetailSection::new("hint", rows)])),
        Box::new(el::<_, Succession, ()>("div", text(format!("[Enter / click panel]  {}", state.stay)))
            .attr("class", "checkpoint-answer")),
    ];
    Box::new(el::<_, Succession, ()>("div", children).attr("class", "checkpoint"))
}
