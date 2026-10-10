// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A roomier visible-body roster, separated feet and a fuller leaf crown:
//! `spaced.toml`.
//!
//! The bodies, the palette and the reasons for them are in the sheet.

use super::*;

/// The jointed palette with its leaf slot widened.
pub fn palette() -> PartPalette {
    datasheet::palette("spaced")
}

bodies!("spaced":
    /// Five broad leaves at distinct, connected branch endpoints.
    producer_shrub => "spaced.producer_shrub",
    /// The branching browser with three fixed, spaced leg situs.
    consumer_browser => "spaced.consumer_browser",
    /// The jointed armoured grazer, unchanged.
    consumer_armoured => "jointed.consumer_armoured",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::mesocosm::axis::Soma;
    use crate::legacy::mesocosm::body::{BodyDocument, Part, SpeciesId, VolumeRef};
    use crate::legacy::mesocosm::development::{develop_body, minimum_body_mass_mg};
    use crate::legacy::mesocosm::organism::Kingdom;
    use crate::process::FeedingMode;

    // The foot's and the broad leaf's tags in the jointed and spaced sheets.
    const FOOT_VOLUME_TAG: u8 = 14;
    const LEAF_VOLUME_TAG: u8 = 25;

    fn body(recipe: &Recipe, seed: u64) -> BodyDocument {
        let soma = Soma::develop(recipe, seed);
        let mass = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap()) + 10_000;
        let body = develop_body(SpeciesId(91), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| body.mass_mg(part.id) > 0));
        body
    }

    fn has_strict_gap(body: &BodyDocument, left: &Part, right: &Part) -> bool {
        let left_at = body.world_pivot(left.id).unwrap();
        let right_at = body.world_pivot(right.id).unwrap();
        (0..3).any(|axis| {
            let left_min = left_at[axis] - left.half_extent[axis].abs();
            let left_max = left_at[axis] + left.half_extent[axis].abs();
            let right_min = right_at[axis] - right.half_extent[axis].abs();
            let right_max = right_at[axis] + right.half_extent[axis].abs();
            left_max < right_min || right_max < left_min
        })
    }

    #[test]
    fn spaced_families_keep_their_readings_and_conserve_mass() {
        for (name, recipe, kingdom, mode) in [
            (
                "shrub",
                producer_shrub as fn() -> Recipe,
                Kingdom::Producer,
                FeedingMode::Producer,
            ),
            (
                "browser",
                consumer_browser,
                Kingdom::Consumer,
                FeedingMode::Grazer,
            ),
            (
                "armoured",
                consumer_armoured,
                Kingdom::Consumer,
                FeedingMode::Grazer,
            ),
        ] {
            for seed in 0..128 {
                let body = body(&recipe(), seed);
                let phenotype = crate::legacy::mesocosm::phenotype::BodyPhenotype::seed(body);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }

    #[test]
    fn broad_leaf_and_same_side_foot_aabbs_stay_disjoint() {
        let browser_recipe = consumer_browser();
        let mut saw_complete_browser = false;
        for seed in 0..128 {
            let shrub = body(&producer_shrub(), seed);
            let leaves: Vec<_> = shrub
                .living()
                .filter(|part| part.volume == VolumeRef::from_tag(LEAF_VOLUME_TAG))
                .collect();
            assert_eq!(leaves.len(), 5, "shrub seed {seed}");
            for (index, left) in leaves.iter().enumerate() {
                for right in &leaves[index + 1..] {
                    assert!(has_strict_gap(&shrub, left, right), "shrub seed {seed}");
                }
            }

            let soma = Soma::develop(&browser_recipe, seed);
            let expected_pairs = [5u8, 7, 9]
                .into_iter()
                .filter(|tagma| !soma.absent.iter().any(|&(missing, _)| missing == *tagma))
                .count();
            saw_complete_browser |= expected_pairs == 3;
            let browser = body(&browser_recipe, seed);
            let mut feet: [Vec<&Part>; 2] = [Vec::new(), Vec::new()];
            for part in browser
                .living()
                .filter(|part| part.volume == VolumeRef::from_tag(FOOT_VOLUME_TAG))
            {
                let side = usize::from(browser.world_pivot(part.id).unwrap()[0] > 0);
                feet[side].push(part);
            }
            for side in feet {
                assert_eq!(side.len(), expected_pairs, "browser seed {seed}");
                for (index, left) in side.iter().enumerate() {
                    for right in &side[index + 1..] {
                        assert!(has_strict_gap(&browser, left, right), "browser seed {seed}");
                    }
                }
            }
        }
        assert!(
            saw_complete_browser,
            "the sample includes a complete six-foot body"
        );
    }
}
