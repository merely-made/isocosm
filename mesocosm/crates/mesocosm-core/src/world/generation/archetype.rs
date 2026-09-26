// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Editable starting anatomy, lowered through ordinary recipe development.
//! Names describe a structural reference, not a taxonomic simulation rule.

use crate::axis::{Anchor, Appendage, AppendageStep, ChainFacing, Stretch, Tagma};
use crate::plan::{Facing, Role};
use crate::{Kingdom, Recipe, Rng, Soma};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    Raccoon,
    Cat,
    Horse,
    Bird,
    Fish,
    Grass,
    Shrub,
    Tree,
}

impl Archetype {
    pub const ALL: [Self; 8] = [
        Self::Raccoon,
        Self::Cat,
        Self::Horse,
        Self::Bird,
        Self::Fish,
        Self::Grass,
        Self::Shrub,
        Self::Tree,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Raccoon => "raccoon-like",
            Self::Cat => "cat-like",
            Self::Horse => "horse-like",
            Self::Bird => "bird-like",
            Self::Fish => "fish-like",
            Self::Grass => "grass",
            Self::Shrub => "shrub",
            Self::Tree => "tree",
        }
    }

    pub fn role(self) -> Kingdom {
        match self {
            Self::Grass | Self::Shrub | Self::Tree => Kingdom::Producer,
            _ => Kingdom::Consumer,
        }
    }

    /// Starting references retain all their authored organ situs. Ordinary
    /// development can omit a situs; generation rejects that draw rather than
    /// changing the developmental rules for existing lineages.
    pub(super) fn accepts(self, soma: &Soma) -> bool {
        soma.absent.is_empty()
    }

    pub(super) fn generate(self, rng: &mut Rng) -> Recipe {
        match self {
            Self::Raccoon | Self::Cat | Self::Horse => quadruped(self, rng),
            Self::Bird => bird(rng),
            Self::Fish => fish(rng),
            Self::Grass => grass(rng),
            Self::Shrub => {
                let mut recipe = crate::axis::archetype::spaced::producer_shrub();
                recipe.tagmata[0].segments = 3 + rng.below(4) as u8;
                recipe
            },
            Self::Tree => tree(rng),
        }
    }
}

#[derive(Default)]
struct Anatomy {
    tagmata: Vec<Tagma>,
    layout: Vec<Stretch>,
    chains: Vec<Vec<AppendageStep>>,
}

impl Anatomy {
    fn add(&mut self, tagma: Tagma, parent: Option<u8>, anchor: Anchor, facing: Facing) -> u8 {
        let index = self.tagmata.len() as u8;
        self.tagmata.push(tagma);
        self.layout.push(Stretch {
            parent,
            anchor,
            facing,
            variance: Some(0),
        });
        self.chains.push(Vec::new());
        index
    }

    fn finish(self) -> Recipe {
        let mut recipe = Recipe::of(self.tagmata)
            .with_layout(self.layout)
            .with_appendage_chains(self.chains);
        // Generation varies the authored counts. Admission separately rejects
        // Soma draws that omit an organ situs through Archetype::accepts.
        recipe.variance = 0;
        recipe
    }
}

fn link(role: Role, shape: u8, facing: ChainFacing, distal: bool) -> AppendageStep {
    AppendageStep {
        role,
        shape,
        facing,
        distal,
    }
}

fn legs(long: bool) -> Vec<AppendageStep> {
    let mut links = vec![link(Role::Limb, 1, ChainFacing::Outward, false)];
    links.push(link(Role::Limb, 2, ChainFacing::Below, true));
    if long {
        links.push(link(Role::Limb, 2, ChainFacing::Below, false));
    }
    links.push(link(Role::Limb, 3, ChainFacing::Front, true));
    links
}

