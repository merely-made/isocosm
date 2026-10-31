// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A quick-start world (case 1023, doctrine World 10): a modded, shared
//! world whose founders are authored and enter by the founding door at its
//! first tick. A pack is a datasheet: the sites and routes it authors (case
//! 1044, Mark: "Quick-starts author their world"), its founders' lineage and
//! social vocabulary, a faction, founders and the dwellings offered to them.
//! Every product reads the same pack.

use crate::{
    Command, Founding, Result, Session,
    arrival::Arrival,
    asserted::{Assertion, Authored, Character, Faction, Place},
    schema::Lineage,
    schema::{Id, Key, Location, Method, Route, Site},
    simulation::{Execution, Genesis},
    social::{Offer, Social, TENANT, Terms, Vocabulary, Work},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// The packs this crate carries, by key.
pub const PACKS: [(&str, &str); 1] = [(
    "mill-ford",
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/datasheets/quickstart/mill-ford.toml")),
)];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub schema: u32,
    pub key: Key,
    pub name: String,
    pub status: String,
    pub sites: Vec<PackSite>,
    #[serde(default)]
    pub routes: Vec<PackRoute>,
    pub lineage: PackLineage,
    pub settle: Settle,
    pub vocabulary: Vocabulary,
    pub faction: Option<PackFaction>,
    pub founders: Vec<Founder>,
    #[serde(default)]
    pub dwellings: Vec<Dwelling>,
}

/// One site the pack authors.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackSite { pub key: Key, pub name: String }

/// A route the pack authors between two of its sites, its weight the travel.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackRoute { pub key: Key, pub from: Key, pub to: Key, pub weight: u32 }

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackLineage { pub key: Key, pub kingdom: Key }

/// Which of the pack's sites, by key, the founders arrive at.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settle { pub site: Key }

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackFaction { pub key: Key, pub name: String, pub governance: Key, pub focus: BTreeSet<Key> }

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Founder {
    pub key: Key,
    pub name: String,
    #[serde(default)]
    pub crafts: BTreeMap<Key, u64>,
    #[serde(default)]
    pub caution: i16,
    #[serde(default)]
    pub member: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dwelling { pub key: Key, pub name: String, pub tenant: Key, pub with: Key, pub work: Work, pub terms: Terms }

/// A pack from its datasheet text, wherever it was read from: carried here
/// or found in a content pack's folder.
pub fn parse(text: &str) -> Result<Pack> {
    let pack: Pack = toml::from_str(text).map_err(|e| format!("quick-start pack: {e}"))?;
    if pack.schema != 1 {
        return Err(format!("quick-start pack {}: unsupported schema {}", pack.key, pack.schema));
    }
    Ok(pack)
}

/// A carried pack by key.
pub fn pack(key: &str) -> Result<Pack> {
    let (_, text) = PACKS.iter().find(|(k, _)| *k == key).ok_or("no quick-start pack under that key")?;
    let pack = parse(text)?;
    if pack.key != key {
        return Err(format!("quick-start pack {key}: its key disagrees"));
    }
    Ok(pack)
}

fn id(outcome: &str, kind: &str) -> Result<Id> {
    outcome.strip_prefix(kind).and_then(|s| s.strip_prefix(':')).and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("expected a {kind} from the founding door, got {outcome}"))
}

impl Pack {
    /// The pack's authored world under `seed`: the stock rules, with no
    /// drawn lineage, process or body, and only the pack's sites and routes,
    /// so `founding` stays absent and the head gate admits it (case 1044).
    fn authored(&self, seed: u64) -> Result<(Genesis, BTreeMap<Key, Id>)> {
        if self.sites.is_empty() {
            return Err(format!("quick-start pack {} authors no site", self.key));
        }
        let stock = Founding { seed, sites: 1, population: 1, lineages: 1, cohort_size: 1, ..Default::default() };
        let mut genesis = stock.generate()?;
        genesis.founding = None;
        let drawn: Vec<Key> = genesis.lineages.keys().filter(|k| *k != "world:ground").cloned().collect();
        for lineage in &drawn {
            genesis.lineages.remove(lineage);
        }
        let r = &mut genesis.rules;
        r.processes.retain(|k, _| !k.starts_with("process:cycle-"));
        r.accounts.retain(|_, kind| !matches!(kind,
            crate::rules::AccountKind::Matter { lineage, .. } if drawn.contains(lineage)));
        r.traits.retain(|t| !t.starts_with("ability:cycle-"));
        r.limits.entities = r.limits.entities.max(1_000_001 + self.sites.len() as u64);
        let ground = genesis.population.groups.values()
            .find(|g| g.entity.lineage == "world:ground").map(|g| g.entity.as_ref().clone())
            .ok_or("the stock world has no ground body")?;
        let ids: BTreeMap<Key, Id> = self.sites.iter().enumerate().map(|(i, s)| (s.key.clone(), i as Id)).collect();
        let site = |key: &str| ids.get(key).copied().ok_or_else(|| format!("route names no site {key}"));
        let mut sites = BTreeMap::new();
        for (i, s) in self.sites.iter().enumerate() {
            let place = Place { key: s.key.clone(), name: s.name.clone(), ..Default::default() };
            sites.insert(i as Id, Site { terrain_seed: crate::draw(seed, "terrain", &[i as u64]),
                conditions: BTreeMap::from([("world:habitable".into(), 1), ("world:weather".into(), 0)]),
                accounts: BTreeMap::new(), routes: vec![], authored: Some(place) });
        }
        for r in &self.routes {
            let (from, to) = (site(&r.from)?, site(&r.to)?);
            let authored = crate::asserted::Route { key: r.key.clone(), from: r.from.clone(), to: r.to.clone(),
                tags: Default::default(), weight: r.weight };
            for (at, toward) in [(from, to), (to, from)] {
                sites.get_mut(&at).expect("an authored site").routes.push(Route { to: toward,
                    travel: u64::from(r.weight).max(1), transmission: 0, border: None, authored: Some(authored.clone()) });
            }
        }
        genesis.sites = sites;
        let mut population = crate::population::Population::default();
        for &id in ids.values() {
            population.insert(crate::schema::Entity { place: id, ..ground.clone() }, 1)?;
        }
        genesis.population = population;
        genesis.world.footprint = None;
        Ok((genesis, ids))
    }

