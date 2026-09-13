// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::world::generation::{Archetype, BodyPlan, Request, Selection};
use crate::{Founding, SpeciesId, develop_body, state_hash};
use std::collections::BTreeSet;

fn request(structure: Structure, role: Option<Kingdom>) -> Request {
    let mut request = Request {
        seed: 7,
        candidates: 1,
        ..Request::default()
    };
    request.criteria.body_plan = BodyPlan::Generated;
    request.criteria.structure = Some(structure);
    request.criteria.role = role;
    request
}

#[test]
fn every_feasible_layout_organ_role_founds_and_replays_paid_anatomy() {
    let palette = Founding::SpacedRoster.palette();
    for layout in StructureLayout::ALL {
        for organs in StructureOrgans::ALL {
            for role in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
                let structure = Structure {
                    layout,
                    organs,
                    branch_count: 6,
                    ..Structure::default()
                };
                if structure
                    .required_role()
                    .is_some_and(|required| required != role)
                {
                    continue;
                }
                let mut request = request(structure, Some(role));
                request.seed = 1;
                let prepared = request.prepare(palette).unwrap();
                let draft = prepared.draft();
                assert_eq!(
                    draft.candidates.len(),
                    1,
                    "{layout:?}/{organs:?}/{role:?}: {:?}",
                    draft.rejected
                );
                let candidate = &draft.candidates[0];
                assert!(structure.accepts(&Soma::develop(&candidate.recipe, candidate.seed)));
                assert_eq!(candidate.body.total_mass_mg(), request.criteria.mass_mg);
                assert!(candidate.body.living().all(|part| part.mass_mg > 0));
                let world = prepared.enter(0).unwrap();
                assert_eq!(
                    world.total_matter_mg(),
                    prepared.habitat_world().total_matter_mg()
                );
                assert_eq!(world.controlled().unwrap().kingdom(), role);
                let saved = serde_json::to_string(&Selection {
                    request,
                    candidate: 0,
                })
                .unwrap();
                let selection: Selection = serde_json::from_str(&saved).unwrap();
                assert_eq!(
                    state_hash(&world),
                    state_hash(&selection.enter(palette).unwrap())
                );
            }
        }
    }
}

fn signature(structure: Structure) -> String {
    let palette = crate::axis::archetype::spaced::palette();
    let recipe = structure.generate(&mut Rng::from_seed(7), Kingdom::Producer, palette);
    let soma = Soma {
        segments: recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
    };
    let body = develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
    // Realized graph offsets and envelopes, not merely declared socket names.
    format!(
        "{:?}",
        body.parts
            .iter()
            .map(|p| (p.attachment.map(|a| (a.parent, a.offset)), p.half_extent))
            .collect::<Vec<_>>()
    )
}

#[test]
fn layouts_and_organ_chains_have_distinct_realized_geometry() {
    let layouts: BTreeSet<_> = StructureLayout::ALL
        .into_iter()
        .map(|layout| {
            signature(Structure {
                layout,
                organs: StructureOrgans::Bare,
                ..Structure::default()
            })
        })
        .collect();
    assert_eq!(layouts.len(), StructureLayout::ALL.len());
    let organs: BTreeSet<_> = StructureOrgans::ALL
        .into_iter()
        .map(|organs| {
            signature(Structure {
                organs,
                ..Structure::default()
            })
        })
        .collect();
    assert_eq!(organs.len(), StructureOrgans::ALL.len());
}

#[test]
fn boundary_counts_and_sparse_palettes_remain_bounded_and_seeded() {
    let mut palette = PartPalette::primitive();
    palette.mass.extra[2] = Some(palette.mass.default);
    for branch_count in [2, 6] {
        for segment_length in [1, 4] {
            for layout in StructureLayout::ALL {
                for organs in StructureOrgans::ALL {
                    let structure = Structure {
                        layout,
                        organs,
                        branch_count,
                        segment_length,
                    };
                    for seed in [1, 7, 42] {
                        let recipe = structure.generate(
                            &mut Rng::from_seed(seed),
                            Kingdom::Producer,
                            palette,
                        );
                        assert_eq!(
                            recipe,
                            structure.generate(
                                &mut Rng::from_seed(seed),
                                Kingdom::Producer,
                                palette
                            )
                        );
                        for tagma in &recipe.tagmata {
                            assert!(matches!(tagma.segment_shape, 0 | 3));
                            assert!((1..=segment_length).contains(&tagma.segments));
                        }
                        let soma = Soma::develop(&recipe, seed);
                        let body =
                            develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
                        assert!(body.parts.len() <= 256);
                        assert!(recipe.segments() <= 32);
                    }
                }
            }
        }
    }
}

