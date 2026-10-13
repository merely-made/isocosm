// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::founding::{ENERGY,FATIGUE,HUNGER};
use isometer_space::places::Places;

/// Exact native debit choices, used only if the existing fall law charges them.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallPayment {
    pub wounds: Vec<Vec<DebitSelection>>,
    pub hunger: Vec<DebitSelection>, pub fatigue: Vec<DebitSelection>,
}

pub(super) fn advance(session:&mut Session, mode:&mut ModeState, actions:&ActionState,
    actor:&PoseBinding, participant:Id, step:u64, input:MotionInput, payment:&FallPayment,
    solver:MotionSolver) -> Result<String,String> {
    let profile = actions.profiles.iter().find(|p|p.subject == actor.subject)
        .ok_or("Eponym movement has no authored subject profile")?;
    let policy = profile.motion.as_ref().ok_or("Eponym movement rules are not supplied")?;
    let record = anatomy(session,mode,profile)?;
    let prior = actor.current(session)?;
    if prior.step.checked_add(1) != Some(step) { return Err("wrong Eponym motion step".into()); }
    if (input.move_x != 0 || input.move_z != 0) && !mobile(session,actor.entity)? {
        return Err("Eponym played body is currently immobile".into());
    }
    policy.rules.validate().map_err(debug)?;
    let rules = match &policy.profile {
        Some(profile) => profile.project_view(record,policy.rules.speed).map_err(debug)?
            .rules_for(policy.rules).map_err(debug)?,
        None => policy.rules,
    };
    if rules.speed == 0 && (input.move_x != 0 || input.move_z != 0) {
        return Err("Eponym current supports cannot move".into());
    }
    let volume = session.sim.volume(actor.site,actor.window.min,actor.window.max,actor.window.depth)?;
    let outcome = solver.advance(&volume.ground(),prior,input,rules).map_err(debug)?;
    if outcome.pose.step != step { return Err("Eponym solver returned another fixed step".into()); }
    outcome.pose.validate().map_err(debug)?;
    let site_cell = crate::walking::to_site(&volume,outcome.pose.cell().map_err(debug)?);
    let patch = Places::derive(&volume,actor.places.unwrap_or_default())?.place_at(site_cell)
        .ok_or("Eponym accepted contact has no native patch")?;
    let mut next = actor.clone(); next.pose = outcome.pose; next.patch = patch;
    // Validate the exact offered contact against current native geometry before
    // publishing its patch. The supplied game solver owns envelope collision.
    if patch.site != actor.site || !volume.material(site_cell)
        .is_some_and(|m|m == volume.materials.air || m == volume.materials.water) {
        return Err("Eponym solver contact is not a clear native cell".into());
    }
    let patch_receipt = session.command(Command::Patch {entity:actor.entity,patch:Some(patch)})?;
    next.current(session)?;
    let mut receipts = vec![patch_receipt];
    if let Some(distance) = outcome.landed_distance.filter(|d|*d > crate::SAFE_FALL) {
        let harm = distance.saturating_sub(crate::SAFE_FALL).saturating_mul(10).clamp(0,100) as u16;
        receipts.extend(super::combat::injury(session,actor.entity,harm,&[],&payment.wounds)?);
        if session.sim.state().population.get(actor.entity).is_some_and(|e|e.alive) {
            for (process,selections) in [(HUNGER,&payment.hunger),(FATIGUE,&payment.fatigue)] {
                let out = session.command(Command::ActOwned {participant,
                    actor:actor.entity,target:None,process:process.into(),cause:None,selections:selections.clone()})?;
                let acted: OwnedActed = serde_json::from_str(&out).map_err(|e|e.to_string())?;
                if !acted.receipt.accepted() { return Err("native fall exertion refused".into()); }
                receipts.push(out);
            }
        } else if !payment.hunger.is_empty() || !payment.fatigue.is_empty() {
            return Err("a dead fall survivor cannot spend exertion selections".into());
        }
    } else if !payment.wounds.is_empty() || !payment.hunger.is_empty() || !payment.fatigue.is_empty() {
        return Err("Eponym motion supplied payments without a damaging landing".into());
    }
    // Preserve the authentic pre-injury geometry stamp. A severed body withdraws
    // this picture until trusted current geometry/bindings are supplied.
    clock::motion(mode,actor,&next)?;
    *mode.bindings.iter_mut().find(|b|b.subject == actor.subject).ok_or("missing Eponym actor binding")? = next;
    Ok(format!("motion {step}: {:?}\n{}",outcome.pose,receipts.join("\n")))
}

fn mobile(session:&Session,entity:Id) -> Result<bool,String> {
    let body = session.sim.state().population.get(entity).ok_or("Eponym body is absent")?;
    let (mut lost,mut all) = (0u64,0u64);
    for (id,part) in &body.parts {
        let cells = u64::from(isocosm::anatomy::capacity(body.extent(*id)));
        all = all.checked_add(cells).ok_or("Eponym cell count overflow")?;
        lost = lost.checked_add(if body.lives(*id) {part.lost.len() as u64} else {cells})
            .ok_or("Eponym wound count overflow")?;
    }
    let wound = lost.checked_mul(100).ok_or("Eponym wound ratio overflow")?.checked_div(all).unwrap_or(0).min(100);
    let fatigue = isocosm::needs::want(&session.sim.genesis().rules,body,ENERGY)
        .map_or(0,|want|want.deficit().min(u64::from(crate::MAX_NEED)));
    Ok(body.alive && session.sim.name_of(entity).is_some()
        && wound < u64::from(crate::MOBILITY_WOUND) && fatigue < u64::from(crate::MAX_NEED))
}
