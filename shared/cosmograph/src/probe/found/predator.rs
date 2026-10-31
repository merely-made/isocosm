// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A lineage that eats the others (ruling 287). A hunter hunts while its
//! body is below its appetite, eating a bite of a living member of any other
//! lineage at its site, drawn by what each holds, and keeps its body and
//! starves as the others do. It takes no part in the competitions, and the
//! mind reads no need of it. Its prey carry fat besides their bodies and
//! stores, drawn per cohort, which nothing but a hunt takes: so prey differ
//! in what they hold, and a draw by holdings differs from one by headcount.

use super::contested::{below, keeping, own, store};
use crate::{generate::process, rules::*};
use serde::{Deserialize, Serialize};

/// Inclusive ranges, drawn per world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorFounding {
    /// Hunters per site.
    pub hunters: [u64; 2],
    /// What a hunter eats of its prey at a time.
    pub bite: [u64; 2],
    /// A hunter hunts while its body is below this.
    pub appetite: [u64; 2],
    /// Fat each prey cohort carries.
    pub fat: [u64; 2],
}

impl Default for PredatorFounding {
    /// Checkpoint 4b's domain: a twentieth to over a third of the prey's
    /// headcount hunting, meals big against a lean body, and fat up to five
    /// times a body, so which prey a hunt takes moves who starves.
    fn default() -> Self {
        Self {
            hunters: [12, 48],
            bite: [2, 6],
            appetite: [2, 12],
            fat: [0, 48],
        }
    }
}

/// The hunters as drawn for one world: their lineage, the lineages before
/// it being their prey.
pub(super) struct Hunters {
    pub lineage: u32,
    pub per_site: u64,
    pub bite: u64,
    pub appetite: u64,
    pub fat: [u64; 2],
}

/// The account a prey lineage keeps its fat in.
pub(super) fn fat(lineage: u32) -> crate::schema::Key {
    store(lineage, 2)
}

impl PredatorFounding {
    pub(super) fn valid(&self) -> bool {
        let ranges = [self.hunters, self.bite, self.appetite];
        ranges.iter().all(|r| r[0] <= r[1] && r[0] > 0) && self.fat[0] <= self.fat[1]
    }

    pub(super) fn draw(&self, pick: impl Fn(&str, [u64; 2]) -> u64, lineage: u32) -> Hunters {
        Hunters {
            lineage,
            per_site: pick("probe-hunters", self.hunters),
            bite: pick("probe-bite", self.bite),
            appetite: pick("probe-appetite", self.appetite),
            fat: self.fat,
        }
    }
}

impl Hunters {
    pub(super) fn identity(&self) -> String {
        format!("ability:probe-{}", self.lineage)
    }

    /// Keeping a body, and hunting every lineage before this one: only a
    /// prey holding a whole bite is taken, so every meal is a bite.
    pub(super) fn processes(&self) -> Vec<Process> {
        let i = self.lineage;
        let body = store(i, 0);
        let mut hunt = process(
            &format!("probe:hunt-{i}"),
            Causation::Choice,
            vec![Effect::Eat {
                from: Binding::Target,
                amount: self.bite.into(),
                into: body.clone(),
                of: vec![],
                whole: false,
            }],
        );
        hunt.requires.extend([
            own(i),
            below(&body, self.appetite),
            Query::Holds {
                who: Binding::Target,
                at_least: self.bite,
            },
        ]);
        hunt.target = Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: None,
            among: (0..i).map(|j| format!("lineage:{j}")).collect(),
            weighted: true,
        });
        hunt.period = Some(1);
        let [upkeep, starve] = keeping(i);
        vec![upkeep, starve, hunt]
    }
}
