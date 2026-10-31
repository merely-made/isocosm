// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    fn custody_ready(&self, change: &Change) -> Result<&str> {
        self.project_control(change.participant, change.actor)?;
        let actor = self.state.population.get(change.actor).ok_or("item carrier is absent")?;
        let item = self.state.population.get(change.item).ok_or("item is absent")?;
        if !actor.alive || actor.body.is_none() || item.method != Method::Inert || item.body.is_none()
            || change.actor == change.item || !actor.colocated(item) {
            return Err("item custody needs distinct resident physical bodies and a living controlled carrier".into());
        }
        let carrier = self.item_carrier(change.item)?;
        match change.action {
            Action::Carry if carrier.is_some() => return Err("item already has native custody".into()),
            Action::Drop | Action::Equip | Action::Unequip if carrier != Some(change.actor) =>
                return Err("only the actual native carrier changes this held item".into()),
            _ => {},
        }
        let provider = self.custody_provider(&change.provider)?;
        if carrier.is_some() && !self.state.relations.iter().any(|relation|
            relation.subject == change.actor && relation.object == change.item && relation.kind == provider.relation) {
            return Err("item is held through another native custody provider".into());
        }
        Ok(provider.process(change.action))
    }

    pub fn prepare_owned_custody(&self, change: &Change) -> Result<crate::projects::DebitPreview> {
        let process = self.custody_ready(change)?;
        self.prepare_owned_act(change.actor, Some(change.item), process, change.cause.clone())
    }

    pub(crate) fn custody_change(&mut self, change: &Change) -> Result<String> {
        if self.state.projects.debit_capture.active() { return Err("native item custody cannot nest an owned action".into()); }
        let process = self.custody_ready(change)?.to_owned();
        let relation = self.custody_provider(&change.provider)?.relation.clone();
        let mut candidate = self.clone();
        candidate.clear_owned_capture();
        let result = candidate.owned_act(change.participant, change.actor, Some(change.item),
            &process, change.cause.clone(), &change.selections);
        candidate.clear_owned_capture();
        let owned: crate::projects::OwnedActed = serde_json::from_str(&result?).map_err(|error| error.to_string())?;
        if !owned.receipt.accepted() { return Err(format!("native item custody act was not admitted: {:?}", owned.receipt.outcome)); }
        let carried = change.action != Action::Drop;
        let actual = candidate.item_carrier(change.item)?;
        let matching = candidate.state.relations.iter().any(|link|
            link.object == change.item && link.kind == relation && link.subject == change.actor);
        if actual != carried.then_some(change.actor) || matching != carried {
            return Err("authored item process did not establish its exact exclusive custody result".into());
        }
        let actor = candidate.state.population.get(change.actor).ok_or("item carrier vanished")?;
        let item = candidate.state.population.get(change.item).ok_or("item vanished")?;
        if !actor.alive || item.method != Method::Inert || actor.body.is_none() || item.body.is_none()
            || actor.resident_site().is_none() || !actor.colocated(item) {
            return Err("item custody process changed the physical action's actual resident bodies".into());
        }
        candidate.synchronize_custody()?;
        let matter = candidate.item_matter(change.item)?;
        let burden = candidate.item_burden(change.actor)?;
        let capacity = candidate.item_capacity(change.actor, &change.provider)?;
        if change.action == Action::Carry && burden > u128::from(capacity) {
            return Err("actual native item and project burden exceeds authored capacity".into());
        }
        let out = CustodyActed { owned, provider: change.provider.clone(), actor: change.actor,
            item: change.item, carried, matter, burden, capacity };
        let encoded = serde_json::to_string(&out).map_err(|error| error.to_string())?;
        self.publish_candidate(candidate)?;
        Ok(encoded)
    }
}
