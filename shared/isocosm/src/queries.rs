// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Result,
    meaning::{Named, Scene},
    rules::*,
    schema::*,
    simulation::*,
};

impl Simulation {
    pub(crate) fn bound(&self, actor: Id, target: Option<Id>, binding: Binding) -> Result<Id> {
        match binding {
            Binding::Actor => Ok(actor),
            Binding::Target => target.ok_or("target is required".into()),
            Binding::Place => Err("place is not a body".into()),
        }
    }
    pub(crate) fn query(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        query: &Query,
    ) -> Result<String> {
        let related = |kind: &Key| -> Result<bool> {
            Ok(self.state.relations.contains(&Relation {
                subject: actor,
                kind: kind.clone(),
                object: target.ok_or("target required")?,
            }))
        };
        let scene = Scene {
            actor: self.state.population.get(actor),
            target: match target {
                None => Named::Unnamed,
                Some(id) => self
                    .state
                    .population
                    .get(id)
                    .map_or(Named::Missing, Named::Found),
            },
            site: self.state.sites.get(&place),
            tick: self.state.tick,
            related: &related,
            needs: crate::meaning::needs(&self.genesis.rules),
        };
        let (yes, reading) = crate::meaning::read(query, &scene)?;
        if yes {
            Ok(format!("{query:?} = {reading}"))
        } else {
            Err(format!("missing requirement: {query:?} = {reading}"))
        }
    }
    pub(crate) fn target_matches(&self, actor: Id, target: Option<Id>, p: &Process) -> bool {
        let Some(selector) = &p.target else {
            return true;
        };
        let Some(target) = target.filter(|id| *id != actor) else {
            return false;
        };
        let Some(a) = self.state.population.get(actor) else {
            return false;
        };
        let Some(b) = self.state.population.get(target) else {
            return false;
        };
        (!selector.same_place || a.place == b.place)
            && selector.alive.is_none_or(|alive| alive == b.alive)
            && selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| lineage == &b.lineage)
    }
}
