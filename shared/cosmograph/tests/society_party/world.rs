// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench world with one authored party and its membership relation.
use super::*;
use cosmograph::party::{Change, Outcome};

pub const BAND: &str = "party:band";
pub const MEMBER: &str = "relation:member-of-band";

pub fn party_world(seed: u64) -> World {
    let mut w = world(seed);
    w.genesis.rules.relations.insert(MEMBER.into());
    w.genesis.rules.party_relations.insert(BAND.into(), MEMBER.into());
    w.genesis.validate().unwrap();
    w
}

pub fn party(change: Change) -> Command { Command::Party(change) }

pub fn join(participant: Id, member: Id) -> Command {
    party(Change::Join { participant, party: BAND.into(), member })
}

pub fn leave(participant: Id, member: Id) -> Command {
    party(Change::Leave { participant, party: BAND.into(), member })
}

pub fn disclose(participant: Id, member: Id, sites: &[Id]) -> Command {
    party(Change::Disclose { participant, party: BAND.into(), member,
        sites: sites.iter().copied().collect(), events: BTreeSet::new() })
}

pub fn outcome(result: cosmograph::Result<String>) -> Outcome { serde_json::from_str(&result.unwrap()).unwrap() }

/// Three played bodies; the first two participants are the party's players.
pub fn table(w: &World) -> (Session, [Id; 3]) {
    let mut s = Session::new(w.genesis.clone(), Execution::Individuals).unwrap();
    let players = w.bodies.map(|body| join_take(&mut s, body));
    (s, players)
}
