// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A lineage that eats the others (ruling 287). A hunter hunts while its
//! body is below its appetite, eating a bite of a living member of any other
//! lineage at its site, drawn by what each holds, and keeps its body and
//! starves as the others do. It takes no part in the competitions, and the
//! mind reads no need of it.

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
}

impl Default for PredatorFounding {
    fn default() -> Self {
        Self {
            hunters: [4, 16],
            bite: [1, 2],
            appetite: [3, 6],
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
}

impl PredatorFounding {
    pub(super) fn valid(&self) -> bool {
        let ranges = [self.hunters, self.bite, self.appetite];
        ranges.iter().all(|r| r[0] <= r[1] && r[0] > 0)
    }

    pub(super) fn draw(&self, pick: impl Fn(&str, [u64; 2]) -> u64, lineage: u32) -> Hunters {
        Hunters {
            lineage,
            per_site: pick("probe-hunters", self.hunters),
            bite: pick("probe-bite", self.bite),
            appetite: pick("probe-appetite", self.appetite),
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
            Shape::Choice,
            vec![Effect::Eat {
                from: Binding::Target,
                amount: self.bite,
                into: body.clone(),
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
