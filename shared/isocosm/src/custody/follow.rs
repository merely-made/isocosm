// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    pub(crate) fn custody_links(&self) -> Result<BTreeMap<Id, Id>> {
        let Some(catalogue) = &self.genesis.rules.item_custody else { return Ok(BTreeMap::new()); };
        let kinds: BTreeSet<_> = catalogue.providers.values().map(|provider| provider.relation.clone()).collect();
        let mut links = BTreeMap::new();
        for relation in &self.state.relations {
            if !kinds.contains(&relation.kind) { continue; }
            if relation.subject == relation.object || links.insert(relation.object, relation.subject).is_some() {
                return Err("item custody is self-referential or has duplicate carriers".into());
            }
            if self.state.population.get(relation.subject).is_none()
                || self.state.population.get(relation.object).is_none_or(|item| item.method != Method::Inert) {
                return Err("item custody names an absent carrier or non-inert item".into());
            }
        }
        for item in links.keys() {
            let mut cursor = *item;
            let mut seen = BTreeSet::new();
            while let Some(carrier) = links.get(&cursor) {
                if !seen.insert(cursor) { return Err("item custody contains a cycle".into()); }
                cursor = *carrier;
            }
        }
        Ok(links)
    }

    pub fn item_carrier(&self, item: Id) -> Result<Option<Id>> {
        Ok(self.custody_links()?.get(&item).copied())
    }

    pub(crate) fn check_custody_changes(&self, before: &Simulation) -> Result<()> {
        let Some(catalogue) = &self.genesis.rules.item_custody else { return Ok(()); };
        let links = self.custody_links()?;
        let mut affected = BTreeSet::new();
        for provider in catalogue.providers.values() {
            for relation in self.state.relations.iter().filter(|relation| relation.kind == provider.relation) {
                if before.state.relations.iter().any(|old| old.subject == relation.subject
                    && old.object == relation.object && old.kind == relation.kind) { continue; }
                let actor = before.state.population.get(relation.subject).ok_or("new item carrier was absent")?;
                let item = before.state.population.get(relation.object).ok_or("new held item was absent")?;
                if !actor.alive || actor.body.is_none() || item.body.is_none() || !actor.colocated(item) {
                    return Err("new item custody requires actual resident physical contact".into());
                }
                let mut carrier = relation.subject;
                loop {
                    affected.insert(carrier);
                    let Some(parent) = links.get(&carrier) else { break; };
                    carrier = *parent;
                }
            }
        }
        for (key, provider) in &catalogue.providers {
            let carriers: BTreeSet<_> = self.state.relations.iter()
                .filter(|relation| relation.kind == provider.relation && affected.contains(&relation.subject))
                .map(|relation| relation.subject).collect();
            for actor in carriers {
                if self.item_burden(actor)? > u128::from(self.item_capacity(actor, key)?) {
                    return Err("new native custody exceeds its authored item capacity".into());
                }
            }
        }
        Ok(())
    }

    /// Relations carry the same saved journey, never a second route or clock.
    pub(crate) fn synchronize_custody(&mut self) -> Result<()> {
        let links = self.custody_links()?;
        let mut done = BTreeSet::new();
        for item in links.keys() { self.follow_item(*item, &links, &mut done)?; }
        Ok(())
    }

    fn follow_item(&mut self, item: Id, links: &BTreeMap<Id, Id>, done: &mut BTreeSet<Id>) -> Result<()> {
        if done.contains(&item) { return Ok(()); }
        let carrier = *links.get(&item).ok_or("item custody link is absent")?;
        if links.contains_key(&carrier) { self.follow_item(carrier, links, done)?; }
        let source = self.state.population.get(carrier).ok_or("item carrier vanished")?.clone();
        let held = self.state.population.get(item).ok_or("held item vanished")?;
        let changed = held.journey != source.journey || held.place != source.place || held.patch.is_some();
        if changed {
            let tick = self.state.tick;
            let held = self.state.population.lift(item)?;
            match &source.journey {
                Some(journey) => {
                    if held.journey.as_ref() != Some(journey) { crate::journey::leave(held, journey.clone()); }
                    held.place = source.place;
                },
                None => {
                    if held.journey.is_some() {
                        held.place = source.place;
                        held.arrived = source.arrived;
                        held.journey = None;
                    } else if held.place != source.place {
                        if held.arrived < tick {
                            held.visits.push(Visit { place: held.place, from: held.arrived, until: tick });
                        }
                        held.place = source.place;
                        held.arrived = tick;
                    }
                },
            }
            held.patch = None;
        }
        done.insert(item);
        Ok(())
    }
}
