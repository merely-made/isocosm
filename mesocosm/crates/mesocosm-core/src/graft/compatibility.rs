// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The ruled, bounded allowance for otherwise disfavoured graft material.
//!
//! This is a world rule for retained material, not [`super::Affinity`]'s
//! answer about whether a branch may cross a tissue boundary. It has no meal
//! admission, body mutation, or world integration caller.

use serde::{Deserialize, Serialize};

use crate::discovery::{ConditionId, resolve};

/// One condition that expands a compatibility allowance when it is held.
///
/// A condition id is content-addressed. An id absent from this ruleset never
/// contributes, even if a caller says it is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConditionAllowance {
    pub condition: ConditionId,
    pub additional_mg: u64,
}

/// A compatibility rule over a proposed retained material transfer.
///
/// The base allowance is one recipient tissue cell plus an explicit amount.
/// Matching declared conditions add bounded milligrams. Two slots match the
/// current PE2 condition-table bound; a later admitted grammar may widen that
/// rule deliberately. Duplicate held condition rows contribute independently
/// in declared slot order. The resulting penalty is a separate amount a caller
/// may price; evaluating a rule has no side effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Compatibility {
    pub allowance_cells: u32,
    pub allowance_mg: u64,
    pub penalty_per_mg: u64,
    pub raised_by: [Option<ConditionAllowance>; 2],
}

impl Default for Compatibility {
    fn default() -> Self {
        Self::native()
    }
}

impl Compatibility {
    /// The provisional native rule: one recipient cell, charged one-for-one.
    pub const fn native() -> Self {
        Self {
            allowance_cells: 1,
            allowance_mg: 0,
            penalty_per_mg: 1,
            raised_by: [None; 2],
        }
    }

    /// The legacy shape: no retained disfavoured material and no price.
    pub const fn legacy_disabled() -> Self {
        Self {
            allowance_cells: 0,
            allowance_mg: 0,
            penalty_per_mg: 0,
            raised_by: [None; 2],
        }
    }

    /// Digest over every rule-bearing field in stable slot order.
    pub fn digest(&self) -> u64 {
        let mut bytes = self.allowance_cells.to_le_bytes().to_vec();
        bytes.extend_from_slice(&self.allowance_mg.to_le_bytes());
        bytes.extend_from_slice(&self.penalty_per_mg.to_le_bytes());
        for raised in self.raised_by {
            match raised {
                Some(raised) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&raised.condition.0.to_le_bytes());
                    bytes.extend_from_slice(&raised.additional_mg.to_le_bytes());
                },
                None => bytes.push(0),
            }
        }
        crate::snapshot::hash_bytes(&bytes)
    }

    /// Checks one incoming transfer against retained material already on a body.
    ///
    /// The callback answers only whether an admitted condition is held. A
    /// missing condition is deliberately not offered to it, so stale or
    /// foreign ids cannot grant an allowance by sharing a number.
    pub fn evaluate<F>(
        &self,
        incoming_mg: u64,
        retained_mg: u64,
        recipient_cell_mg: u64,
        holds: F,
    ) -> Result<CompatibilityReceipt, CompatibilityRefusal>
    where
        F: Fn(ConditionId) -> bool,
    {
        let requested_mg = incoming_mg
            .checked_add(retained_mg)
            .ok_or(CompatibilityRefusal::Overflow)?;
        let base_allowance_mg = u64::from(self.allowance_cells)
            .checked_mul(recipient_cell_mg)
            .and_then(|cells| cells.checked_add(self.allowance_mg))
            .ok_or(CompatibilityRefusal::Overflow)?;
        let mut effective_allowance_mg = base_allowance_mg;
        let mut applied = Vec::new();
        for raised in self.raised_by.into_iter().flatten() {
            if resolve(raised.condition).is_some() && holds(raised.condition) {
                effective_allowance_mg = effective_allowance_mg
                    .checked_add(raised.additional_mg)
                    .ok_or(CompatibilityRefusal::Overflow)?;
                applied.push(raised.condition);
            }
        }
        if requested_mg > effective_allowance_mg {
            return Err(CompatibilityRefusal::OverAllowance {
                requested_mg,
                allowance_mg: effective_allowance_mg,
            });
        }
        let penalty_mg = incoming_mg
            .checked_mul(self.penalty_per_mg)
            .ok_or(CompatibilityRefusal::Overflow)?;
        Ok(CompatibilityReceipt {
            base_allowance_mg,
            effective_allowance_mg,
            incoming_mg,
            retained_mg,
            requested_mg,
            penalty_mg,
            applied,
        })
    }
}

/// The complete calculation a caller can record beside a later transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityReceipt {
    pub base_allowance_mg: u64,
    pub effective_allowance_mg: u64,
    pub incoming_mg: u64,
    pub retained_mg: u64,
    pub requested_mg: u64,
    pub penalty_mg: u64,
    pub applied: Vec<ConditionId>,
}

