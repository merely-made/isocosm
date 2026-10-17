// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::asserted::{Assertion, HistoryLine, Applied};
use isocosm_overlay::{key_handle, key_for_handle};

impl Witness {
    fn order(&self, session: &Session) -> Result<u64, String> {
        session.entries.iter().find(|e| e.id == self.entry).map(|e| e.order)
            .ok_or("pack witness entry is absent".into())
    }
    pub(super) fn validate(&self, session: &Session, record: &Record) -> Result<(), String> {
        let entry = session.entries.iter().find(|e| e.id == self.entry).ok_or("pack witness entry is absent")?;
        let Command::Assert(Assertion::History(line)) = &entry.command else {
            return Err("pack witness is not an accepted native history record".into());
        };
        let event = session.sim.state().events.get(&self.event_key).ok_or("pack witness event is absent")?;
        if line.key != format!("vtt:pack:{}:{}", entry.order, self.tick)
            || line.event_key() != self.event_key || entry.outcome != self.event_key
            || key_handle(&self.event_key) != self.event.0
            || key_for_handle(session.sim.state().events.keys().map(String::as_str),self.event.0)
                != Some(self.event_key.as_str())
            || entry.tick != self.tick || event.tick != self.tick || event.authored.as_ref() != Some(line)
            || line.kind != "history" || !line.participants.is_empty() || line.place.is_some()
            || !line.tags.is_empty() || i64::try_from(self.tick).ok() != Some(line.time)
            || line.text != serde_json::to_string(record).map_err(|e| e.to_string())?
            || serde_json::from_str::<Record>(&line.text).map_err(|e| e.to_string())? != *record {
            return Err("pack accepted entry, event and typed record differ".into());
        }
        Ok(())
    }
}

pub(super) fn append(session: &mut Session, record: &Record) -> Result<Witness, String> {
    let tick = session.sim.state().tick;
    let line = HistoryLine { key:format!("vtt:pack:{}:{tick}",session.entries.len()),kind:"history".into(),
        time:i64::try_from(tick).map_err(|_| "pack tick exceeds history timestamp")?,
        text:serde_json::to_string(record).map_err(|e| e.to_string())?,
        participants:Vec::new(),place:None,tags:BTreeSet::new() };
    let event_key = line.event_key();
    if session.sim.state().events.contains_key(&event_key) { return Err("pack record key already exists".into()); }
    session.command(Command::Assert(Assertion::History(line)))?;
    let witness = Witness { entry:session.entries.last().ok_or("pack record entry is absent")?.id.clone(),
        event_key,event:EventHandle(key_handle(&session.entries.last().ok_or("pack entry is absent")?.outcome)),tick };
    witness.validate(session,record)?;
    Ok(witness)
}

pub fn validate_bindings(session: &Session, catalogue: Option<&Catalogue>) -> Result<(), String> {
    let Some(catalogue) = catalogue else { return Ok(()); };
    let participant = |id| session.sim.state().population.get(id).is_some_and(isocosm::directing::is_participant);
    if let Some(content) = &catalogue.content {
        content.content.validate().map_err(|e| e.to_string())?;
        if !participant(content.participant) { return Err("pack content has no real native author participant".into()); }
        content.witness.validate(session,&Record::Content { participant:content.participant,content:content.content.clone() })?;
    }
    for (key,pack) in &catalogue.packs {
        pack.definition.validate()?;
        if key != &pack.definition.key || !participant(pack.participant) {
            return Err("pack catalogue key or native author differs".into());
        }
        pack.witness.validate(session,&Record::Registered { participant:pack.participant,definition:pack.definition.clone() })?;
        let mut order = pack.witness.order(session)?;
        for preview in &pack.previews {
            if !participant(preview.participant) || preview.prepared.definition != isocosm::digest(&pack.definition)
                || preview.after_native.is_empty() {
                return Err("pack preview definition or native participant differs".into());
            }
            preview.witness.validate(session,&Record::Previewed { participant:preview.participant,key:key.clone(),
                prepared:preview.prepared.clone() })?;
            let next = preview.witness.order(session)?;
            if next <= order { return Err("pack previews are not in accepted native order".into()); }
            order = next;
        }
        if let Some(done) = &pack.activation {
            let preview = pack.previews.last().ok_or("activated pack has no accepted preview")?;
            if !participant(done.participant) || done.participant != preview.participant
                || done.preview != preview.witness.event || done.forced != preview.prepared.forced
                || preview.prepared.probe.native_refusal.is_some()
                || !done.forced && !preview.prepared.probe.compatible
                || done.witness.tick != preview.witness.tick {
                return Err("pack activation differs from its preview or native admission".into());
            }
            done.witness.validate(session,&Record::Activated { participant:done.participant,key:key.clone(),
                preview:done.preview,forced:done.forced })?;
            let commands: Vec<_> = if done.forced {
                vec![Command::Assert(Assertion::PackForced(Applied { key:key.clone(),
                    source:pack.definition.source.clone(),asserts:pack.definition.assertions.clone() }))]
            } else { pack.definition.assertions.iter().cloned().map(|a| Command::Assert(a)).collect() };
            let start = usize::try_from(order.checked_add(1).ok_or("pack order overflow")?).map_err(|_| "pack order overflow")?;
            let end = start.checked_add(commands.len()).ok_or("pack order overflow")?;
            if done.witness.order(session)? != end as u64
                || session.entries.get(start..end).ok_or("pack native assertion entries are absent")?.iter()
                    .zip(commands).any(|(entry,command)| entry.command != command || entry.tick != done.witness.tick) {
                return Err("pack activation is not immediately after its exact admitted assertions".into());
            }
        }
    }
    super::reading::merge_content(catalogue)?;
    Ok(())
}
