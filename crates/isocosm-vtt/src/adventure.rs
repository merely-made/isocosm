// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Adventures and current goals: the DM's pack, storylet and goal decisions
//! on the shared world, with live polity readings beside them.

use std::collections::BTreeSet;

use chirograph::IntentEffect;
use isocosm::{directing::readings::Mode, polities::PolityReading};
use isocosm_host::endpoint::disclosure::{Item, NULL, action, card, value};
use isocosm_overlay::{Seat, vtt::HookIntent};
use isometry_campaign::native::{goal_hooks, hooks::GoalHook, pack::{self, PackInput, PackView}, polity_readings,
    storylets};

use crate::{Act, Built, Cx, VttEndpoint};

const DOMAIN: IntentEffect = IntentEffect::DomainTruth;
const NOTE: &str = r#"{"type":"string"}"#;

fn pack(built: &mut Built, view: PackView) {
    let definition = &view.pack.definition;
    let state = if view.pack.activation.is_some() { "active" }
        else if view.readiness.compatible { "compatible, dormant" } else { "dormant" };
    let mut values = vec![value("State", state), value("Proposed assertions", definition.assertions.len())];
    values.extend([&view.readiness.condition_refusal, &view.readiness.native_refusal].into_iter().flatten()
        .map(|refusal| value("Refused", refusal)));
    values.extend(definition.assertions.iter().map(|assertion|
        value("Assertion", serde_json::to_string(assertion).unwrap_or_else(|error| error.to_string()))));
    if let Some(content) = &definition.content {
        values.extend(content.maps.iter().map(|map| value("Proposed map", &map.map.id)));
        values.extend(content.storylets.values().map(|storylet| value("Proposed storylet", &storylet.entry)));
        values.extend(content.rewards.iter().map(|item| value("Item offer", &item.name)));
    }
    let (mut actions, mut acts) = (Vec::new(), Vec::new());
    if view.pack.activation.is_none() {
        for force in [false, true] {
            actions.push(action(&format!("vtt.pack-preview{}", if force { "-forced" } else { "" }),
                if force { "Preview forced acceptance" } else { "Preview acceptance" },
                "Record a preview of accepting this pack.", NULL, DOMAIN));
            acts.push(Act::Pack(PackInput::Preview { key: definition.key.clone(), force }));
        }
        if view.preview_current && let Some(preview) = view.pack.previews.last() {
            let (event, force) = (preview.witness.event, preview.prepared.forced);
            if preview.prepared.probe.native_refusal.is_none() && (force || preview.prepared.probe.compatible) {
                actions.push(action("vtt.pack-accept", if force { "Explicitly force this preview" } else { "Accept this preview" },
                    "Accept the current preview.", NULL, DOMAIN));
                let key = definition.key.clone();
                acts.push(Act::Pack(if force { PackInput::Force { key, preview: event } }
                    else { PackInput::Accept { key, preview: event } }));
            }
        }
    }
    built.push(Item { id: format!("pack:{}", definition.key), card: card(&definition.name, values, &["pack"]), actions },
        acts);
}

fn goal(built: &mut Built, hook: GoalHook) {
    let thread = &hook.goal;
    let mut values = vec![value("Holder", thread.holder), value("Recorded at", thread.recorded_at),
        value("Adoption", format!("{} at tick {}", thread.adoption.id, thread.adoption.tick))];
    values.extend(thread.participation.iter().map(|part|
        value("Served", format!("actor {} through {} at tick {}", part.actor, part.event.id, part.event.tick))));
    values.extend(thread.causes.iter().map(|link| value("Immediate cause", format!("{} follows {}", link.event, link.cause))));
    values.extend(thread.ending.iter().map(|end| value("Ending", format!("{:?} through {}", end.ending, end.event.id))));
    let (arc, key) = (hook.event, thread.goal.clone());
    built.push(Item { id: format!("goal:{}:{key}", arc.0), card: card(format!("Goal {key}: {}", thread.text), values, &["goal"]),
        actions: vec![
            action("vtt.goal-take-up", "Take up", "Take up this goal.", NULL, DOMAIN),
            action("vtt.goal-drop", "Drop", "Drop this goal.", NULL, DOMAIN),
            action("vtt.goal-reshape", "Reshape", "Reshape this goal; the payload is the note.", NOTE, DOMAIN),
        ] },
        vec![Act::Hook(HookIntent::TakeUpGoal { arc, goal: key.clone() }),
            Act::Hook(HookIntent::DropGoal { arc, goal: key.clone() }), Act::Reshape { arc, goal: key }]);
}

