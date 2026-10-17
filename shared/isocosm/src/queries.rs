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
            Binding::Part => Err("a part is not a body".into()),
        }
    }
    /// The part `p` binds in `actor`: its lowest-numbered live part
    /// expressing the function `p` requires (ruling 338).
    pub(crate) fn bind_part(&self, actor: Id, p: &Process) -> Option<PartId> {
        let function = p.expresses()?;
        expressing(self.body_at_start(actor)?, function)
    }
    pub(crate) fn bind_intake(&self, actor: Id, p: &Process) -> Result<Option<PartId>> {
        p.feeding_part(self.body_at_start(actor).ok_or("actor is absent")?)
    }
    /// Reads `query` for an act by `actor`, with `part` the part its process
    /// binds, against the world as the pass under way began (ruling 454).
    pub(crate) fn query(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        part: Option<PartId>,
        query: &Query,
    ) -> Result<String> {
        if matches!(query, Query::Project { .. } | Query::ProjectDecision | Query::InstitutionDecision | Query::Institution { .. }) {
            crate::society()?;
        }
        if let Query::Project { definition, phase, process } = query {
            return self.project_ready(actor, target, definition, *phase, process);
        }
        if matches!(query, Query::ProjectDecision) {
            return self.project_decision_ready(actor, target);
        }
        if matches!(query, Query::InstitutionDecision) {
            return self.institution_decision_ready(actor,target);
        }
        if let Query::Institution {producer}=query {
            return self.institution_producer_ready(actor,target,producer);
        }
        if let Query::Holds { who: Binding::Target, at_least } = query {
            let id = target.ok_or_else(|| "target is required".to_owned())?;
            let holdings = self.books_at_start(id).ok_or_else(|| "body missing".to_owned())?;
            let reading = crate::meaning::mass(&holdings, &self.genesis.rules);
            return if reading >= u128::from(*at_least) {
                Ok(format!("{query:?} = {reading}"))
            } else {
                Err(format!("missing requirement: {query:?} = {reading}"))
            };
        }
        let related = |kind: &Key| -> Result<bool> {
            let object = target.ok_or("target required")?;
            Ok(related(&self.state.relations, actor, kind, object).is_some())
        };
        let scene = Scene {
            actor: self.body_at_start(actor),
            target: match target {
                None => Named::Unnamed,
                Some(id) => self.body_at_start(id).map_or(Named::Missing, Named::Found),
            },
            part,
            site: self.body_at_start(actor).and_then(Entity::resident_site)
                .filter(|site| *site == place).and_then(|site| self.site_at_start(site)),
            tick: self.state.tick,
            related: &related,
            rules: &self.genesis.rules,
            lineages: Some(&self.state.lineages),
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
        let Some(a) = self.body_at_start(actor) else {
            return false;
        };
        let Some(b) = self.body_at_start(target) else {
            return false;
        };
        // A participant is no body to act on (ruling 688).
        if crate::directing::is_participant(b) {
            return false;
        }
        (!selector.same_place || if self.state.movement.immediate() { a.place == b.place } else { a.colocated(b) })
            && selector.alive.is_none_or(|alive| alive == b.alive)
            && selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| lineage == &b.lineage)
            && (selector.among.is_empty() || selector.among.contains(&b.lineage))
            && p.admits_meal_with(a, b, &self.genesis.rules, &self.state.lineages,
                || self.books_at_start(target).expect("target body exists"))
    }
}
