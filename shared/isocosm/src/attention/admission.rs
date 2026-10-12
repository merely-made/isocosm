// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native admission for one active window's contribution (815).

use super::*;
use crate::{Result, Session, Command, simulation::Simulation, directing};

impl Simulation {
    pub(crate) fn attention_participant(&self, participant: Id) -> Result<()> {
        match self.state.population.get(participant) {
            Some(e) if directing::is_participant(e) => Ok(()),
            _ => Err("attention requires a native participant".into()),
        }
    }

    fn attention_target(&self, target: &Target) -> Result<()> {
        let exists = match target {
            Target::Entity(id) => self.state.population.get(*id).is_some(),
            Target::Site(id) => self.state.sites.contains_key(id),
            Target::Location(id) => self.state.locations.contains_key(id),
            Target::Lineage(key) => self.state.lineages.contains_key(key),
            Target::Faction(id) => self.state.polities.contains_key(id),
            Target::Event(key) => self.state.events.contains_key(key),
        };
        if exists { Ok(()) } else { Err("attention names no native target".into()) }
    }

    fn validate_window(&self, participant: Id, profile: &str, value: &WindowAttention) -> Result<()> {
        self.attention_participant(participant)?;
        self.profile_care(participant, profile)?;
        for target in &value.pinned { self.attention_target(target)?; }
        if let Some(region) = &value.examined {
            self.attention_target(&match region {
                Region::Site(id) => Target::Site(*id),
                Region::Location(id) => Target::Location(*id),
            })?;
        }
        Ok(())
    }

    pub(crate) fn admit_window(&mut self, participant: Id, window: WindowId,
        profile: Key, value: WindowAttention) -> Result<String> {
        self.validate_window(participant, &profile, &value)?;
        self.state.attention.attach_at(participant, window, profile, value)?;
        Ok(format!("attention-window:{}", window.id()))
    }

    pub(crate) fn replace_window(&mut self, participant: Id, window: WindowId,
        profile: Key, value: WindowAttention) -> Result<String> {
        self.state.attention.owned(participant, window)?;
        self.validate_window(participant, &profile, &value)?;
        let changed = self.state.attention.replace(participant, window, profile, value)?;
        Ok(if changed { "attention-replaced" } else { "attention-unchanged" }.into())
    }

    pub(crate) fn close_window(&mut self, participant: Id, window: WindowId) -> Result<String> {
        self.attention_participant(participant)?;
        self.state.attention.close(participant, window)?;
        Ok("attention-closed".into())
    }

    pub fn attention_window(&self, participant: Id, handle: u64)
        -> Result<(WindowId, Key, WindowAttention)> {
        self.attention_participant(participant)?;
        let window = self.state.attention.resolve(participant, handle)?;
        let value = self.state.attention.owned(participant, window)?;
        Ok((window, value.profile.clone(), value.attention.clone()))
    }

    pub fn player_attention(&self, participant: Id) -> Result<PlayerAttention> {
        self.attention_participant(participant)?;
        // Every admitted profile was checked. Derivation never accepts host care.
        let mut attention = self.state.attention.for_player(participant, self.plays(participant), |_| {
            BTreeSet::new()
        });
        for value in self.state.attention.active.values().filter(|v| v.participant == participant) {
            attention.care.extend(self.profile_care(participant, &value.profile)?);
        }
        Ok(attention)
    }
}

impl Session {
    /// Native issues a stable branch/intent-derived token. A collision refuses;
    /// neither replay nor merge silently renumbers subsequent window commands.
    pub fn attach_window(&mut self, participant: Id, profile: Key,
        attention: WindowAttention) -> Result<WindowId> {
        let identity = crate::digest(&(&self.branch, self.sim.state().tick,
            self.entries.len() as u64, participant, &profile, &attention));
        let window = WindowId(u64::from_str_radix(&identity[..16], 16)
            .map_err(|_| "native attention identity does not encode")?);
        self.attention_command(Command::Window(Change::Attach { participant, window, profile, attention }))?;
        Ok(window)
    }

    pub fn replace_attention(&mut self, participant: Id, window: WindowId,
        profile: Key, attention: WindowAttention) -> Result<String> {
        self.attention_command(Command::Window(Change::Replace { participant, window, profile, attention }))
    }

    pub fn close_attention(&mut self, participant: Id, window: WindowId) -> Result<String> {
        self.attention_command(Command::Window(Change::Close { participant, window }))
    }

    fn attention_command(&mut self, command: Command) -> Result<String> {
        let mut proposed = self.clone();
        let answer = proposed.command(command)?;
        *self = proposed;
        Ok(answer)
    }
}

impl Simulation {
    pub(crate) fn change_window(&mut self, change: &Change) -> Result<String> {
        match change {
            Change::Attach { participant, window, profile, attention } =>
                self.admit_window(*participant, *window, profile.clone(), attention.clone()),
            Change::Replace { participant, window, profile, attention } =>
                self.replace_window(*participant, *window, profile.clone(), attention.clone()),
            Change::Close { participant, window } => self.close_window(*participant, *window),
        }
    }
}
