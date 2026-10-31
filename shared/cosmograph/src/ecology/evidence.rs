// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{AccountKind, Rules};

/// Disposable observer state, never native state, hashes, or saved authority.
#[derive(Clone, Debug)]
pub(crate) struct JointCapture {
    pub spec: JointSpec,
    pub acts: Vec<ActionProof>,
    pub kept: usize,
}

impl JointSpec {
    pub(crate) fn totals(&self, acts: &[ActionProof], rules: &Rules) -> crate::Result<JointTotals> {
        let mut totals = JointTotals { captured: 0, returned: 0, births: 0, counter: 0 };
        let add = |total: &mut u128, amount: u128| -> crate::Result<()> {
            *total = total.checked_add(amount).ok_or("joint witness quantity overflow")?;
            Ok(())
        };
        for act in acts {
            if self.capture.selection.matches(act) {
                for credit in &act.credits {
                    if credit.holder != crate::flows::Holder::Entity(act.actor) { continue; }
                    let matches = match &self.capture.stock {
                        CaptureStock::EnergyAccount(key) => key == &credit.account
                            && matches!(rules.accounts.get(key), Some(AccountKind::Energy)),
                        CaptureStock::OwnReserve(key) => key == &credit.account
                            && matches!(rules.accounts.get(key), Some(AccountKind::Matter { lineage, reserve: true, .. })
                                if lineage == &act.actor_lineage)
                            && !credit.parts.is_empty()
                            && credit.parts.iter().map(|(_, n)| u128::from(*n)).sum::<u128>() == u128::from(credit.amount),
                    };
                    if matches { add(&mut totals.captured, u128::from(credit.fresh) * u128::from(act.count))?; }
                }
            }
            if self.returned.selection.matches(act) {
                for credit in &act.credits {
                    if credit.account == self.returned.account
                        && act.site.is_some_and(|site| credit.holder == crate::flows::Holder::Site(site)) {
                        add(&mut totals.returned, u128::from(credit.fresh) * u128::from(act.count))?;
                    }
                }
            }
            if self.reproduction.selection.matches(act) { add(&mut totals.births, act.births.len() as u128)?; }
            if self.counterplay.selection.matches(act) { add(&mut totals.counter, u128::from(act.counter))?; }
        }
        Ok(totals)
    }

    pub fn missing(&self, totals: &JointTotals) -> Vec<String> {
        let counter = match &self.counterplay.proof {
            CounterProof::TargetDied => 1,
            CounterProof::LivingCellsLost { at_least } | CounterProof::TargetDebited { at_least, .. } => *at_least,
        };
        [("energy capture", totals.captured, self.capture.at_least),
            ("matter return", totals.returned, self.returned.at_least),
            ("biological reproduction", totals.births, self.reproduction.at_least),
            ("counterplay", totals.counter, counter)]
            .into_iter().filter(|(_, actual, required)| *actual < u128::from(*required))
            .map(|(role, actual, required)| format!("{role}: {actual} of {required}")).collect()
    }
}
