// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The agentless hazard (ruling 704): the one definition the checkpoint-10
//! probe and generated Mesocosm worlds both run (the 2026-10-18 note to
//! ruling 227). Each tick, with odds `rate` in 64, it wounds a living body at
//! the world body's place, the target drawn by what it holds and the part by
//! its cells (716). A rate of nought never hits: the negative control.

use crate::{generate, rules::*, schema::Key};
use std::collections::BTreeSet;

pub const ID: &str = "body:hazard";

/// The probe's per-world draws, inclusive (704): the odds a tick as a
/// numerator over 64, and the cells a hit takes. Mesocosm's founding takes
/// them; the generator's own default is no hazard.
pub const RATE: [u64; 2] = [2, 8];
pub const CELLS: [u32; 2] = [1, 8];

/// The hazard hitting for `cells` cells, on bodies of `among` lineages, or
/// of any where `among` is empty.
pub fn process(rate: i64, cells: u64, among: BTreeSet<Key>) -> Process {
    let hits = Effect::When {
        guard: Expr::AtLeast(
            Box::new(Expr::Const(rate - 1)),
            Box::new(Expr::Draw {
                below: MAX_DRAW,
                slot: 0,
            }),
        ),
        then: vec![Effect::Wound {
            who: Binding::Target,
            cells: cells.into(),
            slot: 1,
        }],
        otherwise: vec![],
    };
    let mut hazard = generate::process(ID, Causation::Agentless, vec![hits]);
    hazard.period = Some(1);
    hazard.target = Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: None,
        among,
        weighted: true,
    });
    hazard.requires.push(Query::Computed(Expr::AtLeast(
        Box::new(Expr::Read(Reading::LivingCells {
            who: Binding::Target,
        })),
        Box::new(Expr::Const(1)),
    )));
    hazard
}
