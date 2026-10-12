// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Registered product care from current native facts, never binding input.

use super::*;
use crate::{Result, simulation::Simulation, directing::readings::{self, Mode}};

impl Simulation {
    pub fn profile_care(&self, participant: Id, profile: &str) -> Result<BTreeSet<Target>> {
        self.attention_participant(participant)?;
        let mut care = BTreeSet::new();
        match profile {
            "mesocosm" => {
                if let Some(e) = self.plays(participant).and_then(|id| self.state.population.get(id)) {
                    care.insert(Target::Lineage(e.lineage.clone()));
                }
            },
            "eponym" => {
                if let Some(subject) = self.plays(participant) {
                    care.insert(Target::Entity(subject));
                    for (first, count) in readings::view(self, subject, Mode::Survival).things {
                        care.extend((first..first + count).map(Target::Entity));
                    }
                }
            },
            "vtt" => {
                for (first, group) in &self.state.population.groups {
                    if group.entity.authored.is_some() {
                        care.extend((*first..first + group.count).map(Target::Entity));
                    }
                }
                care.extend(self.state.sites.iter().filter(|(_, s)| s.authored.is_some())
                    .map(|(id, _)| Target::Site(*id)));
                care.extend(self.state.polities.iter().filter(|(_, p)| p.authored.is_some())
                    .map(|(id, _)| Target::Faction(*id)));
                care.extend(self.state.events.iter().filter(|(_, e)| e.authored.is_some())
                    .map(|(key, _)| Target::Event(key.clone())));
                // Authored records retain their native nouns. Maps care for their
                // authored place; assertions inside packs/storylets already fill
                // those same nouns. No fabricated handle for a law or pack.
                for map in self.state.asserted.maps.values() {
                    let drawn = map.place.strip_prefix("site:").and_then(|s| s.parse().ok())
                        .filter(|id| self.state.sites.contains_key(id));
                    if let Some(site) = drawn.or_else(|| self.authored_site(&map.place).map(|(id, _)| id)) {
                        care.insert(Target::Site(site));
                    }
                }
                for note in self.state.notes.iter().filter(|n| n.core.cause.starts_with("assert:")) {
                    if let Some(id) = note.core.subject.strip_prefix("entity:").and_then(|s| s.parse().ok()) {
                        if self.state.population.get(id).is_some() { care.insert(Target::Entity(id)); }
                    }
                    if let Some(id) = self.authored_id(&note.core.subject) {
                        if self.state.population.get(id).is_some() { care.insert(Target::Entity(id)); }
                        else if self.state.sites.contains_key(&id) { care.insert(Target::Site(id)); }
                        else if self.state.polities.contains_key(&id) { care.insert(Target::Faction(id)); }
                    }
                    if let Some(id) = note.core.subject.strip_prefix("site:").and_then(|s| s.parse().ok()) {
                        if self.state.sites.contains_key(&id) { care.insert(Target::Site(id)); }
                    }
                }
            },
            _ => return Err("unregistered native attention profile".into()),
        }
        Ok(care)
    }

    /// Individual roots only. A lineage/faction/site pin keeps a group without
    /// turning its fungible members into individual rows (211). Native noted
    /// roots remain protected by kept(); an examined region is full detail.
    pub(crate) fn attention_roots(&self) -> BTreeSet<Id> {
        let mut roots = BTreeSet::new();
        for participant in self.state.attention.active.values().map(|c| c.participant)
            .collect::<BTreeSet<_>>() {
            let Ok(attention) = self.player_attention(participant) else { continue; };
            roots.extend(attention.played);
            for target in attention.pinned.into_iter().chain(attention.care) {
                if let Target::Entity(id) = target { roots.insert(id); }
            }
            for region in attention.examined {
                let sites = match region {
                    Region::Site(id) => BTreeSet::from([id]),
                    Region::Location(id) => self.state.locations.get(&id)
                        .map(|l| l.sites.clone()).unwrap_or_default(),
                };
                for (first, group) in &self.state.population.groups {
                    if group.entity.alive && group.entity.resident_site().is_some_and(|site| sites.contains(&site)) {
                        roots.extend(*first..first + group.count);
                    }
                }
            }
        }
        roots
    }

    pub(crate) fn refresh_attention(&mut self) {
        if self.state.attention.is_empty() { return; }
        let roots = self.attention_roots();
        let windows = &mut self.state.attention;
        for id in windows.root_cache.difference(&roots) {
            windows.released.insert(*id, self.state.tick.saturating_add(self.genesis.rules.collection_buffer));
        }
        windows.released.retain(|id, until| *until > self.state.tick && !roots.contains(id));
        windows.root_cache = roots;
    }

    pub(crate) fn protect_attention(&mut self) {
        if self.state.attention.is_empty() { return; }
        let roots = self.kept();
        let population = &mut self.state.population;
        match &mut self.journal {
            Some(j) => population.restrict_logged(&roots, &mut |first, was| j.group(first, was)),
            None => population.restrict(&roots),
        }
    }
}