fn polity(built: &mut Built, read: PolityReading) {
    let mut values = vec![value("Membership records", read.members.len())];
    if let Some(authored) = &read.authored {
        values.push(value("Authored", format!("{}: {}", authored.name, authored.tags.iter().cloned().collect::<Vec<_>>().join(", "))));
        values.push(value("Authored claims", authored.claims.iter().cloned().collect::<Vec<_>>().join(", ")));
    }
    values.extend(read.member_tags.iter().map(|(tag, members)| value(format!("Member tag {tag}"), format!("{members:?}"))));
    if let Some(disagreement) = &read.disagreement {
        values.push(value("Disagreement", format!("authored-only {:?}; member-only {:?}",
            disagreement.authored_only, disagreement.members_only)));
    }
    for member in &read.members {
        values.push(value(format!("Member {}", member.id),
            format!("constitution {} · native relation {}", member.constitutional, member.relation)));
        values.extend(member.acts.iter().map(|act| value(&act.id,
            format!("tick {} · subject {}, object {:?} · {}", act.tick, act.subject, act.object, act.process))));
    }
    built.push(Item { id: format!("polity:{}", read.id), card: card(format!("Polity {}", read.id), values, &["polity"]),
        actions: Vec::new() }, Vec::new());
}

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let mut built = Built::new();
    let access = cx.cut.access;
    let read = if access.seat != Seat::Dm { Err("Adventure and goal authoring belongs to the DM.".to_string()) }
        else { endpoint.read(|session, game| {
            let packs = pack::read_packs(session, game, access, &BTreeSet::new())?;
            let goals = goal_hooks(session, access.subject.map_or(u64::MAX, |id| id.0), Mode::Creative)?;
            let polities = polity_readings(session, access)?;
            let storylets = storylets::read_storylets(session, game, access)?;
            Ok((packs, goals, polities, storylets))
        }) };
    let (packs, goals, polities, storylets) = match read {
        Ok(read) => read,
        Err(reason) => {
            built.push(Item { id: "adventure".into(), card: card("Adventures and current goals",
                vec![value("Note", reason)], &["note"]), actions: Vec::new() }, Vec::new());
            return Ok(built);
        },
    };
    for view in packs { pack(&mut built, view); }
    for view in storylets {
        let mut values: Vec<_> = view.cast.iter().map(|(role, actor)| value(role, actor)).collect();
        let (actions, acts) = match &view.refusal {
            Some(reason) => { values.push(value("Refused", reason)); (Vec::new(), Vec::new()) },
            None => (vec![action("vtt.storylet", "Play this storylet", "Play this storylet now.", NULL, DOMAIN)],
                vec![Act::Submit(serde_json::to_vec(&view.input).map_err(|e| e.to_string())?)]),
        };
        built.push(Item { id: format!("storylet:{}", view.input.key), card: card(&view.label, values, &["storylet"]),
            actions }, acts);
    }
    for hook in goals { goal(&mut built, hook); }
    for read in polities { polity(&mut built, read); }
    if built.items.is_empty() {
        built.push(Item { id: "adventure".into(), card: card("Adventures and current goals",
            vec![value("Note", "No packs, storylets, current goals or polities in this world.")], &["note"]),
            actions: Vec::new() }, Vec::new());
    }
    Ok(built)
}
