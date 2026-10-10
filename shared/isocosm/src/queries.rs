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
    pub(crate) fn bind_part(&self, actor: Id, p: &Process) -> Option<Id> {
        let function = p.expresses()?;
        expressing(self.body_at_start(actor)?, function)
    }
    /// Reads `query` for an act by `actor`, with `part` the part its process
    /// binds, against the world as the pass under way began (ruling 454).
    pub(crate) fn query(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        part: Option<Id>,
        query: &Query,
    ) -> Result<String> {
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
            site: self.site_at_start(place),
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
        (!selector.same_place || a.place == b.place)
            && selector.alive.is_none_or(|alive| alive == b.alive)
            && selector
                .lineage
                .as_ref()
                .is_none_or(|lineage| lineage == &b.lineage)
            && (selector.among.is_empty() || selector.among.contains(&b.lineage))
    }
}
