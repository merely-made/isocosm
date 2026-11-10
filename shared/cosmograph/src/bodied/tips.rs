// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A plant grown from its tips (cases 1202, 1206): one stem bearing fronds,
//! a grammar that lengthens it, branches it at every other segment to two
//! levels, each branch turned off the last, and caps the whole. A preset to
//! found from; no founding path takes it up by itself.

use super::*;

/// The segments the stem is seeded with, and the grammar's first rule adds.
const STEM: u8 = 6;
const LENGTHEN: u8 = 2;
/// Segments between joints, how many each level is shorter, and the cap.
const EVERY: u8 = 2;
const TAPER: u8 = 2;
const TOTAL: u16 = 72;

/// The plant development the grammar grows: a producer's stem, the `block`,
/// bearing a plate each side, the `broad`, which a proportioned plant's
/// leaves are. The `frond` is too wide for a run turned off the spine to
/// bear: its plates would cross one another along it.
///
/// # Errors
/// Fails if `rules` lacks the stem or frond kind, or the grammar does not
/// unfold.
pub fn tips(rules: &Rules) -> Result<Development> {
    let (stem, frond) = (kind("block"), kind("broad"));
    for k in [&stem, &frond] {
        if !rules.kinds.contains_key(k) {
            return Err(format!("no {k} kind to grow a plant from"));
        }
    }
    let seed = Tagma { socket: FLANK, ..bearing(STEM, "block", "broad", 1, FLANK) };
    let grammar = Grammar {
        seed: vec![seed],
        rules: vec![
            Rule::Repeat { tagma: 0, by: LENGTHEN },
            Rule::Branch {
                tagma: 0,
                every: EVERY,
                depth: 2,
                taper: TAPER,
                anchor: Anchor::Tip,
                facing: LIT,
            },
            Rule::Allocate { total: TOTAL },
        ],
    };
    let recipe = Recipe {
        tagmata: unfold(&grammar)?,
        variance: 0,
        absence: [0, 1],
        riff: [0, 1],
        vary: [0, 1],
    };
    Ok(Development {
        lexicon: BTreeSet::from([stem, frond]),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
        anamorphic: false,
        tracts: vec![],
        systems: Default::default(),
        comfort: 0,
        grammar: Some(grammar),
    })
}
