// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::native;
use super::*;

#[test]
fn every_founding_palette_resolves_and_the_default_names_one() {
    let sheets = sheets();
    for (name, founding) in &sheets.foundings.founding {
        let banks = sheets.banks(&founding.palette).expect(name);
        assert!(
            Bank::ALL.iter().all(|b| !banks.bank(*b).is_empty()),
            "{name}"
        );
    }
    assert!(sheets.founding(native::default_founding()).is_ok());
}

#[test]
fn the_default_roster_lowers_onto_its_own_kinds() {
    let founding = native::default_founding();
    let palette = &sheets().founding(founding).unwrap().palette;
    let kinds = native::kinds(palette).unwrap();
    let roster = native::founding(founding).unwrap();
    assert_eq!(roster.len(), 8, "Mesocosm's eight authored bodies (512)");
    assert_eq!(roster.iter().filter(|r| r.1).count(), 3, "three producers");
    for (name, _, recipe) in &roster {
        assert!(
            recipe.kinds().iter().all(|k| kinds.contains_key(k)),
            "{name}"
        );
    }
}
