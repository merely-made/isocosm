// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The visible-body roster's second arrangement pass: `branching.toml`.
//!
//! Explicit branching layouts, admitted under the base palette. The bodies
//! and the reasons for them are in the sheet.

use super::*;

bodies!("base":
    /// A raised shrub with two lateral, leaf-bearing branches and a crown.
    producer_shrub => "branching.producer_shrub",
    /// A browsing consumer with a raised head, a compact chest and a shorter tail.
    consumer_browser => "branching.consumer_browser",
    /// A low armoured cropper whose legs branch down from the carapace.
    consumer_armoured => "branching.consumer_armoured",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::mesocosm::axis::Soma;
    use crate::legacy::mesocosm::body::SpeciesId;
    use crate::legacy::mesocosm::development::{develop_body, minimum_body_mass_mg};
    use crate::legacy::mesocosm::organism::Kingdom;
    use crate::process::FeedingMode;

    fn grown(recipe: &Recipe, seed: u64) -> crate::legacy::mesocosm::phenotype::BodyPhenotype {
        let soma = Soma::develop(recipe, seed);
        let floor = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap());
        let mass = floor + 10_000;
        let body = develop_body(SpeciesId(42), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| body.mass_mg(part.id) > 0));
        crate::legacy::mesocosm::phenotype::BodyPhenotype::seed(body)
    }

    #[test]
    fn branching_recipes_keep_their_ecological_readings_and_conserve_mass() {
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
            let recipe = recipe();
            for seed in 0..128 {
                let phenotype = grown(&recipe, seed);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }

    #[test]
    fn consumer_variation_is_confined_to_bare_neck_and_tail_regions() {
        for recipe in [consumer_browser(), consumer_armoured()] {
            assert_eq!(recipe.variance, 0);
            for seed in 0..128 {
                let soma = Soma::develop(&recipe, seed);
                for (tagma, stretch) in recipe.layout.iter().enumerate() {
                    let authored = recipe.tagmata[tagma].segments;
                    match stretch.variance {
                        Some(1) => assert!(
                            (authored - 1..=authored + 1).contains(&soma.segments[tagma]),
                            "seed {seed}, tagma {tagma}"
                        ),
                        None => {
                            assert_eq!(soma.segments[tagma], authored, "seed {seed}, tagma {tagma}")
                        },
                        Some(other) => panic!("unexpected regional variance {other}"),
                    }
                }
            }
        }
    }

    #[test]
    fn target_families_keep_mass_segments_at_distinct_non_overlapping_pivots() {
        for (name, recipe) in [
            ("shrub", producer_shrub()),
            ("browser", consumer_browser()),
            ("armoured", consumer_armoured()),
        ] {
            for seed in 0..128 {
                let body = grown(&recipe, seed).body().clone();
                let segments: Vec<_> = body
                    .living()
                    .filter(|part| {
                        crate::legacy::mesocosm::plan::classify(part.half_extent)
                            == crate::legacy::mesocosm::plan::Role::Mass
                    })
                    .collect();
                for (index, left) in segments.iter().enumerate() {
                    let left_at = body.world_pivot(left.id).unwrap();
                    for right in &segments[index + 1..] {
                        let right_at = body.world_pivot(right.id).unwrap();
                        assert_ne!(
                            left_at, right_at,
                            "{name}, seed {seed}: coincident segments"
                        );
                        let overlap = (0..3).all(|axis| {
                            left_at[axis] - left.half_extent[axis].abs()
                                < right_at[axis] + right.half_extent[axis].abs()
                                && right_at[axis] - right.half_extent[axis].abs()
                                    < left_at[axis] + left.half_extent[axis].abs()
                        });
                        assert!(
                            !overlap,
                            "{name}, seed {seed}: mass segments {:?} and {:?} overlap",
                            left.id, right.id
                        );
                    }
                }
            }
        }
    }
}
