// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The lineages a participant knows, and what each regards them (cases
//! 1210, 1214): by playing one, by a fork of one played, by a whole meal of
//! one's part by a played line, by meeting one or by outpacing one (the
//! last two kept as `directing:knows` relations). A pure function of the
//! world and its log, never kept. It reads the log, so it takes a
//! `Session`, unlike its siblings.

use crate::{
    directing::KNOWS, history::Command, lineage::tree, schema::*, simulation::Simulation, Session,
};
use std::collections::{BTreeMap, BTreeSet};

/// The lineages `participant` knows; none unless the world sets regard.
pub fn known(session: &Session, participant: Id) -> BTreeSet<Key> {
    let state = session.sim.state();
    let mut out = BTreeSet::new();
    if session.sim.genesis().rules.directing.as_ref().is_none_or(|d| d.regard.is_none()) {
        return out;
    }
    let lines = &state.lineages;
    let line_of = |id: Id| state.population.get(id).map(|e| e.lineage.clone());
    let played: BTreeSet<Key> = session.entries.iter().filter_map(|e| match &e.command {
        Command::Take { participant: p, critter } | Command::TakeAlso { participant: p, critter }
            if *p == participant => line_of(*critter),
        _ => None,
    }).collect();
    for line in &played {
        out.extend(tree::ancestry(lines, line));
        out.extend(lines.keys().filter(|k| tree::descends_from(lines, k, line)).cloned());
        out.extend(lines.get(line).into_iter().flat_map(|l| l.ate.iter().cloned()));
    }
    out.extend(state.relations.iter()
        .filter(|r| r.subject == participant && r.kind == KNOWS)
        .filter_map(|r| line_of(r.object)));
    out
}

/// Each lineage's regard for `participant`, by lineage.
pub fn regard(sim: &Simulation, participant: Id) -> BTreeMap<Key, i64> {
    sim.state().lineages.iter()
        .filter_map(|(k, l)| l.regard.get(&participant).map(|v| (k.clone(), *v))).collect()
}
