// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Graft compatibility, re-expressed from Mesocosm's (wing ruling 755): the
//! bounded allowance for material a body keeps across a crossing its
//! affinity refuses. Not [`super::Affinity`]'s answer about whether a part
//! may cross; a world rule over what refused material a body may retain,
//! and what that costs. Evaluating it changes nothing.

use super::{Affinity, Verdict};
use crate::schema::{Entity, Key, Lineage};
use crate::{Result, rules::AccountKind, rules::Rules};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The most conditions that may raise an allowance, as Mesocosm's rule.
pub const RAISES: usize = 2;

/// A held condition, by key, that raises the allowance by `additional_mg`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Raise {
    pub condition: Key,
    pub additional_mg: u64,
}

/// One recipient cell's worth plus `allowance_mg`, raised by the conditions
/// held, each refused milligram priced at `penalty_per_mg`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    pub allowance_cells: u32,
    pub allowance_mg: u64,
    pub penalty_per_mg: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raised_by: Vec<Raise>,
}

impl Default for Compatibility {
    /// Mesocosm's provisional rule: one recipient cell, charged one for one.
    fn default() -> Self {
        Self {
            allowance_cells: 1,
            allowance_mg: 0,
            penalty_per_mg: 1,
            raised_by: vec![],
        }
    }
}

/// What an evaluation found, to record beside the act it priced.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraftReceipt {
    pub base_allowance_mg: u64,
    pub effective_allowance_mg: u64,
    pub incoming_mg: u64,
    pub retained_mg: u64,
    pub penalty_mg: u64,
    pub applied: Vec<Key>,
}

impl Compatibility {
    /// Checks `incoming_mg` of refused material against the `retained_mg` a
    /// body already keeps, a recipient cell weighing `cell_mg`; `holds`
    /// answers whether a named condition holds, in declared order.
    pub fn evaluate(
        &self,
        incoming_mg: u64,
        retained_mg: u64,
        cell_mg: u64,
        holds: impl Fn(&str) -> bool,
    ) -> Result<GraftReceipt> {
        if self.raised_by.len() > RAISES {
            return Err(format!("at most {RAISES} conditions raise an allowance"));
        }
        let overflow = || "graft compatibility overflow".to_string();
        let requested = incoming_mg.checked_add(retained_mg).ok_or_else(overflow)?;
        let base = u64::from(self.allowance_cells)
            .checked_mul(cell_mg)
            .and_then(|cells| cells.checked_add(self.allowance_mg))
            .ok_or_else(overflow)?;
        let mut effective = base;
        let mut applied = vec![];
        for raise in self.raised_by.iter().filter(|r| holds(&r.condition)) {
            effective = effective
                .checked_add(raise.additional_mg)
                .ok_or_else(overflow)?;
            applied.push(raise.condition.clone());
        }
        if requested > effective {
            return Err(format!(
                "{requested} mg of refused material over an allowance of {effective} mg"
            ));
        }
        Ok(GraftReceipt {
            base_allowance_mg: base,
            effective_allowance_mg: effective,
            incoming_mg,
            retained_mg,
            penalty_mg: incoming_mg
                .checked_mul(self.penalty_per_mg)
                .ok_or_else(overflow)?,
            applied,
        })
    }
}

/// The matter `e`'s living parts keep whose lineage's tissue domain is
/// refused into `into`: the retained material an allowance is weighed
/// against. Matter of a lineage with no domain is not refused.
pub fn retained(
    e: &Entity,
    rules: &Rules,
    lineages: &BTreeMap<Key, Lineage>,
    affinity: &Affinity,
    into: u16,
) -> u64 {
    let domain = |key: &str| -> Option<u16> {
        let Some(AccountKind::Matter { lineage, .. }) = rules.accounts.get(key) else {
            return None;
        };
        Some(lineages.get(lineage)?.development.as_ref()?.domain)
    };
    e.living()
        .flat_map(|(_, p)| p.matter.iter())
        .filter(|(k, _)| domain(k).is_some_and(|d| affinity.verdict(d, into) == Verdict::Refused))
        .map(|(_, v)| *v)
        .fold(0, u64::saturating_add)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_allows_one_cell_and_prices_each_milligram() {
        let r = Compatibility::default()
            .evaluate(7, 0, 7, |_| false)
            .unwrap();
        assert_eq!((r.effective_allowance_mg, r.penalty_mg), (7, 7));
        assert!(
            Compatibility::default()
                .evaluate(8, 0, 7, |_| false)
                .is_err()
        );
    }

    #[test]
    fn a_held_condition_raises_the_allowance() {
        let rule = Compatibility {
            raised_by: vec![Raise {
                condition: "trait:symbiont".into(),
                additional_mg: 5,
            }],
            ..Compatibility::default()
        };
        assert!(rule.evaluate(10, 0, 7, |_| false).is_err());
        let r = rule.evaluate(10, 0, 7, |k| k == "trait:symbiont").unwrap();
        assert_eq!(r.applied, vec!["trait:symbiont".to_string()]);
    }
}
