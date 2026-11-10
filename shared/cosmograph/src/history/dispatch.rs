// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The native door each logged command enters the world by, split out of
//! `history.rs` to keep it under the line ceiling.

use super::Command;
use crate::{Result, simulation::Simulation};

pub(super) fn dispatch(sim: &mut Simulation, command: &Command) -> Result<String> {
    match command {
        Command::Location { location } => sim.author_location(location)
            .and_then(|o| sim.founding_event(command, o)),
        Command::ActOwned { participant, actor, target, process, cause, selections } =>
            crate::society().and_then(|()| sim.owned_act(*participant,*actor,*target,process,cause.clone(),selections)),
        Command::Owned { .. } => Err("owned direct command needs its native Session".into()),
        Command::AdvanceGeometry { .. } => Err("geometry advance needs its native Session".into()),
        Command::AdvanceSpatial { .. } => Err("spatial advance needs its native Session".into()),
        Command::AdvanceBirthSpatial { .. } => Err("birth spatial advance needs its native Session".into()),
        Command::BirthSpatial { .. } => Err("birth spatial command needs its native Session".into()),
        Command::Window(change) => sim.change_window(change),
        Command::Party(change) => crate::society().and_then(|()| sim.party_change(change)),
        Command::Spread(change) => sim.spread(change),
        Command::Project(change) => crate::society().and_then(|()| sim.project_change(change)),
        Command::Develop { entity, seed } => {
            sim.develop_arrival(*entity, *seed)?;
            Ok("developed".into())
        },
        Command::Act {
            actor,
            target,
            process,
            cause,
        } => {
            let receipt = sim.execute(*actor, *target, process, cause.clone());
            serde_json::to_string(&receipt).map_err(|e| e.to_string())
        },
        Command::Inspect(id) => {
            sim.inspect(*id)?;
            Ok("inspected".into())
        },
        Command::Release(id) => {
            sim.release(*id);
            Ok("released".into())
        },
        Command::Collect => {
            sim.collect();
            Ok("collected".into())
        },
        Command::Learn { entity, event } => Ok(sim.learn(*entity, event)?.to_string()),
        Command::PlaceMatter {
            site,
            account,
            amount,
        } => {
            sim.place(*site, account, *amount)?;
            Ok("placed".into())
        },
        Command::Edit { site, edit, by } => {
            match by {
                None => sim.edit(*site, edit.clone())?,
                Some(actor) => sim.edit_by(*actor, *site, edit.clone())?,
            }
            Ok("edited".into())
        },
        Command::Patch { entity, patch } => {
            sim.set_patch(*entity, *patch)?;
            Ok("placed".into())
        },
        Command::Join => Ok(format!("participant:{}", sim.join()?)),
        Command::Take {
            participant,
            critter,
        } => {
            sim.take(*participant, *critter)?;
            Ok("taken".into())
        },
        Command::TakeAlso {
            participant,
            critter,
        } => {
            sim.take_also(*participant, *critter)?;
            Ok("taken".into())
        },
        Command::Leave {
            participant,
            critter,
        } => {
            sim.leave(*participant, *critter)?;
            Ok("left".into())
        },
        Command::Regard { participant, knows, regard } => sim.set_regard(*participant, knows, regard).map(|()| "regarded".into()),
        Command::Hold { participant, lineage } => sim.hold(*participant, lineage).map(|()| "held".into()),
        Command::Unhold { participant, lineage } => sim.unhold(*participant, lineage).map(|()| "unheld".into()),
        Command::Steer { participant, lineage, on, off } => sim.steer(*participant, lineage, on, off),
        Command::Turn { participant, lineage, shift } => sim.turn(*participant, lineage, *shift).map(|()| "turned".into()),
        Command::Nudge {
            participant,
            critter,
            aim,
            toward,
            act,
        } => {
            sim.nudge(*participant, *critter, *aim, *toward, act.clone())?;
            Ok("nudged".into())
        },
        Command::ForceBirth { parent } => {
            let receipt = sim.force_birth(*parent)?;
            serde_json::to_string(&receipt).map_err(|e| e.to_string())
        },
        Command::Kill { entity } => {
            sim.kill(*entity)?;
            Ok("killed".into())
        },
        Command::Express {
            entity,
            part,
            tracts,
        } => Ok(format!("expressed:{}", sim.express(*entity, *part, tracts)?)),
        Command::Revise {
            lineage,
            development,
        } => {
            sim.revise(lineage, development)?;
            Ok("revised".into())
        },
        Command::Policy { lineage, policy } => {
            sim.adopt_policy(lineage, policy)?;
            Ok("adopted".into())
        },
        Command::Speciate { founder, name } => sim.speciate(*founder, name),
        Command::Assert(assertion) => sim.assert(assertion).and_then(|o| sim.founding_event(command, o)),
        Command::AttachSite { place, attachment } => sim.attach_site(place, attachment),
        Command::Wound {
            entity,
            part,
            cells,
        } => {
            let wounded = sim.wound(*entity, *part, *cells)?;
            serde_json::to_string(&wounded).map_err(|e| e.to_string())
        },
        Command::Arrive(arrival) => {
            let outcome = format!("entity:{}", sim.arrive(arrival)?);
            sim.founding_event(command, outcome)
        },
        Command::Embody { entity, body } => {
            sim.embody(*entity, body.clone())?;
            Ok("embodied".into())
        },
        Command::Name { by, of, name } => {
            sim.name(*by, *of, name)?;
            Ok("named".into())
        },
        Command::Know(knowing) => sim.know(knowing),
        Command::Goal(change) => crate::society().and_then(|()| sim.goal_change(change)),
        Command::Institution(change) => crate::society().and_then(|()| sim.institution_change(change)),
        Command::Custody(change) => crate::society().and_then(|()| sim.custody_change(change)),
        Command::ActGraft { participant, actor, target, process, cause, carry, selections } =>
            sim.act_graft(*participant, *actor, *target, process, cause.clone(), carry, selections),
        Command::Social(act) => {
            let answer = sim.social(act)?;
            serde_json::to_string(&answer).map_err(|e| e.to_string())
        },
    }
}
