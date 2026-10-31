// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_campaign::{DraftMap, ItemProposal, StoryletProposal};
use cosmograph::asserted::{Applied, Assertion};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActiveContent {
    pub maps: BTreeMap<Key, DraftMap>, pub storylets: BTreeMap<Key, StoryletProposal>,
    /// Offers only; these issue no native stock or automatic inventory reward.
    pub rewards: Vec<ItemProposal>, pub choreography: Vec<Choreography>,
}

pub(super) fn merge_content(catalogue: &Catalogue) -> Result<ActiveContent, String> {
    let mut active = ActiveContent::default();
    let mut contents = Vec::new();
    if let Some(base) = &catalogue.content { contents.push(&base.content); }
    for pack in catalogue.packs.values().filter(|p| p.activation.is_some()) {
        if let Some(content) = &pack.definition.content { contents.push(content); }
        active.choreography.extend(pack.definition.choreography.iter().cloned());
    }
    for content in contents {
        for map in &content.maps {
            if active.maps.get(&map.map.id).is_some_and(|old| old != map) {
                return Err(format!("active packs disagree about map {}",map.map.id));
            }
            active.maps.insert(map.map.id.clone(),map.clone());
        }
        for (key,storylet) in &content.storylets {
            if active.storylets.get(key).is_some_and(|old| old != storylet) {
                return Err(format!("active packs disagree about storylet {key}"));
            }
            active.storylets.insert(key.clone(),storylet.clone());
        }
        active.rewards.extend(content.rewards.iter().cloned());
    }
    Ok(active)
}

pub fn active_content(session: &Session, game: &[u8], access: OrdinaryAccess) -> Result<ActiveContent, String> {
    dm(session,access)?;
    let bindings = GameBindings::decode(game)?;
    bindings.validate(session)?;
    merge_content(bindings.catalogue().unwrap_or(&Catalogue::default()))
}

/// Public presentation from accepted packs, without exposing dormant proposals
/// or private content through the DM catalogue reading.
pub fn accepted_choreography(session: &Session, game: &[u8]) -> Result<Vec<Choreography>, String> {
    let bindings = GameBindings::decode(game)?;
    bindings.validate(session)?;
    Ok(bindings.catalogue().into_iter().flat_map(|catalogue| catalogue.packs.values())
        .filter(|pack| pack.activation.is_some())
        .flat_map(|pack| pack.definition.choreography.iter().cloned()).collect())
}

pub(super) fn apply_definition(session: &mut Session, definition: &Definition, force: bool) -> Result<(), String> {
    if force {
        session.command(Command::Assert(Assertion::PackForced(Applied { key:definition.key.clone(),
            source:definition.source.clone(),asserts:definition.assertions.clone() })))?;
    } else {
        for assertion in &definition.assertions { session.command(Command::Assert(assertion.clone()))?; }
    }
    Ok(())
}

fn probe(session: &Session, definition: &Definition, private: &BTreeSet<Key>, force: bool) -> Probe {
    let storylet = StoryletProposal { key:definition.key.clone(),entry:definition.name.clone(),
        requirements:definition.requirements.clone(),roles:definition.roles.clone(),effects:Vec::new(),tags:Vec::new() };
    let resolved = super::super::Reading::new(session).resolve_storylet(&storylet,private.iter().map(String::as_str));
    let (compatible,condition_refusal,cast) = match resolved {
        Ok(result) => (true,None,result.cast), Err(why) => (false,Some(format!("{why:?}")),BTreeMap::new()),
    };
    let mut candidate = session.clone();
    let native_refusal = apply_definition(&mut candidate,definition,force).err();
    Probe { compatible,condition_refusal,cast,native_refusal }
}

pub fn preview_pack(session: &Session, game: &[u8], access: OrdinaryAccess,
    private: &BTreeSet<Key>, key: &str, force: bool) -> Result<Prepared, String> {
    dm(session,access)?;
    let bindings = GameBindings::decode(game)?;
    bindings.validate(session)?;
    let catalogue = bindings.catalogue().ok_or("no saved adventure catalogue")?;
    let pack = catalogue.packs.get(key).ok_or("unknown saved adventure pack")?;
    if pack.activation.is_some() { return Err("adventure pack already activated".into()); }
    let mut result = probe(session,&pack.definition,private,force);
    let mut merged = catalogue.clone();
    let pending = merged.packs.get_mut(key).ok_or("unknown pack")?;
    pending.activation = Some(Activation { participant:access.participant.0,preview:pack.witness.event,
        forced:force,witness:pack.witness.clone() });
    if let Err(why) = merge_content(&merged) { result.native_refusal = Some(why); }
    Ok(Prepared { definition:cosmograph::digest(&pack.definition),game:cosmograph::digest(&bindings.encode()?),
        native:cosmograph::digest(&session.save()),forced:force,probe:result,
        private_requirements:pack.definition.requirements.hidden_facts.iter().filter(|key| private.contains(*key)).cloned().collect() })
}

pub(super) fn current(session: &Session, game: &GameBindings, private: &BTreeSet<Key>, key: &str,
    preview: &Preview) -> Result<(), String> {
    let mut previous = game.clone();
    let mut catalogue = previous.catalogue().ok_or("no saved catalogue")?.clone();
    let pack = catalogue.packs.get_mut(key).ok_or("unknown adventure pack")?;
    if pack.previews.pop().as_ref() != Some(preview) || pack.activation.is_some() {
        return Err("pack preview is no longer its last dormant preview".into());
    }
    previous.record_catalogue(catalogue);
    if preview.prepared.game != cosmograph::digest(&previous.encode()?)
        || preview.after_native != cosmograph::digest(&session.save())
        || session.entries.last().map(|entry| &entry.id) != Some(&preview.witness.entry)
        || session.sim.state().tick != preview.witness.tick {
        return Err("native world or game changed since pack preview; preview again".into());
    }
    let pack = previous.catalogue().ok_or("no catalogue")?.packs.get(key).ok_or("unknown pack")?;
    let required: BTreeSet<_> = pack.definition.requirements.hidden_facts.iter()
        .filter(|key| private.contains(*key)).cloned().collect();
    if required != preview.prepared.private_requirements
        || probe(session,&pack.definition,private,preview.prepared.forced) != preview.prepared.probe {
        return Err("pack requirements, cast or native admission changed since preview".into());
    }
    Ok(())
}

pub fn read_packs(session: &Session, game: &[u8], access: OrdinaryAccess,
    private: &BTreeSet<Key>) -> Result<Vec<PackView>, String> {
    dm(session,access)?;
    let bindings = GameBindings::decode(game)?;
    bindings.validate(session)?;
    let Some(catalogue) = bindings.catalogue() else { return Ok(Vec::new()); };
    Ok(catalogue.packs.iter().map(|(key,pack)| PackView { pack:pack.clone(),
        readiness:probe(session,&pack.definition,private,false),
        preview_current:pack.previews.last().is_some_and(|preview| current(session,&bindings,private,key,preview).is_ok()) }).collect())
}
