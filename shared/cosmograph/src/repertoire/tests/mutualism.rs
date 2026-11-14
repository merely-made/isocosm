// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire with the mutualism rule (`ecology::mutualism`): the exchange
//! parts lower only where it is set, as organs whose port gives and takes; the
//! parts a box cannot show stay out, and each with its reason.

use super::*;
use crate::{ecology::Mutualism, process::Trade};

fn mutual() -> Founding {
    Founding { mutualism: Some(Mutualism::default()), ..founding(all(), false) }
}

#[test]
fn exchange_parts_lower_only_where_the_rule_is_set() {
    let kinds = |g: &Genesis| g.rules.repertoire.as_ref().unwrap().kinds.keys().cloned().collect::<BTreeSet<_>>();
    let (a, b) = (kinds(&founding(all(), false).generate().unwrap()), kinds(&mutual().generate().unwrap()));
    assert!(a.is_subset(&b) && b.len() > a.len(), "{} without, {} with", a.len(), b.len());
    let wanting: Vec<_> = catalog().parts().iter().filter(|p| needs_mutualism(p)).collect();
    assert_eq!(wanting.len(), 13, "twelve parts wait on the rule alone, and the scarce hypha, which also waits on the nutrient rule, lowers where both are set; three more wait on another mechanic as well");
    let mut lowered = 0;
    for part in wanting {
        assert!(!a.contains(&kind_key(&part.key)), "{}", part.key);
        assert_eq!(lowering_for(part, true, false).unwrap_err(), "it needs the mutualism rule");
        match lowering_in(part, true, false, true) {
            Ok(_) => {
                lowered += 1;
                assert!(b.contains(&kind_key(&part.key)), "{}", part.key);
            },
            Err(why) => println!("not lowered: {} ({why})", part.key),
        }
    }
    assert!(lowered >= 6, "{lowered} lowered");
}

#[test]
fn the_lowered_organs_give_and_take_and_are_not_mouths() {
    let g = mutual().generate().unwrap();
    let port = g.rules.kinds["kind:catalog-mycorrhizal-hypha"].port.unwrap();
    assert!(port.trades(Trade::Gives) && port.trades(Trade::Takes) && !port.is_mouth());
    let plain = founding(all(), false).generate().unwrap();
    assert!(plain.rules.kinds.values().filter_map(|t| t.port).all(|p| p.trade_bits() == 0));
    assert!(g.rules.validate().is_ok());
}
