// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire with the detritus rule (`ecology::detritus`): the parts that
//! deposit or eat dung lower only where it is set, and a dung mouth is a
//! dead-stock port limited to the dung class.

use super::*;
use crate::{ecology::{Categories, Detritus}, process::DeadClass};

fn dung() -> Founding {
    Founding { categories: Some(Categories::default()), detritus: Some(Detritus::default()), ..founding(all(), false) }
}

fn with_categories() -> Founding {
    Founding { categories: Some(Categories::default()), ..founding(all(), false) }
}

#[test]
fn dung_parts_lower_only_where_the_rule_is_set() {
    let kinds = |g: &Genesis| g.rules.repertoire.as_ref().unwrap().kinds.keys().cloned().collect::<BTreeSet<_>>();
    let (a, b) = (kinds(&with_categories().generate().unwrap()), kinds(&dung().generate().unwrap()));
    assert!(a.is_subset(&b) && b.len() > a.len(), "{} without, {} with", a.len(), b.len());
    let wanting: Vec<_> = catalog().parts().iter().filter(|p| needs_detritus(p)).collect();
    assert_eq!(wanting.len(), 14, "fourteen parts wait on the dung stock");
    let mut lowered = 0;
    for part in wanting {
        assert!(!a.contains(&kind_key(&part.key)), "{}", part.key);
        assert_eq!(lowering_for(part, true, false).unwrap_err(), "it needs the detritus rule");
        match lowering_all(part, true, false, false, true) {
            Ok(_) => {
                lowered += 1;
                assert!(b.contains(&kind_key(&part.key)), "{}", part.key);
            },
            Err(why) => println!("not lowered: {} ({why})", part.key),
        }
    }
    println!("{lowered} of the dung parts lowered");
    assert!(lowered >= 4, "{lowered} lowered");
}

#[test]
fn the_dung_mouth_is_limited_to_the_dung_class_and_the_authored_mouths_deposit() {
    let g = dung().generate().unwrap();
    let port = |k: &str| g.rules.kinds[k].port.unwrap();
    let mouth = port("kind:catalog-dung-mouth");
    assert!(mouth.admits_deadstock() && mouth.takes(DeadClass::Dung));
    assert!(!mouth.takes(DeadClass::Litter) && !mouth.takes(DeadClass::Carrion) && !mouth.takes(DeadClass::Wood));
    for authored in ["grinding-molars", "carnassial-jaw", "carrion-mouth", "litter-mouth"] {
        assert!(g.rules.kinds.contains_key(&format!("kind:catalog-{authored}")), "{authored}");
    }
    assert!(g.rules.validate().is_ok());
}
