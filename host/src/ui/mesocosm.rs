// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exact offered played-loop answers over the host's current native owner.

use cambium::{el, text};
use isocosm::directing::interim::Happening;
use isocosm_overlay::{EntityHandle, Tick, mesocosm::{BirthAnswer, CheckpointAnswer,
    CheckpointAnswerKind, DeathAnswer, RevisionAnswer}, CandidateHandle};
use mesocosm_runtime::{Occasion, continuation::{LoopAction, LoopAnswer, LoopCut,
    LoopInput, LoopPhase, LoopReading, PersonalQuestion}};
use super::{model::App, view::{Child, button}};
use crate::products::{Input, Mode};

impl App {
    fn mesocosm_loop(&self, key:u64) -> Result<LoopReading,String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let cut=local.native.projection()?;
        self.world.read_game(Mode::Mesocosm, |session, bytes| {
            let game:mesocosm_runtime::spatial::Game=if bytes.is_empty() {Default::default()}
                else {serde_json::from_slice(bytes).map_err(|error|error.to_string())?};
            if game.continuation.as_ref().is_some_and(|saved|!saved.queued.is_empty()) {
                return Err("This saved turn has pending standalone inputs. Resume it in the standalone host.".into());
            }
            mesocosm_runtime::continuation::read_loop(session,&game,cut.access,&cut.visible)
        })
    }

    fn mesocosm_answer(&mut self, key:u64, expected:LoopCut, answer:LoopAnswer) -> Result<(),String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        let tick=self.world.read(|session|session.sim.state().tick)?;
        let input=mesocosm_runtime::spatial::Input::Loop(LoopInput {kind:"mesocosm:loop".into(),
            version:1,participant:access.participant,tick:Tick(tick),expected,
            action:LoopAction::Answer(answer)});
        let request=Input {mode:Mode::Mesocosm,bytes:serde_json::to_vec(&input).map_err(|error|error.to_string())?,
            round:0,region:None};
        local.native.submit_product(&request)?;
        self.refresh();
        Ok(())
    }
}

fn choice(key:u64, cut:&LoopCut, id:String, label:String, answer:LoopAnswer) -> Child {
    let expected=cut.clone();
    button(id,label,key,move |app|app.mesocosm_answer(key,expected.clone(),answer.clone()))
}

pub(super) fn can_step(app:&App, key:u64) -> bool {
    if app.windows.get(&key).and_then(|local|local.native.projection().ok())
        .is_none_or(|cut|cut.access.seat!=isocosm_overlay::Seat::Player) {return false;}
    app.mesocosm_loop(key).is_ok_and(|reading|reading.settings.is_some()
        && matches!(reading.phase,LoopPhase::Running|LoopPhase::Watching))
}

pub(super) fn can_send(app:&App, key:u64) -> bool {
    let Some(input)=app.windows.get(&key).and_then(|local|local.input.as_ref()) else {return false;};
    if input.mode!=Mode::Mesocosm {return false;}
    let Ok(parsed)=serde_json::from_slice::<mesocosm_runtime::spatial::Input>(&input.bytes) else {return false;};
    match parsed {
        mesocosm_runtime::spatial::Input::Loop(_)=>return true,
        mesocosm_runtime::spatial::Input::Game(input) if input.envelope.is_none()=>return true,
        _=>{},
    }
    app.mesocosm_loop(key).is_ok_and(|reading|reading.phase==LoopPhase::Running)
}

