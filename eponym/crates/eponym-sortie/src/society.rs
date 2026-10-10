// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The society the sortie reads, held in Eponym's world on Isocosm (wing
//! rulings 60, 63, 67 and 755): its peers are sim entities this receipt
//! names, and every deed, ask, agreement, wound and dig is a native command
//! at the tick it happens, the sim's clock brought up to it first.

use eponym_play::identity::{SubjectId, Tick};
use eponym_play::{World, WorldError, WorldIntent, founding};
use isocosm::history::Command;
use isocosm::schema::{Id, Key, Method};
use isocosm::social::{
    Agreement, Answer, Craft, Deed, DeedKind, EndReason, Offer, Premise, Response, Ruling, Social,
    Standing, Terms, Work,
};

/// What the society refuses with.
pub type SocialError = WorldError;

#[derive(Clone, Debug)]
pub struct Society {
    world: World,
}

impl Society {
    pub fn on(world: World) -> Self {
        Self { world }
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn sim(&self) -> &isocosm::Simulation {
        self.world.sim()
    }

    /// Brings the sim's clock up to `at`.
    fn at(&mut self, at: Tick) -> Result<(), SocialError> {
        while self.world.sim().state().tick < at.0 {
            self.world.tick()?;
        }
        Ok(())
    }

    /// A peer arrives, is named, and takes a one-part body.
    pub fn admit(&mut self, subject: SubjectId, name: &str, crafts: &[(Craft, u8)], caution: i16) -> Result<Id, SocialError> {
        let mut disposition = [0; 5];
        disposition[isocosm::social::CAUTION] = caution;
        let arrival = isocosm::arrival::Arrival {
            lineage: founding::SOPHONT.into(),
            site: self.world.site(),
            method: Method::Normative,
            accounts: [(founding::RESERVE, founding::FULL), (founding::ENERGY, founding::FULL)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
            traits: Default::default(),
            skills: crafts.iter().map(|(c, g)| (c.skill(), u64::from(*g))).collect(),
            disposition,
        };
        let outcome = self.world.command(Command::Arrive(arrival))?;
        let id: Id = outcome.strip_prefix("entity:").and_then(|s| s.parse().ok()).ok_or(SocialError::Decode)?;
        let body = isometer_core::BodyDocument::new(isometer_core::VolumeRef::from_tag(0), [2, 4, 2]);
        self.world.command(Command::Embody { entity: id, body })?;
        self.world.command(Command::Name { by: id, of: id, name: name.into() })?;
        self.world.bind(subject, id);
        Ok(id)
    }

    pub fn id(&self, subject: SubjectId) -> Id {
        self.world.entity(subject).expect("a peer this receipt admitted")
    }
    pub fn subject_of(&self, id: Id) -> Option<SubjectId> {
        self.world.subjects().find(|(_, e)| *e == id).map(|(s, _)| s)
    }
    pub fn name_of(&self, subject: SubjectId) -> &str {
        self.world.entity(subject).and_then(|id| self.sim().name_of(id)).unwrap_or("someone")
    }
    pub fn name_of_entity(&self, id: Id) -> &str {
        self.sim().name_of(id).unwrap_or("someone")
    }
    pub fn offer(&self, by: SubjectId, of: SubjectId, work: Work, terms: Terms) -> Offer {
        Offer::new(self.id(by), self.id(of), work, terms)
    }

    fn social(&mut self, act: Social, at: Tick) -> Result<Answer, SocialError> {
        self.at(at)?;
        let outcome = self.world.command(Command::Social(act))?;
        serde_json::from_str(&outcome).map_err(|_| SocialError::Decode)
    }

    fn ruling(&mut self, act: Social, at: Tick) -> Result<Ruling, SocialError> {
        match self.social(act, at)? {
            Answer::Ruling(r) => Ok(r),
            _ => Err(SocialError::Decode),
        }
    }

    pub fn record(&mut self, at: Tick, doer: SubjectId, toward: Option<SubjectId>, kind: DeedKind) -> Result<Key, SocialError> {
        let act = Social::Deed { doer: self.id(doer), toward: toward.map(|t| self.id(t)), kind };
        match self.social(act, at)? {
            Answer::Deed(key) => Ok(key),
            _ => Err(SocialError::Decode),
        }
    }
    pub fn consider(&mut self, offer: &Offer, at: Tick) -> Result<Response, SocialError> {
        match self.social(Social::Consider(*offer), at)? {
            Answer::Response(r) => Ok(r),
            _ => Err(SocialError::Decode),
        }
    }
    pub fn form(&mut self, offer: &Offer, at: Tick) -> Result<Ruling, SocialError> {
        self.ruling(Social::Form(*offer), at)
    }
    pub fn exercise(&mut self, agreement: Id, work: &Work, at: Tick) -> Result<Ruling, SocialError> {
        self.ruling(Social::Exercise { agreement, work: *work }, at)
    }
    pub fn propose_change(&mut self, agreement: Id, by: SubjectId, terms: Terms, at: Tick) -> Result<Ruling, SocialError> {
        self.ruling(Social::Renegotiate { agreement, by: self.id(by), terms }, at)
    }
    pub fn end(&mut self, agreement: Id, by: SubjectId, why: EndReason, at: Tick) -> Result<Ruling, SocialError> {
        self.ruling(Social::End { agreement, by: self.id(by), why }, at)
    }
    pub fn offer_home(&mut self, offer: &Offer, dwelling: Id, at: Tick) -> Result<Ruling, SocialError> {
        self.ruling(Social::OfferHome { offer: *offer, dwelling }, at)
    }

    /// A dwelling, asserted as an authored place (769).
    pub fn found_dwelling(&mut self, key: &str, name: &str) -> Result<Id, SocialError> {
        let place = isocosm::asserted::Place { key: key.into(), name: name.into(), ..Default::default() };
        let outcome = self.world.command(Command::Assert(isocosm::asserted::Assertion::Place(place)))?;
        outcome.strip_prefix("site:").and_then(|s| s.parse().ok()).ok_or(SocialError::Decode)
    }

    pub fn agreement(&self, id: Id) -> Option<&Agreement> {
        self.sim().state().agreements.get(&id)
    }
    pub fn agreements(&self) -> impl Iterator<Item = &Agreement> {
        self.sim().state().agreements.values()
    }
    pub fn standing(&self, holder: SubjectId, toward: SubjectId) -> Standing {
        self.sim().standing(self.id(holder), self.id(toward))
    }
    pub fn deeds(&self) -> Vec<Deed> {
        self.sim().deeds()
    }
    pub fn explain(&self, premises: &[Premise]) -> Vec<String> {
        self.sim().explain(premises)
    }
    pub fn home_of(&self, subject: SubjectId) -> Option<Id> {
        self.sim().home_of(self.id(subject))
    }

    /// A wound of `cells` to `subject`, at a part drawn by its cells.
    pub fn wound(&mut self, subject: SubjectId, cells: u32, at: Tick) -> Result<(), SocialError> {
        self.at(at)?;
        let entity = self.id(subject);
        self.world.command(Command::Wound { entity, part: None, cells })?;
        Ok(())
    }
    /// The body's revision in the sim (717: a severing moves it).
    pub fn body_revision(&self, subject: SubjectId) -> u64 {
        self.sim().state().population.get(self.id(subject)).map_or(0, |e| e.body_revision)
    }

    /// `by` carves a sphere where it stands; returns the cells removed.
    pub fn carve(&mut self, by: SubjectId, centre: [i32; 3], radius: i32, at: Tick) -> Result<u32, SocialError> {
        self.at(at)?;
        match self.world.apply(WorldIntent::Carve { tick: at, by, centre, radius })? {
            eponym_play::WorldEvent::Carved { removed, .. } => Ok(removed),
            _ => Ok(0),
        }
    }

    /// The world's hash, which every social fact is in.
    pub fn hash(&self) -> u64 {
        self.world.state_hash().unwrap_or(0)
    }
}