    /// Adds the pack's lineage, social vocabulary and relations to `genesis`.
    pub fn lay(&self, genesis: &mut Genesis) -> Result<()> {
        if genesis.lineages.contains_key(&self.lineage.key) {
            return Err("the pack's lineage is already the world's".into());
        }
        genesis.lineages.insert(self.lineage.key.clone(), Lineage { parent: None, revision: 1,
            traits: BTreeSet::new(), kingdom: self.lineage.kingdom.clone(), development: None, decides: None });
        let r = &mut genesis.rules;
        r.relations.insert(TENANT.to_string());
        r.processes.extend(crate::social::processes(&self.vocabulary));
        r.social = Some(self.vocabulary.clone());
        r.founding_events = true;
        genesis.validate()
    }

    /// Enters the founders by the founding door at the first tick, then
    /// forms each dwelling's tenancy; `run` is the session's command door.
    pub fn found(&self, run: &mut dyn FnMut(Command) -> Result<String>, sites: &BTreeMap<Key, Id>) -> Result<()> {
        let site = *sites.get(&self.settle.site).ok_or("the pack settles at no site of its own")?;
        let mut claims = BTreeSet::new();
        if let Some(f) = &self.faction {
            let authored = Authored { key: f.key.clone(), name: f.name.clone(), ..Default::default() };
            let faction = Faction { authored, governance: f.governance.clone(), focus: f.focus.clone() };
            claims.insert(id(&run(Command::Assert(Assertion::Faction(faction)))?, "polity")?);
        }
        let mut people = BTreeMap::new();
        for f in &self.founders {
            let faction = self.faction.as_ref().filter(|_| f.member).map(|x| x.key.clone());
            let character = Character { key: f.key.clone(), name: f.name.clone(), faction, ..Default::default() };
            run(Command::Assert(Assertion::Character(character)))?;
            let mut disposition = [0; 5];
            disposition[crate::social::CAUTION] = f.caution;
            let arrival = Arrival { lineage: self.lineage.key.clone(), site, method: Method::Normative,
                accounts: Default::default(), traits: Default::default(), skills: f.crafts.clone(),
                disposition, character: Some(f.key.clone()) };
            people.insert(f.key.clone(), id(&run(Command::Arrive(arrival))?, "entity")?);
        }
        let who = |key: &str| people.get(key).copied().ok_or_else(|| format!("no founder {key}"));
        let mut held = BTreeSet::from([site]);
        let mut homes = Vec::new();
        for d in &self.dwellings {
            let place = Place { key: d.key.clone(), name: d.name.clone(), ..Default::default() };
            let dwelling = id(&run(Command::Assert(Assertion::Place(place)))?, "site")?;
            held.insert(dwelling);
            homes.push((Offer::new(who(&d.with)?, who(&d.tenant)?, d.work.clone(), d.terms), dwelling));
        }
        let location = Location { sites: held, claims, parent: None, ratio: 1 };
        run(Command::Location { location })?;
        for (offer, dwelling) in homes {
            let answer = run(Command::Social(Social::OfferHome { offer, dwelling }))?;
            if !answer.contains("Formed") {
                return Err(format!("a pack tenancy was not formed: {answer}"));
            }
        }
        Ok(())
    }
}

impl Pack {
    /// The pack's authored world under `seed`, laid, and its sites by key,
    /// before anyone enters.
    pub fn genesis(&self, seed: u64) -> Result<(Genesis, BTreeMap<Key, Id>)> {
        let (mut genesis, sites) = self.authored(seed)?;
        self.lay(&mut genesis)?;
        Ok((genesis, sites))
    }
}

/// The pack `key`'s authored world under `seed`, founded.
pub fn session(key: &str, seed: u64, mode: Execution) -> Result<Session> {
    let pack = pack(key)?;
    let (genesis, sites) = pack.genesis(seed)?;
    let mut session = Session::new(genesis, mode)?;
    pack.found(&mut |c| session.command(c), &sites)?;
    Ok(session)
}
