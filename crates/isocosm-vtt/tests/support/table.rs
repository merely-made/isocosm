// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A table world: one authored party, a DM and a player, each playing one
//! embodied subject. Shared by the endpoint tests and the receipt example.

#![allow(dead_code)]

use isocosm::{
    Command, Execution, Founding,
    directing::Played,
    schema::{Id, Method},
};
use isocosm_host::{founding::Purpose, world::World};
use isocosm_overlay::ParticipantHandle;

pub const BAND: &str = "party:band";
pub const MEMBER: &str = "relation:member-of-band";

pub struct Table {
    pub world: World,
    pub dm: ParticipantHandle,
    pub player: ParticipantHandle,
    pub bodies: [Id; 2],
}

pub fn table() -> Table {
    table_with_sites(3)
}

pub fn table_with_sites(sites: u32) -> Table {
    prepared_table(sites, false)
}

pub fn table_with_native_geometry() -> Table {
    prepared_table(4, true)
}

fn prepared_table(sites: u32, physical: bool) -> Table {
    let mut genesis = Founding {
        seed: 11,
        sites,
        population: if physical { 12 } else { 24 },
        cohort_size: 1,
        lineages: if physical { 2 } else { 3 },
        ecology: true,
        map: physical.then(|| {
            isocosm::map::Layout::Grid(isocosm::map::Grid {
                shape: "shape:plane".into(),
                width: 2,
                height: 2,
                side: 64,
                elevation: [0, 0],
                relief: [0, 0],
                sea_per_mille: 0,
            })
        }),
        bodies: Some(Default::default()),
        played: Some(Played {
            lineage: 1,
            region_sites: 1,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    if physical {
        isocosm::spatial::embodiment::declare_site(&mut genesis).unwrap();
    }
    genesis.rules.relations.insert(MEMBER.into());
    genesis
        .rules
        .party_relations
        .insert(BAND.into(), MEMBER.into());
    genesis.validate().unwrap();
    let world =
        isocosm_host::founding::found(genesis, Execution::Individuals, Purpose::Bench, None)
            .unwrap();
    if physical {
        // A founding stand declares a pose; native Patch admits its residence,
        // just as the Mesocosm runtime does when it first lifts the site.
        let placements: Vec<_> = world
            .read(|session| {
                let template = session
                    .sim
                    .genesis()
                    .rules
                    .spatial_templates
                    .as_ref()
                    .unwrap();
                session
                    .sim
                    .state()
                    .spatial_poses
                    .iter()
                    .map(|(&id, resident)| {
                        let window =
                            isocosm::spatial::embodiment::site_window(template, resident.pose.at);
                        let terrain = session
                            .sim
                            .site_geometry(resident.site, &window, template.places)
                            .unwrap();
                        (id, terrain.places.place_at(resident.pose.at).unwrap())
                    })
                    .collect()
            })
            .unwrap();
        for (entity, patch) in placements {
            world
                .command(Command::Patch {
                    entity,
                    patch: Some(patch),
                })
                .unwrap();
        }
    }
    let bodies: Vec<Id> = world
        .read(|session| {
            let candidates: Vec<Id> = session
                .sim
                .state()
                .population
                .groups
                .iter()
                .filter(|(id, group)| {
                    group.entity.method == Method::Deliberative
                        && (!physical || session.sim.state().spatial_poses.contains_key(id))
                })
                .map(|(first, _)| *first)
                .collect();
            if physical {
                let mut sites = std::collections::BTreeMap::<Id, Vec<Id>>::new();
                for id in candidates {
                    if let Some(site) = session
                        .sim
                        .state()
                        .population
                        .get(id)
                        .and_then(|b| b.resident_site())
                    {
                        sites.entry(site).or_default().push(id);
                    }
                }
                sites
                    .into_values()
                    .find(|ids| ids.len() >= 2)
                    .expect("two paid native bodies share a physical site")
                    .into_iter()
                    .take(2)
                    .collect()
            } else {
                candidates.into_iter().take(2).collect()
            }
        })
        .unwrap();
    let mut participants = Vec::new();
    for body in &bodies {
        let joined = world.command(Command::Join).unwrap();
        let participant: Id = joined
            .strip_prefix("participant:")
            .unwrap()
            .parse()
            .unwrap();
        world
            .command(Command::Take {
                participant,
                critter: *body,
            })
            .unwrap();
        participants.push(ParticipantHandle(participant));
    }
    Table {
        world,
        dm: participants[0],
        player: participants[1],
        bodies: [bodies[0], bodies[1]],
    }
}
