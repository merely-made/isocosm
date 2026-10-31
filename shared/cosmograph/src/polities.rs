// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored polity fill and actual member readings retain their sources (867).

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberAttributes {
    pub alive: bool, pub resident_site: Option<Id>, pub lineage: Key,
    pub tags: BTreeSet<Key>, pub traits: BTreeSet<Key>, pub skills: BTreeMap<Key,u64>,
    pub tenets: BTreeMap<Key,Tenet>, pub disposition: [i16;5],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberReading {
    pub id: Id,
    pub constitutional: bool,
    pub relation: bool,
    /// Missing member state stays unavailable, never inferred from a label.
    pub attributes: Option<MemberAttributes>,
    pub acts: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDisagreement { pub authored_only: BTreeSet<Key>, pub members_only: BTreeSet<Key> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolityReading {
    pub id: Id,
    pub authored: Option<Authored>,
    pub constitution: Constitution,
    pub ended_by: Option<Key>,
    pub members: Vec<MemberReading>,
    /// Exact living native members supporting each authored character tag.
    pub member_tags: BTreeMap<Key,BTreeSet<Id>>,
    /// No members means no derived tag claim, rather than disagreement with emptiness.
    pub disagreement: Option<TagDisagreement>,
}

impl Simulation {
    /// A trusted native read. Consumers must impose their own admitted knowledge cut.
    pub fn polity_reading(&self, id: Id) -> Result<PolityReading> {
        crate::society()?;
        let polity = self.state.polities.get(&id).ok_or("unknown native polity")?;
        let related: BTreeSet<_> = self.state.relations.iter()
            .filter(|relation| relation.kind == "member" && relation.object == id)
            .map(|relation| relation.subject).collect();
        let members: BTreeSet<_> = polity.constitution.members.union(&related).copied().collect();
        let mut member_tags: BTreeMap<Key,BTreeSet<Id>> = BTreeMap::new();
        let mut living = 0;
        let members = members.into_iter().map(|member| {
            let attributes = self.state.population.get(member).map(|entity| {
                let tags = entity.authored.as_ref().map_or_else(BTreeSet::new,|authored| authored.tags.clone());
                if entity.alive {
                    living += 1;
                    for tag in &tags { member_tags.entry(tag.clone()).or_default().insert(member); }
                }
                MemberAttributes { alive:entity.alive,resident_site:entity.resident_site(),
                    lineage:entity.lineage.clone(),tags,traits:entity.traits.clone(),skills:entity.skills.clone(),
                    tenets:entity.tenets.clone(),disposition:entity.disposition }
            });
            let mut acts: Vec<_> = self.state.events.values().filter(|event| event.authored.is_none()
                && (event.subject == member || event.object == Some(member))).cloned().collect();
            acts.sort_by(|a,b| (a.tick,&a.id).cmp(&(b.tick,&b.id)));
            MemberReading { id:member,constitutional:polity.constitution.members.contains(&member),
                relation:related.contains(&member),attributes,acts }
        }).collect();
        let tags: BTreeSet<_> = member_tags.keys().cloned().collect();
        let disagreement = polity.authored.as_ref().filter(|_| living > 0).and_then(|authored| {
            let authored_only = authored.tags.difference(&tags).cloned().collect::<BTreeSet<_>>();
            let members_only = tags.difference(&authored.tags).cloned().collect::<BTreeSet<_>>();
            (!authored_only.is_empty() || !members_only.is_empty()).then_some(TagDisagreement { authored_only,members_only })
        });
        Ok(PolityReading { id,authored:polity.authored.clone(),constitution:polity.constitution.clone(),
            ended_by:polity.ended_by.clone(),members,member_tags,disagreement })
    }
}
