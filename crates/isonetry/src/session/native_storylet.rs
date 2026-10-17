// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Plain native Fact/History admission. Physical effects use complete composition.

use super::*;
use isocosm::{Execution, asserted::{Applied, Assertion, Fact}};

impl HostSession {
    pub(super) fn commit_native_storylet(&mut self, key: &str, _item_owner: Option<TokenId>)
        -> Result<Vec<Outbound>, String> {
        let previous = self.state.native.as_ref().ok_or("no native campaign")?;
        let old = previous.load(Execution::Grouped)?;
        let mut offers = old.content().storylets.clone();
        if let Some(table) = &self.state.native_table {
            table.validate_history(old.table_session())?;
            let access = isocosm_overlay::OrdinaryAccess {
                participant:isocosm_overlay::ParticipantHandle(table.host_participant),seat:isocosm_overlay::Seat::Dm,
                subject:old.table_session().sim.plays(table.host_participant).map(isocosm_overlay::EntityHandle) };
            for (key,storylet) in isocosm_vtt::native::pack::active_content(old.table_session(),&table.game,access)?.storylets {
                if offers.get(&key).is_some_and(|existing| existing != &storylet) {
                    return Err(format!("native campaign and accepted pack disagree about storylet {key}"));
                }
                offers.insert(key,storylet);
            }
        }
        let storylet = offers.get(key).ok_or_else(|| format!("unknown native storylet: {key}"))?;
        if storylet.effects.iter().any(|effect|matches!(effect,StoryletEffect::LocalMap {..}|StoryletEffect::Item {..})) {
            return Err("physical native storylet effects require complete supplied inputs through composed Quote/Apply".into());
        }
        let resolved = old.reading().resolve_storylet(storylet, self.campaign.secret_ids())
            .map_err(|error| format!("storylet does not match native state: {error:?}"))?;
        let occurrence = format!("storylet:{key}:{}", old.reading().entries().len());
        let mut asserts = Vec::new();
        let mut facts = Vec::new();
        for (index, effect) in resolved.effects.into_iter().enumerate() {
            match effect {
                StoryletEffect::Fact { fact } => {
                    asserts.push(Assertion::Fact(Fact { about:fact.id.clone(),
                        key:format!("{occurrence}:fact:{index}"), kind:fact.kind.clone(),
                        text:fact.text.clone(), tags:fact.tags.iter().cloned().collect() }));
                    facts.push(fact);
                },
                StoryletEffect::History { event } => asserts.push(Assertion::History(event)),
                StoryletEffect::LocalMap {..}|StoryletEffect::Item {..}=>
                    return Err("physical native storylets use complete composed Quote/Apply".into()),
            }
        }
        let admitted = old.with_assertions(&[Assertion::Storylet(Applied {
            key:occurrence, source:storylet.key.clone(), asserts,
        })]).map_err(|error| error.to_string())?;
        let publication = super::native::NativePublication::update(previous, &admitted)?;
        let mut candidate = Self::with_history(self.state.clone(), self.campaign.clone(), self.history.clone());
        let mut out = candidate.try_commit(GameEvent::NativePublished(publication))?;
        for fact in facts { out.extend(candidate.try_commit(GameEvent::Fact(fact))?); }
        // Peer attribution, pending requests and private custody stay on this
        // authority. Only a fully admitted public state/history is published.
        self.state = candidate.state;
        self.history = candidate.history;
        self.seq = candidate.seq;
        self.log_hash = candidate.log_hash;
        Ok(out)
    }
}
