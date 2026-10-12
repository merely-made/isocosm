// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Recipe admission for an unbodied arrival, preserving every supplied stock.

use crate::{Result, anatomy, development, geometry::Body, rules::AccountKind, schema::*, Simulation};

impl Simulation {
    /// The native body this line develops under `seed`, before stocks are issued.
    pub fn development_body(&self, lineage: &str, seed: u64) -> Result<Body> {
        let d = self.state.lineages.get(lineage)
            .and_then(|l| l.development.as_ref()).ok_or("a lineage with no recipe")?;
        let soma = development::soma(&self.genesis.rules, &d.recipe, seed);
        development::develop(&self.genesis.rules, d, &soma)
    }

    pub(crate) fn develop_arrival(&mut self, entity: Id, seed: u64) -> Result<()> {
        let held = self.state.population.get(entity).ok_or("unknown entity")?;
        if !held.alive { return Err("development of the dead".into()); }
        if held.body.is_some() || !held.parts.is_empty() {
            return Err("development replaces an existing body".into());
        }
        let d = self.state.lineages.get(&held.lineage)
            .and_then(|l| l.development.as_ref()).ok_or("a lineage with no recipe")?;
        let rules = &self.genesis.rules;
        let soma = development::soma(rules, &d.recipe, seed);
        let mut next = held.clone();
        next.embody(development::develop(rules, d, &soma)?);
        next.soma = soma.segments;
        next.systems = d.systems.clone();
        crate::systems::inherit(&mut next);
        crate::systems::vary(&mut next, rules, &d.recipe, seed);
        crate::systems::riff(&mut next, &d.recipe, seed);
        let own: Vec<(Key, u64)> = next.accounts.iter()
            .filter(|(key, amount)| **amount > 0 && matches!(rules.accounts.get(*key),
                Some(AccountKind::Matter { lineage, .. }) if *lineage == next.lineage))
            .map(|(key, amount)| (key.clone(), *amount)).collect();
        for (key, amount) in own {
            // The native allocator refuses overflow; no min, clipping or fallback.
            anatomy::give(&mut next, rules, &key, amount)
                .ok_or("a developed body with no matter-bearing parts")??;
            next.accounts.remove(&key);
        }
        if anatomy::books(&next) != anatomy::books(held) {
            return Err("development changed supplied stocks".into());
        }
        next.body_revision = next.body_revision.checked_add(1).ok_or("body revision overflow")?;
        *self.state.population.lift(entity)? = next;
        Ok(())
    }
}
