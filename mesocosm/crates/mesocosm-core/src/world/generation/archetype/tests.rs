// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{BodyPhenotype, Soma, SpeciesId};

#[test]
fn starts_replay_vary_and_develop_with_their_declared_ecology() {
    let palette = crate::axis::archetype::spaced::palette();
    for archetype in Archetype::ALL {
        let mut recipes = std::collections::BTreeSet::new();
        for seed in 0..16 {
            let recipe = archetype.generate(&mut Rng::from_seed(seed));
            assert_eq!(recipe, archetype.generate(&mut Rng::from_seed(seed)));
            recipes.insert(serde_json::to_string(&recipe).unwrap());
            let soma = Soma::develop(&recipe, seed);
            assert!(
                soma.total_segments() <= 32,
                "{archetype:?}: default segment admission"
            );
            let body =
                crate::development::develop_body(SpeciesId(2), &recipe, &soma, 100_000, palette)
                    .unwrap_or_else(|error| panic!("{archetype:?} seed {seed}: {error:?}"));
            assert!(body.living().count() <= 128);
            let phenotype = BodyPhenotype::seed(body);
            assert_eq!(
                Kingdom::of(&phenotype),
                archetype.role(),
                "{archetype:?} seed {seed}"
            );
            let wire = serde_json::to_string(&recipe).unwrap();
            assert_eq!(serde_json::from_str::<Recipe>(&wire).unwrap(), recipe);
        }
        assert!(
            recipes.len() > 1,
            "{archetype:?} must actually vary across seeds"
        );
    }
}

#[test]
fn limb_situs_and_branching_distinguish_the_starting_anatomies() {
    for seed in 0..16 {
        for kind in [Archetype::Raccoon, Archetype::Cat, Archetype::Horse] {
            let recipe = kind.generate(&mut Rng::from_seed(seed));
            assert_eq!(
                recipe
                    .tagmata
                    .iter()
                    .filter(|t| t.appendage == Appendage::Limb)
                    .map(|t| t.segments as usize * t.per_segment as usize * 2)
                    .sum::<usize>(),
                4
            );
            assert!(
                recipe
                    .tagmata
                    .iter()
                    .any(|t| t.appendage == Appendage::Feeler)
            );
            assert!(
                recipe
                    .appendage_chains
                    .iter()
                    .filter(|c| !c.is_empty())
                    .all(|c| c.len() == if kind == Archetype::Horse { 4 } else { 3 })
            );
        }
        let bird = Archetype::Bird.generate(&mut Rng::from_seed(seed));
        assert_eq!(
            bird.tagmata
                .iter()
                .filter(|t| t.appendage == Appendage::Limb)
                .count(),
            1
        );
        assert_eq!(
            bird.tagmata
                .iter()
                .filter(|t| t.appendage == Appendage::Vane)
                .count(),
            1
        );
        let fish = Archetype::Fish.generate(&mut Rng::from_seed(seed));
        assert!(!fish.tagmata.iter().any(|t| t.appendage == Appendage::Limb));
        assert!(fish.tagmata.iter().any(|t| t.appendage == Appendage::Vane));
        let grass = Archetype::Grass.generate(&mut Rng::from_seed(seed));
        let tree = Archetype::Tree.generate(&mut Rng::from_seed(seed));
        assert!(tree.tagmata[0].segments > grass.tagmata[0].segments);
        assert!(tree.layout.iter().filter(|s| s.parent == Some(0)).count() >= 4);
    }
}

#[test]
fn all_starts_admit_with_the_bench_palette_and_retain_authored_organs() {
    // The bench content generator preserves these exact palette envelopes; it changes volume references only.
    let palette = crate::axis::archetype::spaced::palette();
    for seed in [1, 7, 42] {
        for archetype in Archetype::ALL {
            let mut request = super::super::Request {
                seed,
                ..Default::default()
            };
            request.criteria.archetype = Some(archetype);
            let prepared = request.prepare(palette).unwrap();
            let draft = prepared.draft();
            assert_eq!(
                draft.candidates.len(),
                4,
                "{archetype:?} seed {seed}: {:?}",
                draft.rejected
            );
            for candidate in &draft.candidates {
                assert_eq!(candidate.role, archetype.role());
                assert!(archetype.accepts(&Soma::develop(&candidate.recipe, candidate.seed)));
            }
            prepared.enter(0).unwrap();
        }
    }
}

#[test]
fn mammal_ears_do_not_bury_the_working_eyes() {
    let palette = crate::axis::archetype::spaced::palette();
    for kind in [Archetype::Raccoon, Archetype::Cat, Archetype::Horse] {
        let recipe = kind.generate(&mut Rng::from_seed(1));
        let soma = Soma::develop(&recipe, 1);
        let body = crate::development::develop_body(SpeciesId(2), &recipe, &soma, 100_000, palette)
            .unwrap();
        let mut centres = vec![[0i32; 3]; body.parts.len()];
        for part in &body.parts {
            if let Some(at) = part.attachment {
                centres[part.id.0 as usize] = std::array::from_fn(|axis| {
                    centres[at.parent.0 as usize][axis] + at.offset[axis]
                });
            }
        }
        for eye in body
            .living()
            .filter(|p| crate::plan::classify(p.half_extent) == Role::Sensor)
        {
            for part in body.living().filter(|p| p.id != eye.id) {
                let overlaps = (0..3).all(|axis| {
                    let distance = (centres[eye.id.0 as usize][axis]
                        - centres[part.id.0 as usize][axis])
                        .abs();
                    // Rendered widths are max(2h, 1). Shared faces are
                    // contact; only positive interior intersection buries.
                    2 * distance
                        < (2 * eye.half_extent[axis].abs()).max(1)
                            + (2 * part.half_extent[axis].abs()).max(1)
                });
                assert!(
                    !overlaps,
                    "{kind:?}: part {:?} buries eye {:?}",
                    part.id, eye.id
                );
            }
        }
    }
}
