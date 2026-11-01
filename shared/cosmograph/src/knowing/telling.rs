// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Telling (case 1066): one act in which a teller passes a hearer everything
//! it knows about a place or an event, its notes and what it knows by reach.
//! The hearer is at the teller's site (letters and messengers are later).
//! Whether the hearer takes it is a seeded baseline draw over what Processes
//! 11 names that exists today: whether the two are related, and the hearer's
//! caution; the fuller weighing is still to come. Its numbers are the world's
//! (`Rules::telling`), tuned in play. Every copy names the teller.

use super::{CLAIM, REPORT, holder, told};
use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};

/// What a telling is about.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Topic {
    Place(Id),
    Event(Key),
}

/// What one telling did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Telling {
    pub id: Key,
    pub chance: i64,
    pub taken: bool,
    /// The hearer's new notes, by id or event.
    pub copied: Vec<Key>,
}

impl Simulation {
    /// The baseline chance that `hearer` takes what `teller` tells it.
    pub fn telling_chance(&self, teller: Id, hearer: Id) -> Result<i64> {
        let h = self.state.population.get(hearer).ok_or("hearer is absent")?;
        let related = self.state.relations.iter().any(|r|
            (r.subject == teller && r.object == hearer) || (r.subject == hearer && r.object == teller));
        let rules = self.genesis.rules.telling();
        let caution = i64::from(crate::social::caution_of(h)).clamp(-rules.caution_clamp, rules.caution_clamp);
        Ok((rules.base + if related { rules.related } else { 0 } - caution * rules.per_caution)
            .clamp(rules.floor, rules.ceiling))
    }

    pub(crate) fn tell(&mut self, teller: Id, hearer: Id, topic: &Topic) -> Result<Telling> {
        if teller == hearer { return Err("a body tells another, not itself".into()); }
        let (t, h) = (self.state.population.get(teller).ok_or("teller is absent")?,
            self.state.population.get(hearer).ok_or("hearer is absent")?);
        if !t.alive || !h.alive { return Err("teller and hearer must be living".into()); }
        let site = t.resident_site().ok_or("the teller is at no site")?;
        if h.resident_site() != Some(site) { return Err("the hearer is not at the teller's site".into()); }
        let chance = self.telling_chance(teller, hearer)?;
        let tick = self.state.tick;
        let id = format!("tell:{}", crate::digest(&(tick, teller, hearer, topic, self.state.notes.len())));
        let drawn = crate::draw(self.genesis.seed, "know:tell", &[tick, teller, hearer,
            u64::from_str_radix(&crate::digest(topic)[..15], 16).unwrap_or(0)]) % 1000;
        let taken = (drawn as i64) < chance;
        let mut copied = Vec::new();
        if taken {
            let mut candidate = self.clone();
            candidate.copy_known(teller, hearer, topic, &id, &mut copied)?;
            self.state = candidate.state;
        }
        Ok(Telling { id, chance, taken, copied })
    }

    fn copy_known(&mut self, teller: Id, hearer: Id, topic: &Topic, id: &str, copied: &mut Vec<Key>) -> Result<()> {
        let about_site = |site: Option<Id>| match topic { Topic::Place(place) => site == Some(*place), Topic::Event(_) => false };
        // Events: what the teller knows, by note or by reach, becomes the hearer's.
        let events: Vec<Key> = self.state.events.iter().filter(|(key, event)| match topic {
            Topic::Event(wanted) => *key == wanted,
            Topic::Place(_) => about_site(event.place.site()),
        }).map(|(key, _)| key.clone()).collect();
        for key in events {
            if !self.knows(teller, &key)? || self.knows_by_note(hearer, &key) { continue; }
            let object = self.state.events[&key].id.clone();
            self.add_note(hearer, object.clone(), "sim:discover", String::new(), None, id.into())?;
            copied.push(object);
        }
        // Claims and reports about those events pass as reports naming the teller.
        if self.genesis.rules.note_kinds.contains(REPORT) {
            let claims: Vec<(Key, super::Proposition)> = self.state.notes.iter()
                .filter(|n| holder(n) == Some(teller) && (n.core.kind == CLAIM || n.core.kind == REPORT))
                .filter_map(|n| told(n).map(|p| (n.core.cause.clone(), p)))
                .filter(|(_, p)| match topic {
                    Topic::Event(wanted) => self.state.events.get(wanted).is_some_and(|e| e.id == p.event),
                    Topic::Place(_) => self.state.events.values().any(|e| e.id == p.event && about_site(e.place.site())),
                }).collect();
            for (claim, proposition) in claims {
                let claim = self.state.notes.iter().find(|n| n.core.cause == claim)
                    .and_then(|n| n.extra.get("claim").cloned()).unwrap_or(claim);
                let extra = [("claim", claim), ("reporter", teller.to_string()), ("telling", id.to_string())];
                copied.push(self.hold_note(hearer, REPORT, &proposition, &extra)?);
            }
        }
        if let Topic::Place(place) = topic {
            // Where the world keeps place notes, the place itself: the teller
            // knows it by living or having been there, or by being told.
            let rules = &self.genesis.rules;
            if rules.telling().place_notes && self.knows_place(teller, *place) && !self.has_place_note(hearer, *place) {
                self.add_note(hearer, format!("site:{place}"), crate::rules::PLACE_NOTE, String::new(), None, id.into())?;
                copied.push(format!("site:{place}"));
            }
            // Held statements naming the place, and terrain seen there.
            let statements: Vec<Key> = self.held_statements(teller)?.into_iter()
                .filter(|held| super::statements::names_site(&held.statement, *place)).map(|held| held.id).collect();
            for held in statements { copied.push(self.tell_statement(teller, hearer, &held)?); }
            let terrain: Vec<Key> = self.held_terrain(teller)?.into_iter()
                .filter(|held| held.source.observation.named.site == *place).map(|held| held.id).collect();
            for held in terrain { copied.push(self.tell_terrain(teller, hearer, &held)?); }
        }
        Ok(())
    }

