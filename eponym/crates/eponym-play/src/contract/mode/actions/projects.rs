// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPose {
    pub project:ProjectId, pub site:Id, pub window:Window,
    /// Fixed creation pose. This supplement neither moves nor owns native stock.
    pub at:[i64;3], pub entry:Key,
}

pub(super) fn act(session:&mut Session, mode:&ModeState, actions:&mut ActionState,
    actor:&PoseBinding, participant:Id, visible:&View, process:&str, target:Option<Id>,
    project:Option<ProjectId>, lots:&[Key], selections:&[DebitSelection]) -> Result<String,String> {
    if !lots.is_empty() {
        if let Some(target) = target.filter(|id|*id != actor.entity) {
            if !visible.shows(target) { return Err("Eponym material target is outside native awareness".into()); }
            if mode.bindings.iter().any(|b|b.entity == target) {
                check_target(session,mode,actor,Some(target),visible)?;
            } else {
                let imported = actions.imported.as_ref().ok_or("Eponym material target has no actual saved location")?;
                let (site,point) = imported.item_at(session,mode,crate::ItemId(target))?;
                let volume = session.sim.volume(actor.site,actor.window.min,actor.window.max,actor.window.depth)?;
                let here = crate::walking::to_site(&volume,actor.current(session)?.cell().map_err(debug)?);
                if site != actor.site || point != here { return Err("Eponym saved material target is not at the actor".into()); }
            }
        }
    }
    let volume = session.sim.volume(actor.site,actor.window.min,actor.window.max,actor.window.depth)?;
    let at = crate::walking::to_site(&volume,actor.current(session)?.cell().map_err(debug)?);
    let previous: BTreeSet<_> = session.sim.state().projects.records.keys().copied().collect();
    if let Some(id) = project {
        let held = session.sim.state().projects.get(id).ok_or("Eponym project is absent")?;
        let pose = actions.projects.iter().find(|p|p.project == id)
            .ok_or("Eponym project has no accepted physical location")?;
        if session.sim.project_site(held) != Some(actor.site) || pose.site != actor.site
            || pose.at != at || held.carrier.is_some() {
            return Err("Eponym project work needs its fixed current creation position".into());
        }
    }
    let out = session.command(Command::Project(Change::ActOwned {participant,actor:actor.entity,target,
        process:process.into(),project,cause:None,lots:lots.to_vec(),selections:selections.to_vec()}))?;
    let accepted: OwnedProjectActed = serde_json::from_str(&out).map_err(|e|e.to_string())?;
    if !accepted.acted.receipt.accepted() { return Err("native Eponym project refused".into()); }
    if let Some(id) = accepted.acted.project {
        if previous.contains(&id) && project != Some(id) {
            return Err("Eponym process selected physically unchecked existing work".into());
        }
        if !actions.projects.iter().any(|p|p.project == id) {
            if previous.contains(&id) { return Err("Eponym existing project lacks an accepted pose".into()); }
            actions.projects.push(ProjectPose {project:id,site:actor.site,window:actor.window.clone(),at,
                entry:session.entries.last().ok_or("missing accepted project entry")?.id.clone()});
        }
        if let Some(output) = accepted.acted.output {
            if session.sim.state().population.get(output).and_then(|e|e.resident_site()) != Some(actor.site) {
                return Err("native Eponym project output is not resident at its creation site".into());
            }
        }
    } else if accepted.acted.output.is_some() { return Err("native project output lacks a project identity".into()); }
    Ok(out)
}
