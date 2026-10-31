// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Named individuals (Naming 2: an entity people name is a denizen, a
//! sapient one a sophont), their kin and relations, and where they stand
//! with one another, read from names, relations and deeds.

use super::*;

/// The record's birth relations: a child to its parent, a parent to its child.
pub const PARENT: &str = "sim:parent";
pub const CHILD: &str = "sim:child";

/// One relation an individual stands in, toward or from another.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tie {
    pub kind: Key,
    pub other: Id,
    /// Whether the individual is the relation's subject.
    pub outward: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Denizen {
    pub id: Id,
    pub name: String,
    pub lineage: Key,
    /// A sapient one (Naming 2): its method is normative.
    pub sophont: bool,
    pub alive: bool,
    pub site: Option<Id>,
    /// Where it lives and what it does, read from its tenancy.
    pub round: DailyRound,
    /// Parents and children by the birth relations.
    pub kin: Vec<Tie>,
    /// Every other relation it stands in, either way.
    pub relations: Vec<Tie>,
    /// How it stands with each other individual whose deeds reach it.
    pub standing: Vec<(Id, Standing)>,
}

impl Simulation {
    /// Every named individual, by id.
    pub fn denizen_ids(&self) -> Vec<Id> {
        let about = |n: &Note| n.core.object.strip_prefix("entity:").and_then(|id| id.parse().ok());
        let named = self.state.notes.iter().filter(|n| n.core.kind == crate::arrival::NAME).filter_map(about);
        let authored = self.state.population.groups.iter()
            .filter(|(_, g)| g.entity.authored.is_some()).map(|(id, _)| *id);
        let ids: BTreeSet<Id> = named.chain(authored)
            .filter(|id| self.state.population.get(*id).is_some() && self.name_of(*id).is_some()).collect();
        ids.into_iter().collect()
    }

    /// A named individual as the record reads it; `None` where it has no name.
    pub fn denizen(&self, id: Id) -> Result<Option<Denizen>> {
        crate::society()?;
        let (Some(e), Some(name)) = (self.state.population.get(id), self.name_of(id)) else { return Ok(None) };
        let (mut kin, mut relations) = (Vec::new(), Vec::new());
        for r in &self.state.relations {
            let tie = if r.subject == id { Tie { kind: r.kind.clone(), other: r.object, outward: true } }
                else if r.object == id { Tie { kind: r.kind.clone(), other: r.subject, outward: false } }
                else { continue };
            if r.kind == PARENT || r.kind == CHILD { kin.push(tie) } else { relations.push(tie) }
        }
        let deeds = self.deeds();
        let reach: BTreeSet<Id> = deeds.iter().filter(|d| d.toward == Some(id) && d.doer != id).map(|d| d.doer).collect();
        let standing = reach.into_iter()
            .map(|other| (other, standing_in(&self.genesis.rules, &deeds, id, other)))
            .filter(|(_, s)| !s.from_deeds.is_empty()).collect();
        Ok(Some(Denizen {
            id, name: name.to_owned(), lineage: e.lineage.clone(), sophont: e.method == Method::Normative,
            alive: e.alive, site: e.resident_site(), round: self.daily(id), kin, relations, standing,
        }))
    }

    /// Every named individual.
    pub fn denizens(&self) -> Result<Vec<Denizen>> {
        crate::society()?;
        self.denizen_ids().into_iter().filter_map(|id| self.denizen(id).transpose()).collect()
    }

    /// Living kin by the birth relations, either way.
    pub(super) fn living_kin(&self, id: Id) -> BTreeSet<Id> {
        self.state.relations.iter().filter(|r| r.kind == PARENT || r.kind == CHILD)
            .filter_map(|r| if r.subject == id { Some(r.object) } else if r.object == id { Some(r.subject) } else { None })
            .filter(|k| self.state.population.get(*k).is_some_and(|e| e.alive)).collect()
    }
}
