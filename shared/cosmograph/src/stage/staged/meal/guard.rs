// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The guard rule at a bite (`crate::ecology::guard`): what the prey part's
//! armour resists, and the wounds its spines and glands cost the eater. The
//! numbers are the rule's (`ecology::guard::read`); this reads the stage.

use super::*;
use crate::ecology::Ward;

/// What the rule makes of one bite, read before the bite changes anything.
#[derive(Clone, Copy, Debug)]
pub(super) struct Guarded {
    /// Tenths of the bite the part's armour resists.
    pub resisted: u64,
    /// Cells of wound the part's spines cost the eater.
    pub spined: u32,
    /// Cells of wound the body's glands cost the eater.
    pub toxic: u32,
}

/// A ward's harm to the eater, or the matter armour withheld.
#[derive(Clone, Copy, Debug)]
pub(super) struct Warded {
    pub ward: Ward,
    pub cells: u32,
    pub withheld: u64,
}

impl Staged<'_> {
    /// The guard rule's reading of this bite; none where the world sets no
    /// rule or the meal has no declared mouth.
    pub(super) fn guarded(&self, from: Binding) -> Result<Option<Guarded>> {
        let (Some(rule), Some(feeding), Some(id)) =
            (self.sim.genesis.rules.guard, self.stage.port_feeding.as_ref(), self.stage.intake)
        else {
            return Ok(None);
        };
        let eater = &self.stage.bodies[&self.stage.actor];
        let Some(mouth) = crate::mosaic::ports::active(feeding.intake(eater, id)?) else { return Ok(None) };
        let (attack, body) = (rule.attack(mouth), crate::mosaic::ports::ports(eater));
        let prey = sim_body(self, from)?;
        let part = self.stage.bitten;
        Ok(Some(Guarded {
            resisted: part.map_or(0, |part| rule.resisted(prey, part, attack)),
            spined: part.map_or(0, |part| rule.spined(prey, part, attack, body)),
            toxic: rule.toxic(prey, body),
        }))
    }

    /// The wards that acted on a bite that withheld `withheld` of it.
    pub(super) fn wards(g: Option<Guarded>, withheld: u64) -> Vec<Warded> {
        let Some(g) = g else { return vec![] };
        let mut out = vec![];
        if withheld > 0 {
            out.push(Warded { ward: Ward::Armour, cells: 0, withheld });
        }
        if g.spined > 0 {
            out.push(Warded { ward: Ward::Spines, cells: g.spined, withheld: 0 });
        }
        if g.toxic > 0 {
            out.push(Warded { ward: Ward::Toxin, cells: g.toxic, withheld: 0 });
        }
        out
    }
}

fn sim_body<'s>(staged: &'s Staged<'_>, who: Binding) -> Result<&'s Entity> {
    let id = staged.sim.bound(staged.stage.actor, staged.stage.target, who)?;
    Ok(staged.stage.bodies.get(&id).map(std::sync::Arc::as_ref).ok_or("body missing")?)
}
