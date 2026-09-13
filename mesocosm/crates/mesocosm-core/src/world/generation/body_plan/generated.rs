// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded draws from the existing stretch and appendage grammar. Parentage,
//! lengths, sockets and admitted shapes vary independently of the habitat.
//! Declared sockets are distinct; anchors on very short stretches may still
//! coincide. This is not a whole-body self-intersection solver or a claim of
//! flight/swimming capability.

use crate::axis::{Anchor, Appendage, AppendageStep, ChainFacing, Stretch, Tagma};
use crate::plan::{Facing, Role};
use crate::{Kingdom, PartPalette, Recipe, Rng};

pub(super) fn draw(rng: &mut Rng, role: Kingdom, palette: PartPalette) -> Recipe {
    let count = 3 + rng.below(6) as usize;
    let mut tagmata = Vec::with_capacity(count);
    let mut layout = Vec::with_capacity(count);
    let mut chains = Vec::with_capacity(count);
    for index in 0..count {
        let appendage = match (role, index) {
            (Kingdom::Consumer, 0) => Appendage::Mouth,
            (Kingdom::Producer, i) if i == count - 1 => Appendage::Plate,
            (Kingdom::Producer, _) => match rng.below(4) {
                0 => Appendage::Plate,
                _ => Appendage::None,
            },
            (_, 0) => Appendage::None,
            _ => match rng.below(6) {
                0 | 1 => Appendage::Limb,
                2 => Appendage::Vane,
                3 => Appendage::Feeler,
                _ => Appendage::None,
            },
        };
        let segment_shape = shape(rng, palette, Role::Mass);
        let appendage_shape = appendage
            .role(0)
            .map(|r| shape(rng, palette, r))
            .unwrap_or(0);
        let tagma = Tagma::new(1 + rng.below(4) as u8, appendage)
            .with_shapes(segment_shape, appendage_shape);
        chains.push(chain(rng, tagma, palette));
        tagmata.push(tagma);
        layout.push(if index == 0 {
            Stretch {
                parent: None,
                anchor: Anchor::Tip,
                facing: if role == Kingdom::Producer {
                    Facing::Above
                } else {
                    Facing::Back
                },
                variance: Some(0),
            }
        } else {
            socket(rng, &layout, role)
        });
    }
    let mut recipe = Recipe::of(tagmata)
        .with_layout(layout)
        .with_appendage_chains(chains);
    // Segment count is already drawn. Keep the admitted tree's lengths exact
    // instead of adding a second independent length perturbation in Soma.
    recipe.variance = 0;
    recipe
}

fn shape(rng: &mut Rng, palette: PartPalette, role: Role) -> u8 {
    let bank = palette.shapes(role);
    let mut admitted = vec![0];
    admitted.extend(
        bank.extra
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| slot.map(|_| i as u8 + 1)),
    );
    admitted[rng.below(admitted.len() as u64) as usize]
}

fn socket(rng: &mut Rng, layout: &[Stretch], role: Kingdom) -> Stretch {
    let directions: &[Facing] = if role == Kingdom::Producer {
        &[
            Facing::Above,
            Facing::Left,
            Facing::Right,
            Facing::Front,
            Facing::Back,
        ]
    } else {
        &[
            Facing::Above,
            Facing::Below,
            Facing::Left,
            Facing::Right,
            Facing::Front,
            Facing::Back,
        ]
    };
    let mut choices = Vec::new();
    for parent in 0..layout.len() {
        for anchor in [Anchor::Base, Anchor::Middle, Anchor::Tip] {
            for &facing in directions {
                if !layout.iter().any(|s| {
                    s.parent == Some(parent as u8) && s.anchor == anchor && s.facing == facing
                }) {
                    choices.push(Stretch {
                        parent: Some(parent as u8),
                        anchor,
                        facing,
                        variance: Some(0),
                    });
                }
            }
        }
    }
    choices[rng.below(choices.len() as u64) as usize]
}

fn chain(rng: &mut Rng, tagma: Tagma, palette: PartPalette) -> Vec<AppendageStep> {
    if !matches!(
        tagma.appendage,
        Appendage::Limb | Appendage::Vane | Appendage::Plate
    ) || rng.below(2) == 0
    {
        return Vec::new();
    }
    let leaf = tagma.appendage == Appendage::Plate;
    let role = tagma
        .appendage
        .role(tagma.appendage_shape)
        .expect("eligible appendage");
    let first_role = if leaf { Role::Mass } else { role };
    vec![
        AppendageStep {
            role: first_role,
            shape: shape(rng, palette, first_role),
            facing: if leaf {
                ChainFacing::Above
            } else {
                ChainFacing::Outward
            },
            distal: false,
        },
        AppendageStep {
            role,
            shape: tagma.appendage.shape_index(tagma.appendage_shape),
            facing: if leaf {
                ChainFacing::Above
            } else if tagma.appendage == Appendage::Vane {
                ChainFacing::Outward
            } else {
                ChainFacing::Below
            },
            distal: !leaf,
        },
    ]
}

#[cfg(test)]
mod tests;
