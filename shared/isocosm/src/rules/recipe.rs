// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a lineage's bodies develop from (rulings 468, 478, 495, 510 to 512,
//! 516, 530 and 531): Mesocosm's axial recipe and isometer's placement
//! policy, re-expressed as world data. A kind is a part template, a box and
//! the cells each function takes (511); a recipe runs head to tail as
//! tagmata of segments, each segment bearing a kind; and a lineage keeps its
//! recipe, its policy, its affinity domain, its lexicon of kinds and its
//! clutch.

use crate::schema::Key;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Where a part faces from its parent: isometer's six, along the axes
/// right +x, above +y and back +z.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Facing {
    Front,
    Back,
    Left,
    Right,
    Above,
    Below,
}

impl Facing {
    pub const ALL: [Facing; 6] = [
        Facing::Front,
        Facing::Back,
        Facing::Left,
        Facing::Right,
        Facing::Above,
        Facing::Below,
    ];

    /// The axis it lies along and its sign.
    pub fn axis(self) -> (usize, i32) {
        match self {
            Facing::Right => (0, 1),
            Facing::Left => (0, -1),
            Facing::Above => (1, 1),
            Facing::Below => (1, -1),
            Facing::Back => (2, 1),
            Facing::Front => (2, -1),
        }
    }

    /// Its mirror across the body's midline: left and right swap.
    pub fn mirrored(self) -> Facing {
        match self {
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
            other => other,
        }
    }

    pub fn lateral(self) -> bool {
        matches!(self, Facing::Left | Facing::Right)
    }
}

/// A part template (ruling 511): a box, and the cells each function takes
/// of it, never more in all than the box's capacity. A declared shape is
/// for the hollows a box cannot show, tube and shell (494).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub half_extent: [i32; 3],
    pub cells: BTreeMap<Key, u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub shape: Key,
}

/// Where a tagma's first segment joins the segments placed before it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    Base,
    Middle,
    #[default]
    Tip,
}

/// One tagma (ruling 478): its segments, each of the `segment` kind, and
/// the kind each segment bears, `per_segment` of them (a pair each for
/// lateral kinds). `parent` names an earlier tagma it branches from, at
/// `anchor`; absent, it continues from the segment placed last.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tagma {
    pub segments: u8,
    pub segment: Key,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bears: Option<Key>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub per_segment: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u8>,
    #[serde(default)]
    pub anchor: Anchor,
    /// The way the tagma runs from what it joins; the spine runs back.
    #[serde(default = "back")]
    pub facing: Facing,
    /// Where on a segment its borne kind attaches, as Mesocosm's sockets
    /// do (510): a flank, mirrored where the plan is bilateral, by default.
    #[serde(default = "right")]
    pub socket: Facing,
    /// Overrides the recipe's variance for this tagma.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variance: Option<u8>,
}

/// A lineage's axial recipe (ruling 478): tagmata head to tail; the drift
/// each tagma's segment count may take either way; and the odds that a
/// segment's borne kind develops absent, as a numerator over a denominator,
/// none where the numerator is nought (531).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub tagmata: Vec<Tagma>,
    #[serde(default)]
    pub variance: u8,
    #[serde(default = "never")]
    pub absence: [u32; 2],
}

/// How a body places what it grows or takes in (isometer's policy, kept
/// as world data by ruling 478): the facing each box name prefers, the
/// plan's symmetry, and how many facings beyond the preferred it tries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub bilateral: bool,
    pub preferences: BTreeMap<Key, Facing>,
    pub tolerance: u8,
}

impl Default for Policy {
    /// isometer's default plan: bilateral, a lump below, a rod right, a
    /// sheet above and a point to the front, two further facings tried.
    fn default() -> Self {
        let preferences = [
            ("part-shape:lump", Facing::Below),
            ("part-shape:rod", Facing::Right),
            ("part-shape:sheet", Facing::Above),
            ("part-shape:point", Facing::Front),
        ];
        Self {
            bilateral: true,
            preferences: preferences.map(|(k, f)| (k.to_string(), f)).into(),
            tolerance: 2,
        }
    }
}

impl Policy {
    /// The facings tried for a box read as `name`, in order: the preferred,
    /// its mirror where the plan mirrors, then the rest in isometer's order,
    /// `tolerance` of them beyond the preferred.
    pub fn candidates(&self, name: &str) -> Vec<Facing> {
        let first = self.preferences.get(name).copied().unwrap_or(Facing::Front);
        let mut out = vec![first];
        if self.mirrors(first) {
            out.push(first.mirrored());
        }
        for f in Facing::ALL {
            if out.len() > usize::from(self.tolerance) {
                break;
            }
            if !out.contains(&f) {
                out.push(f);
            }
        }
        out
    }

    /// Whether a part placed facing `f` is matched by its mirror.
    pub fn mirrors(&self, f: Facing) -> bool {
        self.bilateral && f.lateral()
    }
}

/// How a part from one tissue domain lands on a body of another (ruling
/// 516, Mesocosm's affinity).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Verdict {
    /// The same domain: it lands as it was.
    Native,
    /// A favoured crossing: it lands expressing nothing.
    Adapter,
    /// It does not land, and is eaten as a meal.
    Refused,
}

/// The world's tissue domains and the crossings it favours.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affinity {
    pub domains: u16,
    pub favoured: BTreeSet<(u16, u16)>,
}

impl Default for Affinity {
    /// Mesocosm's: three domains, each favouring the next.
    fn default() -> Self {
        let domains = 3;
        let favoured = (0..domains).map(|d| (d, (d + 1) % domains)).collect();
        Self { domains, favoured }
    }
}

impl Affinity {
    pub fn verdict(&self, from: u16, into: u16) -> Verdict {
        if from >= self.domains || into >= self.domains {
            Verdict::Refused
        } else if from == into {
            Verdict::Native
        } else if self.favoured.contains(&(from, into)) {
            Verdict::Adapter
        } else {
            Verdict::Refused
        }
    }
}

/// What a lineage's bodies develop from: its recipe and policy, its tissue
/// domain, the kinds it has learned (its lexicon, 468), and how many eggs a
/// birth lays (530).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Development {
    pub recipe: Recipe,
    #[serde(default)]
    pub policy: Policy,
    #[serde(default)]
    pub domain: u16,
    pub lexicon: BTreeSet<Key>,
    #[serde(default = "one")]
    pub clutch: u32,
}

impl Recipe {
    /// Every kind the recipe names, segments and what they bear.
    pub fn kinds(&self) -> BTreeSet<Key> {
        let segments = self.tagmata.iter().map(|t| t.segment.clone());
        let borne = self.tagmata.iter().filter_map(|t| t.bears.clone());
        segments.chain(borne).collect()
    }
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

fn back() -> Facing {
    Facing::Back
}

fn right() -> Facing {
    Facing::Right
}

fn never() -> [u32; 2] {
    [0, 1]
}

fn one() -> u32 {
    1
}
