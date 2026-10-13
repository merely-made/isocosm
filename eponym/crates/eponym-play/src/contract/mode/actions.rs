// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product action data beside a borrowed native candidate. The host publishes
//! this data and its candidate Session together, or discards both on refusal.

use super::*;
use std::collections::BTreeSet;
use crate::{AnatomyRecord, CombatRules, MotionInput, MotionRules, MotionSolver, MovementProfile};
use crate::identity::{BodyRevisionId, Tick};
use isocosm::{Command, directing::readings::View, lineage::{LineageBody, SpeciesId}, projects::*};
pub use isocosm_overlay::OrdinaryAccess;
use isocosm_overlay::Seat;
use isometer_core::PartId;
use crate::timed_action::{Direction, TimedActionRules};
use wing_functions::FunctionalNetwork;

mod motion;
mod combat;
mod timed;
mod projects;
pub mod handoff;
pub use timed::ChargeState;
pub use projects::ProjectPose;
pub use motion::FallPayment;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotionPolicy { pub rules: MotionRules, pub profile: Option<MovementProfile> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatPolicy {
    pub rules: CombatRules, pub timed: TimedActionRules, pub network: FunctionalNetwork,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectProfile {
    pub subject: SubjectId, pub species: SpeciesId,
    /// Explicit product revision and its exact native geometry source.
    pub revision: BodyRevisionId, pub native_revision: u64,
    pub motion: Option<MotionPolicy>, pub combat: Option<CombatPolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionState {
    pub profiles: Vec<SubjectProfile>, pub charges: Vec<ChargeState>,
    pub projects: Vec<ProjectPose>, pub records: Vec<ActionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported: Option<handoff::ImportedState>,
}

/// Accepted product inputs retain actual native entry references, not another
/// NativeSaved/history runtime. Native replay remains owned by the host.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRecord {
    pub input:ActionInput, pub native_after:Key, pub entries:Vec<Key>, pub outcome:String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionConfig {
    pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub profiles: Vec<SubjectProfile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionInput {
    pub kind: String, pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub subject: SubjectId, pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Motion { step: u64, input: MotionInput, payment: FallPayment },
    Prepare { direction: Direction }, JoinLimb { part: PartId },
    Charge { tick: Tick }, Gate { node: wing_functions::NodeId, open: bool },
    ReleaseAgainst { target: SubjectId, tick: Tick, wounds: Vec<Vec<DebitSelection>> },
    Act { process: Key, target: Option<Id>, selections: Vec<DebitSelection> },
    Project { process: Key, target: Option<Id>, project: Option<ProjectId>, lots: Vec<Key>,
        selections: Vec<DebitSelection> },
    Consent { project: ProjectId, owner: Owner, worker: Id, grant: bool, process: Option<Key> },
    /// Native advancement is requested explicitly, never as an action side effect.
    Advance { ticks: u64 },
}

pub fn game_source(bytes: &[u8]) -> Key { isocosm::digest(&bytes) }

impl ActionState {
    pub fn validate(&self, mode: &ModeState) -> Result<(),String> {
        let mut subjects = BTreeSet::new();
        for profile in &self.profiles {
            if !subjects.insert(profile.subject) || !mode.bindings.iter().any(|b|b.subject == profile.subject) {
                return Err("Eponym action profiles need distinct explicitly mapped subjects".into());
            }
            if let Some(motion) = &profile.motion {
                motion.rules.validate().map_err(debug)?;
                if (motion.rules.revision == crate::MOVEMENT_PROFILE_REVISION) != motion.profile.is_some() {
                    return Err("Eponym movement revision and supplied anatomy profile disagree".into());
                }
            }
            if let Some(combat) = &profile.combat {
                combat.rules.validate().map_err(debug)?;
                combat.timed.validate().map_err(debug)?;
                combat.network.validate().map_err(debug)?;
            }
        }
        let mut charged = BTreeSet::new();
        for charge in &self.charges {
            if !charged.insert(charge.subject) { return Err("duplicate Eponym charge owner".into()); }
            let policy = self.profiles.iter().find(|p|p.subject == charge.subject)
                .and_then(|p|p.combat.as_ref()).ok_or("charge has no authored combat policy")?;
            charge.validate(policy)?;
        }
        let mut projects = BTreeSet::new();
        if self.projects.iter().any(|p| !projects.insert(p.project)) {
            return Err("duplicate Eponym project pose".into());
        }
        Ok(())
    }
}

/// Trusted author input. Reconfiguration preserves paid charge and project
/// records; changing a charged network requires a separate authoring policy.
pub fn configure(session: &Session, bytes: &[u8], input: &ActionConfig) -> Result<Vec<u8>,String> {
    check_source(session,bytes,input.version,&input.expected_native,&input.expected_game)?;
    let mut mode = ModeState::decode(bytes)?.ok_or("Eponym actions need marked physical bindings")?;
    let old = mode.actions.take();
    let mut actions = ActionState { profiles:input.profiles.clone(), charges:vec![], projects:vec![], records:vec![], imported:None };
    if let Some(old) = old {
        for charge in &old.charges {
            let before = old.profiles.iter().find(|p|p.subject == charge.subject).and_then(|p|p.combat.as_ref());
            let after = actions.profiles.iter().find(|p|p.subject == charge.subject).and_then(|p|p.combat.as_ref());
            if before != after { return Err("reconfiguration cannot replace a paid combat network".into()); }
        }
        actions.charges = old.charges; actions.projects = old.projects; actions.records = old.records;
        actions.imported = old.imported;
    }
    for profile in &actions.profiles {
        let record = anatomy(session,&mode,profile)?;
        if let Some(policy) = &profile.motion {
            if let Some(profile) = &policy.profile { profile.validate_at(&record).map_err(debug)?; }
        }
    }
    mode.version = if actions.imported.is_some() {3} else {2}; mode.actions = Some(actions); mode.validate()?;
    serde_json::to_vec(&mode).map_err(|e|e.to_string())
}

/// Mutates only the host's unpublished candidate. On any error the caller must
/// discard it, including accepted commands preceding a later game refusal.
pub fn submit(session: &mut Session, bytes: Vec<u8>, input: &ActionInput,
    access: OrdinaryAccess, visible: &View, solver: MotionSolver) -> Result<(String,Vec<u8>),String> {
    if input.kind != "eponym:actions" { return Err("not an Eponym action envelope".into()); }
    check_source(session,&bytes,input.version,&input.expected_native,&input.expected_game)?;
    let mut mode = ModeState::decode(&bytes)?.ok_or("Eponym actions need marked mode state")?;
    let mut actions = mode.actions.take().ok_or("Eponym actions need authored configuration")?;
    let binding = binding(&mode,input.subject)?.clone();
    let participant = access.participant.0;
    if access.seat == Seat::CreativeObserver || !session.sim.state().population.get(participant)
        .is_some_and(isocosm::directing::is_participant)
        || access.subject.map(|s|s.0) != Some(binding.entity)
        || session.sim.plays(participant) != Some(binding.entity) || !visible.shows(binding.entity) {
        return Err("Eponym action requires the actual current native played body".into());
    }
    let first_entry = session.entries.len();
    let outcome = match &input.action {
        Action::Advance { ticks } => {
            let work = session.advance(*ticks)?;
            format!("native advanced to {}: {work:?}",session.sim.state().tick)
        },
        action => {
            binding.current(session)?;
            if !session.sim.state().population.get(binding.entity).is_some_and(|e|e.alive) {
                return Err("Eponym physical act requires a living played body".into());
            }
            match action {
                Action::Motion {step,input,payment} => motion::advance(session,&mut mode,&actions,&binding,participant,*step,*input,payment,solver)?,
                Action::Act {process,target,selections} => {
                    if target.is_some_and(|id| !visible.shows(id)) {
                        return Err("Eponym target is outside native awareness".into());
                    }
                    let out = session.command(Command::ActOwned { participant,actor:binding.entity,target:*target,
                        process:process.clone(),cause:None,selections:selections.clone() })?;
                    let acted: OwnedActed = serde_json::from_str(&out).map_err(|e|e.to_string())?;
                    if !acted.receipt.accepted() { return Err("native Eponym process refused".into()); }
                    out
                },
                Action::Project {process,target,project,lots,selections} => projects::act(
                    session,&mode,&mut actions,&binding,participant,visible,process,*target,*project,lots,selections)?,
                Action::Consent {project,owner,worker,grant,process} => session.command(Command::Project(Change::Consent {
                    participant,actor:binding.entity,project:*project,owner:*owner,worker:*worker,grant:*grant,
                    process:process.clone(),cause:None }))?,
                _ => combat::action(session,&mode,&mut actions,&binding,action,visible)?,
            }
        },
    };
    actions.records.push(ActionRecord {input:input.clone(),native_after:source(session),
        entries:session.entries[first_entry..].iter().map(|entry|entry.id.clone()).collect(),outcome:outcome.clone()});
    if actions.records.len() > session.sim.genesis().rules.limits.history {
        return Err("Eponym action receipts exceed native history limit".into());
    }
    mode.version = if actions.imported.is_some() {3} else {2}; mode.actions = Some(actions); mode.validate()?;
    Ok((outcome,serde_json::to_vec(&mode).map_err(|e|e.to_string())?))
}

pub fn submit_bytes(session:&mut Session, bytes:Vec<u8>, input:&[u8], access:OrdinaryAccess,
    visible:&View, solver:MotionSolver) -> Result<(String,Vec<u8>),String> {
    let input:ActionInput = serde_json::from_slice(input).map_err(|e|e.to_string())?;
    submit(session,bytes,&input,access,visible,solver)
}

fn check_source(session:&Session, bytes:&[u8], version:u32, native:&str, game:&str) -> Result<(),String> {
    if version != 1 || native != source(session) || game != game_source(bytes) {
        return Err("Eponym action input differs from its current native/game source".into());
    }
    Ok(())
}
fn debug(error: impl std::fmt::Debug) -> String { format!("{error:?}") }
fn binding(mode:&ModeState, subject:SubjectId) -> Result<&PoseBinding,String> {
    mode.bindings.iter().find(|b|b.subject == subject).ok_or("Eponym subject has no explicit physical binding".into())
}
fn anatomy(session:&Session, mode:&ModeState, profile:&SubjectProfile) -> Result<AnatomyRecord,String> {
    let binding = binding(mode,profile.subject)?;
    binding.current(session)?;
    let entity = session.sim.state().population.get(binding.entity).ok_or("Eponym body is absent")?;
    if profile.native_revision != entity.body_revision { return Err("Eponym authored anatomy profile is stale".into()); }
    Ok(AnatomyRecord { subject:profile.subject,revision:profile.revision,
        document:LineageBody::snapshot(profile.species,entity,&session.sim.genesis().rules)?,native_source:None })
}
fn check_target(session:&Session, mode:&ModeState, actor:&PoseBinding, target:Option<Id>, visible:&View) -> Result<(),String> {
    let Some(target) = target else { return Ok(()); };
    if !visible.shows(target) { return Err("Eponym target is outside native awareness".into()); }
    if target == actor.entity { return Ok(()); }
    let target = mode.bindings.iter().find(|b|b.entity == target).ok_or("Eponym target has no actual physical binding")?;
    if target.site != actor.site || target.current(session)?.cell().map_err(debug)? !=
        actor.in_frame(session,&session.sim.volume(target.site,target.window.min,target.window.max,target.window.depth)?)?
            .cell().map_err(debug)? {
        return Err("Eponym material target is not at the actor's actual position".into());
    }
    Ok(())
}