#[test]
fn conflicts_are_explicit_and_absent_structure_keeps_old_requests() {
    let structure = Structure {
        organs: StructureOrgans::Leaves,
        ..Structure::default()
    };
    let implicit = request(structure, None);
    assert_eq!(
        implicit
            .prepare(Founding::Drawn.palette())
            .unwrap()
            .draft()
            .candidates
            .len(),
        1
    );
    assert!(
        implicit
            .prepare(Founding::Drawn.palette())
            .unwrap()
            .draft()
            .candidates
            .iter()
            .all(|c| c.role == Kingdom::Producer)
    );
    assert!(
        request(structure, Some(Kingdom::Consumer))
            .validate()
            .is_err()
    );
    for (branch_count, segment_length) in [(1, 1), (7, 1), (2, 0), (2, 5)] {
        assert!(
            request(
                Structure {
                    branch_count,
                    segment_length,
                    ..structure
                },
                None
            )
            .validate()
            .is_err()
        );
    }
    let mut bad = implicit;
    bad.criteria.body_plan = BodyPlan::Axial;
    assert!(bad.validate().is_err());
    bad.criteria.body_plan = BodyPlan::Generated;
    bad.criteria.archetype = Some(Archetype::Tree);
    assert!(bad.validate().is_err());
    let old = Request::default();
    let mut json = serde_json::to_value(&old).unwrap();
    json["criteria"]
        .as_object_mut()
        .unwrap()
        .remove("structure");
    let decoded: Request = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, old);
    let palette = Founding::Drawn.palette();
    assert_eq!(
        state_hash(&old.prepare(palette).unwrap().enter(0).unwrap()),
        state_hash(&decoded.prepare(palette).unwrap().enter(0).unwrap())
    );
}

#[test]
fn minimal_stretches_do_not_converge_and_fin_pairs_remain_separate() {
    let palette = PartPalette::primitive();
    for layout in StructureLayout::ALL {
        let structure = Structure {
            layout,
            organs: StructureOrgans::Bare,
            branch_count: 6,
            segment_length: 1,
        };
        let recipe = structure.generate(&mut Rng::from_seed(1), Kingdom::Decomposer, palette);
        let soma = Soma::develop(&recipe, 1);
        let body = develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
        let positions = centres(&body);
        let unique: BTreeSet<_> = positions.iter().copied().collect();
        assert_eq!(
            unique.len(),
            positions.len(),
            "{layout:?} has convergent structural parts"
        );
    }
    let structure = Structure {
        organs: StructureOrgans::Fins,
        branch_count: 2,
        segment_length: 1,
        ..Structure::default()
    };
    let recipe = structure.generate(&mut Rng::from_seed(1), Kingdom::Decomposer, palette);
    let soma = Soma {
        segments: recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
    };
    let body = develop_body(SpeciesId(1), &recipe, &soma, 800, palette).unwrap();
    let positions = centres(&body);
    let mut paired_parents = 0;
    for parent in &body.parts {
        let links: Vec<_> = body
            .parts
            .iter()
            .filter(|p| {
                p.attachment.is_some_and(|a| a.parent == parent.id)
                    && crate::plan::classify(p.half_extent) == crate::plan::Role::Limb
            })
            .collect();
        if links.len() == 2 {
            paired_parents += 1;
            assert_ne!(
                positions[links[0].id.0 as usize],
                positions[links[1].id.0 as usize]
            );
        }
    }
    assert_eq!(paired_parents, usize::from(structure.branch_count));
}

fn centres(body: &crate::BodyDocument) -> Vec<[i32; 3]> {
    let mut positions = vec![[0; 3]; body.parts.len()];
    for part in &body.parts {
        if let Some(at) = part.attachment {
            positions[part.id.0 as usize] =
                std::array::from_fn(|axis| positions[at.parent.0 as usize][axis] + at.offset[axis]);
        }
    }
    positions
}
