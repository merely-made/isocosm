// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::physiology::{c, due};
use super::*;

/// Inclusive per-world draws. Hazard is a numerator over 64 ticks;
/// cells and rot are amounts per accepted hit and rotting act.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarmFounding {
    pub hazard: [u64; 2],
    pub cells: [u32; 2],
    pub rot: [u64; 2],
    pub heals: u64,
    pub fragments: u64,
}

impl Default for HarmFounding {
    fn default() -> Self {
        Self {
            hazard: [2, 8],
            cells: [1, 8],
            rot: [1, 8],
            heals: 500,
            fragments: 500,
        }
    }
}

impl HarmFounding {
    pub(super) fn valid(&self) -> bool {
        self.hazard[0] <= self.hazard[1]
            && self.hazard[1] <= MAX_DRAW
            && self.cells[0] <= self.cells[1]
            && self.cells[0] > 0
            && self.rot[0] <= self.rot[1]
            && self.heals <= 1000
            && self.fragments <= 1000
    }

    pub(super) fn processes(&self, f: &BodyFounding) -> [Process; 2] {
        let rate = f.pick("body-hazard", 0, self.hazard) as i64;
        let cells = f.pick("body-wound-cells", 0, self.cells.map(u64::from));
        let hits = Effect::When {
            guard: Expr::AtLeast(
                Box::new(c(rate - 1)),
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
        let mut hazard = due("body:hazard", Causation::Agentless, vec![hits], 0);
        hazard.target = Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: None,
            among: BTreeSet::new(),
            weighted: true,
        });
        hazard.requires.push(Query::Computed(Expr::AtLeast(
            Box::new(Expr::Read(Reading::LivingCells {
                who: Binding::Target,
            })),
            Box::new(c(1)),
        )));
        let effect = Effect::Rot {
            who: Binding::Target,
            amount: f.pick("body-rot", 0, self.rot).into(),
        };
        let mut rot = due("body:rot", Causation::Agentless, vec![effect], 7);
        rot.target = Some(Target {
            same_place: true,
            alive: Some(false),
            lineage: None,
            among: BTreeSet::new(),
            weighted: true,
        });
        [hazard, rot]
    }
}
