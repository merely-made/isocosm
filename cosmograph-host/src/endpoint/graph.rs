// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world as one graph (case 1038): regions and sites as clusters with
//! route edges, opening into resident stored groups (the far aggregates,
//! Processes 6), their kin and relations, and the locations and polities the
//! ladder reads. A closed cluster is a sceno fold behind its own card; a
//! filter narrows the graph by domain (Naming 3: "filter", never "lens").

use std::collections::{BTreeMap, BTreeSet};

use chirograph::IntentEffect;
use cosmograph::Session;
use cosmograph::directing::readings::regions;
use cosmograph::schema::Id;

use super::disclosure::{Folded, Item, NULL, action, card, value};

pub const OPEN: &str = "cosmograph.open";
pub const CLOSE: &str = "cosmograph.close";

/// The domains a filter keeps (case 1038).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain { Geography, Ecology, Society, Polities }

impl Domain {
    pub const ALL: [Domain; 4] = [Domain::Geography, Domain::Ecology, Domain::Society, Domain::Polities];
    pub fn word(self) -> &'static str {
        match self {
            Domain::Geography => "geography",
            Domain::Ecology => "ecology",
            Domain::Society => "society",
            Domain::Polities => "polities",
        }
    }
    pub fn parse(word: &str) -> Option<Self> { Self::ALL.into_iter().find(|d| d.word() == word) }
}

/// The viewer's curation of the graph: what is opened and what is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    /// Opened clusters by id; regions start open, sites closed.
    pub opened: BTreeSet<String>,
    pub closed: BTreeSet<String>,
    pub domains: BTreeSet<Domain>,
}

impl Default for View {
    fn default() -> Self {
        Self { opened: BTreeSet::new(), closed: BTreeSet::new(), domains: Domain::ALL.into_iter().collect() }
    }
}

impl View {
    pub fn is_open(&self, id: &str) -> bool {
        if id.starts_with("region:") { !self.closed.contains(id) } else { self.opened.contains(id) }
    }
    pub fn open(&mut self, id: &str) { self.closed.remove(id); self.opened.insert(id.into()); }
    pub fn close(&mut self, id: &str) { self.opened.remove(id); self.closed.insert(id.into()); }
    fn keeps(&self, domain: Domain) -> bool { self.domains.contains(&domain) }
}

/// One graph: its items, edges (from, to, kind) and folds.
#[derive(Default)]
pub struct Graph {
    pub items: Vec<Item>,
    pub links: Vec<(String, String, &'static str)>,
    pub folds: Vec<Folded>,
}

fn toggle(view: &View, id: &str, what: &str) -> chirograph::AdvertisedAction {
    if view.is_open(id) {
        action(CLOSE, "Close", &format!("Fold this {what} behind its card."), NULL, IntentEffect::Curation)
    } else {
        action(OPEN, "Open", &format!("Show what this {what} holds."), NULL, IntentEffect::Curation)
    }
}

/// The stored group holding `entity`, by its first id.
fn group_of(session: &Session, entity: Id) -> Option<Id> {
    let groups = &session.sim.state().population.groups;
    let (&first, group) = groups.range(..=entity).next_back()?;
    (entity - first < group.count).then_some(first)
}

/// The graph of `session` under `view`; `acts` are offered on each group.
pub fn graph(session: &Session, view: &View, acts: &[chirograph::AdvertisedAction]) -> Graph {
    let sim = &session.sim;
    let state = sim.state();
    let mut g = Graph::default();
    let regions = regions(sim);
    // Per site: its region and its resident groups' ids.
    let mut region_of = BTreeMap::new();
    for region in &regions {
        let id = format!("region:{}", region.sites.first().copied().unwrap_or_default());
        for &site in &region.sites { region_of.insert(site, id.clone()); }
    }
    let mut residents: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    for (&first, group) in &state.population.groups {
        if let Some(site) = group.entity.resident_site() { residents.entry(site).or_default().push(first); }
    }
    let shown_sites: BTreeSet<Id> = if view.keeps(Domain::Geography) { state.sites.keys().copied().collect() }
        else { BTreeSet::new() };
    // Regions, each a cluster of sites.
    if view.keeps(Domain::Geography) {
        for region in &regions {
            let id = format!("region:{}", region.sites.first().copied().unwrap_or_default());
            let mut values = vec![value("Sites", region.sites.len()), value("Collapsed", region.collapsed)];
            values.extend(region.biomass.iter().map(|(level, m)| value(level.as_str(), m)));
            g.items.push(Item { id: id.clone(), card: card(format!("Region of site {}", &id[7..]), values,
                &["region"]), actions: vec![toggle(view, &id, "region")] });
        }
    }
    // Sites, with their routes and their region.
    for &site in &shown_sites {
        let at = &state.sites[&site];
        let id = format!("site:{site}");
        let here = residents.get(&site).map(Vec::as_slice).unwrap_or_default();
        let alive: u64 = here.iter().map(|f| &state.population.groups[f])
            .filter(|g| g.entity.alive).map(|g| g.count).sum();
        let name = at.authored.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| format!("Site {site}"));
        g.items.push(Item { id: id.clone(), card: card(name, vec![value("Alive", alive),
            value("Stored groups", here.len()), value("Routes", at.routes.len())], &["site"]),
            actions: vec![toggle(view, &id, "site")] });
        for route in &at.routes {
            if shown_sites.contains(&route.to) { g.links.push((id.clone(), format!("site:{}", route.to), "route")); }
        }
        if let Some(region) = region_of.get(&site) { g.links.push((id, region.clone(), "in-region")); }
    }
    // Resident stored groups: every group of a shown site's residents.
    let mut shown_groups = BTreeSet::new();
    if view.keeps(Domain::Ecology) {
        for (&site, firsts) in &residents {
            for &first in firsts {
                let group = &state.population.groups[&first];
                let e = &group.entity;
                let named = sim.name_of(first).filter(|_| group.count == 1);
                let title = named.map(str::to_owned).unwrap_or_else(|| format!("{} ×{}", e.lineage, group.count));
                let badges: &[&str] = if named.is_some() { &["group", "denizen"] } else { &["group"] };
                g.items.push(Item { id: format!("group:{first}"), card: card(title, vec![
                    value("Lineage", &e.lineage), value("Kingdom", &e.kingdom), value("Members", group.count),
                    value("Alive", e.alive), value("Born", e.born)], badges), actions: acts.to_vec() });
                if shown_sites.contains(&site) {
                    g.links.push((format!("group:{first}"), format!("site:{site}"), "resides"));
                }
                shown_groups.insert(first);
            }
        }
    }
    // Kin and relations between shown groups.
    if view.keeps(Domain::Society) {
        let mut seen = BTreeSet::new();
        for r in &state.relations {
            let (Some(a), Some(b)) = (group_of(session, r.subject), group_of(session, r.object)) else { continue };
            if a == b || !shown_groups.contains(&a) || !shown_groups.contains(&b) { continue; }
            let kind = match r.kind.as_str() {
                cosmograph::social::ladder::PARENT | cosmograph::social::ladder::CHILD => "kin",
                "member" => continue,
                _ => "relation",
            };
            if seen.insert((a.min(b), a.max(b), kind)) {
                g.links.push((format!("group:{a}"), format!("group:{b}"), kind));
            }
        }
    }
    society(session, view, &shown_sites, &shown_groups, &mut g);
    fold(view, &regions, &residents, &mut g);
    g
}