    /// The events `teller` could tell `hearer` that matter to it (1066):
    /// known to the teller, not to the hearer, and within the world's
    /// `event_hops` of what the hearer knows. One hop is an event at a place
    /// the hearer knows, with the hearer or a body it holds a relation with as
    /// a party, or causing or caused by an event it knows; each further hop
    /// reaches events sharing a party or cause with one offered at the hop
    /// before. Each comes with its hop, nearest first.
    pub fn tellable_events(&self, teller: Id, hearer: Id, hops: u32) -> Vec<(Key, u32)> {
        use crate::directing::readings::{Mode, view};
        use std::collections::BTreeSet;
        let heard = view(self, hearer, Mode::Survival);
        let told = view(self, teller, Mode::Survival);
        let mut parties: BTreeSet<Id> = self.state.relations.iter().filter_map(|r|
            if r.subject == hearer { Some(r.object) } else if r.object == hearer { Some(r.subject) } else { None })
            .collect();
        parties.insert(hearer);
        let mut causes: BTreeSet<Key> = heard.events.iter().filter_map(|k| self.state.events.get(k))
            .map(|e| e.id.clone()).collect();
        let caused_by: BTreeSet<Key> = heard.events.iter().filter_map(|k| self.state.events.get(k))
            .filter_map(|e| e.cause.clone()).collect();
        let mut left: BTreeSet<&Key> = told.events.iter().filter(|k| !heard.events.contains(*k)).collect();
        let mut offered = Vec::new();
        for hop in 1..=hops {
            let reached: Vec<&Key> = left.iter().copied().filter(|key| self.state.events.get(*key).is_some_and(|e|
                (hop == 1 && e.place.site().is_some_and(|site| heard.sites.contains(&site)))
                || parties.contains(&e.subject) || e.object.is_some_and(|o| parties.contains(&o))
                || e.cause.as_ref().is_some_and(|c| causes.contains(c))
                || (hop == 1 && caused_by.contains(&e.id)))).collect();
            if reached.is_empty() { break; }
            for key in reached {
                left.remove(key);
                let e = &self.state.events[key];
                parties.insert(e.subject);
                parties.extend(e.object);
                causes.insert(e.id.clone());
                offered.push((key.clone(), hop));
            }
        }
        offered
    }

    /// Whether `entity` knows site `place`: it lives or has been there, or
    /// holds a place note of it.
    pub fn knows_place(&self, entity: Id, place: Id) -> bool {
        self.state.population.get(entity).is_some_and(|e| e.resident_site() == Some(place)
            || e.visits.iter().any(|v| v.place == place)) || self.has_place_note(entity, place)
    }

    fn has_place_note(&self, entity: Id, place: Id) -> bool {
        let (subject, object) = (format!("entity:{entity}"), format!("site:{place}"));
        self.state.notes.iter().any(|n| n.core.kind == crate::rules::PLACE_NOTE && n.core.subject == subject
            && n.core.object == object)
    }

    fn knows_by_note(&self, entity: Id, event: &str) -> bool {
        let Some(e) = self.state.events.get(event) else { return false };
        let subject = format!("entity:{entity}");
        self.state.notes.iter().any(|n| n.core.subject == subject && n.core.object == e.id && n.core.kind == "sim:discover")
    }
}
