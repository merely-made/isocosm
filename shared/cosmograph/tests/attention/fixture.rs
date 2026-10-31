// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Command, Execution, Founding, Session, directing::Played, schema::{Id, Method}};

/// Reuses the declared core-host seed-11 body fixture and native Join/Take.
pub fn fixture() -> (Session, [Id; 2], [Id; 2]) {
    let mut genesis = Founding { seed: 11, sites: 3, population: 24,
        cohort_size: 1, lineages: 3, ecology: true, bodies: Some(Default::default()),
        played: Some(Played { lineage: 1, region_sites: 1 }), ..Default::default()
    }.generate().unwrap();
    // The buffer is declared world data; a short setting makes its gate exact.
    genesis.rules.collection_buffer = 3;
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    let subjects: Vec<_> = session.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.method == Method::Deliberative)
        .map(|(id, _)| *id).take(2).collect();
    assert_eq!(subjects.len(), 2, "declared fixture requires two native played bodies");
    let mut participants = Vec::new();
    for subject in &subjects {
        let joined = session.command(Command::Join).unwrap();
        let participant = joined.strip_prefix("participant:").unwrap().parse().unwrap();
        session.command(Command::Take { participant, critter: *subject }).unwrap();
        participants.push(participant);
    }
    (session, [participants[0], participants[1]], [subjects[0], subjects[1]])
}
