// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::rules::{Anchor, Condition, Facing, Grammar, Rule, Symmetry};

/// Compose a topology, a distribution of functional tissue, and a growth
/// budget. All operator addresses point into the stable editable seed, even
/// when another operator appends runs. The root stays available to reproduce.
pub(super) fn compose(grammar: &mut Grammar, seed: u64, lineage: u32, repeated: usize) {
    let draw = |domain, slot| crate::draw(seed, domain, &[u64::from(lineage), slot]);
    let anchors = [Anchor::Base, Anchor::Middle, Anchor::Tip];
    let horizontal = [Facing::Front, Facing::Right, Facing::Back, Facing::Left];
    // Attach organs to the trunk, a side axis, or one another. Parent indices
    // are always earlier, so adding an organ never invalidates an old joint.
    for index in 2..grammar.seed.len() {
        let slot = index as u64;
        let parent = (draw("growth-bearer", slot) % slot) as usize;
        let bearing = grammar.seed[parent].facing;
        let run = &mut grammar.seed[index];
        run.parent = Some(parent as u8);
        run.anchor = anchors[(draw("growth-anchor", slot) % 3) as usize];
        run.facing = if bearing.horizontal() {
            bearing.turned(1 + 2 * (draw("growth-side", slot) % 2) as usize)
        } else {
            horizontal[(draw("growth-side", slot) % 4) as usize]
        };
    }

    // Duplicate an actual working organ, where present. Expanding this run
    // changes how much of the grown body feeds, defends, draws, or exchanges.
    let run_count = grammar.seed.len();
    let organ = if run_count > 2 {
        2 + (draw("growth-organ", 0) % (run_count - 2) as u64) as usize
    } else {
        1
    };
    let tagma = organ as u8;
    let facing = grammar.seed[organ].facing;
    match draw("growth-topology", 0) % 3 {
        0 => {
            let side = if facing.horizontal() {
                facing.turned(1)
            } else {
                Facing::Right
            };
            grammar.rules.push(Rule::Branch {
                tagma,
                every: 1,
                depth: 1,
                taper: 0,
                anchor: anchors[(draw("growth-joint", 0) % 3) as usize],
                facing: side,
            });
        },
        1 if facing.horizontal() => grammar.rules.push(Rule::Radiate {
            tagma,
            arms: 2 + (draw("growth-arms", 0) % 3) as u8,
            fused: draw("growth-joined", 0) % 2 == 0,
        }),
        _ => {
            grammar.rules.push(Rule::Repeat {
                tagma,
                by: 1 + (draw("growth-extent", 0) % 2) as u8,
            });
            if grammar.seed[organ].bears.is_some() {
                grammar.seed[organ].socket = if facing.lateral() {
                    Facing::Front
                } else {
                    Facing::Right
                };
                grammar.rules.push(Rule::Symmetry {
                    tagma,
                    symmetry: Symmetry::Radial { arms: 4 },
                });
            }
        },
    }

    // Some lines differentiate working tissue along a body axis. Its distal
    // section then performs that function directly, even between appendages.
    if run_count > 2 && draw("growth-differentiation", 0) % 3 == 0 {
        let donor = 2 + (draw("growth-tissue", 0) % (run_count - 2) as u64) as usize;
        if let Some(kind) = grammar.seed[donor].bears.clone() {
            grammar.seed[1].segments = grammar.seed[1].segments.max(2);
            grammar.rules.push(Rule::Differentiate {
                tagma: 1,
                at: 1,
                kind,
            });
        }
    }
    if draw("growth-pool", 0) % 3 == 0 {
        grammar.rules.push(Rule::Fuse {
            runs: vec![0, tagma],
        });
    }
    if draw("growth-plasticity", 0) % 3 == 0 {
        let when = if draw("growth-trigger", 0) % 2 == 0 {
            Condition::FullReserves
        } else {
            Condition::Crowding {
                bodies: 2 + (draw("growth-density", 0) % 5) as u32,
            }
        };
        grammar.rules.push(Rule::Switch {
            when,
            rules: vec![Rule::Repeat {
                tagma: repeated as u8,
                by: 2,
            }],
        });
    }
    grammar.rules.push(Rule::Allocate {
        total: 6 + (draw("growth-budget", 0) % 9) as u16,
    });
}