/// Locations at their sites, and polities with their claims and members.
fn society(session: &Session, view: &View, sites: &BTreeSet<Id>, groups: &BTreeSet<Id>, g: &mut Graph) {
    let state = session.sim.state();
    if view.keeps(Domain::Society) {
        for reading in session.sim.place_readings().unwrap_or_default() {
            let id = format!("location:{}", reading.location);
            let tier = reading.tier.map(|t| t.word()).unwrap_or("none");
            g.items.push(Item { id: id.clone(), card: card(format!("Location {}", reading.location), vec![
                value("Tier", tier), value("Residents", reading.residents.len()),
                value("Households", reading.households.len())], &["location"]), actions: vec![] });
            for site in reading.sites.intersection(sites) { g.links.push((id.clone(), format!("site:{site}"), "at")); }
        }
    }
    if !view.keeps(Domain::Polities) { return; }
    for (&id, p) in &state.polities {
        let node = format!("polity:{id}");
        let kind = if p.constitution.members.is_empty() { "faction" } else { "polity" };
        let name = p.authored.as_ref().map(|a| a.name.clone()).unwrap_or_else(|| format!("Group {id}"));
        g.items.push(Item { id: node.clone(), card: card(name, vec![value("Governance", &p.constitution.governance),
            value("Ended", p.ended_by.is_some())], &[kind]), actions: vec![] });
        let members: BTreeSet<Id> = p.constitution.members.iter().copied()
            .chain(state.relations.iter().filter(|r| r.kind == "member" && r.object == id).map(|r| r.subject))
            .filter_map(|m| group_of(session, m)).filter(|m| groups.contains(m)).collect();
        g.links.extend(members.into_iter().map(|m| (format!("group:{m}"), node.clone(), "member")));
        if view.keeps(Domain::Society) {
            for (&location, l) in &state.locations {
                if l.claims.contains(&id) { g.links.push((node.clone(), format!("location:{location}"), "claims")); }
            }
        }
    }
}

/// Closed clusters fold behind their card: a closed region holds its sites
/// and their residents; a closed site in an open region holds its residents.
fn fold(view: &View, regions: &[cosmograph::directing::readings::Region], residents: &BTreeMap<Id, Vec<Id>>,
    g: &mut Graph) {
    let shown: BTreeSet<&str> = g.items.iter().map(|i| i.id.as_str()).collect();
    let held = |site: Id| residents.get(&site).into_iter().flatten().map(|f| format!("group:{f}"));
    let mut folds = Vec::new();
    for region in regions {
        let id = format!("region:{}", region.sites.first().copied().unwrap_or_default());
        if !shown.contains(id.as_str()) { continue; }
        if !view.is_open(&id) {
            let members = region.sites.iter().flat_map(|&s| std::iter::once(format!("site:{s}")).chain(held(s)))
                .filter(|m| shown.contains(m.as_str())).collect();
            folds.push(Folded { stand_in: id.clone(), members, label: format!("{}'s sites", &id) });
            continue;
        }
        for &site in &region.sites {
            let id = format!("site:{site}");
            if view.is_open(&id) || !shown.contains(id.as_str()) { continue; }
            let members = held(site).filter(|m| shown.contains(m.as_str())).collect();
            folds.push(Folded { stand_in: id, members, label: format!("site {site}'s residents") });
        }
    }
    g.folds = folds;
}
