// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1173's predicate over hand-made monthly residents.

use super::{
    Outcome,
    recovery::{DIETS, Resolution, Residents, judge, outcome},
};
use std::collections::BTreeMap;

/// Every diet resident at both sites for `months` + 1 samples.
fn whole(months: usize) -> Vec<Residents> {
    let by: BTreeMap<&'static str, u64> = DIETS.iter().map(|&d| (d, 3)).collect();
    vec![BTreeMap::from([(0, by.clone()), (1, by)]); months + 1]
}

fn set(months: &mut [Residents], range: std::ops::Range<usize>, site: u64, diet: &'static str, n: u64) {
    for m in &mut months[range] {
        m.get_mut(&site).unwrap().insert(diet, n);
    }
}

#[test]
fn a_patch_loss_returned_natively_within_six_months_passes() {
    let mut months = whole(24);
    set(&mut months, 5..9, 0, "grazer", 0);
    let (patch, global) = judge(&months);
    assert!(global.is_empty());
    assert_eq!(patch.len(), 1);
    assert_eq!((patch[0].lost_month, patch[0].returned_month, patch[0].resolution), (5, Some(9), Resolution::Returned));
    assert_eq!(outcome(&patch, &global, 24), Outcome::Passed);
}

#[test]
fn a_patch_loss_held_elsewhere_past_six_months_passes() {
    let mut months = whole(24);
    set(&mut months, 3..24 + 1, 1, "predator", 0);
    let (patch, global) = judge(&months);
    assert_eq!(patch[0].resolution, Resolution::HeldElsewhere);
    assert_eq!(outcome(&patch, &global, 24), Outcome::Passed);
}

#[test]
fn a_diet_gone_everywhere_fails() {
    let mut months = whole(24);
    set(&mut months, 10..12, 0, "omnivore", 0);
    set(&mut months, 10..12, 1, "omnivore", 0);
    let (patch, global) = judge(&months);
    assert_eq!(global.iter().map(|g| (g.diet, g.month)).collect::<Vec<_>>(), [("omnivore", 10), ("omnivore", 11)]);
    // A return within six months does not excuse the footprint's loss.
    assert!(patch.iter().all(|l| l.resolution == Resolution::Returned));
    assert_eq!(outcome(&patch, &global, 24), Outcome::Failed);
    assert_eq!(outcome(&patch, &global, 11), Outcome::Failed, "a failure shows before two years");
}

#[test]
fn a_lineage_loss_is_reported_only() {
    // Two lineages share each diet in a web of ten; one of a pair dying out
    // leaves the diet held, so residents by diet never fall to nothing.
    let months = whole(24);
    let (patch, global) = judge(&months);
    assert!(patch.is_empty() && global.is_empty());
    assert_eq!(outcome(&patch, &global, 24), Outcome::Passed);
    let living = [BTreeMap::from([("lineage:3".to_string(), 2u64)]), BTreeMap::from([("lineage:3".to_string(), 0)])];
    let lost = super::recovery::lineage_losses(&living);
    assert_eq!(lost.len(), 1);
    assert_eq!((lost[0].lineage.as_str(), lost[0].month), ("lineage:3", 1));
}
