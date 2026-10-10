// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's readings of the moves: a body's accounts (DT2) and the
//! stand's trend (plan §3), facts and their windows, never a verdict.

use super::Process;
use crate::flows::Flow;
use crate::legacy::mesocosm::organism::OrganismId;
use serde::{Deserialize, Serialize};

/// What one body earned and spent over a window: what reached it from
/// outside, what left it as rent, and what left it any other way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accounts {
    pub ticks: u64,
    pub income_mg: u64,
    pub rent_mg: u64,
    pub outflow_mg: u64,
}

impl Accounts {
    /// Folds one tick's moves in; a tick with none still widens the window.
    pub fn absorb(&mut self, organism: OrganismId, flows: &[Flow]) {
        self.ticks = self.ticks.saturating_add(1);
        for flow in flows.iter().filter(|f| !f.internal()) {
            if flow.to_organism() == Some(organism) {
                self.income_mg = self.income_mg.saturating_add(flow.amount);
            }
            if flow.from_organism() == Some(organism) {
                match flow.process() {
                    Some(Process::Upkeep) => {
                        self.rent_mg = self.rent_mg.saturating_add(flow.amount)
                    },
                    _ => self.outflow_mg = self.outflow_mg.saturating_add(flow.amount),
                }
            }
        }
    }

    pub fn net_mg(self) -> i128 {
        i128::from(self.income_mg) - i128::from(self.rent_mg)
    }
}

/// Consecutive ticks the stand must read short before the game says so:
/// one whole sixty-tick window, measured on eight seeds of the shipping
/// roster (`mesocosm-runtime/tests/readings.rs`).
pub const WARN_AFTER_TICKS: u64 = 60;

/// What the bounded windows currently read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trend {
    /// Ticks the replacement counts cover.
    pub replacement_ticks: u64,
    pub matured: u32,
    pub died: u32,
    /// Ticks the stand reading covers.
    pub stand_ticks: u64,
    /// The standing plant matter's change over that window: a stock trend,
    /// not a flow ratio.
    pub stand_change_mg: i64,
    /// What mouths took out of producers in the same window, stated beside
    /// the change rather than divided into it.
    pub grazed_mg: u64,
    /// Consecutive ticks the stand has read short over `stand_ticks`.
    pub shortfall_ticks: u64,
}

impl Trend {
    /// Maturation against mortality, per mille; `None` when nothing died.
    pub fn replacement_permille(&self) -> Option<u64> {
        (self.died > 0).then(|| u64::from(self.matured) * 1_000 / u64::from(self.died))
    }

    pub fn warns(&self) -> bool {
        self.shortfall_ticks >= WARN_AFTER_TICKS
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Account, Subject};
    use super::*;
    use crate::legacy::mesocosm::body::SpeciesId;
    use crate::legacy::mesocosm::organism::Kingdom;

    const A: OrganismId = OrganismId(1);

    #[test]
    fn a_bodys_accounts_separate_income_rent_and_everything_else() {
        let me = Subject {
            organism: A,
            lineage: SpeciesId(3),
            kingdom: Kingdom::Consumer,
        };
        let other = Subject {
            organism: OrganismId(2),
            lineage: SpeciesId(3),
            kingdom: Kingdom::Producer,
        };
        let (s, r) = (Account::Substance, Account::Reserve);
        let tick = vec![
            Flow::uptake(me, r, 50),
            Flow::between(Process::Feeding, other, s, me, s, 30),
            Flow::returned(Process::Upkeep, me, r, 7),
            Flow::returned(Process::Travel, me, s, 4),
            Flow::returned(Process::Develop, me, r, 9),
            Flow::uptake(other, s, 900),
            Flow::between(Process::Uptake, me, s, me, r, 20),
        ];
        let mut accounts = Accounts::default();
        accounts.absorb(A, &tick);
        assert_eq!(
            (
                accounts.ticks,
                accounts.income_mg,
                accounts.rent_mg,
                accounts.outflow_mg
            ),
            (1, 80, 7, 13)
        );
        assert_eq!(accounts.net_mg(), 73);
        accounts.absorb(A, &[]);
        assert_eq!((accounts.ticks, accounts.income_mg), (2, 80));
    }

    #[test]
    fn a_trend_states_its_windows_rather_than_a_verdict() {
        let trend = Trend {
            replacement_ticks: 240,
            matured: 6,
            died: 4,
            stand_ticks: 60,
            stand_change_mg: -7_930,
            grazed_mg: 15_771,
            shortfall_ticks: WARN_AFTER_TICKS,
        };
        assert_eq!(trend.replacement_permille(), Some(1_500));
        assert!(trend.warns());
        let quiet = Trend {
            shortfall_ticks: WARN_AFTER_TICKS - 1,
            ..trend
        };
        assert!(!quiet.warns());
        assert_eq!(Trend::default().replacement_permille(), None);
    }
}
