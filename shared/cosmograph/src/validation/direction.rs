// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn effect(rules: &Rules, process: &Process, from: crate::schema::Id,
    destinations: &std::collections::BTreeSet<crate::schema::Id>) -> Result<()> {
    if rules.spread_capacity.is_none() || process.causation != Causation::Choice {
        return Err("territory direction needs declared capacity and an authored native Choice process".into());
    }
    if destinations.is_empty() || destinations.contains(&from) || process.target.is_some() {
        return Err("territory direction needs explicit outgoing destinations without a body target".into());
    }
    let all = process.commitments.iter().chain(&process.effects);
    if all.clone().filter(|e| matches!(e, Effect::DirectOverflow { .. })).count() != 1 {
        return Err("one authored native process directs its territory once".into());
    }
    fn nested(effect: &Effect) -> bool {
        effect.branches().any(|branch| matches!(branch, Effect::DirectOverflow { .. }) || nested(branch))
    }
    if all.into_iter().any(nested) || process.risk.as_ref().is_some_and(|risk|
        risk.effects.iter().any(|effect| matches!(effect, Effect::DirectOverflow { .. }) || nested(effect))) {
        return Err("conditional or risky direction needs a separately authored explicit Choice process".into());
    }
    Ok(())
}
