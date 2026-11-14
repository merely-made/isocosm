// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire with the nutrient rule (`ecology::nutrients`): the parts
//! tuned to the scarce nutrient lower only where it is set, as organs whose
//! port draws and which are not mouths; the parts that make a nutrient out of
//! air stay out, waiting on their mechanic.

use super::*;
use crate::{ecology::{Mutualism, Nutrients}, process::Trade};

fn soil() -> Founding {
    Founding { nutrients: Some(Nutrients::default()), ..founding(all(), false) }
}

#[test]
fn scarce_uptake_parts_lower_only_where_the_rule_is_set() {
    let kinds = |g: &Genesis| g.rules.repertoire.as_ref().unwrap().kinds.keys().cloned().collect::<BTreeSet<_>>();
    let (a, b) = (kinds(&founding(all(), false).generate().unwrap()), kinds(&soil().generate().unwrap()));
    assert!(a.is_subset(&b) && b.len() > a.len(), "{} without, {} with", a.len(), b.len());
    let wanting: Vec<_> = catalog().parts().iter().filter(|p| needs_nutrients(p)).collect();
    assert_eq!(wanting.len(), 4, "four parts wait on the rule alone: two authored, two generated");
    for part in wanting {
        assert!(!a.contains(&kind_key(&part.key)), "{}", part.key);
        // Hypha that hands over what it takes also waits on the mutualism rule, which is named first.
        let mutual = needs_mutualism(part);
        assert_eq!(lowering_for(part, true, false).unwrap_err(), if mutual { "it needs the mutualism rule" } else { "it needs the nutrients rule" });
        assert_eq!(lowering_full(part, true, false, !mutual, (false, true)).is_ok(), !mutual, "{}", part.key);
        let ready = lowering_full(part, true, false, true, (false, true));
        assert!(ready.is_ok(), "{}: {ready:?}", part.key);
    }
    // The nodule parts make nutrient from air, a mechanic the rule does not build: still waiting.
    for key in ["part:nodule", "part:nodule-bacterium"] {
        assert!(!needs_nutrients(catalog().part_by_key(key).unwrap()), "{key}");
        assert!(catalog().part_by_key(key).unwrap().waits_on.contains(&"nutrient-axes".to_string()), "{key}");
    }
}

#[test]
fn the_lowered_organs_draw_and_are_not_mouths() {
    let g = soil().generate().unwrap();
    let port = g.rules.kinds["kind:catalog-nutrient-root-mat"].port.unwrap();
    assert!(port.trades(Trade::Draws) && !port.trades(Trade::Gives) && !port.trades(Trade::Takes) && !port.is_mouth());
    let plain = founding(all(), false).generate().unwrap();
    assert!(plain.rules.kinds.values().filter_map(|t| t.port).all(|p| !p.trades(Trade::Draws)));
    assert!(g.rules.validate().is_ok());
    // With the mutualism rule too the hypha both draws and hands over.
    let both = Founding { mutualism: Some(Mutualism::default()), ..soil() }.generate().unwrap();
    let hypha = both.rules.kinds["kind:catalog-scarce-hypha"].port.unwrap();
    assert!(hypha.trades(Trade::Draws) && hypha.trades(Trade::Gives));
    assert!(!g.rules.kinds.contains_key("kind:catalog-scarce-hypha"), "without mutualism the hypha waits on it");
}
