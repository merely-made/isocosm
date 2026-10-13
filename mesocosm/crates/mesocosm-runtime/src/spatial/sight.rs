// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored ranges refine physical pictures from current admitted cells only.

use super::*;
use crate::perception::{InSite, Sight};
use isocosm::directing::readings::View;

/// The range belongs to an explicitly named native observer. Its eye and
/// window remain the actual saved Placement's; no rendering spot supplies them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SightConfig { pub version: u32, pub ranges: BTreeMap<Id, u64> }

/// A new marked setup beside the original placement-array/driver grammar.
/// Empty placements with no profile update only sight on existing bindings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SightedSetup {
    pub version: u32, pub placements: Vec<Placement>,
    pub profile: Option<Profile>, pub sight: SightConfig,
}

impl SightConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.ranges.is_empty()
            || self.ranges.values().any(|range| *range > i64::MAX as u64) {
            return Err("sight source needs its supported version and explicit bounded observer ranges".into());
        }
        Ok(())
    }
}

impl Game {
    /// Trusted authored setup only. This changes no native knowledge or clock.
    pub fn set_sight(&mut self, session: &Session, config: &SightConfig) -> Result<(), String> {
        self.version()?;
        config.validate()?;
        if config.ranges.len() as u64 > session.sim.genesis().rules.limits.entities {
            return Err("sight source exceeds the native entity limit".into());
        }
        for id in config.ranges.keys() {
            let at = self.members.get(id).ok_or("sight observer has no explicitly supplied physical binding")?;
            at.realized(&session.sim)?;
        }
        self.sight = Some(config.clone());
        self.version = self.current_version();
        Ok(())
    }

    pub(super) fn setup_sighted(&mut self, session: &Session, authored: &SightedSetup) -> Result<(), String> {
        if authored.version != 1 { return Err("unsupported marked sight setup".into()); }
        let mut candidate = self.clone();
        if let Some(profile) = &authored.profile {
            candidate.configure(session, &Authored { placements: authored.placements.clone(), profile: profile.clone() })?;
        } else if !authored.placements.is_empty() {
            candidate.replace(&session.sim, &authored.placements)?;
        }
        candidate.set_sight(session, &authored.sight)?;
        *self = candidate;
        Ok(())
    }

    /// None is only the old unmarked presentation path. A marked picture
    /// requires this actual observer's complete range and current eye binding.
    /// Targets use their current saved root cells, including resident corpses
    /// in the supplied native cut. Unbound/departed bodies provide no ray cell.
    pub fn sight(&self, sim: &Simulation, observer: Id, visible: &View)
        -> Result<Option<(InSite, Sight)>, String> {
        self.version()?;
        let Some(config) = &self.sight else {
            if self.has_sight_source() {
                return Err("marked newborn sight source has no admitted observer range".into());
            }
            return Ok(None);
        };
        let range = *config.ranges.get(&observer).ok_or("marked sight has no authored range for the current observer")?;
        let at = self.members.get(&observer).ok_or("marked sight observer has no current physical eye binding")?;
        at.realized(sim)?;
        let volume = sim.volume(at.site.0, at.window.min, at.window.max, at.window.depth)?;
        let site = InSite::of(sim, volume, at.places.unwrap_or_default())?;
        let mut targets = BTreeMap::new();
        for (&id, held) in &self.members {
            if id == observer || !visible.shows(id) || held.site != at.site { continue; }
            let Some(member) = sim.state().population.get(id) else { continue; };
            if member.body.is_none() || member.resident_site() != Some(at.site.0) { continue; }
            held.realized(sim)?;
            let cell = held.pose.at;
            if !site.volume().holds(cell[0], cell[2]) || cell[1] < site.volume().datum
                || cell[1] >= site.volume().ceiling() { continue; }
            targets.insert(id, cell);
        }
        let sight = Sight { observer, eye: at.eye, range, targets };
        site.seen(sim, &sight)?;
        Ok(Some((site, sight)))
    }
}
