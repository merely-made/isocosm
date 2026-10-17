// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::records::append;

/// Borrowed authority and one game-byte candidate. Refusal publishes neither.
/// Private IDs are supplied by the authenticated host's separate private store.
pub fn submit_pack(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    private: &BTreeSet<Key>, envelope: &PackEnvelope) -> Result<(String, Vec<u8>), String> {
    dm(session,access)?;
    if envelope.participant != access.participant.0 || envelope.tick != session.sim.state().tick {
        return Err("pack input names another authenticated participant or native tick".into());
    }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut catalogue = bindings.catalogue().cloned().unwrap_or_default();
    let mut candidate = session.clone();
    let label = match &envelope.intent {
        PackInput::Content { content } => {
            content.validate().map_err(|e| e.to_string())?;
            if catalogue.content.is_some() { return Err("base native catalogue content already registered".into()); }
            let witness = append(&mut candidate,&Record::Content { participant:envelope.participant,content:content.clone() })?;
            catalogue.content = Some(ContentBinding { content:content.clone(),participant:envelope.participant,witness });
            "registered campaign content"
        },
        PackInput::Register { definition } => {
            definition.validate()?;
            if catalogue.packs.contains_key(&definition.key) { return Err("adventure pack key already registered".into()); }
            let witness = append(&mut candidate,&Record::Registered { participant:envelope.participant,definition:definition.clone() })?;
            catalogue.packs.insert(definition.key.clone(),Registered { definition:definition.clone(),
                participant:envelope.participant,witness,previews:Vec::new(),activation:None });
            "registered dormant adventure pack"
        },
        PackInput::Preview { key, force } => {
            let prepared = preview_pack(session,&game,access,private,key,*force)?;
            let witness = append(&mut candidate,&Record::Previewed { participant:envelope.participant,key:key.clone(),
                prepared:prepared.clone() })?;
            catalogue.packs.get_mut(key).ok_or("unknown pack")?.previews.push(Preview {
                prepared,participant:envelope.participant,witness,after_native:isocosm::digest(&candidate.save()) });
            "recorded adventure preview; explicit acceptance still required"
        },
        PackInput::Accept { key, preview } | PackInput::Force { key, preview } => {
            let force = matches!(&envelope.intent,PackInput::Force { .. });
            let pack = catalogue.packs.get(key).ok_or("unknown saved adventure pack")?;
            let selected = pack.previews.last().ok_or("preview the dormant pack before accepting it")?;
            if selected.witness.event != *preview || selected.participant != envelope.participant
                || selected.prepared.forced != force {
                return Err("acceptance names another DM preview or acceptance mode".into());
            }
            reading::current(session,&bindings,private,key,selected)?;
            if let Some(why) = &selected.prepared.probe.native_refusal { return Err(format!("pack native admission refused: {why}")); }
            if !force && !selected.prepared.probe.compatible { return Err("pack requirements are unmet; ordinary acceptance remains dormant".into()); }
            reading::apply_definition(&mut candidate,&pack.definition,force)?;
            let witness = append(&mut candidate,&Record::Activated { participant:envelope.participant,key:key.clone(),
                preview:*preview,forced:force })?;
            catalogue.packs.get_mut(key).ok_or("unknown pack")?.activation = Some(Activation {
                participant:envelope.participant,preview:*preview,forced:force,witness });
            if force { "forced adventure pack" } else { "accepted adventure pack" }
        },
    };
    bindings.record_catalogue(catalogue);
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    Ok((label.into(),next))
}
