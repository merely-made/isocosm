// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Homes offered under agreements (rulings 54 and 63): a dwelling is a site,
//! asserted as an authored place (769), and a tenancy a relation from the
//! tenant to it whose value names the agreement it stands under, so where a
//! peer lives is read from that agreement's standing: moving out is the
//! agreement ending.

use super::*;
use crate::{Result, schema::related, simulation::Simulation};

/// A tenancy, its value the agreement's id.
pub const TENANT: &str = "social:tenant";

/// Where someone lives and what they do daily, read from agreements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyRound {
    pub home: Option<Id>,
    pub does: Option<Work>,
}

impl Simulation {
    /// Offers `dwelling` as a home under `offer`; the tenancy holds while
    /// the agreement it forms stands.
    pub(crate) fn offer_home(&mut self, offer: &Offer, dwelling: Id) -> Result<Ruling> {
        if !self.state.sites.contains_key(&dwelling) {
            return Err("no such dwelling".into());
        }
        if let Some(sitting) = self.tenant_of(dwelling) {
            return Err(format!("the dwelling is {sitting}'s"));
        }
        let ruling = self.form(offer)?;
        if let RulingKind::Formed(id) = ruling.kind {
            let tenant = offer.asked_of;
            let rs = &mut self.state.relations;
            if let Some(held) = related(rs, tenant, TENANT, dwelling).cloned() {
                rs.remove(&held);
            }
            rs.insert(Relation { value: id as i64, ..Relation::new(tenant, TENANT, dwelling) });
        }
        Ok(ruling)
    }

    /// The tenancies whose agreements stand, as (tenant, dwelling,
    /// agreement), latest agreement first.
    fn tenancies(&self) -> Vec<(Id, Id, Id)> {
        let stands = |a: Id| self.state.agreements.get(&a).is_some_and(Agreement::standing);
        let mut held: Vec<(Id, Id, Id)> = self
            .state
            .relations
            .iter()
            .filter(|r| r.kind == TENANT && stands(r.value as Id))
            .map(|r| (r.subject, r.object, r.value as Id))
            .collect();
        held.sort_by(|a, b| b.2.cmp(&a.2));
        held
    }

    pub fn home_of(&self, who: Id) -> Option<Id> {
        self.tenancies().into_iter().find(|t| t.0 == who).map(|t| t.1)
    }

    pub fn tenant_of(&self, dwelling: Id) -> Option<Id> {
        self.tenancies().into_iter().find(|t| t.1 == dwelling).map(|t| t.0)
    }

    pub fn daily(&self, who: Id) -> DailyRound {
        let Some((_, home, a)) = self.tenancies().into_iter().find(|t| t.0 == who) else {
            return DailyRound::default();
        };
        let does = self.state.agreements.get(&a).map(|a| a.work.clone());
        DailyRound { home: Some(home), does }
    }
}
