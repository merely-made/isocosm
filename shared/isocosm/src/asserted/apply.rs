// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Writing each assertion onto its native noun (rulings 760 and 769).

use super::*;
use crate::schema::{
    Constitution, Entity, Event, Id, Key, Ledger, Method, Note, Polity, Provenance, Relation, Site,
    Tick,
};
use crate::{Result, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

impl Simulation {
    /// The polity asserted under an authored key.
    pub fn authored_polity(&self, key: &str) -> Option<(Id, &Polity)> {
        let named = |p: &Polity| p.authored.as_ref().is_some_and(|a| a.key == key);
        let mut found = self.state.polities.iter().filter(|(_, p)| named(p));
        found.next().map(|(id, p)| (*id, p))
    }

    pub(super) fn assert_faction(&mut self, f: &Faction) -> Result<Id> {
        let polity = Polity {
            constitution: Constitution {
                members: BTreeSet::new(),
                governance: f.governance.clone(),
                focus: f.focus.clone(),
                support_account: Key::new(),
                host: None,
                founded_by: cause(&f.authored.key),
            },
            accounts: BTreeMap::new(),
            ended_by: None,
            authored: Some(f.authored.clone()),
        };
        if let Some((id, was)) = self.authored_polity(&f.authored.key) {
            return held(was, &polity, "faction", &f.authored.key).map(|_| id);
        }
        let id = self.fresh_id();
        self.state.polities.insert(id, polity);
        Ok(id)
    }

    /// A fresh id from the entity id space, which polities and authored
    /// sites share with entities.
    fn fresh_id(&mut self) -> Id {
        let id = self.state.population.next_id;
        self.state.population.next_id += 1;
        id
    }

    pub(super) fn assert_fact(&mut self, f: &Fact) -> Result<()> {
        if !self.genesis.rules.note_kinds.contains(&f.kind) {
            return Err("unknown note kind".into());
        }
        let mut extra = BTreeMap::new();
        if !f.tags.is_empty() {
            extra.insert(
                "tags".into(),
                f.tags.iter().cloned().collect::<Vec<_>>().join(" "),
            );
        }
        let note = Note {
            core: wing_impresa::Record {
                subject: f.about.clone(),
                object: f.key.clone(),
                kind: f.kind.clone(),
                canon_revision: self.genesis.world.canon.revision,
                cause: cause(&f.key),
                tick: self.state.tick,
                stance: wing_impresa::Stance::Associate,
            },
            djot: f.text.clone(),
            extra,
            expires: None,
        };
        let was = self
            .state
            .notes
            .iter()
            .find(|n| n.core.cause == note.core.cause);
        if let Some(was) = was {
            let same = (&was.core.subject, &was.core.kind, &was.djot, &was.extra)
                == (&note.core.subject, &note.core.kind, &note.djot, &note.extra);
            return held(&same, &true, "fact", &f.key).map(|_| ());
        }
        self.room(self.state.notes.len())?;
        self.state.notes.push(note);
        Ok(())
    }

    /// The site asserted under an authored key.
    pub fn authored_site(&self, key: &str) -> Option<(Id, &Site)> {
        let named = |s: &Site| s.authored.as_ref().is_some_and(|a| a.key == key);
        let mut found = self.state.sites.iter().filter(|(_, s)| named(s));
        found.next().map(|(id, s)| (*id, s))
    }

    /// The entity asserted under an authored key.
    pub fn authored_entity(&self, key: &str) -> Option<(Id, &Entity)> {
        let named = |e: &Entity| e.authored.as_ref().is_some_and(|a| a.key == key);
        let groups = self.state.population.groups.iter();
        let mut found = groups.filter(|(_, g)| named(&g.entity));
        found.next().map(|(id, g)| (*id, &g.entity))
    }

    /// The native id an authored key names: a polity, site or entity.
    pub fn authored_id(&self, key: &str) -> Option<Id> {
        let polity = self.authored_polity(key).map(|(id, _)| id);
        let site = || self.authored_site(key).map(|(id, _)| id);
        let entity = || self.authored_entity(key).map(|(id, _)| id);
        polity.or_else(site).or_else(entity)
    }

    pub(super) fn assert_place(&mut self, p: &Place) -> Result<Id> {
        if let Some((id, was)) = self.authored_site(&p.key) {
            return held(&was.authored, &Some(p.clone()), "place", &p.key).map(|_| id);
        }
        if self.state.sites.len() >= self.genesis.rules.limits.sites {
            return Err("site limit".into());
        }
        let site = Site {
            terrain_seed: 0,
            conditions: BTreeMap::new(),
            accounts: Ledger::new(),
            routes: vec![],
            authored: Some(p.clone()),
        };
        let id = self.fresh_id();
        self.state.sites.insert(id, site);
        Ok(id)
    }

    /// A route runs both ways between its places' sites, each side carrying
    /// the authored route.
    pub(super) fn assert_route(&mut self, r: &Route) -> Result<()> {
        let routes = self.state.sites.values().flat_map(|s| &s.routes);
        let mut found = routes.filter_map(|x| x.authored.as_ref());
        if let Some(was) = found.find(|a| a.key == r.key) {
            return held(was, r, "route", &r.key).map(|_| ());
        }
        let site = |key: &str| self.authored_site(key).map(|(id, _)| id);
        let (Some(from), Some(to)) = (site(&r.from), site(&r.to)) else {
            return Err(format!("route {} names an unasserted place", r.key));
        };
        for (at, toward) in [(from, to), (to, from)] {
            let route = crate::schema::Route {
                to: toward,
                travel: Tick::from(r.weight),
                transmission: 0,
                border: None,
                authored: Some(r.clone()),
            };
            let site = self.state.sites.get_mut(&at).expect("an asserted site");
            site.routes.push(route);
        }
        Ok(())
    }

    /// A placeless entity carrying its fill; its faction, where asserted,
    /// takes it in as a member relation (760).
    pub(super) fn assert_character(&mut self, c: &Character) -> Result<Id> {
        if let Some((id, was)) = self.authored_entity(&c.key) {
            return held(&was.authored, &Some(c.clone()), "character", &c.key).map(|_| id);
        }
        if self.state.population.count() >= self.genesis.rules.limits.entities {
            return Err("entity limit".into());
        }
        let faction = c.faction.as_deref().and_then(|f| self.authored_polity(f));
        let faction = faction.map(|(id, _)| id);
        let entity = authored_entity(c, self.state.tick);
        let id = self.state.population.insert(entity, 1)?;
        if let Some(polity) = faction {
            self.state
                .relations
                .insert(Relation::new(id, "member", polity));
        }
        Ok(id)
    }

    pub(super) fn assert_law(&mut self, l: &Law) -> Result<()> {
        if let Some(was) = self.state.laws.get(&l.key) {
            return held(was, l, "law", &l.key).map(|_| ());
        }
        self.state.laws.insert(l.key.clone(), l.clone());
        Ok(())
    }

    /// An event at its place's site and about its first participant, where
    /// those are asserted, placeless otherwise; it spreads nowhere.
    pub(super) fn assert_line(&mut self, h: &HistoryLine) -> Result<Key> {
        let id = format!("line:{}", h.key);
        if let Some(was) = self.state.events.get(&id) {
            let line = Some(h.clone());
            return held(&was.authored, &line, "history line", &h.key).map(|_| id);
        }
        if self.state.events.len() >= self.genesis.rules.limits.history {
            return Err("history limit".into());
        }
        let placeless = crate::directing::PLACELESS;
        let place = h.place.as_deref().and_then(|p| self.authored_site(p));
        let subject = h.participants.first().and_then(|p| self.authored_id(p));
        let event = Event {
            id: id.clone(),
            tick: self.state.tick,
            place: place.map_or(placeless, |(id, _)| id),
            subject: subject.unwrap_or(placeless),
            object: None,
            process: h.kind.clone(),
            cause: Some(cause(&h.key)),
            strength: 0,
            legend: false,
            authored: Some(h.clone()),
        };
        self.state.events.insert(id.clone(), event);
        Ok(id)
    }
}

/// A character's placeless entity, its lineage and kingdom placeholders.
fn authored_entity(c: &Character, tick: Tick) -> Entity {
    Entity {
        lineage: "authored:character".into(),
        kingdom: "kingdom:character".into(),
        scale: "scale:meso".into(),
        provenance: Provenance::Intrinsic("authored:character".into()),
        method: Method::Inert,
        place: crate::directing::PLACELESS,
        arrived: tick,
        visits: vec![],
        born: tick,
        alive: true,
        body_revision: 1,
        body: None,
        parts: Default::default(),
        traits: Default::default(),
        accounts: Default::default(),
        skills: Default::default(),
        tenets: Default::default(),
        disposition: [0; 5],
        soma: vec![],
        systems: Default::default(),
        varied: vec![],
        patch: None,
        authored: Some(c.clone()),
    }
}
