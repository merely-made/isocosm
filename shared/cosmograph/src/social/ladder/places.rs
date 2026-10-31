// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A place read off its dwellings, tenancies, work and institutions
//! (doctrine World 10): a `Location` whose sites hold its dwellings (case
//! 1027), its tier counting living residents, the factions and polities
//! acting through them, and the record that explains it. Never stored.

use super::*;

/// A faction or polity with residents among its members: a group is a
/// polity once its constitution names members, a faction otherwise (cases
/// 1024 and 1032).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// The group's native record, which every group has.
    pub id: Id,
    pub polity: bool,
    pub name: Option<String>,
    pub governance: Key,
    /// Residents its constitution names.
    pub constitutional: Vec<Id>,
    /// Residents related to it by membership alone.
    pub related: Vec<Id>,
    pub ended: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceReading {
    pub location: Id,
    pub sites: BTreeSet<Id>,
    /// Polities the location names as claiming it.
    pub claims: BTreeSet<Id>,
    pub households: Vec<Household>,
    /// Living members of its households (case 1027).
    pub residents: Vec<Id>,
    /// Hamlet, town or city by residents; none below the first threshold.
    pub tier: Option<Tier>,
    pub groups: Vec<Group>,
    /// What the record holds about it, earliest first: at its sites, under
    /// its tenancies, by or toward its residents, or by its claimants.
    pub record: Vec<Event>,
}

impl Simulation {
    pub fn place_reading(&self, location: Id) -> Result<PlaceReading> {
        crate::society()?;
        let l = self.state.locations.get(&location).ok_or("no such location")?;
        let households = self.households_at(&l.sites)?;
        let residents: BTreeSet<Id> = households.iter().flat_map(|h| h.members.iter().copied()).collect();
        let tier = tiers().0.tier(residents.len() as u64);
        let mut groups = Vec::new();
        for (&id, p) in &self.state.polities {
            let constitutional: Vec<Id> = residents.iter().copied()
                .filter(|r| p.constitution.members.contains(r)).collect();
            let related: Vec<Id> = residents.iter().copied().filter(|r| !p.constitution.members.contains(r)
                && self.state.relations.iter().any(|x| x.kind == "member" && x.subject == *r && x.object == id)).collect();
            if constitutional.is_empty() && related.is_empty() { continue; }
            groups.push(Group { id, polity: !p.constitution.members.is_empty(), name: p.authored.as_ref().map(|a| a.name.clone()),
                governance: p.constitution.governance.clone(), constitutional, related, ended: p.ended_by.is_some() });
        }
        let agreements: BTreeSet<Id> = households.iter().map(|h| h.agreement).collect();
        let touches = |e: &Event| matches!(e.place, EventPlace::Site(s) if l.sites.contains(&s))
            || e.agreement.is_some_and(|a| agreements.contains(&a))
            || residents.contains(&e.subject) || e.object.is_some_and(|o| residents.contains(&o))
            || l.claims.contains(&e.subject);
        let mut record: Vec<Event> = self.state.events.values().filter(|e| touches(e)).cloned().collect();
        // What came in by the founding door comes before what was done at its tick.
        let door = |e: &Event| !e.id.starts_with("founding:");
        record.sort_by(|a, b| (a.tick, door(a), &a.id).cmp(&(b.tick, door(b), &b.id)));
        Ok(PlaceReading { location, sites: l.sites.clone(), claims: l.claims.clone(), households,
            residents: residents.into_iter().collect(), tier, groups, record })
    }

    /// Every location's reading, by id.
    pub fn place_readings(&self) -> Result<Vec<PlaceReading>> {
        crate::society()?;
        self.state.locations.keys().map(|id| self.place_reading(*id)).collect()
    }
}
