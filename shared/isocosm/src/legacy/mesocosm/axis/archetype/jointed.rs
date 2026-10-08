// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The third visible-body roster, separated leaves and jointed limbs:
//! `jointed.toml`.
//!
//! The bodies, the palette and the reasons for them are in the sheet.

use super::*;

/// The base palette plus two directional limb links and its own leaf.
pub fn palette() -> PartPalette {
    datasheet::palette("jointed")
}

bodies!("jointed":
    /// A branched producer with each leaf held on a paid stalk.
    producer_shrub => "jointed.producer_shrub",
    /// A grazer whose three pairs of legs each have upper, lower and foot links.
    consumer_browser => "jointed.consumer_browser",
    /// A low armoured grazer whose legs branch from either end of the shell run.
    consumer_armoured => "jointed.consumer_armoured",
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::mesocosm::axis::Soma;
    use crate::legacy::mesocosm::body::SpeciesId;
    use crate::legacy::mesocosm::development::{develop_body, minimum_body_mass_mg};
    use crate::legacy::mesocosm::organism::Kingdom;
    use crate::legacy::mesocosm::process::FeedingMode;

    fn grown(recipe: &Recipe, seed: u64) -> crate::legacy::mesocosm::phenotype::BodyPhenotype {
        let soma = Soma::develop(recipe, seed);
        let mass = u64::from(minimum_body_mass_mg(recipe, &soma).unwrap()) + 10_000;
        let body = develop_body(SpeciesId(88), recipe, &soma, mass, palette()).unwrap();
        assert_eq!(body.total_mass_mg(), mass);
        assert!(body.living().all(|part| part.mass_mg > 0));
        crate::legacy::mesocosm::phenotype::BodyPhenotype::seed(body)
    }

    #[test]
    fn jointed_families_keep_their_feeding_readings_and_mass() {
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
                let phenotype = grown(&recipe(), seed);
                assert_eq!(Kingdom::of(&phenotype), kingdom, "{name}, seed {seed}");
                assert_eq!(FeedingMode::of(&phenotype), mode, "{name}, seed {seed}");
            }
        }
    }
}