pub(super) fn panel(app:&App, key:u64) -> Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h2",text("Played turn")))];
    let reading=match app.mesocosm_loop(key) {
        Ok(reading)=>reading,
        Err(reason)=>{rows.push(Box::new(el("p",text(reason))));
            return Box::new(el("section",rows).attr("class","panel"));},
    };
    let Some(settings)=&reading.settings else {
        rows.push(Box::new(el("p",text("Choose a loop setup input with your mode, pace and collapse choice."))));
        return Box::new(el("section",rows).attr("class","panel"));
    };
    rows.push(Box::new(el("p",text(format!("{:?} · {} ticks per round · {:?}",
        settings.mode,settings.pace.round,settings.on_collapse)))));
    let phase=match reading.phase {
        LoopPhase::Running=>"Playing",LoopPhase::Held=>"Waiting for your answer",
        LoopPhase::LineageEnded=>"Your lineage has ended",LoopPhase::Watching=>"Watching",
        LoopPhase::NewWorldRequested=>"New world requested",
        LoopPhase::WorldEnded=>"This world has ended",
    };
    rows.push(Box::new(el("p",text(phase))));
    if let Some(tick)=reading.world_ended_at {
        rows.push(Box::new(el("p",text(format!("This world's game ended at tick {tick}; its history remains watchable.")))));
    }
    for happening in &reading.happenings {
        let label=match happening {
            Happening::Born {parent,child}=>format!("Birth: member {parent} had member {child}"),
            Happening::Died {critter,next}=>format!("Member {critter} died; next life: {next:?}"),
            Happening::Boundary {tick,..}=>format!("Epoch boundary at tick {tick}"),
            Happening::Collapsed {moved_to,..}=>format!("A region collapsed; next life: {moved_to:?}"),
            Happening::LineageEnded {lineage}=>format!("Lineage {lineage} ended"),
            Happening::WorldEnded {tick}=>format!("This world ended at tick {tick}"),
        };
        rows.push(Box::new(el("p",text(label))));
    }
    if app.windows.get(&key).and_then(|local|local.native.projection().ok())
        .is_none_or(|cut|cut.access.seat!=isocosm_overlay::Seat::Player) {
        rows.push(Box::new(el("p",text("The player's window answers this turn."))));
        return Box::new(el("section",rows).attr("class","panel"));
    }
    if reading.phase==LoopPhase::Held {
        if reading.checkpoint.is_none() {
            if let Some(PersonalQuestion::Birth {parent})=&reading.personal {
                rows.push(choice(key,&reading.cut,"mesocosm-birth-keep-personal".into(),"Keep parent".into(),
                    LoopAnswer::Checkpoint(CheckpointAnswer {entity:EntityHandle(*parent),
                        kind:CheckpointAnswerKind::Birth(BirthAnswer::KeepParent)})));
            }
        }
        if let Some(checkpoint)=&reading.checkpoint {
            match &checkpoint.occasion {
                Occasion::Birth(birth)=>for (suffix,label,kind) in [
                    ("parent","Keep parent",BirthAnswer::KeepParent),
                    ("child","Take offspring",BirthAnswer::TakeOffspring)] {
                    rows.push(choice(key,&reading.cut,format!("mesocosm-birth-{suffix}"),label.into(),
                        LoopAnswer::Checkpoint(CheckpointAnswer {entity:EntityHandle(birth.parent),
                            kind:CheckpointAnswerKind::Birth(kind)})));
                },
                Occasion::Loss(loss)=>for &heir in &checkpoint.heirs {
                    rows.push(choice(key,&reading.cut,format!("mesocosm-heir-{heir}"),format!("Take member {heir}"),
                        LoopAnswer::Checkpoint(CheckpointAnswer {entity:EntityHandle(loss.critter),
                            kind:CheckpointAnswerKind::Death(DeathAnswer {next:EntityHandle(heir)})})));
                },
                Occasion::Epoch(_)=>for offer in &reading.offers {
                    if let Some(reason)=&offer.why_not {
                        rows.push(Box::new(el("p",text(format!("{}: {reason}",offer.name)))));
                    } else if let Some(subject)=app.windows.get(&key)
                        .and_then(|local|local.native.projection().ok()).and_then(|cut|cut.access.subject) {
                        rows.push(choice(key,&reading.cut,format!("mesocosm-review-{}",offer.index),
                            format!("Choose {}",offer.name),LoopAnswer::Checkpoint(CheckpointAnswer {entity:subject,
                                kind:CheckpointAnswerKind::EpochReview(RevisionAnswer {
                                    revision:CandidateHandle(offer.index as u64)})})));
                    }
                },
            }
        }
    }
    if matches!(reading.phase,LoopPhase::LineageEnded|LoopPhase::Watching|LoopPhase::WorldEnded) {
        for &member in &reading.lineages {
            rows.push(choice(key,&reading.cut,format!("mesocosm-lineage-{}",member.0),
                format!("Play member {}",member.0),LoopAnswer::OtherLineage(member)));
        }
        if matches!(reading.phase,LoopPhase::LineageEnded|LoopPhase::WorldEnded) {
            rows.push(choice(key,&reading.cut,"mesocosm-watch".into(),"Keep watching".into(),LoopAnswer::Watch));
        }
        rows.push(choice(key,&reading.cut,"mesocosm-new-world".into(),"Request a new world".into(),LoopAnswer::NewWorld));
    }
    if reading.phase==LoopPhase::NewWorldRequested {
        rows.push(choice(key,&reading.cut,"mesocosm-cancel-new-world".into(),
            "Keep watching this world".into(),LoopAnswer::Watch));
        rows.push(Box::new(el("p",text("Save this world, then reopen with your chosen founding input to start the next one."))));
    }
    Box::new(el("section",rows).attr("class","panel"))
}
