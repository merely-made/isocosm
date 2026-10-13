// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual DM pack, storylet and goal decisions on this host's native owner.

use cambium::{el,text,lens,caret_text_field,TextInput};
use isocosm_overlay::{Intent,Seat,Tick,vtt::{VttIntent,VttIntentEnvelope,HookIntent}};
use isometry_campaign::native::{pack::{self,PackEnvelope,PackInput},storylets};
use super::{model::App,view::{Child,button}};
use crate::products::Mode;
use std::collections::BTreeSet;

impl App {
    fn vtt_pack(&mut self,key:u64,intent:PackInput)->Result<(),String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        let tick=self.world.read(|session|session.sim.state().tick)?;
        let input=PackEnvelope {participant:access.participant.0,tick,intent};
        self.vtt_bytes(key,serde_json::to_vec(&input).map_err(|error|error.to_string())?)
    }
    fn vtt_hook(&mut self,key:u64,intent:HookIntent)->Result<(),String> {
        let local=self.windows.get(&key).ok_or("window is closed")?;
        let access=local.native.projection()?.access;
        let tick=self.world.read(|session|session.sim.state().tick)?;
        let input=VttIntentEnvelope {participant:access.participant,tick:Tick(tick),intent:Intent::Game(VttIntent::Hook(intent))};
        self.vtt_bytes(key,serde_json::to_vec(&input).map_err(|error|error.to_string())?)
    }
}

pub(super) fn panel(app:&App,key:u64)->Child {
    let mut rows:Vec<Child>=vec![Box::new(el("h3",text("Adventures and current goals")))];
    let read=(|| {
        let access=app.windows.get(&key).ok_or("window is closed")?.native.projection()?.access;
        if access.seat!=Seat::Dm {return Err("Adventure and goal authoring belongs to the DM.".into());}
        app.world.read_game(Mode::Vtt,|session,game| {
            let packs=pack::read_packs(session,game,access,&BTreeSet::new())?;
            let goals=isometry_campaign::native::goal_hooks(session,access.subject.map_or(u64::MAX,|id|id.0),
                isocosm::directing::readings::Mode::Creative)?;
            let polities=isometry_campaign::native::polity_readings(session,access)?;
            let storylets=storylets::read_storylets(session,game,access)?;
            Ok((packs,goals,polities,storylets))
        })
    })();
    let (packs,goals,polities,storylets)=match read {Ok(read)=>read,Err(reason)=>{
        rows.push(Box::new(el("p",text(reason))));return Box::new(el("section",rows).attr("class","panel"));
    }};
    for view in packs {
        let definition=&view.pack.definition;
        let state=if view.pack.activation.is_some() {"active"}
            else if view.readiness.compatible {"compatible, dormant"} else {"dormant"};
        rows.push(Box::new(el("p",text(format!("{}: {state}",definition.name)))));
        for refusal in [&view.readiness.condition_refusal,&view.readiness.native_refusal].into_iter().flatten() {
            rows.push(Box::new(el("p",text(refusal.clone()))));
        }
        rows.push(Box::new(el("p",text(format!("{} proposed assertions",definition.assertions.len())))));
        for assertion in &definition.assertions {
            rows.push(Box::new(el("p",text(serde_json::to_string(assertion).unwrap_or_else(|error|error.to_string())))));
        }
        if let Some(content)=&definition.content {
            for map in &content.maps {rows.push(Box::new(el("p",text(format!("Proposed map {}",map.map.id)))));}
            for storylet in content.storylets.values() {rows.push(Box::new(el("p",text(format!("Proposed storylet {}",storylet.entry)))));}
            for item in &content.rewards {rows.push(Box::new(el("p",text(format!("Item offer {}",item.name)))));}
        }
        if view.pack.activation.is_none() {
            for force in [false,true] {
                let pack=definition.key.clone();
                rows.push(button(format!("pack-preview-{pack}-{force}"),if force {"Preview forced acceptance"} else {"Preview acceptance"}.into(),key,
                    move |app|app.vtt_pack(key,PackInput::Preview {key:pack.clone(),force})));
            }
            if view.preview_current && let Some(preview)=view.pack.previews.last() {
                let pack=definition.key.clone();let event=preview.witness.event;let force=preview.prepared.forced;
                if preview.prepared.probe.native_refusal.is_none() && (force || preview.prepared.probe.compatible) {
                    rows.push(button(format!("pack-accept-{pack}"),if force {"Explicitly force this preview"} else {"Accept this preview"}.into(),key,
                        move |app|app.vtt_pack(key,if force {PackInput::Force {key:pack.clone(),preview:event}}
                            else {PackInput::Accept {key:pack.clone(),preview:event}})));
                }
            }
        }
    }
    for view in storylets {
        rows.push(Box::new(el("p",text(view.label))));
        for (role,actor) in view.cast {rows.push(Box::new(el("p",text(format!("{role}: {actor}")))));}
        if let Some(reason)=view.refusal {rows.push(Box::new(el("p",text(reason))));}
        else {
            let input=view.input;
            rows.push(button(format!("storylet-{}",input.key),"Play this storylet".into(),key,
                move |app|app.vtt_bytes(key,serde_json::to_vec(&input).map_err(|error|error.to_string())?)));
        }
    }
    rows.push(Box::new(el("label",(text("Reshape note "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),move |app:&mut App|
            &mut app.windows.get_mut(&key).expect("the open window owns this field").hook_note)))));
    for hook in goals {
        for line in isometry_views::native_goal_lines(&hook) {rows.push(Box::new(el("p",text(line))));}
        for (label,intent) in [("Take up",HookIntent::TakeUpGoal {arc:hook.event,goal:hook.goal.goal.clone()}),
            ("Drop",HookIntent::DropGoal {arc:hook.event,goal:hook.goal.goal.clone()})] {
            rows.push(button(format!("hook-{}-{label}",hook.goal.goal),label.into(),key,move |app|app.vtt_hook(key,intent.clone())));
        }
        let goal=hook.goal.goal;let arc=hook.event;
        rows.push(button(format!("hook-{goal}-reshape"),"Reshape".into(),key,move |app| {
            let note=app.windows.get(&key).ok_or("window is closed")?.hook_note.text().to_owned();
            app.vtt_hook(key,HookIntent::ReshapeGoal {arc,goal:goal.clone(),note})
        }));
    }
    rows.push(Box::new(el("h3",text("Current polity readings"))));
    for polity in polities {for line in isometry_views::native_polity_lines(&polity) {rows.push(Box::new(el("p",text(line))));}}
    Box::new(el("section",rows).attr("class","panel"))
}
