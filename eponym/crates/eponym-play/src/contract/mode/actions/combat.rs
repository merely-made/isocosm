// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::combat::{precision,resolve};

pub(super) fn action(session:&mut Session, mode:&ModeState, actions:&mut ActionState,
    actor:&PoseBinding, action:&Action, visible:&View) -> Result<String,String> {
    let profile = actions.profiles.iter().find(|p|p.subject == actor.subject)
        .ok_or("Eponym combat has no authored subject profile")?.clone();
    let policy = profile.combat.as_ref().ok_or("Eponym combat policy is not supplied")?;
    let record = anatomy(session,mode,&profile)?;
    let index = match actions.charges.iter().position(|c|c.subject == actor.subject) {
        Some(index)=>index,
        None=> {
            actions.charges.push(ChargeState {subject:actor.subject,network:policy.network.clone(),action:None});
            actions.charges.len()-1
        },
    };
    let charge = &mut actions.charges[index];
    charge.validate(policy)?;
    match action {
        Action::Prepare{direction} => charge.prepare(record,*direction,Tick(session.sim.state().tick)),
        Action::JoinLimb{part} => charge.join(record,policy,*part),
        Action::Charge{tick} => charge.charge(record,policy,*tick),
        Action::Gate{node,open} => charge.gate(*node,*open),
        Action::ReleaseAgainst{target,tick,wounds} => {
            let target = binding(mode,*target)?;
            if target.site != actor.site || !visible.shows(target.entity) {
                return Err("Eponym strike target is outside the actual native site cut".into());
            }
            let target_profile = actions.profiles.iter().find(|p|p.subject == target.subject)
                .ok_or("Eponym strike target has no authored anatomy label")?;
            let target_record = anatomy(session,mode,target_profile)?;
            let volume = session.sim.volume(actor.site,actor.window.min,actor.window.max,actor.window.depth)?;
            let target_pose = target.in_frame(session,&volume)?;
            let strikes = charge.release(record,policy,*tick)?;
            if strikes.is_empty() {
                if !wounds.is_empty() { return Err("uncharged Eponym release cannot carry wound payments".into()); }
                return Ok("uncharged release".into());
            }
            let result = if policy.rules.revision == crate::COMBAT_RULES_REVISION {
                precision::resolve(actor.subject,profile.revision,record.document,actor.current(session)?,
                    target.subject,target_record.document,target_pose,&volume.ground(),&strikes,policy.rules)
            } else {
                resolve(actor.subject,profile.revision,record.document,actor.current(session)?.cell().map_err(debug)?,
                    target.subject,target_record.document,target_pose.cell().map_err(debug)?,
                    &volume.ground(),&strikes,policy.rules)
            }.map_err(debug)?;
            let receipts = injury(session,target.entity,result.harm,&result.severed_parts,wounds)?;
            Ok(format!("volley {:?}; harm {}\n{}",result.strikes,result.harm,receipts.join("\n")))
        },
        _=>Err("not a borrowed Eponym combat operation".into()),
    }
}

/// The existing game harm law lowered through actual native Wound/Owned
/// commands. Selections may describe these exact debits, never set the harm.
pub(super) fn injury(session:&mut Session, entity:Id, harm:u16, severed:&[PartId],
    payments:&[Vec<DebitSelection>]) -> Result<Vec<String>,String> {
    let body = session.sim.state().population.get(entity).ok_or("Eponym wound target is absent")?;
    let cells = if harm == 0 || !severed.is_empty() {0} else {
        let living = body.living().try_fold(0u64,|all,(id,part)|all
            .checked_add(u64::from(cosmograph::anatomy::living_cells(body.extent(id),part)))
            .ok_or("Eponym living cell count overflow"))?;
        let amount = living.checked_mul(u64::from(harm)).ok_or("Eponym harm cell count overflow")?
            .div_ceil(100).max(1);
        u32::try_from(amount).map_err(|e|e.to_string())?
    };
    let mut wounds:Vec<_> = severed.iter().map(|part|(Some(*part),u32::MAX)).collect();
    if cells > 0 { wounds.push((None,cells)); }
    if payments.len() != wounds.len() { return Err("Eponym wound payments must name every resolved native wound".into()); }
    wounds.into_iter().zip(payments).map(|((part,cells),selections)| {
        let command = Command::Wound {entity,part,cells};
        session.command(Command::Owned {command:Box::new(command),selections:selections.clone()})
    }).collect()
}
