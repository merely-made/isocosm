// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an intake organ admits and tolerates: the port a part carries
//! (wing ruling 766), its living kinds, dead stock, tolerances and, under the
//! categories rule, the dead classes it is limited to. Split from the parent
//! module along its own seam; the parent re-exports every name.

use super::ProcessRef;
use serde::{Deserialize, Serialize};

/// The living provenance a port may admit. Dead stock is deliberately not a
/// fourth kind: it is an intake-target condition. TG2 later decides how its
/// retained provenance is accounted before it returns to soil.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NisKind {
    Producer,
    Consumer,
    Decomposer,
}

/// A compact, part-local declaration of what one intake organ can accept.
///
/// This is anatomy carried beside the part's allocation. Geometry supplies a
/// founding default, but later declarations are retained through body changes.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct IntakePort {
    kinds: u8,
    deadstock: bool,
    /// The exact allocation that keeps this declaration active. A declaration
    /// survives rearrangement, while a port goes dormant when its tissue does.
    #[serde(default)]
    support: Option<ProcessRef>,
    /// What this mouth eats without harm under the world's spoilage rule
    /// (`crate::ecology::Spoilage`), beside what it admits: bits of
    /// [`Tolerance`]. None unless set, which serializes and hashes as before.
    #[serde(default, skip_serializing_if = "is_none_tolerated")]
    tolerance: u8,
    /// The classes of dead matter a dead-stock mouth is limited to under the
    /// categories rule (`crate::ecology::Categories`): bits of [`DeadClass`].
    /// None means every class. None unless set, which serializes and hashes
    /// as before.
    #[serde(default, skip_serializing_if = "is_none_tolerated")]
    classes: u8,
    /// What the organ's tissue does under the guard rule
    /// (`crate::ecology::Guard`): see [`Strength`]. None unless set, which
    /// serializes and hashes as before.
    #[serde(default, skip_serializing_if = "Strength::is_none")]
    strength: Strength,
}

/// The strengths a part carries under the guard rule (`crate::ecology::
/// Guard`; lineage-rounds D39 model 2), each on the catalog's nought to ten
/// scale (`catalog::Strength`). **On a mouth** they are its attack by kind
/// (crush, pierce). **On any other organ** they are what it gives the part
/// it is on: the armour that resists a bite of that kind, the wound its
/// contact deals the biter (`spine`), and the toxin its secretion charges the
/// eater (`toxin`). Which one is read is decided by whether the port is a
/// mouth, so one small record serves both sides of a bite.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Strength {
    pub crush: u8,
    pub pierce: u8,
    pub spine: u8,
    pub toxin: u8,
}

impl Strength {
    /// The strength of nothing.
    pub const NONE: Self = Self { crush: 0, pierce: 0, spine: 0, toxin: 0 };

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.crush == 0 && self.pierce == 0 && self.spine == 0 && self.toxin == 0
    }

    /// The larger of each strength of the two.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        Self {
            crush: self.crush.max(other.crush),
            pierce: self.pierce.max(other.pierce),
            spine: self.spine.max(other.spine),
            toxin: self.toxin.max(other.toxin),
        }
    }
}

fn is_none_tolerated(bits: &u8) -> bool {
    *bits == 0
}

/// Matter a mouth can be made to eat without harm, under the spoilage rule
/// (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md` D36, D37).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tolerance {
    /// The flesh of the dead once it has aged.
    AgedFlesh,
    /// The flesh of the dead once it is toxic.
    ToxicFlesh,
    /// The living flesh of a decomposer, which is toxic to ordinary mouths.
    DecomposerFlesh,
    /// Tough plant matter, living or dead (the categories rule): a mouth
    /// with a grinding capability eats it unharmed.
    ToughPlant,
    /// The contact harm of spines and stings (the guard rule): a mouth
    /// that tolerates it is not wounded for biting a spined part.
    Spines,
    /// The toxin of a gland (the guard rule): a mouth, or an organ elsewhere
    /// on the body, that clears it is not harmed by eating a secreting body.
    Toxin,
}

/// What dead matter came from, as the categories rule classes it
/// (`mesocosm/design_docs/2026-11-09_lineage_rounds_plan.md` D37). Derived
/// from the body and the part bitten, never stored. There is no dung: the
/// stock and A1 ecologies deposit no unkept remainder of a meal (a meal is
/// kept whole; only world time's partitioned meal returns a waste, and the
/// rule does not layer on world time).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DeadClass {
    /// Dead soft plant matter: leaf, frond, blade.
    Litter,
    /// Dead tough plant matter.
    Wood,
    /// The dead flesh of anything not a plant.
    Carrion,
}

