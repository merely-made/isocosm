// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::founding::{ENERGY,FATIGUE,HUNGER};

/// Exact native debit choices, used only if the existing fall law charges them.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FallPayment {
    pub wounds: Vec<Vec<DebitSelection>>,
    pub hunger: Vec<DebitSelection>, pub fatigue: Vec<DebitSelection>,
}

/// One held input over consecutive fixed steps (case 1117).
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run { pub input: MotionInput, pub steps: u32 }

/// A damaging landing's payment, by the fixed step that lands.
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fall { pub step: u64, pub payment: FallPayment }

/// A round's minute holds at most this many fixed 1/60 s steps.
pub const STEPS_PER_MINUTE: u64 = 60 * 60;

pub(super) fn advance(session:&mut Session, mode:&mut ModeState, actions:&ActionState,
    actor:&PoseBinding, participant:Id, step:u64, input:MotionInput, payment:&FallPayment,
    solver:MotionSolver) -> Result<String,String> {
    let rules = rules(session,mode,actions,actor)?;
    let prior = actor.current(session)?;
    if prior.step.checked_add(1) != Some(step) { return Err("wrong Eponym motion step".into()); }
    if (input.move_x != 0 || input.move_z != 0) && !mobile(session,actor.entity)? {
        return Err("Eponym played body is currently immobile".into());
    }
    if rules.speed == 0 && (input.move_x != 0 || input.move_z != 0) {
        return Err("Eponym current supports cannot move".into());
    }
    let volume = crate::contract::mode::memo::volume(&session.sim, actor.site, &actor.window)?;
    let outcome = solver.advance(&volume.ground(),prior,input,rules).map_err(debug)?;
    if outcome.pose.step != step { return Err("Eponym solver returned another fixed step".into()); }
    outcome.pose.validate().map_err(debug)?;
    let site_cell = crate::walking::to_site(&volume,outcome.pose.cell().map_err(debug)?);
    let patch = crate::contract::mode::memo::places(&session.sim,actor.site,&actor.window,actor.places.unwrap_or_default())?.place_at(site_cell)
        .ok_or("Eponym accepted contact has no native patch")?;
    let mut next = actor.clone(); next.pose = outcome.pose; next.patch = patch;
    // Validate the exact offered contact against current native geometry before
    // publishing its patch. The supplied game solver owns envelope collision.
    if patch.site != actor.site || !volume.material(site_cell)
        .is_some_and(|m|m == volume.materials.air || m == volume.materials.water) {
        return Err("Eponym solver contact is not a clear native cell".into());
    }
    // In a round the sim learns the patch from the minute's handback, not
    // from every step (cases 1095, 1107).
    let mut receipts = Vec::new();
    if !super::super::clock::rounds(session) {
        receipts.push(session.command(Command::Patch {entity:actor.entity,patch:Some(patch)})?);
    }
    next.current(session)?;
    receipts.extend(fall(session,actor.entity,participant,outcome.landed_distance,payment)?);
    // Preserve the authentic pre-injury geometry stamp. A severed body withdraws
    // this picture until trusted current geometry/bindings are supplied.
    clock::motion(mode,actor,&next)?;
    *mode.bindings.iter_mut().find(|b|b.subject == actor.subject).ok_or("missing Eponym actor binding")? = next;
    Ok(format!("motion {step}: {:?}\n{}",outcome.pose,receipts.join("\n")))
}

