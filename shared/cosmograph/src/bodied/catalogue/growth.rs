// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! How a founder grows (case 1206): one set of growth operators, drawn by
//! the weights and within the bounds the catalogue gives its kingdom
//! (`datasheets/catalog/kingdoms.toml`). Each draw proposes a rule on a run of
//! the body as it then stands; a proposal that leaves the body invalid or
//! unchanged is dropped and another drawn, so no draw can fail a founding
//! and no kingdom is a fixed plan. A kingdom is a row of weights here and
//! nothing else.

use crate::{
    catalog::Kingdom,
    rules::{Accretion, Anchor, Condition, Facing, Grammar, Rule, Symmetry, YEAR, unfold},
};

/// The operators a founder may draw, by the key the kingdoms sheet weighs
/// each under. `allocate` closes every draw and is no entry; a deposit
/// waits on its lowering at founding and a partition on an organ's own
/// geometry.
pub(crate) const OPERATORS: [&str; 10] =
    ["repeat", "branch", "radiate", "symmetry", "differentiate", "harden", "cover", "fuse", "partner", "switch"];

/// The numbers a draw takes within a kingdom's bounds, with the bounds a
/// sheet that names none of them leaves.
pub(crate) const BOUNDS: [(&str, [u16; 2]); 8] = [
    ("by", [1, 3]),
    ("every", [1, 3]),
    ("depth", [1, 2]),
    ("taper", [0, 2]),
    ("arms", [2, 5]),
    ("oldest", [1, 3]),
    ("crowd", [2, 6]),
    ("total", [6, 14]),
];

/// Dead matter a run may lay down (D42): the world declares both with no
/// cells where its founders are composed.
pub(crate) const HARDENED: &str = "kind:wood";
pub(crate) const COVERING: &str = "kind:bark";

/// Declares the dead matter among `kinds`: the box of a living kind with no
/// cells, so it holds and does nothing.
pub(crate) fn inert(kinds: &mut std::collections::BTreeMap<crate::schema::Key, crate::rules::Template>) {
    for (dead, living) in [(HARDENED, "kind:block"), (COVERING, "kind:frond")] {
        if let Some(shape) = kinds.get(living).cloned() {
            let template = crate::rules::Template { cells: Default::default(), port: None, ..shape };
            kinds.entry(dead.into()).or_insert(template);
        }
    }
}

/// The most segments a founder's body may unfold to in any stage.
pub(super) const MOST_SEGMENTS: usize = 32;

/// Join each organ run to the trunk, a side axis or an earlier organ, at a
/// drawn joint and side. A parent is always an earlier run, so no joint
/// names one not yet placed; a layout the world refuses is left as it was.
pub(super) fn attach(grammar: &mut Grammar, draw: &impl Fn(&str, u64) -> u64, valid: &impl Fn(&Grammar) -> bool) {
    for index in 2..grammar.seed.len() {
        let slot = index as u64;
        let before = grammar.seed[index].clone();
        let parent = (draw("growth-bearer", slot) % slot) as usize;
        let bearing = grammar.seed[parent].facing;
        let run = &mut grammar.seed[index];
        run.parent = Some(parent as u8);
        run.anchor = ANCHORS[(draw("growth-joint", slot) % 3) as usize];
        run.facing = if bearing.horizontal() {
            bearing.turned(1 + 2 * (draw("growth-side", slot) % 2) as usize)
        } else {
            FACINGS[(draw("growth-side", slot) % 4) as usize]
        };
        if !valid(grammar) {
            grammar.seed[index] = before;
        }
    }
}

const ANCHORS: [Anchor; 3] = [Anchor::Base, Anchor::Middle, Anchor::Tip];
const FACINGS: [Facing; 6] = [Facing::Front, Facing::Right, Facing::Back, Facing::Left, Facing::Above, Facing::Below];

pub(crate) struct Sampler<'a, D, V> {
    pub kingdom: &'a Kingdom,
    /// A draw in a named domain at a slot, seeded for this founder.
    pub draw: D,
    /// Whether a grammar unfolds to a body the world accepts.
    pub valid: V,
    /// Whether the world has a year for a seasonal switch to read.
    pub seasons: bool,
}

