// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared-host offers from explicitly installed or accepted native pack content.

use super::{pack,reading::Reading,movement::current_access};
use isocosm::{Session,Command,asserted::{Assertion,Applied,Fact},schema::{Id,Tick}};
use isocosm_overlay::{OrdinaryAccess,Seat};
use crate::StoryletEffect;
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryletInput {
    pub participant:Id,pub tick:Tick,pub key:String,pub expected_native:String,pub expected_content:String,
}

#[derive(Clone,Debug)]
pub struct StoryletView {pub input:StoryletInput,pub label:String,pub refusal:Option<String>,pub cast:Vec<(String,String)>}

fn resolve(session:&Session,storylet:&crate::StoryletProposal)->Result<crate::StoryletResolution,String> {
    Reading::new(session).resolve_storylet(storylet,std::iter::empty::<&str>())
        .map_err(|error|format!("{error:?}"))
}

fn supported(effects:&[StoryletEffect])->Result<(),String> {
    if effects.iter().any(|effect|matches!(effect,StoryletEffect::LocalMap{..}|StoryletEffect::Item{..})) {
        return Err("This storylet needs map or item placement that is not available in this host yet.".into());
    }
    Ok(())
}

pub fn read_storylets(session:&Session,game:&[u8],access:OrdinaryAccess)->Result<Vec<StoryletView>,String> {
    current_access(session,access)?;
    if access.seat!=Seat::Dm {return Err("storylet authoring requires the DM seat".into());}
    let content=pack::active_content(session,game,access)?;
    let source=isocosm::digest(&content);
    let native=isocosm::digest(&session.save());
    Ok(content.storylets.iter().map(|(key,storylet)| {
        let (cast,refusal)=match resolve(session,storylet) {
            Ok(resolved)=>(resolved.cast.into_iter().collect(),supported(&resolved.effects).err()),
            Err(reason)=>(Vec::new(),Some(reason)),
        };
        StoryletView {input:StoryletInput {participant:access.participant.0,tick:session.sim.state().tick,
            key:key.clone(),expected_native:native.clone(),expected_content:source.clone()},
            label:storylet.entry.clone(),refusal,cast}
    }).collect())
}

pub fn submit_storylet(session:&mut Session,game:Vec<u8>,access:OrdinaryAccess,input:&StoryletInput)
    ->Result<(String,Vec<u8>),String> {
    current_access(session,access)?;
    if access.seat!=Seat::Dm || input.participant!=access.participant.0 || input.tick!=session.sim.state().tick
        || input.expected_native!=isocosm::digest(&session.save()) {
        return Err("storylet input names another DM or a changed native preview".into());
    }
    let content=pack::active_content(session,&game,access)?;
    if input.expected_content!=isocosm::digest(&content) {return Err("storylet content changed since preview".into());}
    let storylet=content.storylets.get(&input.key).ok_or("storylet offer is absent")?;
    let resolved=resolve(session,storylet)?; supported(&resolved.effects)?;
    let occurrence=format!("storylet:{}:{}",input.key,session.entries.len());
    let assertions=resolved.effects.into_iter().enumerate().map(|(index,effect)| match effect {
        StoryletEffect::History{event}=>Ok(Assertion::History(event)),
        StoryletEffect::Fact{fact}=>Ok(Assertion::Fact(Fact {about:fact.id,key:format!("{occurrence}:fact:{index}"),
            kind:fact.kind,text:fact.text,tags:fact.tags.into_iter().collect::<BTreeSet<_>>() })),
        _=>Err("storylet effect requires unavailable placement".into()),
    }).collect::<Result<Vec<_>,String>>()?;
    let mut candidate=session.clone();
    let accepted=candidate.command(Command::Assert(Assertion::Storylet(Applied {key:occurrence,
        source:storylet.key.clone(),asserts:assertions})))?;
    *session=candidate;
    Ok((accepted,game))
}
