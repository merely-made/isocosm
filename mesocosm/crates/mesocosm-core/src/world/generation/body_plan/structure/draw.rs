// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::axis::{Anchor, Appendage, AppendageStep, ChainFacing, Stretch, Tagma};
use crate::plan::{Facing, Role};

pub(super) fn recipe(
    spec: Structure,
    rng: &mut Rng,
    role: Kingdom,
    palette: PartPalette,
) -> Recipe {
    let root = match role {
        Kingdom::Producer => Appendage::Plate,
        Kingdom::Consumer => Appendage::Mouth,
        Kingdom::Decomposer => Appendage::None,
    };
    let mut tagmata = vec![Tagma::new(1, root).with_shapes(shape(rng, palette, Role::Mass), 0)];
    let mut layout = vec![Stretch {
        parent: None,
        anchor: Anchor::Tip,
        facing: Facing::Back,
        variance: Some(0),
    }];
    let mut chains = vec![vec![]];
    for index in 1..=spec.branch_count {
        let appendage = match spec.organs {
            StructureOrgans::Bare => Appendage::None,
            StructureOrgans::Legs => Appendage::Limb,
            StructureOrgans::Wings | StructureOrgans::Fins => Appendage::Vane,
            StructureOrgans::Leaves => Appendage::Plate,
            StructureOrgans::Feelers => Appendage::Feeler,
        };
        let organ_shape = appendage
            .role(0)
            .map_or(0, |role| shape(rng, palette, role));
        tagmata.push(
            Tagma::new(
                1 + rng.below(u64::from(spec.segment_length)) as u8,
                appendage,
            )
            .with_shapes(shape(rng, palette, Role::Mass), organ_shape),
        );
        layout.push(placement(spec.layout, index));
        chains.push(organ_chain(spec.organs, organ_shape, rng, palette));
    }
    // Producers retain paid leaves above structural tips, so a climbing
    // branch cannot leave its only feeding anatomy buried at the root.
    if role == Kingdom::Producer && spec.organs != StructureOrgans::Leaves {
        for parent in 1..=spec.branch_count {
            tagmata.push(Tagma::new(1, Appendage::Plate).with_shapes(
                shape(rng, palette, Role::Mass),
                shape(rng, palette, Role::Plate),
            ));
            layout.push(Stretch {
                parent: Some(parent),
                anchor: Anchor::Tip,
                facing: Facing::Above,
                variance: Some(0),
            });
            chains.push(vec![]);
        }
    }
    let mut recipe = Recipe::of(tagmata)
        .with_layout(layout)
        .with_appendage_chains(chains);
    recipe.variance = 0;
    recipe
}

fn placement(kind: StructureLayout, index: u8) -> Stretch {
    use Facing::*;
    let (parent, anchor, facing) = match kind {
        StructureLayout::Chain => (index - 1, Anchor::Tip, Back),
        // The feeding root is separate from the hub. A structural Mass below
        // that root would otherwise be read as a mouth by ordinary ecology.
        StructureLayout::Radial if index == 1 => (0, Anchor::Tip, Back),
        StructureLayout::Radial => (
            1,
            Anchor::Tip,
            [Left, Right, Back, Above, Below][usize::from(index - 2)],
        ),
        StructureLayout::Crown if index == 1 => (0, Anchor::Tip, Above),
        StructureLayout::Crown => (
            1,
            Anchor::Tip,
            [Left, Right, Front, Back, Above][usize::from(index - 2)],
        ),
        StructureLayout::Mat => (
            index.saturating_sub(2),
            Anchor::Base,
            if index % 2 == 1 { Left } else { Back },
        ),
        StructureLayout::Vine => (
            index - 1,
            Anchor::Tip,
            [Above, Left, Above, Right, Above, Front][usize::from(index - 1)],
        ),
        StructureLayout::Roots => (
            index.saturating_sub(2),
            Anchor::Tip,
            if index == 1 {
                Left
            } else if index == 2 {
                Right
            } else {
                Below
            },
        ),
    };
    Stretch {
        parent: Some(parent),
        anchor,
        facing,
        variance: Some(0),
    }
}

fn shape(rng: &mut Rng, palette: PartPalette, role: Role) -> u8 {
    let mut slots = vec![0];
    slots.extend(
        palette
            .shapes(role)
            .extra
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.map(|_| i as u8 + 1)),
    );
    slots[rng.below(slots.len() as u64) as usize]
}

fn organ_chain(
    kind: StructureOrgans,
    terminal_shape: u8,
    rng: &mut Rng,
    palette: PartPalette,
) -> Vec<AppendageStep> {
    use ChainFacing::*;
    let (intermediate, terminal, first, second) = match kind {
        StructureOrgans::Bare => return vec![],
        StructureOrgans::Legs => (Role::Limb, Role::Limb, Outward, Below),
        StructureOrgans::Wings => (Role::Limb, Role::Limb, Outward, Outward),
        StructureOrgans::Fins => (Role::Limb, Role::Limb, Outward, Back),
        StructureOrgans::Leaves => (Role::Mass, Role::Plate, Above, Above),
        StructureOrgans::Feelers => (Role::Mass, Role::Sensor, Outward, Above),
    };
    vec![
        AppendageStep {
            role: intermediate,
            shape: shape(rng, palette, intermediate),
            facing: first,
            distal: false,
        },
        AppendageStep {
            role: terminal,
            shape: terminal_shape,
            facing: second,
            distal: false,
        },
    ]
}
