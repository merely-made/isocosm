// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lossless product records transferred from the consumed standalone owner.

use super::*;
use crate::{ItemId,ItemPlaces,NativeAnatomySource};
use crate::timed_action::{TimedAction,TimedActionRules};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductRecords {
    pub items:ItemPlaces, pub projects:Vec<ImportedProjectPose>,
    pub movement:Vec<MovementRoles>, pub anatomies:Vec<AnatomyLabel>,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedProjectPose {
    pub project:ProjectId, pub site:Id, pub created_site:Id,
    pub created_at:[i32;3], pub at:[i32;3],
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementRoles { pub subject:SubjectId, pub profile:MovementProfile }

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnatomyLabel {
    pub subject:SubjectId, pub revision:BodyRevisionId, pub species:SpeciesId,
    pub native_source:Option<NativeAnatomySource>,
}

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedState {
    pub products:ProductRecords,
    /// Exact former Ground origin in native site coordinates. Historical item
    /// and project cells retain their original meaning after a terrain change.
    pub site:Id, pub origin:[i64;3],
}

impl ImportedState {
    pub fn item_at(&self,session:&Session,mode:&ModeState,item:ItemId) -> Result<(Id,[i64;3]),String> {
        let entity = session.sim.state().population.get(item.0).ok_or("imported Eponym item is absent")?;
        if !entity.alive { return Err("imported Eponym item is consumed".into()); }
        let carriers:Vec<_> = session.sim.state().relations.iter()
            .filter(|r|r.kind == crate::founding::CARRIES && r.object == item.0).collect();
        if carriers.len() > 1 { return Err("imported Eponym item has ambiguous native custody".into()); }
        if let Some(carrier) = carriers.first() {
            let binding = mode.bindings.iter().find(|b|b.entity == carrier.subject)
                .ok_or("imported Eponym carrier has no explicit subject binding")?;
            let volume = session.sim.volume(binding.site,binding.window.min,binding.window.max,binding.window.depth)?;
            return Ok((binding.site,crate::walking::to_site(&volume,binding.current(session)?.cell().map_err(debug)?)));
        }
        if entity.resident_site() != Some(self.site) { return Err("imported Eponym item has departed its saved place".into()); }
        let at = self.products.items.at.get(&item).ok_or("imported Eponym item has no saved physical location")?;
        Ok((self.site,self.site_cell(*at)?))
    }

    pub(super) fn site_cell(&self,at:[i32;3]) -> Result<[i64;3],String> {
        let mut out = self.origin;
        for axis in 0..3 { out[axis] = out[axis].checked_add(i64::from(at[axis]))
            .ok_or("imported Eponym site coordinate overflow")?; }
        Ok(out)
    }
}

/// Preflight borrows the original native owner. All bytes and product state
/// are complete before the caller consumes/moves that owner.
pub(crate) fn prepare(session:&Session, bytes:&[u8], config:&ActionConfig,
    played:SubjectId, network:&FunctionalNetwork, rules:&TimedActionRules, action:Option<&TimedAction>,
    imported:ImportedState) -> Result<Vec<u8>,String> {
    let configured = configure(session,bytes,config)?;
    let mut mode = ModeState::decode(&configured)?.ok_or("missing configured Eponym handoff")?;
    let actor = binding(&mode,played)?;
    let profile = config.profiles.iter().find(|p|p.subject == played)
        .ok_or("timed Eponym handoff lacks its actual played profile")?;
    let policy = profile.combat.as_ref().ok_or("timed Eponym handoff lacks supplied combat policy")?;
    if &policy.timed != rules { return Err("handoff cannot replace the original timed-action rules".into()); }
    let current = anatomy(session,&mode,profile)?;
    let charged = ChargeState {subject:played,network:network.clone(),action:action.cloned()};
    charged.validate(policy)?;
    if let Some(action) = &charged.action {
        for c in action.contributors.values() {
            if c.state != crate::timed_action::ContributionState::Cancelled
                && (c.binding.revision != current.revision || !current.document.part(c.binding.part).is_some_and(|p|!p.severed)) {
                return Err("handoff cannot reinterpret a stale paid limb preparation".into());
            }
        }
    }
    let mut known = BTreeSet::new();
    for label in &imported.products.anatomies {
        if !known.insert(label.subject) { return Err("duplicate imported anatomy identity".into()); }
        if let Some(p) = config.profiles.iter().find(|p|p.subject == label.subject) {
            if p.species != label.species || p.revision != label.revision {
                return Err("handoff cannot relabel an existing admitted anatomy".into());
            }
        }
    }
    for roles in &imported.products.movement {
        let supplied = config.profiles.iter().find(|p|p.subject == roles.subject)
            .and_then(|p|p.motion.as_ref()).and_then(|p|p.profile.as_ref());
        if supplied != Some(&roles.profile) { return Err("handoff cannot drop or replace declared movement roles".into()); }
    }
    for (item,part) in &imported.products.items.worn {
        if !imported.products.items.at.contains_key(item) {
            return Err(format!("imported worn item {} lacks its original physical record",item.0));
        }
        // Preserve historical attachment addresses. Their current usability is
        // a native/anatomy reading, not a new ability or custody grant.
        let _ = part;
    }
    let mut projects = Vec::new();
    for saved in &imported.products.projects {
        let held = session.sim.state().projects.get(saved.project).ok_or("imported native project is absent")?;
        if saved.site != imported.site || saved.created_site != imported.site || held.site != saved.site
            || held.carrier.is_some() || saved.at != saved.created_at {
            return Err("handoff requires supported fixed native project creation locations; carried/relocated work is unavailable".into());
        }
        let entry = session.entries.iter().find(|entry| match &entry.command {
            Command::Project(Change::Act{..}) => serde_json::from_str::<Acted>(&entry.outcome)
                .is_ok_and(|a|a.project == Some(saved.project) && a.receipt.accepted()),
            Command::Project(Change::ActOwned{..}) => serde_json::from_str::<OwnedProjectActed>(&entry.outcome)
                .is_ok_and(|a|a.acted.project == Some(saved.project) && a.acted.receipt.accepted()),
            _=>false,
        }).ok_or("imported project lacks an actual accepted native project entry")?;
        projects.push(ProjectPose {project:saved.project,site:saved.site,window:actor.window.clone(),
            at:imported.site_cell(saved.at)?,entry:entry.id.clone()});
    }
    let actions = mode.actions.as_mut().ok_or("missing Eponym action handoff")?;
    actions.charges = vec![charged]; actions.projects = projects; actions.imported = Some(imported);
    mode.version = 3; mode.validate()?;
    serde_json::to_vec(&mode).map_err(|e|e.to_string())
}
