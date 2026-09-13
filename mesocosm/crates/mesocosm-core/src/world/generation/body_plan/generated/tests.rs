// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Soma, SpeciesId};
use std::collections::BTreeSet;

#[test]
fn seeded_trees_vary_parentage_and_use_paid_admitted_parts() {
    for role in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
        let palette = crate::axis::archetype::jointed::palette();
        let mut parent_graphs = BTreeSet::new();
        let mut has_chain = false;
        let mut shapes = BTreeSet::new();
        for seed in 0..64 {
            let recipe = draw(&mut Rng::from_seed(seed), role, palette);
            assert_eq!(recipe, draw(&mut Rng::from_seed(seed), role, palette));
            let wire = serde_json::to_string(&recipe).unwrap();
            assert_eq!(recipe, serde_json::from_str::<Recipe>(&wire).unwrap());
            parent_graphs.insert(recipe.layout.iter().map(|s| s.parent).collect::<Vec<_>>());
            for (index, stretch) in recipe.layout.iter().enumerate().skip(1) {
                assert!(usize::from(stretch.parent.unwrap()) < index);
            }
            has_chain |= recipe
                .appendage_chains
                .iter()
                .any(|chain| !chain.is_empty());
            shapes.extend(recipe.tagmata.iter().map(|t| t.segment_shape));
            let soma = Soma::develop(&recipe, seed);
            let body = crate::develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
            assert_eq!(body.total_mass_mg(), 800);
            assert!(body.living().all(|part| part.mass_mg > 0));
        }
        assert!(
            parent_graphs.len() >= 32,
            "{role:?}: only {} parent graphs",
            parent_graphs.len()
        );
        assert!(has_chain);
        assert!(shapes.len() > 1);
    }
}

#[test]
fn a_sparse_palette_never_draws_a_missing_selector() {
    let mut palette = PartPalette::primitive();
    palette.mass.extra[2] = Some(palette.mass.default);
    for seed in 0..32 {
        let recipe = draw(&mut Rng::from_seed(seed), Kingdom::Consumer, palette);
        for tagma in &recipe.tagmata {
            assert!(matches!(tagma.segment_shape, 0 | 3));
            let Some(role) = tagma.appendage.role(tagma.appendage_shape) else {
                continue;
            };
            if role == Role::Mass {
                assert!(matches!(tagma.appendage_shape, 0 | 3));
            } else {
                assert_eq!(tagma.appendage_shape, 0);
            }
        }
        let soma = Soma::develop(&recipe, seed);
        crate::develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
    }
}

#[test]
fn generated_candidates_found_and_replay_through_normal_admission() {
    use crate::world::generation::{BodyPlan, Request};
    for role in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
        let mut request = Request::default();
        request.criteria.body_plan = BodyPlan::Generated;
        request.criteria.role = Some(role);
        let palette = crate::world::Founding::Drawn.palette();
        let prepared = request.prepare(palette).unwrap();
        assert_eq!(
            prepared.draft().candidates.len(),
            4,
            "{role:?}: {:?}",
            prepared.draft().rejected
        );
        let selected = crate::world::generation::Selection {
            request,
            candidate: 0,
        };
        let world = prepared.enter(0).unwrap();
        assert_eq!(world.controlled().unwrap().kingdom(), role);
        assert_eq!(
            crate::state_hash(&world),
            crate::state_hash(&selected.enter(palette).unwrap())
        );
    }
}