/// A round's minute of motion (cases 1113, 1117): the batch is re-solved
/// over one derivation of the window, must end at the claimed pose, and
/// charges each damaging landing by its step. The sim learns the pose from
/// the minute's handback.
pub(super) fn steps(session:&mut Session, mode:&mut ModeState, actions:&ActionState,
    actor:&PoseBinding, participant:Id, runs:&[Run], end:MotionPose, falls:&[Fall],
    solver:MotionSolver) -> Result<String,String> {
    if !super::super::clock::rounds(session) {
        return Err("outside a round each motion step is its own action".into());
    }
    let total = runs.iter().try_fold(0u64,|n,r| (r.steps > 0).then(|| n + u64::from(r.steps)))
        .ok_or("an Eponym motion run holds no steps")?;
    if total == 0 || total > STEPS_PER_MINUTE { return Err("Eponym minute motion holds 1 to 3600 steps".into()); }
    if !falls.windows(2).all(|w| w[0].step < w[1].step) { return Err("Eponym falls are out of step order".into()); }
    let mut pose = actor.current(session)?;
    let first = pose.step;
    let volume = crate::contract::mode::memo::volume(&session.sim, actor.site, &actor.window)?;
    let ground = volume.ground();
    let mut rules = rules(session,mode,actions,actor)?;
    let mut mobile = mobile(session,actor.entity)?;
    let mut falls = falls.iter().peekable();
    let (mut receipts, mut landed) = (Vec::new(), false);
    let inputs = runs.iter().flat_map(|r| std::iter::repeat_n(r.input,r.steps as usize));
    for input in inputs {
        let step = pose.step.checked_add(1).ok_or("Eponym motion step overflows")?;
        if landed {
            // After a damaging landing the next step is checked as the next
            // single-step action would be.
            if !session.sim.state().population.get(actor.entity).is_some_and(|e|e.alive) {
                return Err("an Eponym minute ends at its fatal landing".into());
            }
            let mut held = actor.clone(); held.pose = pose;
            held.current(session)?;
            rules = self::rules(session,mode,actions,actor)?;
            mobile = self::mobile(session,actor.entity)?;
        }
        let moving = input.move_x != 0 || input.move_z != 0;
        if moving && !mobile { return Err("Eponym played body is currently immobile".into()); }
        if moving && rules.speed == 0 { return Err("Eponym current supports cannot move".into()); }
        let outcome = solver.advance(&ground,pose,input,rules).map_err(debug)?;
        if outcome.pose.step != step { return Err("Eponym solver returned another fixed step".into()); }
        outcome.pose.validate().map_err(debug)?;
        let cell = crate::walking::to_site(&volume,outcome.pose.cell().map_err(debug)?);
        if !volume.material(cell).is_some_and(|m|m == volume.materials.air || m == volume.materials.water) {
            return Err("Eponym solver contact is not a clear native cell".into());
        }
        let payment = falls.next_if(|f| f.step == step).map(|f| &f.payment);
        let empty = FallPayment { wounds: Vec::new(), hunger: Vec::new(), fatigue: Vec::new() };
        let charged = fall(session,actor.entity,participant,outcome.landed_distance,payment.unwrap_or(&empty))?;
        landed = !charged.is_empty();
        receipts.extend(charged);
        pose = outcome.pose;
    }
    if falls.next().is_some() { return Err("an Eponym fall payment names no step of this minute".into()); }
    if pose != end { return Err("Eponym minute motion ends away from its claimed pose".into()); }
    let cell = crate::walking::to_site(&volume,pose.cell().map_err(debug)?);
    let patch = crate::contract::mode::memo::places(&session.sim,actor.site,&actor.window,actor.places.unwrap_or_default())?.place_at(cell)
        .ok_or("Eponym accepted contact has no native patch")?;
    let mut next = actor.clone(); next.pose = pose; next.patch = patch;
    if patch.site != actor.site { return Err("Eponym minute motion left its site".into()); }
    clock::motion(mode,actor,&next)?;
    *mode.bindings.iter_mut().find(|b|b.subject == actor.subject).ok_or("missing Eponym actor binding")? = next;
    // The end pose is the input's; the outcome names the steps and charges.
    let mut outcome = format!("motion {}..={}",first + 1,pose.step);
    for receipt in &receipts { outcome.push('\n'); outcome.push_str(receipt); }
    Ok(outcome)
}

/// The subject's motion rules as its current anatomy projects them.
pub(super) fn rules(session:&Session, mode:&ModeState, actions:&ActionState, actor:&PoseBinding)
    -> Result<crate::MotionRules,String> {
    let profile = actions.profiles.iter().find(|p|p.subject == actor.subject)
        .ok_or("Eponym movement has no authored subject profile")?;
    let policy = profile.motion.as_ref().ok_or("Eponym movement rules are not supplied")?;
    let record = anatomy(session,mode,profile)?;
    policy.rules.validate().map_err(debug)?;
    match &policy.profile {
        Some(profile) => profile.project_view(record,policy.rules.speed).map_err(debug)?
            .rules_for(policy.rules).map_err(debug),
        None => Ok(policy.rules),
    }
}

/// Charges a damaging landing with its exact payment; a payment without one
/// is refused.
fn fall(session:&mut Session, entity:Id, participant:Id, landed:Option<i32>, payment:&FallPayment)
    -> Result<Vec<String>,String> {
    let mut receipts = Vec::new();
    if let Some(distance) = landed.filter(|d|*d > crate::SAFE_FALL) {
        let harm = distance.saturating_sub(crate::SAFE_FALL).saturating_mul(10).clamp(0,100) as u16;
        receipts.extend(super::combat::injury(session,entity,harm,&[],&payment.wounds)?);
        if session.sim.state().population.get(entity).is_some_and(|e|e.alive) {
            for (process,selections) in [(HUNGER,&payment.hunger),(FATIGUE,&payment.fatigue)] {
                let out = session.command(Command::ActOwned {participant,
                    actor:entity,target:None,process:process.into(),cause:None,selections:selections.clone()})?;
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
    Ok(receipts)
}

fn mobile(session:&Session,entity:Id) -> Result<bool,String> {
    let body = session.sim.state().population.get(entity).ok_or("Eponym body is absent")?;
    let wound = cosmograph::harm::wound_share(body);
    let fatigue = cosmograph::needs::want(&session.sim.genesis().rules,body,ENERGY)
        .map_or(0,|want|want.deficit().min(u64::from(crate::MAX_NEED)));
    Ok(body.alive && session.sim.name_of(entity).is_some()
        && wound < u64::from(crate::MOBILITY_WOUND) && fatigue < u64::from(crate::MAX_NEED))
}
