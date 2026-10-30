// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Current DM goal and polity readings, separate from historical map facts.

use cambium::{el,text,lens,clickable,caret_text_field,TextInput};
use isocosm_overlay::vtt::HookIntent;
use isocosm_vtt::native::hooks::GoalHook;
use isocosm::polities::PolityReading;
use isocosm_vtt::native::places::PlaceView;
use crate::{UiState,UiChild,PartyRequest,PartyView};

pub fn native_goal_lines(hook:&GoalHook)->Vec<String> {
    let goal=&hook.goal;
    let mut lines=vec![format!("Goal {} · holder {} · recorded at {} · {}",goal.goal,goal.holder,goal.recorded_at,goal.text),
        format!("Adoption act {} at tick {}",goal.adoption.id,goal.adoption.tick)];
    for part in &goal.participation {
        lines.push(format!("Actor {} served this goal through {} at tick {}",part.actor,part.event.id,part.event.tick));
    }
    for link in &goal.causes {lines.push(format!("Immediate cause: {} follows {}",link.event,link.cause));}
    if let Some(end)=&goal.ending {lines.push(format!("{:?} through {}",end.ending,end.event.id));}
    lines
}

pub fn native_polity_lines(read:&PolityReading)->Vec<String> {
    let mut lines=vec![format!("Polity {} · {} actual membership records",read.id,read.members.len())];
    if let Some(authored)=&read.authored {
        lines.push(format!("Authored {}: {}",authored.name,authored.tags.iter().cloned().collect::<Vec<_>>().join(", ")));
        lines.push(format!("Authored claims: {}",authored.claims.iter().cloned().collect::<Vec<_>>().join(", ")));
    }
    for (tag,members) in &read.member_tags {lines.push(format!("Member tag {tag}: {members:?}"));}
    if let Some(disagreement)=&read.disagreement {
        lines.push(format!("Authored-only tags: {:?}; member-only tags: {:?}",disagreement.authored_only,disagreement.members_only));
    }
    for member in &read.members {
        lines.push(format!("Member {} · constitution {} · native relation {}",member.id,member.constitutional,member.relation));
        for act in &member.acts {lines.push(format!("{} at {} · subject {}, object {:?} · {}",act.id,act.tick,act.subject,act.object,act.process));}
    }
    lines
}

pub fn native_place_lines(view:&PlaceView)->Vec<String> {
    let read=&view.reading;
    let name=|id:&u64| view.names.get(id).cloned().unwrap_or_else(|| format!("#{id}"));
    let tier=read.tier.map_or("no tier",|t| t.word());
    let mut lines=vec![format!("Location {} · {} · {} living residents · sites {:?}",read.location,tier,read.residents.len(),read.sites)];
    for h in &read.households {
        let members=h.members.iter().map(name).collect::<Vec<_>>().join(", ");
        lines.push(format!("Dwelling {} ({}): {} does {} (grade {}) under a tenancy with {}; household: {}",
            h.dwelling,h.name.as_deref().unwrap_or("unnamed"),name(&h.tenant),
            isocosm::social::craft_name(&h.does.craft),h.does.grade,name(&h.with),members));
    }
    for g in &read.groups {
        lines.push(format!("{} (polity {}) · governance {} · {} residents by constitution, {} by membership{}",
            g.name.as_deref().unwrap_or("Unnamed group"),g.polity,g.governance,g.constitutional.len(),g.related.len(),
            if g.ended {" · ended"} else {""}));
    }
    lines.push("Why it is here, from the history log:".into());
    for e in &view.entries {lines.push(format!("Entry {} at tick {}: {} -> {}",e.order,e.tick,e.what,e.outcome));}
    for e in &read.record {
        let toward=e.object.map(|o| format!(" toward {}",name(&o))).unwrap_or_default();
        lines.push(format!("Tick {} · {} by {}{}",e.tick,e.process,name(&e.subject),toward));
    }
    lines
}

pub(crate) fn panel(_ui:&UiState,read:&PartyView)->Vec<UiChild> {
    if !read.dm {return Vec::new();}
    let mut rows:Vec<UiChild>=vec![Box::new(el("h3",text("Current native goals and polities")))];
    rows.push(Box::new(el("label",(text("Reshape note "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),|ui:&mut UiState|&mut ui.native.party.hook_note)))));
    for hook in &read.hooks {
        for line in native_goal_lines(hook) {rows.push(Box::new(el("div",text(line)).attr("class","side-line")));}
        for (label,intent) in [("Take up",HookIntent::TakeUpGoal {arc:hook.event,goal:hook.goal.goal.clone()}),
            ("Drop",HookIntent::DropGoal {arc:hook.event,goal:hook.goal.goal.clone()})] {
            rows.push(Box::new(clickable(el("button",text(label)).attr("class","btn"),
                move |ui:&mut UiState,_|ui.request_party(PartyRequest::Hook(intent.clone())))));
        }
        let arc=hook.event; let goal=hook.goal.goal.clone();
        rows.push(Box::new(clickable(el("button",text("Reshape")).attr("class","btn"),move |ui:&mut UiState,_| {
            let note=ui.native.party.hook_note.text().to_owned();
            ui.request_party(PartyRequest::Hook(HookIntent::ReshapeGoal {arc,goal:goal.clone(),note}));
        })));
    }
    if !read.places.is_empty() {rows.push(Box::new(el("h3",text("Places read off their residents"))));}
    for place in &read.places {for line in native_place_lines(place) {
        rows.push(Box::new(el("div",text(line)).attr("class","side-line")));
    }}
    for polity in &read.polities {for line in native_polity_lines(polity) {
        rows.push(Box::new(el("div",text(line)).attr("class","side-line")));
    }}
    rows
}
