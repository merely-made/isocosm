// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A household: a dwelling, the tenancy standing on it, and who lives under
//! it (doctrine World 10, rulings 54 and 63). Read, never kept.

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Household {
    pub dwelling: Id,
    /// The dwelling's authored name, where it has one.
    pub name: Option<String>,
    pub tenant: Id,
    pub agreement: Id,
    /// Whom the tenancy stands with: the agreement's asker. Not an owner:
    /// ownership is asserted, never read off a tenancy (Processes 2).
    pub with: Id,
    /// The work the tenancy stands on.
    pub does: Work,
    /// The tenant and its living kin with no home of their own who share
    /// its site.
    pub members: Vec<Id>,
}

impl Simulation {
    fn household_of(&self, tenant: Id, dwelling: Id, agreement: Id) -> Option<Household> {
        let a = self.state.agreements.get(&agreement)?;
        let site = self.state.population.get(tenant).filter(|e| e.alive)?.resident_site();
        let homed: BTreeSet<Id> = self.tenancies().into_iter().map(|t| t.0).collect();
        let kin = self.living_kin(tenant).into_iter().filter(|k| !homed.contains(k)
            && self.state.population.get(*k).and_then(Entity::resident_site) == site);
        let members = std::iter::once(tenant).chain(kin).collect::<BTreeSet<_>>().into_iter().collect();
        let name = self.state.sites.get(&dwelling).and_then(|s| s.authored.as_ref()).map(|p| p.name.clone());
        Some(Household { dwelling, name, tenant, agreement, with: a.asker, does: a.work.clone(), members })
    }

    /// The household standing on `dwelling`, if a living tenant holds it.
    pub fn household(&self, dwelling: Id) -> Result<Option<Household>> {
        crate::society()?;
        Ok(self.tenancies().into_iter().find(|t| t.1 == dwelling).and_then(|(t, d, a)| self.household_of(t, d, a)))
    }

    /// The households standing on any of `sites`, by dwelling.
    pub fn households_at(&self, sites: &BTreeSet<Id>) -> Result<Vec<Household>> {
        crate::society()?;
        let mut seen = BTreeSet::new();
        let mut held: Vec<Household> = self.tenancies().into_iter()
            .filter(|t| sites.contains(&t.1) && seen.insert(t.1))
            .filter_map(|(t, d, a)| self.household_of(t, d, a)).collect();
        held.sort_by_key(|h| h.dwelling);
        Ok(held)
    }
}
