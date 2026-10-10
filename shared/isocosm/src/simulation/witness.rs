// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The state's labelled entries, through mere's `state-witness`: one per
//! field for saves (ruling 633), one per entity for traces (610, 649).

use super::*;
use state_witness::{Witness, hash_value, label};

fn put<T: Serialize + ?Sized>(w: &mut Witness, label: impl Into<String>, value: &T) {
    w.insert(label, value)
        .expect("labels are unique and NUL-free, and the sim schema encodes");
}

impl Simulation {
    /// One entry per field, the world's seed, revision and traits beside the
    /// state's, so the digest covers what the whole-state hash did (652).
    pub fn witness(&self) -> Witness {
        let (seed, revision, world, s) = self.witnessed();
        let mut w = Witness::new();
        put(&mut w, "seed", &seed);
        put(&mut w, "revision", revision);
        put(&mut w, "world", world);
        put(&mut w, "tick", &s.tick);
        put(&mut w, "next_action", &s.next_action);
        put(&mut w, "population", &s.population);
        put(&mut w, "sites", &s.sites);
        put(&mut w, "lineages", &s.lineages);
        put(&mut w, "locations", &s.locations);
        put(&mut w, "polities", &s.polities);
        put(&mut w, "relations", &s.relations);
        put(&mut w, "notes", &s.notes);
        put(&mut w, "roots", &s.roots);
        put(&mut w, "released", &s.released);
        put(&mut w, "record", &s.record);
        put(&mut w, "events", &s.events);
        put(&mut w, "reach", &s.reach);
        if !s.edits.is_empty() {
            put(&mut w, "edits", &s.edits);
        }
        if !s.nudges.is_empty() {
            put(&mut w, "nudges", &s.nudges);
        }
        if !s.laws.is_empty() {
            put(&mut w, "laws", &s.laws);
        }
        if !s.agreements.is_empty() {
            put(&mut w, "agreements", &s.agreements);
        }
        w
    }
    /// One entry per entity, every critter by id, its cohort's digest
    /// computed once (649); notes by their append-only index.
    pub fn entity_witness(&self) -> Witness {
        let (seed, revision, world, s) = self.witnessed();
        let mut w = Witness::new();
        put(&mut w, "seed", &seed);
        put(&mut w, "revision", revision);
        put(&mut w, "world", world);
        put(&mut w, "tick", &s.tick);
        put(&mut w, "next_action", &s.next_action);
        put(&mut w, "next_id", &s.population.next_id);
        for (&first, cohort) in &s.population.groups {
            let digest = hash_value(&cohort.entity).expect("sim schema encodes");
            for id in first..first + cohort.count {
                put(&mut w, label("entity", [id]), &digest);
            }
        }
        for (id, site) in &s.sites {
            put(&mut w, label("site", [id]), site);
        }
        for (key, lineage) in &s.lineages {
            put(&mut w, label("lineage", [key]), lineage);
        }
        for (id, location) in &s.locations {
            put(&mut w, label("location", [id]), location);
        }
        for (id, polity) in &s.polities {
            put(&mut w, label("polity", [id]), polity);
        }
        for r in &s.relations {
            let parts = [r.subject.to_string(), r.kind.clone(), r.object.to_string()];
            put(&mut w, label("relation", parts), r);
        }
        for (index, note) in s.notes.iter().enumerate() {
            put(&mut w, label("note", [index]), note);
        }
        for (index, edit) in s.edits.iter().enumerate() {
            put(&mut w, label("edit", [index]), edit);
        }
        for (index, nudge) in s.nudges.iter().enumerate() {
            put(&mut w, label("nudge", [index]), nudge);
        }
        for (key, law) in &s.laws {
            put(&mut w, label("law", [key]), law);
        }
        for (id, agreement) in &s.agreements {
            put(&mut w, label("agreement", [id]), agreement);
        }
        for id in &s.roots {
            put(&mut w, label("root", [id]), &());
        }
        for (id, until) in &s.released {
            put(&mut w, label("released", [id]), until);
        }
        for axis in s.record.axes() {
            put(&mut w, label("mark", [axis]), &s.record.standing(axis));
        }
        for (key, event) in &s.events {
            put(&mut w, label("event", [key]), event);
        }
        for (event, sites) in &s.reach.arrivals {
            for (site, arrival) in sites {
                put(
                    &mut w,
                    label("arrival", [event.to_string(), site.to_string()]),
                    arrival,
                );
            }
        }
        w
    }
}
