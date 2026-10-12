// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionContext {
    pub actor: Id, pub project: ProjectId, pub polity: Id, pub worker: Id,
    pub grant: bool, pub process: Key, pub constitution: Key,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionStamp { pub decision: DecisionContext, pub action: u64 }

impl Simulation {
    pub(crate) fn project_decision_ready(&self, actor:Id, target:Option<Id>) -> Result<String> {
        let input = self.project_decision.as_ref().ok_or("no native collective decision is proposed")?;
        let project = self.project_at_start(input.project).ok_or("collective decision project is absent")?;
        if input.actor != actor || target != Some(input.worker)
            || self.project_consent_ready(project,Owner::Polity(input.polity),actor)?.as_deref() != Some(input.process.as_str())
            || self.state.polities.get(&input.polity).map(|p| crate::digest(&p.constitution)).as_ref() != Some(&input.constitution) {
            return Err("native collective decision names a stale polity, constitution or acting member".into());
        }
        Ok(format!("collective project decision ready: {}",input.polity))
    }
}