fn quadruped(kind: Archetype, rng: &mut Rng) -> Recipe {
    let mut a = Anatomy::default();
    let broad = if kind == Archetype::Cat { 2 } else { 0 };
    // The ordinary feeding reader recognizes a mouth under the root head.
    let head = a.add(
        Tagma::new(1, Appendage::Mouth)
            .with_shapes(0, if kind == Archetype::Horse { 3 } else { 1 }),
        None,
        Anchor::Tip,
        Facing::Back,
    );
    let eyes = a.add(
        Tagma::new(1, Appendage::Feeler).with_shapes(1, 0),
        Some(head),
        Anchor::Base,
        Facing::Above,
    );
    // A raised crown clears the working eye boxes before the ear bases
    // spread sideways. All these links are structural mass, not sensors.
    let crown = a.add(
        Tagma::bare(1).with_shapes(3, 0),
        Some(eyes),
        Anchor::Tip,
        Facing::Above,
    );
    for facing in [Facing::Left, Facing::Right] {
        let base = a.add(
            Tagma::bare(1).with_shapes(3, 0),
            Some(crown),
            Anchor::Base,
            facing,
        );
        a.add(
            Tagma::bare(if kind == Archetype::Cat { 2 } else { 1 }).with_shapes(3, 0),
            Some(base),
            Anchor::Tip,
            Facing::Above,
        );
    }
    // A short backward run clears the mouth before the neck descends.
    let nape = a.add(
        Tagma::bare(1).with_shapes(1, 0),
        Some(head),
        Anchor::Tip,
        Facing::Back,
    );
    let neck = a.add(
        Tagma::bare(if kind == Archetype::Horse {
            3 + rng.below(2) as u8
        } else {
            1
        })
        .with_shapes(1, 0),
        Some(nape),
        Anchor::Tip,
        Facing::Below,
    );
    let front = a.add(
        Tagma::new(1, Appendage::Limb).with_shapes(broad, 3),
        Some(neck),
        Anchor::Tip,
        Facing::Back,
    );
    a.chains[front as usize] = legs(kind == Archetype::Horse);
    let trunk = a.add(
        Tagma::bare(2 + rng.below(3) as u8).with_shapes(broad, 0),
        Some(front),
        Anchor::Tip,
        Facing::Back,
    );
    let rear = a.add(
        Tagma::new(1, Appendage::Limb).with_shapes(broad, 3),
        Some(trunk),
        Anchor::Tip,
        Facing::Back,
    );
    a.chains[rear as usize] = legs(kind == Archetype::Horse);
    let tail_length = match kind {
        Archetype::Cat => 5,
        Archetype::Horse => 2,
        _ => 3,
    } + rng.below(3) as u8;
    a.add(
        Tagma::bare(tail_length).with_shapes(if kind == Archetype::Raccoon { 0 } else { 1 }, 0),
        Some(rear),
        Anchor::Tip,
        Facing::Back,
    );
    a.finish()
}

fn bird(rng: &mut Rng) -> Recipe {
    let mut a = Anatomy::default();
    let head = a.add(
        Tagma::new(1, Appendage::Mouth).with_shapes(0, 1),
        None,
        Anchor::Tip,
        Facing::Back,
    );
    a.add(
        Tagma::new(1, Appendage::Feeler).with_shapes(1, 0),
        Some(head),
        Anchor::Base,
        Facing::Above,
    );
    let nape = a.add(
        Tagma::bare(1).with_shapes(1, 0),
        Some(head),
        Anchor::Tip,
        Facing::Back,
    );
    let neck = a.add(
        Tagma::bare(1 + rng.below(3) as u8).with_shapes(1, 0),
        Some(nape),
        Anchor::Tip,
        Facing::Below,
    );
    let body = a.add(
        Tagma::new(1, Appendage::Vane).with_shapes(2, 0),
        Some(neck),
        Anchor::Tip,
        Facing::Back,
    );
    a.chains[body as usize] = vec![
        link(Role::Limb, 0, ChainFacing::Outward, false),
        link(Role::Limb, 0, ChainFacing::Outward, false),
    ];
    let feet = a.add(
        Tagma::new(1, Appendage::Limb).with_shapes(0, 3),
        Some(body),
        Anchor::Tip,
        Facing::Back,
    );
    a.chains[feet as usize] = legs(false);
    a.add(
        Tagma::bare(2 + rng.below(3) as u8).with_shapes(1, 0),
        Some(feet),
        Anchor::Tip,
        Facing::Back,
    );
    a.finish()
}