impl<D: Fn(&str, u64) -> u64, V: Fn(&Grammar) -> bool> Sampler<'_, D, V> {
    fn within(&self, number: &str, slot: u64) -> u16 {
        let default = BOUNDS.iter().find(|(name, _)| *name == number).map_or([1, 1], |(_, b)| *b);
        let [least, most] = self.kingdom.bounds.get(number).copied().unwrap_or(default);
        least + ((self.draw)(number, slot) % u64::from(most.saturating_sub(least) + 1)) as u16
    }

    fn pick<T: Copy>(&self, domain: &str, slot: u64, of: &[T]) -> T {
        of[((self.draw)(domain, slot) % of.len() as u64) as usize]
    }

    /// One operator, by the kingdom's weights.
    fn operator(&self, slot: u64, among: &[&'static str]) -> Option<&'static str> {
        let weight = |name: &str| u64::from(self.kingdom.operators.get(name).copied().unwrap_or(0));
        let total: u64 = among.iter().map(|name| weight(name)).sum();
        let mut at = (self.draw)("growth-operator", slot) % total.max(1);
        among.iter().copied().find(|name| {
            let w = weight(name);
            if at < w {
                return true;
            }
            at -= w;
            false
        })
    }

    /// A rule of `operator` on a run of `body`, the grammar's body as it
    /// stands; none where the body gives the operator nothing to act on.
    pub(crate) fn propose(&self, operator: &str, body: &[crate::rules::Tagma], slot: u64) -> Option<Rule> {
        let run = ((self.draw)("growth-run", slot) % body.len() as u64) as usize;
        let tagma = run as u8;
        Some(match operator {
            "repeat" => Rule::Repeat { tagma, by: self.within("by", slot) as u8 },
            "branch" => Rule::Branch {
                tagma,
                every: self.within("every", slot) as u8,
                depth: self.within("depth", slot) as u8,
                taper: self.within("taper", slot) as u8,
                anchor: self.pick("growth-anchor", slot, &ANCHORS),
                facing: self.pick("growth-facing", slot, &FACINGS),
            },
            "radiate" => Rule::Radiate {
                tagma,
                arms: self.within("arms", slot) as u8,
                fused: (self.draw)("growth-joined", slot) % 2 == 0,
            },
            "symmetry" => {
                let bearing: Vec<u8> = (0..body.len()).filter(|i| body[*i].bears.is_some()).map(|i| i as u8).collect();
                let arms = self.within("arms", slot) as u8;
                Rule::Symmetry {
                    tagma: *bearing.get(((self.draw)("growth-run", slot) % bearing.len().max(1) as u64) as usize)?,
                    symmetry: self.pick("growth-symmetry", slot, &[Symmetry::Bilateral, Symmetry::Radial { arms }, Symmetry::None]),
                }
            },
            "differentiate" => {
                let kinds: Vec<_> = body.iter().filter_map(|t| t.bears.clone()).filter(|k| *k != body[run].segment).collect();
                let at = 1 + ((self.draw)("growth-at", slot) % u64::from(body[run].segments.max(2) - 1)) as u8;
                Rule::Differentiate { tagma, at, kind: kinds.get(((self.draw)("growth-tissue", slot) % kinds.len().max(1) as u64) as usize)?.clone() }
            },
            "harden" => Rule::Accrete {
                tagma,
                how: Accretion::Harden { oldest: self.within("oldest", slot) as u8, kind: HARDENED.into() },
            },
            "cover" => Rule::Accrete {
                tagma,
                how: Accretion::Cover { kind: COVERING.into(), socket: self.pick("growth-socket", slot, &FACINGS) },
            },
            "fuse" => {
                let other = ((self.draw)("growth-with", slot) % body.len() as u64) as u8;
                Rule::Fuse { runs: if other == tagma { vec![tagma] } else { vec![tagma, other] } }
            },
            "partner" => Rule::Partner { tagma, kept: None },
            "switch" => {
                let from = ((self.draw)("growth-season", slot) % u64::from(YEAR)) as u16;
                let conditions = [
                    Condition::FullReserves,
                    Condition::Crowding { bodies: u32::from(self.within("crowd", slot)) },
                    Condition::Season { from, to: (from + YEAR / 4 + ((self.draw)("growth-span", slot) % u64::from(YEAR / 4)) as u16) % YEAR },
                ];
                let when = self.pick("growth-condition", slot, &conditions[..2 + usize::from(self.seasons)]);
                // What a switch brings on names nothing the body lacks: it
                // lengthens, branches or copies what is there.
                let inner = self.operator(slot ^ (1 << 32), &["repeat", "branch", "radiate"])?;
                Rule::Switch { when, rules: vec![self.propose(inner, body, slot ^ (1 << 32))?] }
            },
            _ => return None,
        })
    }

    /// Draw this founder's further rules onto `grammar`, then its budget.
    pub(super) fn compose(&self, grammar: &mut Grammar) {
        let [least, most] = self.kingdom.rules;
        let wanted = u64::from(least) + (self.draw)("growth-rules", 0) % (u64::from(most.saturating_sub(least)) + 1);
        let (mut added, mut slot) = (0, 0);
        while added < wanted && slot < 8 * wanted + 8 {
            slot += 1;
            let Ok(body) = unfold(grammar) else { return };
            let Some(rule) = self.operator(slot, &OPERATORS).and_then(|name| self.propose(name, &body, slot)) else {
                continue;
            };
            grammar.rules.push(rule);
            let changed = match grammar.rules.last() {
                Some(Rule::Switch { .. }) => true,
                _ => unfold(grammar).is_ok_and(|grown| grown != body),
            };
            if changed && (self.valid)(grammar) {
                added += 1;
            } else {
                grammar.rules.pop();
            }
        }
        grammar.rules.push(Rule::Allocate { total: self.within("total", 0) });
        if !(self.valid)(grammar) {
            grammar.rules.pop();
        }
    }
}
