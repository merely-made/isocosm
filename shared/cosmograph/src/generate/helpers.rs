// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Small builders the generator and the probes share.

use crate::{rules::*, schema::*};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn set(values: &[&str]) -> BTreeSet<Key> {
    values.iter().map(|s| s.to_string()).collect()
}
pub(crate) fn process(id: &str, causation: Causation, effects: Vec<Effect>) -> Process {
    Process {
        id: id.into(),
        causation,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: set(&["sim:matter", "sim:nonnegative"]),
        note: false,
        port_feeding: None,
        goal_directives: Vec::new(), turn: None,
    }
}

/// The processes every generated world holds, whatever its ecology: a mark
/// to note, a gift, a birth and a death, the founding of a polity, the
/// reserve's record and the weather.
pub(super) fn world_processes(processes: &mut BTreeMap<Key, Process>) {
    let mut mark = process(
        "sim:remember",
        Causation::Choice,
        vec![Effect::Note {
            kind: "sim:observed".into(),
            text: "Observed at the specimen bench.".into(),
            lifetime: None,
        }],
    );
    mark.note = true;
    processes.insert(mark.id.clone(), mark);
    let mut donate = process(
        "sim:give",
        Causation::Choice,
        vec![Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Target,
            account: "world:soil".into(),
            amount: 1.into(),
        }],
    );
    donate.note = true;
    processes.insert(donate.id.clone(), donate);
    let mut birth = process(
        "sim:birth",
        Causation::Transition,
        vec![Effect::Birth {
            provision: BTreeMap::from([("world:soil".into(), 2)]),
            scatter: None,
        }],
    );
    birth.note = true;
    processes.insert(birth.id.clone(), birth);
    let mut death = process("sim:death", Causation::Transition, vec![Effect::Death]);
    death.note = true;
    processes.insert(death.id.clone(), death);
    let polity = process(
        "sim:found-polity",
        Causation::Transition,
        vec![Effect::FoundPolity {
            governance: "governance:consent".into(),
            focus: set(&["sim:give"]),
            support: "world:soil".into(),
        }],
    );
    processes.insert(polity.id.clone(), polity);
    let mut reckon = process(
        "sim:reckon",
        Causation::Transition,
        vec![Effect::Record {
            axis: "feat:reserve".into(),
            account: "world:soil".into(),
        }],
    );
    reckon.note = true;
    processes.insert(reckon.id.clone(), reckon);
    let mut weather = process(
        "world:weather",
        Causation::Agentless,
        vec![Effect::Condition {
            key: "world:weather".into(),
            delta: 1,
        }],
    );
    weather.requires.clear();
    weather.period = Some(3);
    weather.priority = -1;
    processes.insert(weather.id.clone(), weather);
}
