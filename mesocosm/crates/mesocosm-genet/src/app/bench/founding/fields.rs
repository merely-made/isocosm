// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::app::bench::{Bench, Child, Logic};
use cambium_genet_winit_host::{FocusedTextSlot, Runner};
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

/// Connect native clipboard, selection, caret and IME to the same TextInput
/// that Cambium paints. Both template fields edit the currently selected path.
pub(in crate::app::bench) fn focused(
    runner: &Runner<Bench, Logic, Child>,
) -> Option<FocusedTextSlot<Bench>> {
    runner.state().founding.as_ref()?;
    let node = runner.focus()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    let attr = |id, name: &str| dom.attribute(id, &Namespace::from(""), &LocalName::from(name));
    attr(node, "data-cambium-text-value")?;
    let class = attr(dom.parent(node)?, "class")?;
    macro_rules! field {
        ($key:ident) => {
            FocusedTextSlot {
                node,
                get: Box::new(|b: &Bench| &b.founding.as_ref().unwrap().$key),
                get_mut: Box::new(|b: &mut Bench| &mut b.founding.as_mut().unwrap().$key),
            }
        };
    }
    Some(
        match class
            .split_ascii_whitespace()
            .find(|c| c.starts_with("sim-"))?
        {
            "sim-world-seed" => field!(seed),
            "sim-starting-population" => field!(population),
            "sim-founder-cohort-size" => field!(cohort),
            "sim-minimum-stock" => field!(stock_min),
            "sim-maximum-stock" => field!(stock_max),
            "sim-template-or-save-path" | "sim-body-map-path" => field!(path),
            _ => return None,
        },
    )
}
