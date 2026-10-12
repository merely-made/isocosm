// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

mod access;
mod binding;
mod file_save;
mod lifecycle;
#[cfg(all(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
mod products;
#[cfg(all(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
mod reading;
mod save;
mod spatial;
mod world;
#[cfg(all(feature = "mesocosm", feature = "vtt"))]
mod viewport;

use std::collections::BTreeSet;

use isocosm::{Command, Execution, Founding, Session};
use isocosm::directing::Played;
use isocosm::schema::{Entity, Id, Method};
use isocosm_overlay::{AttentionSet, ParticipantHandle, Seat};

use crate::binding::Binding;
use crate::window::Window;
use crate::world::World;

struct Fixture {
    world: World,
    participants: [ParticipantHandle; 2],
    subjects: [Id; 2],
}

fn fixture() -> Fixture {
    let genesis = Founding {
        seed: 11,
        sites: 3,
        population: 24,
        cohort_size: 1,
        lineages: 3,
        ecology: true,
        bodies: Some(Default::default()),
        played: Some(Played { lineage: 1, region_sites: 1 }),
        ..Default::default()
    }.generate().unwrap();
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let subjects: Vec<Id> = session.sim.state().population.groups.iter()
        .filter(|(_, group)| group.entity.method == Method::Deliberative)
        .map(|(first, _)| *first)
        .take(2)
        .collect();
    assert_eq!(subjects.len(), 2, "fixture needs two distinct played bodies");
    let mut participants = Vec::new();
    for subject in &subjects {
        let joined = session.command(Command::Join).unwrap();
        let participant: Id = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        session.command(Command::Take { participant, critter: *subject }).unwrap();
        participants.push(ParticipantHandle(participant));
    }
    Fixture {
        world: World::new(session),
        participants: [participants[0], participants[1]],
        subjects: [subjects[0], subjects[1]],
    }
}

fn binding(mode: &str) -> Binding {
    Binding {
        mode: mode.into(),
        controls: vec![format!("{mode}:fixture-control")],
        view_defaults: mode.as_bytes().to_vec(),
        attention: AttentionSet::default(),
    }
}

fn window(f: &Fixture, participant: usize, seat: Seat, mode: &str) -> Window {
    Window::new(&f.world, f.participants[participant], seat, binding(mode)).unwrap()
}

fn body(f: &Fixture, subject: Id) -> Entity {
    let entity = f.world.read(|session| {
        session.sim.state().population.get(subject).unwrap().clone()
    }).unwrap();
    assert!(entity.body.is_some(), "fixture must preserve an actual embodiment");
    entity
}

fn all_subjects(f: &Fixture) -> BTreeSet<Id> {
    f.world.read(|session| session.sim.state().population.groups.keys().copied().collect()).unwrap()
}