/// Why a compatibility calculation cannot authorize a transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityRefusal {
    OverAllowance {
        requested_mg: u64,
        allowance_mg: u64,
    },
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::conditions;

    #[test]
    fn native_rule_allows_one_recipient_cell_and_prices_each_milligram() {
        let receipt = Compatibility::native()
            .evaluate(7, 0, 7, |_| false)
            .unwrap();
        assert_eq!(
            receipt,
            CompatibilityReceipt {
                base_allowance_mg: 7,
                effective_allowance_mg: 7,
                incoming_mg: 7,
                retained_mg: 0,
                requested_mg: 7,
                penalty_mg: 7,
                applied: Vec::new(),
            }
        );
    }

    #[test]
    fn held_named_conditions_raise_the_allowance_in_declared_order() {
        let condition = conditions()[0].id();
        let rule = Compatibility {
            allowance_cells: 1,
            allowance_mg: 2,
            penalty_per_mg: 3,
            raised_by: [
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 5,
                }),
                None,
            ],
        };
        let receipt = rule.evaluate(11, 0, 4, |found| found == condition).unwrap();
        assert_eq!(receipt.base_allowance_mg, 6);
        assert_eq!(receipt.effective_allowance_mg, 11);
        assert_eq!(receipt.penalty_mg, 33);
        assert_eq!(receipt.applied, vec![condition]);
    }

    #[test]
    fn unknown_condition_never_calls_holds_or_grants_an_allowance() {
        let rule = Compatibility {
            allowance_cells: 0,
            allowance_mg: 2,
            penalty_per_mg: 1,
            raised_by: [
                Some(ConditionAllowance {
                    condition: ConditionId(u64::MAX),
                    additional_mg: 9,
                }),
                None,
            ],
        };
        let receipt = rule
            .evaluate(2, 0, 1, |_| panic!("unknown conditions are not offered"))
            .unwrap();
        assert_eq!(receipt.effective_allowance_mg, 2);
        assert!(receipt.applied.is_empty());
    }

    #[test]
    fn request_past_the_effective_allowance_is_refused_by_name() {
        assert_eq!(
            Compatibility::legacy_disabled().evaluate(1, 0, 10, |_| false),
            Err(CompatibilityRefusal::OverAllowance {
                requested_mg: 1,
                allowance_mg: 0,
            })
        );
    }

    #[test]
    fn checked_arithmetic_refuses_allowance_and_penalty_overflow() {
        let allowance = Compatibility {
            allowance_cells: u32::MAX,
            allowance_mg: 1,
            penalty_per_mg: 0,
            raised_by: [None; 2],
        };
        assert_eq!(
            allowance.evaluate(0, 0, u64::MAX, |_| false),
            Err(CompatibilityRefusal::Overflow)
        );
        let penalty = Compatibility {
            allowance_cells: 0,
            allowance_mg: u64::MAX,
            penalty_per_mg: 2,
            raised_by: [None; 2],
        };
        assert_eq!(
            penalty.evaluate(u64::MAX, 0, 1, |_| false),
            Err(CompatibilityRefusal::Overflow)
        );
    }

    #[test]
    fn retained_material_counts_toward_the_cap_but_not_the_new_penalty() {
        let receipt = Compatibility::native()
            .evaluate(3, 4, 7, |_| false)
            .unwrap();
        assert_eq!((receipt.requested_mg, receipt.penalty_mg), (7, 3));
    }

    #[test]
    fn duplicate_held_rows_add_in_declared_slot_order() {
        let condition = conditions()[0].id();
        let rule = Compatibility {
            allowance_cells: 0,
            allowance_mg: 1,
            penalty_per_mg: 0,
            raised_by: [
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 2,
                }),
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 3,
                }),
            ],
        };
        let receipt = rule.evaluate(6, 0, 1, |_| true).unwrap();
        assert_eq!(receipt.effective_allowance_mg, 6);
        assert_eq!(receipt.applied, vec![condition, condition]);
    }

    #[test]
    fn digest_moves_for_every_rule_bearing_field_and_slot_presence() {
        let base = Compatibility::native();
        let condition = conditions()[0].id();
        let other_condition = conditions()[1].id();
        let variations = [
            Compatibility {
                allowance_cells: 2,
                ..base
            },
            Compatibility {
                allowance_mg: 1,
                ..base
            },
            Compatibility {
                penalty_per_mg: 2,
                ..base
            },
            Compatibility {
                raised_by: [
                    Some(ConditionAllowance {
                        condition,
                        additional_mg: 1,
                    }),
                    None,
                ],
                ..base
            },
            Compatibility {
                raised_by: [
                    None,
                    Some(ConditionAllowance {
                        condition,
                        additional_mg: 1,
                    }),
                ],
                ..base
            },
        ];
        for changed in variations {
            assert_ne!(base.digest(), changed.digest());
        }
        let first = Compatibility {
            raised_by: [
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 1,
                }),
                None,
            ],
            ..base
        };
        let changed_condition = Compatibility {
            raised_by: [
                Some(ConditionAllowance {
                    condition: other_condition,
                    additional_mg: 1,
                }),
                None,
            ],
            ..base
        };
        let changed_bonus = Compatibility {
            raised_by: [
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 2,
                }),
                None,
            ],
            ..base
        };
        assert_ne!(first.digest(), changed_condition.digest());
        assert_ne!(first.digest(), changed_bonus.digest());
    }

    #[test]
    fn serialized_rule_round_trips() {
        let rule = Compatibility::native();
        let bytes = crate::snapshot::encode(&rule).unwrap();
        assert_eq!(
            crate::snapshot::decode::<Compatibility>(&bytes).unwrap(),
            rule
        );
    }

    #[test]
    fn receipt_round_trips() {
        let receipt = Compatibility::native()
            .evaluate(3, 0, 3, |_| false)
            .unwrap();
        let bytes = crate::snapshot::encode(&receipt).unwrap();
        assert_eq!(
            crate::snapshot::decode::<CompatibilityReceipt>(&bytes).unwrap(),
            receipt
        );
    }
}
