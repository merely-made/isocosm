// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Authored storylet conditions over accepted native nouns, not a table fold.

use super::Reading;
use isocosm_campaign::{StoryletError, StoryletProposal, StoryletResolution};
use std::collections::{BTreeMap, BTreeSet};

impl Reading<'_> {
    /// Preserve the declared faction tags, private fact IDs, law keys and role
    /// tags. Matching grants neither control nor a native act to its cast.
    pub fn resolve_storylet<'a, I>(&self, storylet: &StoryletProposal, hidden_fact_ids: I)
        -> Result<StoryletResolution, StoryletError>
    where I: IntoIterator<Item = &'a str> {
        let state = self.state();
        let hidden: BTreeSet<_> = hidden_fact_ids.into_iter().collect();
        for tag in &storylet.requirements.faction_tags {
            if !state.polities.values().any(|polity| polity.authored.as_ref()
                .is_some_and(|authored| authored.tags.contains(tag))) {
                return Err(StoryletError::MissingFactionTag(tag.clone()));
            }
        }
        for fact in &storylet.requirements.hidden_facts {
            if !hidden.contains(fact.as_str()) { return Err(StoryletError::MissingHiddenFact(fact.clone())); }
        }
        for law in &storylet.requirements.world_laws {
            if !state.laws.contains_key(law) { return Err(StoryletError::MissingWorldLaw(law.clone())); }
        }
        let mut characters = BTreeMap::new();
        for group in state.population.groups.values() {
            if let Some(fill) = &group.entity.authored {
                characters.entry(&fill.key).or_insert(fill);
            }
        }
        let mut used = BTreeSet::new();
        let mut cast = BTreeMap::new();
        for role in &storylet.roles {
            let candidate = characters.values().find(|character| !used.contains(&character.key)
                && role.tags.iter().all(|tag| character.tags.contains(tag)))
                .ok_or_else(|| StoryletError::UncastRole(role.key.clone()))?;
            used.insert(candidate.key.clone());
            cast.insert(role.key.clone(), candidate.key.clone());
        }
        Ok(StoryletResolution { cast, effects: storylet.effects.clone() })
    }
}
