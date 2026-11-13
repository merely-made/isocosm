// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual exposed, admitted part offerings. Role labels, appetite and
//! process clocks do not participate in the food contract.

use super::*;
use crate::rules::{Binding, Query};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoodOffering {
    pub part: Option<PartId>,
    pub offered: Ledger,
}

impl Process {
    pub fn food_offerings(
        &self,
        actor: &Entity,
        prey: &Entity,
        rules: &Rules,
        lineages: &std::collections::BTreeMap<Key, Lineage>,
    ) -> Result<Vec<FoodOffering>> {
        if crate::directing::is_participant(prey) || prey.journey.is_some() {
            return Ok(vec![]);
        }
        let feeding = self
            .port_feeding
            .as_ref()
            .ok_or("food contract has no intake")?;
        let (id, _) = feeding.part(actor)?;
        let intake = feeding.intake(actor, id)?;
        if ports::active(intake).is_none() {
            return Err("food contract intake is dormant".into());
        }
        if let Some(target) = &self.target {
            if target.alive.is_some_and(|alive| alive != prey.alive)
                || target
                    .lineage
                    .as_ref()
                    .is_some_and(|lineage| *lineage != prey.lineage)
                || (!target.among.is_empty() && !target.among.contains(&prey.lineage))
            {
                return Ok(vec![]);
            }
        }
        let Some(Effect::Eat { of, .. }) = self.effects.first() else {
            return Err("food contract has no ordinary Eat".into());
        };
        let offered = |part: Option<PartId>, ledger: &Ledger| -> Result<Option<FoodOffering>> {
            let raw = crate::schedule::edible(ledger, of);
            let mut food = feeding.offered(intake, &raw, ports::admission::Eaten::part(prey, part, rules), rules, lineages)?;
            food.retain(|key, n| {
                *n > 0
                    && matches!(
                        rules.accounts.get(key),
                        Some(crate::rules::AccountKind::Matter { .. })
                    )
            });
            Ok((!food.is_empty()).then_some(FoodOffering {
                part,
                offered: food,
            }))
        };
        if !prey.living().any(|(id, _)| prey.bodied(id)) {
            return Ok(offered(None, &prey.accounts)?.into_iter().collect());
        }
        prey.living()
            .filter(|(id, _)| {
                prey.bodied(*id) && crate::anatomy::root_store::exposed(prey, rules, *id)
            })
            .filter_map(|(id, part)| offered(Some(id), &part.matter).transpose())
            .collect()
    }

    pub(crate) fn food_weight(
        &self,
        actor: &Entity,
        prey: &Entity,
        rules: &Rules,
        lineages: &std::collections::BTreeMap<Key, Lineage>,
    ) -> u128 {
        self.food_offerings(actor, prey, rules, lineages)
            .map_or(0, |parts| {
                parts
                    .iter()
                    .flat_map(|p| p.offered.values())
                    .map(|n| u128::from(*n))
                    .sum()
            })
    }
}

/// A new meal contract also declares its inherited retained tissue. This
/// keeps that tissue in parts after native Speciate changes holder lineage.
pub(crate) fn retained(e: &Entity, rules: &Rules, key: &str) -> bool {
    rules.processes.values().any(|p| {
        p.port_feeding.as_ref().is_some_and(|f| f.meal.is_some())
            && matches!(p.effects.as_slice(), [Effect::Eat { into, .. }] if into == key)
            && p.requires.iter().any(|q| {
                matches!(
                    q,
                    Query::Trait {
                        who: Binding::Actor,
                        ..
                    }
                )
            })
            && p.requires
                .iter()
                .filter_map(|q| match q {
                    Query::Trait {
                        who: Binding::Actor,
                        key,
                    } => Some(key),
                    _ => None,
                })
                .all(|trait_key| e.traits.contains(trait_key))
    })
}
