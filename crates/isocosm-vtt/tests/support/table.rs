// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A table world: one authored party, a DM and a player, each playing one
//! embodied subject. Shared by the endpoint tests and the receipt example.

#![allow(dead_code)]

use isocosm::{Command, Execution, Founding, Session, directing::Played, schema::{Id, Method}};
use isocosm_host::world::World;
use isocosm_overlay::ParticipantHandle;

pub const BAND: &str = "party:band";
pub const MEMBER: &str = "relation:member-of-band";

pub struct Table { pub world: World, pub dm: ParticipantHandle, pub player: ParticipantHandle, pub bodies: [Id; 2] }

pub fn table() -> Table {
    let mut genesis = Founding { seed: 11, sites: 3, population: 24, cohort_size: 1, lineages: 3, ecology: true,
        bodies: Some(Default::default()), played: Some(Played { lineage: 1, region_sites: 1 }), ..Default::default() }
        .generate().unwrap();
    genesis.rules.relations.insert(MEMBER.into());
    genesis.rules.party_relations.insert(BAND.into(), MEMBER.into());
    genesis.validate().unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let bodies: Vec<Id> = session.sim.state().population.groups.iter()
        .filter(|(_, group)| group.entity.method == Method::Deliberative).map(|(first, _)| *first).take(2).collect();
    let mut participants = Vec::new();
    for body in &bodies {
        let joined = session.command(Command::Join).unwrap();
        let participant: Id = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        session.command(Command::Take { participant, critter: *body }).unwrap();
        participants.push(ParticipantHandle(participant));
    }
    Table { world: World::new(session), dm: participants[0], player: participants[1], bodies: [bodies[0], bodies[1]] }
}
