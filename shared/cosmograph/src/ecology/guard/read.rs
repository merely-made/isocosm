// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the guard rule reads of a bite (`super`): pure readings over the
//! prey's parts and the eater's mouth, so the meal's glue states no number.

use super::{Guard, SECRETE};
use crate::{
    mosaic::ports::active,
    process::{IntakePort, Strength, Tolerance},
    schema::{Entity, Ledger, PartId},
};
use serde::{Deserialize, Serialize};

/// What a ward is, on a harm record (`flows::Harm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ward {
    /// Armour withheld some of a bite: the record's `withheld`.
    Armour,
    /// Spines or stings wounded the eater.
    Spines,
    /// A gland's toxin wounded the eater.
    Toxin,
}

/// The unit of a resisted bite: the bite takes `TENTHS` less the resistance,
/// in tenths.
const TENTHS: u64 = 10;
/// The most armour resists: nine tenths, so no armour is proof against a bite.
const MOST: u64 = 9;

impl Guard {
    /// The attack `mouth` bites with: its own, or, where it declares none,
    /// the founded crush of [`Guard::bite`].
    #[must_use]
    pub fn attack(&self, mouth: IntakePort) -> Strength {
        let own = mouth.strength();
        if own.crush == 0 && own.pierce == 0 { Strength { crush: self.bite, ..Strength::NONE } } else { own }
    }

    /// The armour of part `id` of `prey` as a bite by `attack` meets it, in
    /// tenths of the bite it resists: the part's armour of the attack's
    /// stronger kind, scaled by the cells it still holds, less the attack.
    #[must_use]
    pub fn resisted(&self, prey: &Entity, id: PartId, attack: Strength) -> u64 {
        let Some(armour) = organ(prey, id).map(|(s, held, whole)| scaled(s, held, whole)) else { return 0 };
        let by_crush = attack.crush >= attack.pierce;
        let defence = if by_crush { armour.crush } else { armour.pierce };
        u64::from(defence.saturating_sub(attack.crush.max(attack.pierce))).min(MOST)
    }

    /// Takes `resisted` tenths off a bite: each account's share scaled to
    /// `(share * (10 - resisted) + luck % 10) / 10`, which is the tenths' worth
    /// on average and a whole number always, so a bite of one unit against
    /// four tenths of armour is taken six times in ten. `luck` is the bite's
    /// own draw (the meal's, keyed by the act), so no state is read and both
    /// executions agree. Returns the matter withheld.
    pub fn withhold(taken: &mut Ledger, resisted: u64, luck: u64) -> u64 {
        let before: u64 = taken.values().sum();
        if resisted == 0 || before == 0 {
            return 0;
        }
        for n in taken.values_mut() {
            *n = (*n * (TENTHS - resisted) + luck % TENTHS) / TENTHS;
        }
        taken.retain(|_, n| *n > 0);
        before - taken.values().sum::<u64>()
    }

    /// The cells of wound the part bitten costs its eater for its spines:
    /// none where it has none, or the eater's body tolerates them, or its
    /// mouth's pierce attack is at least theirs.
    #[must_use]
    pub fn spined(&self, prey: &Entity, id: PartId, attack: Strength, body: IntakePort) -> u32 {
        let spine = organ(prey, id).map_or(0, |(s, _, _)| s.spine);
        if spine == 0 || body.tolerates(Tolerance::Spines) || attack.pierce >= spine { 0 } else { self.spine_harm }
    }

    /// The cells of wound eating `prey` costs for its glands: the load of its
    /// secreting cells over the dose, at least one where there is any, at most
    /// the cap; none where the eater's body tolerates it.
    #[must_use]
    pub fn toxic(&self, prey: &Entity, body: IntakePort) -> u32 {
        if body.tolerates(Tolerance::Toxin) {
            return 0;
        }
        let load: u64 = prey
            .living()
            .map(|(_, p)| {
                let cells = u64::from(p.cells.get(SECRETE).copied().unwrap_or(0));
                let strength = p.port.map(|port| port.strength().toxin).filter(|s| *s > 0).unwrap_or(self.gland);
                cells * u64::from(strength)
            })
            .sum();
        match load {
            0 => 0,
            load => u32::try_from(load.div_ceil(u64::from(self.dose))).unwrap_or(u32::MAX).clamp(1, self.toxin_cap),
        }
    }
}

/// The guard organ on part `id`: its strength, the cells it holds of the
/// function under it, and the part's cells, where the port is active and is
/// not a mouth (a mouth's strength is its attack, not its defence).
fn organ(prey: &Entity, id: PartId) -> Option<(Strength, u32, u32)> {
    let part = prey.parts.get(&id).filter(|_| prey.lives(id))?;
    let port = active(part).filter(|port| !port.is_mouth() && !port.strength().is_none())?;
    let function = crate::mosaic::ports::supporting(port)?;
    let held = part.cells.get(&function).copied().unwrap_or(0);
    Some((port.strength(), held, crate::anatomy::capacity(prey.extent(id)).max(1)))
}

/// `s` as a part holding `held` of `whole` cells of its tissue has it: in
/// proportion, and never nought while any cell is held.
fn scaled(s: Strength, held: u32, whole: u32) -> Strength {
    let by = |n: u8| match n {
        0 => 0,
        n => u8::try_from((u32::from(n) * held.min(whole) / whole).max(1)).unwrap_or(n),
    };
    Strength { crush: by(s.crush), pierce: by(s.pierce), spine: s.spine, toxin: s.toxin }
}