impl DeadClass {
    pub const ALL: [Self; 3] = [Self::Litter, Self::Wood, Self::Carrion];

    const fn bit(self) -> u8 {
        match self {
            Self::Litter => 1,
            Self::Wood => 2,
            Self::Carrion => 4,
        }
    }
}

impl Tolerance {
    const fn bit(self) -> u8 {
        match self {
            Self::AgedFlesh => 1,
            Self::ToxicFlesh => 2,
            Self::DecomposerFlesh => 4,
            Self::ToughPlant => 8,
            Self::Spines => 16,
            Self::Toxin => 32,
        }
    }
}

impl IntakePort {
    const PRODUCER: u8 = 1;
    const CONSUMER: u8 = 2;
    const DECOMPOSER: u8 = 4;

    pub const fn none() -> Self {
        Self {
            kinds: 0,
            deadstock: false,
            support: None,
            tolerance: 0,
            classes: 0,
            strength: Strength::NONE,
        }
    }

    pub const fn live(kind: NisKind) -> Self {
        Self::none().with_live(kind)
    }

    pub const fn deadstock() -> Self {
        Self {
            kinds: 0,
            deadstock: true,
            support: None,
            tolerance: 0,
            classes: 0,
            strength: Strength::NONE,
        }
    }

    pub const fn with_live(mut self, kind: NisKind) -> Self {
        self.kinds |= match kind {
            NisKind::Producer => Self::PRODUCER,
            NisKind::Consumer => Self::CONSUMER,
            NisKind::Decomposer => Self::DECOMPOSER,
        };
        self
    }

    pub const fn with_deadstock(mut self) -> Self {
        self.deadstock = true;
        self
    }

    /// This mouth, eating `what` without harm.
    pub const fn tolerating(mut self, what: Tolerance) -> Self {
        self.tolerance |= what.bit();
        self
    }

    pub const fn tolerates(self, what: Tolerance) -> bool {
        self.tolerance & what.bit() != 0
    }

    /// Whether the mouth tolerates anything: the merge of ports reads this.
    pub const fn tolerance_bits(self) -> u8 {
        self.tolerance
    }

    pub const fn with_tolerance_bits(mut self, bits: u8) -> Self {
        self.tolerance |= bits & 63;
        self
    }

    /// This organ, with `strength` (the guard rule).
    pub const fn with_strength(mut self, strength: Strength) -> Self {
        self.strength = strength;
        self
    }

    pub const fn strength(self) -> Strength {
        self.strength
    }

    /// Whether this port is a mouth: it admits living or dead matter.
    pub const fn is_mouth(self) -> bool {
        self.kinds != 0 || self.deadstock
    }

    /// This dead-stock mouth, limited also to dead matter of `class`: the
    /// first call limits it, each further call adds a class it may eat.
    pub const fn taking(mut self, class: DeadClass) -> Self {
        self.classes |= class.bit();
        self
    }

    /// Whether a dead-stock mouth takes dead matter of `class`: every class
    /// unless it is limited to some.
    pub const fn takes(self, class: DeadClass) -> bool {
        self.classes == 0 || self.classes & class.bit() != 0
    }

    /// The classes it is limited to, as bits; nought for every class.
    pub const fn class_bits(self) -> u8 {
        self.classes
    }

    pub const fn with_class_bits(mut self, bits: u8) -> Self {
        self.classes |= bits & 7;
        self
    }

    pub const fn supported_by(mut self, process: ProcessRef) -> Self {
        self.support = Some(process);
        self
    }

    pub const fn support(self) -> Option<ProcessRef> {
        self.support
    }

    pub const fn admits_live(self, kind: NisKind) -> bool {
        self.kinds
            & match kind {
                NisKind::Producer => Self::PRODUCER,
                NisKind::Consumer => Self::CONSUMER,
                NisKind::Decomposer => Self::DECOMPOSER,
            }
            != 0
    }

    pub const fn admits_deadstock(self) -> bool {
        self.deadstock
    }

    pub const fn has_live(self) -> bool {
        self.kinds != 0
    }
}

