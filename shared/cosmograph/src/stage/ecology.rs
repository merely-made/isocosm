// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Observe only a fully published scheduled act, including biological identity.

use super::*;
use crate::ecology::{ActionProof, BiologicalBirth, CounterProof};

pub(super) struct EcologyPublication {
    act: ActionProof,
    target_before: Option<Entity>,
    biological: Vec<(Id, bool)>,
}

fn cells(entity: &Entity) -> Option<u64> {
    let doc = entity.body.as_ref()?;
    entity.living().try_fold(0u64, |sum, (id, part)| {
        let geometry = doc.part(id)?;
        sum.checked_add(u64::from(crate::anatomy::living_cells(geometry.half_extent, part)))
    })
}

impl Simulation {
    pub(super) fn ecology_publication(&self, stage: &Stage) -> Option<EcologyPublication> {
        if !stage.scheduled { return None; }
        let spec = self.ecology_spec(&stage.process)?;
        let actor = self.state.population.get(stage.actor)?;
        let target = stage.target.and_then(|id| self.state.population.get(id));
        let act = ActionProof {
            action: stage.action.clone(), tick: self.state.tick, process: stage.process.clone(),
            actor: stage.actor, actor_lineage: actor.lineage.clone(), target: stage.target,
            target_lineage: target.map(|entity| entity.lineage.clone()), count: stage.count,
            site: actor.resident_site(), credits: stage.ecology_credits.clone(), births: vec![], counter: 0,
        };
        if !spec.selects(&act) { return None; }
        Some(EcologyPublication { act, target_before: target.cloned(),
            biological: stage.birth_places.iter().filter(|birth|
                birth.parent == stage.actor && birth.action == stage.action && birth.process == stage.process)
                .map(|birth| (birth.child, birth.parent_alive)).collect() })
    }

    pub(super) fn published_ecology(&mut self, publication: Option<EcologyPublication>) {
        let Some(mut publication) = publication else { return; };
        let act = &mut publication.act;
        let Some(spec) = self.ecology_spec(&act.process) else { return; };
        for (id, parent_alive) in publication.biological {
            let Some(child) = self.state.population.get(id) else { continue; };
            let parent = Relation { subject: id, object: act.actor, kind: "sim:parent".into(), value: 0 };
            if parent_alive && child.born == act.tick && child.alive && self.state.relations.contains(&parent) {
                let matter = crate::meaning::mass(&crate::anatomy::books(child), &self.genesis.rules);
                if matter > 0 { act.births.push(BiologicalBirth { child: id, lineage: child.lineage.clone(), matter }); }
            }
        }
        if spec.counterplay.selection.matches(act) && act.target != Some(act.actor) {
            if let (Some(before), Some(after)) = (publication.target_before.as_ref(),
                act.target.and_then(|id| self.state.population.get(id))) {
                if before.alive {
                    act.counter = match &spec.counterplay.proof {
                        CounterProof::TargetDied => u64::from(!after.alive),
                        CounterProof::LivingCellsLost { .. } => cells(before).zip(cells(after))
                            .map_or(0, |(was, now)| was.saturating_sub(now)),
                        CounterProof::TargetDebited { account, .. } =>
                            crate::anatomy::held(before, &self.genesis.rules, account)
                                .saturating_sub(crate::anatomy::held(after, &self.genesis.rules, account)),
                    };
                }
            }
        }
        self.record_ecology(publication.act);
    }
}
