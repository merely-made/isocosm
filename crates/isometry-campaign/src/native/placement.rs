// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit table geometry and native control, separate from authored fill.

use super::RecipeBinding;
use isocosm::{Command, Session, schema::{Id, Key}};
use isometer_space::places::{Cap, PlaceId, Places, Rules};
use serde::{Deserialize, Serialize};

/// Exact authored base-cell window, not a camera or identity-cell fallback.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementWindow { pub min: [i64; 2], pub max: [i64; 2], pub depth: i64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterPlacement {
    pub character: Key,
    pub site: Id,
    pub patch: PlaceId,
    pub body_revision: u64,
    pub terrain_revision: String,
    pub at: [i64; 3],
    pub window: PlacementWindow,
    pub places: Rules,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionBinding {
    pub entity: Id,
    pub placement: CharacterPlacement,
    pub patch_entry: Key,
    /// Missing in older bindings: retain the record, but require explicit setup
    /// before presenting a physical pose in a later or unknown visit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resident_since: Option<isocosm::schema::Tick>,
}

/// Join is explicit and always creates a new native participant. Existing
/// participants keep their native identity; table owner metadata is unused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantInput { Existing(Id), Join }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterControl { pub character: Key, pub participant: ParticipantInput }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlBinding {
    pub entity: Id,
    pub participant: Id,
    pub join_entry: Option<Key>,
    pub take_entry: Key,
}

impl CharacterPlacement {
    pub(crate) fn check(&self, session: &Session, entity: Id, placed: bool) -> Result<(), String> {
        if self.window.depth < 0 || matches!(self.places.cap, Cap::Own(sides) if sides.contains(&0)) {
            return Err("character geometry contains an invalid authored depth or patch cap".into());
        }
        let native = session.sim.state().population.get(entity).ok_or("placement character is absent")?;
        if native.authored.as_ref().map(|fill| &fill.key) != Some(&self.character)
            || (!placed && !native.alive) || native.body.is_none() || native.body_revision != self.body_revision
            || native.resident_site() != Some(self.site) || self.patch.site != self.site
            || session.sim.terrain_revision() != self.terrain_revision
            || placed && native.patch != Some(self.patch)
        { return Err("character geometry names a stale native body, site, patch or terrain".into()); }
        let volume = session.sim.volume(self.site, self.window.min, self.window.max, self.window.depth)?;
        if !volume.holds(self.at[0], self.at[2]) || self.at[1] < volume.datum
            || self.at[1] >= volume.ceiling()
        { return Err("character position is outside the authored native window".into()); }
        let places = Places::derive(&volume, self.places)?;
        if !places.places.contains_key(&self.patch) || places.place_at(self.at) != Some(self.patch)
            || !volume.material(self.at).is_some_and(|material|
                material == volume.materials.air || material == volume.materials.water)
        { return Err("character position is outside the realized clear patch".into()); }
        Ok(())
    }
}

impl PositionBinding {
    pub(crate) fn validate(&self, session: &Session, character: &str, recipe: &RecipeBinding)
        -> Result<(), String> {
        if self.entity != recipe.entity || self.placement.character != character {
            return Err("position binding differs from its admitted character".into());
        }
        let entry = session.entries.iter().find(|entry| entry.id == self.patch_entry)
            .ok_or("position binding's native patch entry is absent")?;
        let developed = session.entries.iter().find(|entry| entry.id == recipe.develop_entry)
            .ok_or("position binding's native development entry is absent")?;
        if !matches!(&entry.command, Command::Patch { entity, patch }
            if *entity == self.entity && *patch == Some(self.placement.patch))
            || entry.outcome != "placed" || entry.order <= developed.order
        { return Err("position binding differs from accepted native placement history".into()); }
        Ok(())
    }

    /// Current physical geometry includes a resident corpse (847). Admission
    /// and moves still require a living actor; this read grants no control.
    pub fn current(&self, session: &Session) -> Result<[i64; 3], String> {
        self.placement.check(session, self.entity, true)?;
        let native = session.sim.state().population.get(self.entity).ok_or("position character is absent")?;
        if self.resident_since != Some(native.arrived) {
            return Err("character position belongs to an unstamped or earlier native visit".into());
        }
        Ok(self.placement.at)
    }
}

impl ControlBinding {
    pub(crate) fn validate(&self, session: &Session, recipe: &RecipeBinding) -> Result<(), String> {
        if self.entity != recipe.entity || !session.sim.state().population.get(self.participant)
            .is_some_and(isocosm::directing::is_participant)
        { return Err("control binding names another character or no native participant".into()); }
        let taken = session.entries.iter().find(|entry| entry.id == self.take_entry)
            .ok_or("control binding's native take entry is absent")?;
        let developed = session.entries.iter().find(|entry| entry.id == recipe.develop_entry)
            .ok_or("control binding's native development entry is absent")?;
        if !matches!(&taken.command, Command::Take { participant, critter }
            if *participant == self.participant && *critter == self.entity)
            || taken.outcome != "taken" || taken.order <= developed.order
        { return Err("control binding differs from accepted native take history".into()); }
        if let Some(join) = &self.join_entry {
            let joined = session.entries.iter().find(|entry| &entry.id == join)
                .ok_or("control binding's native join entry is absent")?;
            if !matches!(&joined.command, Command::Join)
                || joined.outcome != format!("participant:{}", self.participant)
                || joined.order >= taken.order || joined.tick != taken.tick
            { return Err("control binding differs from accepted native join history".into()); }
        }
        Ok(())
    }

    /// Native plays relations, not these historical bindings, own control.
    pub fn current(&self, session: &Session) -> bool {
        session.sim.plays(self.participant) == Some(self.entity)
    }
}

pub(crate) fn place_character(session: &mut Session, input: &CharacterPlacement,
    recipe: &RecipeBinding) -> Result<PositionBinding, String> {
    input.check(session, recipe.entity, false)?;
    session.command(Command::Patch { entity: recipe.entity, patch: Some(input.patch) })?;
    let binding = PositionBinding { entity: recipe.entity, placement: input.clone(),
        patch_entry: session.entries.last().ok_or("native patch entry is absent")?.id.clone(),
        resident_since: Some(session.sim.state().population.get(recipe.entity)
            .ok_or("position character is absent")?.arrived) };
    binding.validate(session, &input.character, recipe)?;
    binding.current(session)?;
    Ok(binding)
}

pub(crate) fn take_character(session: &mut Session, input: &CharacterControl,
    recipe: &RecipeBinding, dm: Id) -> Result<ControlBinding, String> {
    let (participant, join_entry) = match input.participant {
        ParticipantInput::Existing(id) => (id, None),
        ParticipantInput::Join => {
            let joined = session.command(Command::Join)?;
            let id = joined.strip_prefix("participant:").ok_or("native join returned no participant")?
                .parse::<Id>().map_err(|_| "native join returned an invalid participant")?;
            let entry = session.entries.last().ok_or("native join entry is absent")?.id.clone();
            (id, Some(entry))
        },
    };
    if participant == dm && session.sim.state().relations.iter().any(|relation|
        relation.kind == isocosm::directing::PLAYS && relation.object == recipe.entity
            && relation.subject != dm)
    { return Err("the DM may take up only an unclaimed character".into()); }
    session.command(Command::Take { participant, critter: recipe.entity })?;
    let binding = ControlBinding { entity: recipe.entity, participant, join_entry,
        take_entry: session.entries.last().ok_or("native take entry is absent")?.id.clone() };
    binding.validate(session, recipe)?;
    Ok(binding)
}
