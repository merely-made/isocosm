// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An opt-in meal grammar. Its port is an actual organ, and its source
//! selector reads matter provenance rather than the target's lineage.

use crate::{Result, mosaic::ports, rules::{Binding, Effect, Process, Rules}, schema::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedingSources {
    All,
    /// An empty set admits nothing.
    Only(BTreeSet<Key>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortFeeding {
    /// The supporting native function of the addressed intake organ.
    pub function: Key,
    pub sources: FeedingSources,
}

impl PortFeeding {
    fn addresses(&self, part: &Part) -> bool {
        part.port.is_some_and(|port| match port.support() {
            Some(_) => ports::supporting(port).as_ref() == Some(&self.function),
            None => part.functions.contains(&self.function),
        })
    }
    /// Lowest live declared organ with this support. Binding does not skip a
    /// dormant port: losing its allocated cells blocks the addressed meal.
    pub(crate) fn part<'a>(&self, actor: &'a Entity) -> Result<(PartId, &'a Part)> {
        actor.living().find(|(_, part)| self.addresses(part))
            .ok_or_else(|| format!("no declared intake organ for {}", self.function))
    }

    pub(crate) fn intake<'a>(&self, actor: &'a Entity, id: PartId) -> Result<&'a Part> {
        actor.parts.get(&id).map(std::sync::Arc::as_ref).filter(|p| actor.lives(id) && self.addresses(p))
            .ok_or_else(|| "the bound intake organ is no longer declared or living".into())
    }

    pub(crate) fn offered(
        &self, intake: &Part, holdings: &Ledger, alive: bool, rules: &Rules,
        lineages: &std::collections::BTreeMap<Key, Lineage>,
    ) -> Result<Ledger> {
        use ports::admission::{Sources, offered};
        let sources = match &self.sources {
            FeedingSources::All => Sources::All,
            FeedingSources::Only(lines) => Sources::Only(lines),
        };
        offered(intake, sources, holdings, alive, rules, lineages)
    }
}

impl Process {
    pub(crate) fn validate_feeding_sources(
        &self, lineages: &std::collections::BTreeMap<Key, Lineage>,
    ) -> Result<()> {
        if let Some(PortFeeding { sources: FeedingSources::Only(lines), .. }) = &self.port_feeding {
            if let Some(line) = lines.iter().find(|l| !lineages.contains_key(*l)) {
                return Err(format!("{} intake selector names absent lineage {line}", self.id));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_feeding(&self, rules: &Rules) -> Result<()> {
        let Some(feeding) = &self.port_feeding else { return Ok(()) };
        crate::validation::key(&feeding.function)?;
        if !rules.functions.contains_key(&feeding.function) {
            return Err(format!("{} declares unknown intake function {}", self.id, feeding.function));
        }
        if let FeedingSources::Only(lines) = &feeding.sources {
            for line in lines { crate::validation::key(line)?; }
        }
        let meal = |e: &Effect| matches!(e, Effect::Eat { from: Binding::Target, .. });
        if !self.commitments.iter().chain(&self.effects)
            .chain(self.risk.iter().flat_map(|r| &r.effects))
            .any(|e| meal(e) || e.branches().any(meal))
        {
            return Err(format!("{} declares port feeding without a target meal", self.id));
        }
        Ok(())
    }

    pub(crate) fn feeding_part(&self, actor: &Entity) -> Result<Option<PartId>> {
        let Some(feeding) = &self.port_feeding else { return Ok(None) };
        let (id, part) = feeding.part(actor)?;
        if ports::active(part).is_none() { return Err("intake port is dormant".into()); }
        Ok(Some(id))
    }

    /// Prospectively reads the same organ and provenance as an actual bite.
    /// It does not draw a new bite or change the whole-prey weight/quota.
    pub(crate) fn admits_meal(
        &self, actor: &Entity, prey: &Entity, rules: &Rules,
        lineages: &std::collections::BTreeMap<Key, Lineage>,
    ) -> bool {
        self.admits_meal_with(actor, prey, rules, lineages, || {
            std::sync::Arc::new(crate::anatomy::books(prey))
        })
    }

    /// Same eligibility check, with a lazy pass-start holdings source. Keep
    /// the source lazy so non-feeding processes and actors without an intake
    /// organ do not pay to construct the prey ledger.
    pub(crate) fn admits_meal_with(
        &self, actor: &Entity, prey: &Entity, rules: &Rules,
        lineages: &std::collections::BTreeMap<Key, Lineage>,
        holdings: impl FnOnce() -> std::sync::Arc<Ledger>,
    ) -> bool {
        let Some(feeding) = &self.port_feeding else { return true };
        let Ok((_, intake)) = feeding.part(actor) else { return false };
        let holdings = holdings();
        let admits = |e: &Effect| match e {
            Effect::Eat { from: Binding::Target, of, .. } => {
                let offered = crate::schedule::edible(holdings.as_ref(), of);
                feeding.offered(intake, &offered, prey.alive, rules, lineages)
                    .is_ok_and(|held| !held.is_empty())
            },
            _ => false,
        };
        let top = || self.commitments.iter().chain(&self.effects)
            .chain(self.risk.iter().flat_map(|r| &r.effects));
        top().any(|e| admits(e) || e.branches().any(admits))
    }
}

impl crate::rules::Template {
    pub(crate) fn validate_port(&self, rules: &Rules) -> Result<()> {
        let Some(port) = self.port.filter(|p| p.support().is_some()) else { return Ok(()) };
        let function = ports::supporting(port).ok_or("kind declares unknown native port support")?;
        if !rules.functions.contains_key(&function) {
            return Err(format!("kind port supports unknown catalogue function {function}"));
        }
        Ok(())
    }
}
