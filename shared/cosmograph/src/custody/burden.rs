// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    pub fn item_matter(&self, item: Id) -> Result<u128> {
        let entity = self.state.population.get(item).ok_or("carried item is absent")?;
        crate::anatomy::books(entity).iter().try_fold(0u128, |total, (key, amount)| {
            let matter = matches!(self.genesis.rules.accounts.get(key), Some(AccountKind::Matter { .. }))
                || self.state.spreads.identities.iter().any(|identity| identity.key() == *key);
            if matter { total.checked_add(u128::from(*amount)).ok_or_else(|| "item matter overflows".into()) }
            else { Ok(total) }
        })
    }

    pub fn item_burden(&self, actor: Id) -> Result<u128> {
        let mut total = 0u128;
        let links = self.custody_links()?;
        let follows = |mut item| {
            while let Some(carrier) = links.get(&item) {
                if *carrier == actor { return true; }
                item = *carrier;
            }
            false
        };
        for item in links.keys() {
            if follows(*item) {
                total = total.checked_add(self.item_matter(*item)?).ok_or("carried item burden overflows")?;
            }
        }
        for (id, project) in &self.state.projects.records {
            if project.carrier.is_some_and(|carrier| carrier == actor || follows(carrier)) {
                total = total.checked_add(self.project_matter(*id)?).ok_or("carried project burden overflows")?;
            }
        }
        Ok(total)
    }

    pub fn item_capacity(&self, actor: Id, provider: &str) -> Result<u64> {
        let entity = self.state.population.get(actor).ok_or("item carrier is absent")?;
        let expression = &self.custody_provider(provider)?.capacity;
        let rules = &self.genesis.rules;
        let mut read = |reading: &Reading| -> Result<i64> {
            if reading.who() != Binding::Actor || matches!(reading, Reading::Kept { .. }) {
                return Err("item capacity reads an unsupported binding".into());
            }
            match reading {
                Reading::Account { key, .. } => i64::try_from(crate::anatomy::held(entity, rules, key))
                    .map_err(|error| error.to_string()),
                other => Ok(crate::meaning::body_reading(entity, other, rules, Some(&self.state.lineages))),
            }
        };
        let mut parts = |who| {
            if who != Binding::Actor { return Err("item capacity sums an unsupported body".into()); }
            Ok((entity.living().map(|(id, part)| (entity.extent(id), part.clone())).collect(), rules.body()))
        };
        let value = expression.eval_in(&mut read, &mut |_, _| Err("item capacity cannot draw".into()), &mut parts)?;
        Ok(value.max(0) as u64)
    }
}