fn fish(rng: &mut Rng) -> Recipe {
    let mut a = Anatomy::default();
    let head = a.add(
        Tagma::new(1, Appendage::Mouth).with_shapes(0, 1),
        None,
        Anchor::Tip,
        Facing::Back,
    );
    a.add(
        Tagma::new(1, Appendage::Feeler).with_shapes(1, 0),
        Some(head),
        Anchor::Base,
        Facing::Above,
    );
    let fins = a.add(
        Tagma::new(1, Appendage::Vane).with_shapes(2, 0),
        Some(head),
        Anchor::Tip,
        Facing::Back,
    );
    let trunk = a.add(
        Tagma::bare(2 + rng.below(4) as u8).with_shapes(0, 0),
        Some(fins),
        Anchor::Tip,
        Facing::Back,
    );
    let tail = a.add(
        Tagma::bare(2).with_shapes(1, 0),
        Some(trunk),
        Anchor::Tip,
        Facing::Back,
    );
    // A vertical fork is structural mass. Vane pairs remain actual fluid limbs.
    for facing in [Facing::Above, Facing::Below] {
        a.add(
            Tagma::bare(1 + rng.below(2) as u8).with_shapes(1, 0),
            Some(tail),
            Anchor::Tip,
            facing,
        );
    }
    a.finish()
}

fn grass(rng: &mut Rng) -> Recipe {
    let mut a = Anatomy::default();
    let root = a.add(
        Tagma::bare(1).with_shapes(1, 0),
        None,
        Anchor::Tip,
        Facing::Above,
    );
    for facing in [Facing::Left, Facing::Right, Facing::Front, Facing::Back] {
        let base = a.add(
            Tagma::bare(1).with_shapes(1, 0),
            Some(root),
            Anchor::Base,
            facing,
        );
        let blade = a.add(
            Tagma::new(1, Appendage::Plate).with_shapes(1, 0),
            Some(base),
            Anchor::Tip,
            Facing::Above,
        );
        let height = 1 + rng.below(3) as usize;
        a.chains[blade as usize] = (0..height)
            .map(|_| link(Role::Mass, 1, ChainFacing::Above, false))
            .chain([link(Role::Plate, 0, ChainFacing::Above, false)])
            .collect();
    }
    a.finish()
}

fn tree(rng: &mut Rng) -> Recipe {
    let mut a = Anatomy::default();
    let trunk = a.add(
        Tagma::bare(7 + rng.below(4) as u8),
        None,
        Anchor::Tip,
        Facing::Above,
    );
    for (ordinal, facing) in [Facing::Left, Facing::Right, Facing::Front, Facing::Back]
        .into_iter()
        .enumerate()
    {
        let branch = a.add(
            Tagma::bare(2 + rng.below(3) as u8).with_shapes(1, 0),
            Some(trunk),
            if ordinal < 2 {
                Anchor::Middle
            } else {
                Anchor::Tip
            },
            facing,
        );
        let crown = a.add(
            Tagma::new(1, Appendage::Plate).with_shapes(1, 1),
            Some(branch),
            Anchor::Tip,
            Facing::Above,
        );
        a.chains[crown as usize] = vec![
            link(Role::Mass, 1, ChainFacing::Above, false),
            link(Role::Plate, 1, ChainFacing::Above, false),
        ];
    }
    a.add(
        Tagma::new(1, Appendage::Plate).with_shapes(1, 1),
        Some(trunk),
        Anchor::Tip,
        Facing::Above,
    );
    a.finish()
}

#[cfg(test)]
mod tests;
