// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored route duration and exact native stock costs (860).

use super::*;
use crate::Result;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaceProfile {
    pub duration: PaceDuration,
    pub costs: Vec<PaceCost>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaceDuration {
    Ticks { ticks: Tick },
    RouteScale { numerator: u64, denominator: u64 },
}

/// Ordered actor stocks paid through the existing native Spend meaning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaceCost {
    pub from: Vec<Key>,
    pub to: Binding,
    pub into: Option<Key>,
    pub amount: u64,
}

impl PaceProfile {
    /// Native ticks are exact integers; an authored fraction is never rounded.
    pub fn duration(&self, route_travel: Tick) -> Result<Tick> {
        if route_travel == 0 { return Err("route travel must be positive".into()); }
        let ticks = match self.duration {
            PaceDuration::Ticks { ticks } => ticks,
            PaceDuration::RouteScale { numerator, denominator } => {
                if numerator == 0 || denominator == 0 {
                    return Err("pace route scale must be positive".into());
                }
                let product = u128::from(route_travel) * u128::from(numerator);
                if product % u128::from(denominator) != 0 {
                    return Err("pace duration is not a whole native tick".into());
                }
                u64::try_from(product / u128::from(denominator))
                    .map_err(|_| "pace duration overflows native ticks")?
            },
        };
        if ticks == 0 { return Err("pace duration must be positive".into()); }
        Ok(ticks)
    }

    pub fn validate(&self, rules: &Rules) -> Result<()> {
        match self.duration {
            PaceDuration::Ticks { ticks: 0 }
                | PaceDuration::RouteScale { numerator: 0, .. }
                | PaceDuration::RouteScale { denominator: 0, .. } => {
                return Err("pace duration must be positive".into());
            },
            _ => {},
        }
        if self.costs.len() > rules.limits.operations {
            return Err("pace costs exceed the world's operation limit".into());
        }
        for cost in &self.costs {
            if cost.from.is_empty() || cost.from.iter().collect::<BTreeSet<_>>().len() != cost.from.len() {
                return Err("pace costs need distinct ordered source accounts".into());
            }
            for key in &cost.from {
                if !rules.accounts.contains_key(key) { return Err(format!("unknown pace source account {key}")); }
            }
            if !matches!(cost.to, Binding::Place | Binding::Target) {
                return Err("pace costs must pay a place or target".into());
            }
            if let Some(into) = &cost.into {
                if !matches!(rules.accounts.get(into), Some(AccountKind::Matter { .. }))
                    || cost.from.iter().any(|key| !matches!(rules.accounts.get(key), Some(AccountKind::Matter { .. })))
                {
                    return Err("pace mineralization must move matter only".into());
                }
            }
        }
        Ok(())
    }
}

impl PaceCost {
    pub(crate) fn effect(&self) -> Effect {
        Effect::Spend {
            from: self.from.clone(), to: self.to, into: self.into.clone(),
            amount: Amount::Fixed(self.amount),
        }
    }
}

impl Rules {
    /// A finite adapter may select one direct authored route, not a branch or
    /// risk outcome. Actual process requirements and debit admission still run.
    pub fn paced_movement(&self, process: &str, destination: Id) -> Result<(&Key, &PaceProfile)> {
        fn moves(effect: &Effect) -> bool {
            matches!(effect, Effect::Move { .. } | Effect::MovePaced { .. })
                || effect.branches().any(moves)
        }
        let process = self.processes.get(process).ok_or("unknown movement process")?;
        let mut selected = None;
        for effect in process.commitments.iter().chain(&process.effects) {
            match effect {
                Effect::MovePaced { destination: to, profile } if *to == destination && selected.is_none() => {
                    selected = Some(profile);
                },
                other if moves(other) => return Err("movement process has another or conditional route".into()),
                _ => {},
            }
        }
        if process.risk.iter().flat_map(|risk| &risk.effects).any(moves) {
            return Err("movement process has a risky route".into());
        }
        let name = selected.ok_or("process has no direct paced route to destination")?;
        Ok((name, self.pace_profiles.get(name).ok_or("unknown pace profile")?))
    }
}

pub(crate) fn validate(rules: &Rules) -> Result<()> {
    for (name, profile) in &rules.pace_profiles {
        crate::validation::key(name)?;
        profile.validate(rules)?;
    }
    Ok(())
}

pub(crate) fn validate_effect(rules: &Rules, process: &Process, name: &str) -> Result<()> {
    let profile = rules.pace_profiles.get(name).ok_or_else(|| format!("unknown pace profile {name}"))?;
    if profile.costs.iter().any(|cost| cost.to == Binding::Target) && process.target.is_none() {
        return Err("paced movement costs require an authored target".into());
    }
    Ok(())
}

pub(crate) fn validate_lineages(rules: &Rules, lineages: &BTreeMap<Key, Lineage>) -> Result<()> {
    let world = |key: &Key| matches!(rules.accounts.get(key), Some(AccountKind::Matter { lineage, .. })
        if lineages.get(lineage).is_some_and(|lineage| lineage.kingdom == "kingdom:world"));
    for profile in rules.pace_profiles.values() {
        for cost in &profile.costs {
            if let Some(into) = &cost.into
                && (cost.from.iter().all(|key| world(key)) || !world(into))
            {
                return Err("pace mineralization must return living matter to world matter".into());
            }
        }
    }
    Ok(())
}
