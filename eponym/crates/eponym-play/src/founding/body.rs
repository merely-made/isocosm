// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The compact six-part native recipe selected in rulings 816 and 818.

use isocosm::rules::{Anchor, Development, Facing, Policy, Recipe, Rules, Tagma, Template};
use std::collections::BTreeMap;

const ROOT: &str = "kind:eponym-root";
const LIMB: &str = "kind:eponym-limb";
const SENSOR: &str = "kind:eponym-sensor";

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells.iter().map(|(name, cells)| (format!("function:{name}"), *cells)).collect(),
        shape: String::new(),
    }
}

pub(super) fn install(rules: &mut Rules) -> Development {
    rules.functions.extend(isocosm::rules::default_functions());
    rules.shapes.extend(isocosm::rules::default_shapes());
    rules.kinds.extend(BTreeMap::from([
        (ROOT.into(), template([4, 4, 4], &[("intake", 22), ("store", 5)])),
        (LIMB.into(), template([4, 1, 1], &[("contract", 1), ("support", 1), ("grip", 1)])),
        (SENSOR.into(), template([1, 1, 1], &[("sense", 1)])),
    ]));
    let recipe = Recipe {
        tagmata: vec![
            Tagma {
                segments: 1, segment: ROOT.into(), bears: Some(LIMB.into()), per_segment: 2,
                parent: None, anchor: Anchor::Base, facing: Facing::Back, socket: Facing::Right,
                variance: None,
            },
            Tagma {
                segments: 1, segment: SENSOR.into(), bears: None, per_segment: 0,
                parent: Some(0), anchor: Anchor::Base, facing: Facing::Above, socket: Facing::Right,
                variance: None,
            },
        ],
        variance: 0, absence: [0, 1], riff: [0, 1], vary: [0, 1],
    };
    Development {
        lexicon: recipe.kinds(), recipe, policy: Policy::default(), domain: 0, clutch: 1,
        anamorphic: false, tracts: vec![], systems: Default::default(),
    }
}
